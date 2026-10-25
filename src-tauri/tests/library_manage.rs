//! v0.24 library management: archive, Trash, restore and removal for good
//! (sources and Bibles), the Sources list (filters, sorting, pages, a large
//! library), metadata, collections and tags. All material is invented.

use std::fs;
use std::path::Path;
use std::time::Instant;

use abiding_word_lib::db::library_manage::{
    self, ItemState, ManageQuery, SortBy, UpdateSource, TRASH_RETENTION_DAYS,
};
use abiding_word_lib::db::{bibles, library, Database};
use abiding_word_lib::error::{AppError, AppResult};
use abiding_word_lib::library::import::{self, StagedSource};
use abiding_word_lib::library::{CommitSourceImport, LicenceStatus, SearchLibrary, SourceType};
use abiding_word_lib::scripture::reference::ScriptureRef;
use rusqlite::Connection;

fn with_db<T>(f: impl FnOnce(&Connection) -> AppResult<T>) -> T {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(f).unwrap()
}

fn stage(dir: &Path, name: &str, text: &str) -> StagedSource {
    let path = dir.join(name);
    fs::write(&path, text).unwrap();
    import::stage(&path).unwrap()
}

fn add(conn: &Connection, staged: &StagedSource, title: &str) -> AppResult<String> {
    let id = abiding_word_lib::db::new_id();
    let relative = library::managed_relative_path(&id, staged);
    library::commit_import(
        conn,
        &id,
        staged,
        &CommitSourceImport {
            preview_id: staged.id.clone(),
            title: title.into(),
            allow_duplicate: true,
            ..Default::default()
        },
        &relative,
    )?;
    Ok(id)
}

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn code<T: std::fmt::Debug>(result: AppResult<T>) -> String {
    match result {
        Err(AppError::Conflict(m)) | Err(AppError::InvalidInput(m)) => {
            m.code.unwrap_or_default().to_string()
        }
        Err(AppError::NotFound { .. }) => "notFound".into(),
        other => format!("{other:?}"),
    }
}

#[test]
fn archive_trash_and_restore_move_a_source_between_lists_without_losing_anything() {
    let temp = tempfile::tempdir().unwrap();
    let staged = stage(
        temp.path(),
        "abide.md",
        "# Abide\n\nAbiding in the vine (John 15:5).\n",
    );
    with_db(|conn| {
        let id = add(conn, &staged, "Abide")?;
        let john = ScriptureRef::from_canonical("JHN.15.5").unwrap();
        let search = |conn: &Connection| {
            library::search(
                conn,
                &SearchLibrary {
                    query: "vine".into(),
                    offset: None,
                },
            )
        };
        assert_eq!(library::mentions(conn, &john)?.len(), 1);

        // Archived: out of the Library list and the Sources pane, still searchable.
        library_manage::set_source_state(conn, &ids(&[&id]), ItemState::Archived)?;
        assert!(library::list(conn, &Default::default())?.is_empty());
        assert!(library::mentions(conn, &john)?.is_empty());
        let hits = search(conn)?.hits;
        assert_eq!(hits.len(), 1);
        assert!(hits[0].archived);

        // In Trash: hidden everywhere, text and links kept.
        library_manage::set_source_state(conn, &ids(&[&id]), ItemState::Trashed)?;
        assert!(search(conn)?.hits.is_empty());
        let page = library_manage::list(conn, &ManageQuery::default())?;
        assert_eq!((page.total, page.trashed), (0, 1));

        // Restored: exactly as before.
        library_manage::set_source_state(conn, &ids(&[&id]), ItemState::Active)?;
        assert_eq!(library::list(conn, &Default::default())?.len(), 1);
        assert_eq!(library::mentions(conn, &john)?.len(), 1);
        assert_eq!(search(conn)?.hits.len(), 1);
        assert!(library::get(conn, &id)?.segment_count > 0);
        Ok(())
    });
}

#[test]
fn only_trashed_sources_are_removed_for_good_and_a_refused_batch_removes_nothing() {
    let temp = tempfile::tempdir().unwrap();
    with_db(|conn| {
        let a = add(conn, &stage(temp.path(), "a.md", "# A\n\nFirst."), "A")?;
        let b = add(conn, &stage(temp.path(), "b.md", "# B\n\nSecond."), "B")?;
        library_manage::set_source_state(conn, &ids(&[&a]), ItemState::Trashed)?;
        // b is still active: the whole batch is refused, a stays in Trash.
        assert_eq!(
            code(library_manage::remove_sources(conn, &ids(&[&a, &b]))),
            "library.notInTrash"
        );
        assert_eq!(library::get(conn, &a)?.title, "A");
        // From Trash: removed, its file returned for deletion, a tombstone left.
        let files = library_manage::remove_sources(conn, &ids(&[&a]))?;
        assert_eq!(files.len(), 1);
        assert!(files[0].1.is_some());
        assert_eq!(code(library::get(conn, &a)), "notFound");
        let tombstone: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sources WHERE id = ?1 AND deleted_at IS NOT NULL",
            [&a],
            |r| r.get(0),
        )?;
        assert_eq!(tombstone, 1);
        // Unknown ids change nothing.
        assert_eq!(
            code(library_manage::set_source_state(
                conn,
                &ids(&[&b, "nope"]),
                ItemState::Archived
            )),
            "notFound"
        );
        assert_eq!(library::list(conn, &Default::default())?.len(), 1);
        Ok(())
    });
}

#[test]
fn trash_older_than_the_retention_period_is_found_for_removal() {
    let temp = tempfile::tempdir().unwrap();
    with_db(|conn| {
        let old = add(conn, &stage(temp.path(), "old.md", "# Old\n\nText."), "Old")?;
        let recent = add(conn, &stage(temp.path(), "new.md", "# New\n\nText."), "New")?;
        library_manage::set_source_state(conn, &ids(&[&old, &recent]), ItemState::Trashed)?;
        conn.execute(
            "UPDATE sources SET state_changed_at = '2026-01-01T00:00:00.000Z' WHERE id = ?1",
            [&old],
        )?;
        let (sources, bibles) = library_manage::expired(conn, "2026-10-25T00:00:00.000Z")?;
        assert_eq!(sources, vec![old.clone()]);
        assert!(bibles.is_empty());
        // Exactly the retention period later is still kept.
        let (sources, _) = library_manage::expired(conn, "2026-01-31T00:00:00.000Z")?;
        assert!(sources.is_empty(), "{TRASH_RETENTION_DAYS} days");
        Ok(())
    });
}

fn fixture_bible(conn: &Connection, short: &str) -> AppResult<String> {
    use abiding_word_lib::db::bibles::SourceRecord;
    use abiding_word_lib::scripture::import::{self as bible_import, ImportOrigin};
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usfm-a");
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    let staged = bible_import::stage(&files, ImportOrigin::UserFiles)?;
    let id = bibles::new_translation_id();
    let bible = bibles::commit_import(
        conn,
        &id,
        &staged.content,
        &abiding_word_lib::scripture::CommitImport {
            preview_id: staged.id.clone(),
            name: format!("Fixture {short}"),
            short_name: short.into(),
            language: "English".into(),
            language_code: Some("en".into()),
            publisher: None,
            copyright_notice: None,
            license_type: None,
            words: Default::default(),
            as_document: false,
        },
        &SourceRecord {
            origin: "managed_copy",
            path: Some(format!("sources/bibles/{id}")),
            original_file_name: Some("fixture".into()),
            format: "usfm",
            is_builtin: false,
        },
    )?;
    Ok(bible.id)
}

#[test]
fn a_bible_in_trash_leaves_the_translation_list_and_comes_back_whole() {
    with_db(|conn| {
        let id = fixture_bible(conn, "FXA")?;
        abiding_word_lib::db::settings::update(
            conn,
            &abiding_word_lib::domain::UpdateSettings {
                scripture_translation_id: Some(id.clone()),
                ..Default::default()
            },
        )?;
        let verses = bibles::get(conn, &id)?.verse_count;
        library_manage::set_bible_state(conn, &ids(&[&id]), ItemState::Trashed)?;
        assert!(bibles::list(conn)?.is_empty());
        assert_eq!(bibles::list_all(conn)?[0].state, "trashed");
        // No longer the one being read.
        assert!(abiding_word_lib::db::settings::get(conn)?
            .scripture_translation_id
            .is_none());
        // Not removed while it is active again.
        library_manage::set_bible_state(conn, &ids(&[&id]), ItemState::Active)?;
        assert_eq!(
            code(library_manage::remove_bibles(conn, &ids(&[&id]))),
            "library.notInTrash"
        );
        assert_eq!(bibles::get(conn, &id)?.verse_count, verses);
        library_manage::set_bible_state(conn, &ids(&[&id]), ItemState::Trashed)?;
        let removed = library_manage::remove_bibles(conn, &ids(&[&id]))?;
        assert_eq!(removed.len(), 1);
        assert!(bibles::list_all(conn)?.is_empty());
        Ok(())
    });
}

#[test]
fn details_can_be_edited_without_touching_the_text() {
    let temp = tempfile::tempdir().unwrap();
    with_db(|conn| {
        let id = add(
            conn,
            &stage(temp.path(), "c.md", "# C\n\nGrace (Eph 2:8)."),
            "C",
        )?;
        let segments = library::get(conn, &id)?.segment_count;
        let saved = library_manage::update_source(
            conn,
            &UpdateSource {
                id: id.clone(),
                title: "  Ephesians Commentary ".into(),
                subtitle: None,
                author: Some("A. Writer".into()),
                publisher: None,
                edition: None,
                publication_year: Some(1999),
                isbn: None,
                doi: None,
                language: Some("English".into()),
                source_type: SourceType::Commentary,
                licence_status: LicenceStatus::Private,
                licence: None,
                notes: None,
            },
        )?;
        assert_eq!(saved.title, "Ephesians Commentary");
        assert_eq!(saved.source_type, SourceType::Commentary);
        assert_eq!(saved.segment_count, segments);
        Ok(())
    });
}

#[test]
fn collections_nest_hold_many_sources_and_refuse_cycles() {
    let temp = tempfile::tempdir().unwrap();
    with_db(|conn| {
        let a = add(conn, &stage(temp.path(), "a.md", "# A\n\nOne."), "A")?;
        let b = add(conn, &stage(temp.path(), "b.md", "# B\n\nTwo."), "B")?;
        let sermons = library_manage::create_collection(conn, "Sermons", None)?;
        let romans = library_manage::create_collection(conn, "Romans", Some(&sermons.id))?;
        assert_eq!(
            code(library_manage::create_collection(
                conn,
                "romans",
                Some(&sermons.id)
            )),
            "library.collectionExists"
        );
        assert_eq!(
            code(library_manage::move_collection(
                conn,
                &sermons.id,
                Some(&romans.id)
            )),
            "library.collectionCycle"
        );
        // One source in two collections, the file never copied.
        library_manage::collect(conn, &sermons.id, &ids(&[&a, &b]), true)?;
        library_manage::collect(conn, &romans.id, &ids(&[&a]), true)?;
        let in_romans = library_manage::list(
            conn,
            &ManageQuery {
                collection_id: Some(romans.id.clone()),
                ..Default::default()
            },
        )?;
        assert_eq!(
            in_romans
                .items
                .iter()
                .map(|s| s.title.as_str())
                .collect::<Vec<_>>(),
            ["A"]
        );
        assert_eq!(in_romans.items[0].collections.len(), 2);
        let files: i64 = conn.query_row("SELECT COUNT(*) FROM source_files", [], |r| r.get(0))?;
        assert_eq!(files, 2);
        // Trashed sources are not counted; removing a collection keeps the sources.
        library_manage::set_source_state(conn, &ids(&[&b]), ItemState::Trashed)?;
        let counts = library_manage::collections(conn)?;
        assert_eq!(counts.iter().find(|c| c.id == sermons.id).unwrap().count, 1);
        library_manage::remove_collection(conn, &sermons.id)?;
        assert!(library_manage::collections(conn)?.is_empty());
        assert_eq!(library::get(conn, &a)?.title, "A");
        Ok(())
    });
}

#[test]
fn tags_are_shared_case_insensitively_and_applied_to_many_at_once() {
    let temp = tempfile::tempdir().unwrap();
    with_db(|conn| {
        let a = add(conn, &stage(temp.path(), "a.md", "# A\n\nOne."), "A")?;
        let b = add(conn, &stage(temp.path(), "b.md", "# B\n\nTwo."), "B")?;
        library_manage::retag(conn, &ids(&[&a, &b]), &ids(&["Grace", "Romans"]), &[])?;
        library_manage::retag(conn, &ids(&[&b]), &ids(&["grace"]), &[])?;
        let tags = library_manage::tags(conn)?;
        assert_eq!(
            tags.iter()
                .map(|t| (t.name.as_str(), t.count))
                .collect::<Vec<_>>(),
            [("Grace", 2), ("Romans", 2)]
        );
        let romans = tags.iter().find(|t| t.name == "Romans").unwrap().id.clone();
        library_manage::retag(conn, &ids(&[&a]), &[], std::slice::from_ref(&romans))?;
        let tagged = library_manage::list(
            conn,
            &ManageQuery {
                tag_id: Some(romans),
                ..Default::default()
            },
        )?;
        assert_eq!(tagged.total, 1);
        assert_eq!(
            code(library_manage::rename_tag(conn, &tags[1].id, "GRACE")),
            "library.tagExists"
        );
        Ok(())
    });
}

#[test]
fn the_sources_list_filters_sorts_and_pages_a_large_library_quickly() {
    with_db(|conn| {
        // 3,000 sources written directly (importing each would only test the importer).
        let workspace = abiding_word_lib::db::workspaces::get_default(conn)?;
        let tx = conn.unchecked_transaction()?;
        for n in 0..3000 {
            let kind = ["book", "commentary", "sermon", "article", "notes"][n % 5];
            let language = ["English", "Deutsch", "한국어"][n % 3];
            tx.execute(
                "INSERT INTO sources (id, workspace_id, source_type, title, author, language,
                     content_hash, created_at, updated_at, state)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9)",
                rusqlite::params![
                    format!("s{n:05}"),
                    workspace.id,
                    kind,
                    format!("Title {n:05}"),
                    format!("Author {}", n % 40),
                    language,
                    format!("hash{}", n % 2900),
                    format!("2026-01-01T00:{:02}:{:02}.000Z", n / 60 % 60, n % 60),
                    if n % 10 == 0 {
                        "archived"
                    } else if n % 25 == 1 {
                        "trashed"
                    } else {
                        "active"
                    },
                ],
            )?;
        }
        tx.commit()?;
        let started = Instant::now();
        let page = library_manage::list(
            conn,
            &ManageQuery {
                limit: Some(50),
                ..Default::default()
            },
        )?;
        assert_eq!(page.items.len(), 50);
        assert_eq!(page.total, 3000 - page.trashed);
        assert_eq!(page.archived, 300);
        let page = library_manage::list(
            conn,
            &ManageQuery {
                text: Some("author 7".into()),
                source_type: Some(SourceType::Sermon),
                language: Some("Deutsch".into()),
                sort: SortBy::Title,
                descending: true,
                limit: Some(10),
                ..Default::default()
            },
        )?;
        assert!(page
            .items
            .iter()
            .all(|s| s.author.as_deref() == Some("Author 7")
                && s.source_type == SourceType::Sermon
                && s.language.as_deref() == Some("Deutsch")));
        assert!(page.items.windows(2).all(|w| w[0].title >= w[1].title));
        // The last page and a query with no matches.
        let last = library_manage::list(
            conn,
            &ManageQuery {
                offset: 2_990,
                limit: Some(50),
                ..Default::default()
            },
        )?;
        assert!(last.items.len() < 50);
        let none = library_manage::list(
            conn,
            &ManageQuery {
                text: Some("100%_".into()),
                ..Default::default()
            },
        )?;
        assert_eq!(none.total, 0, "LIKE wildcards are matched literally");
        let duplicates = library_manage::list(
            conn,
            &ManageQuery {
                duplicates: true,
                state: Some(ItemState::Active),
                ..Default::default()
            },
        )?;
        assert!(duplicates.items.iter().all(|s| s.duplicates > 0));
        let elapsed = started.elapsed();
        assert!(elapsed.as_secs_f64() < 2.0, "{elapsed:?}");
        Ok(())
    });
}

#[test]
fn on_restart_expired_trash_and_leftover_folders_go_and_live_files_stay() {
    use abiding_word_lib::filesystem::DataDirs;
    use abiding_word_lib::state::AppState;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Abiding Word");
    let input = tempfile::tempdir().unwrap();
    let keep_staged = stage(input.path(), "keep.md", "# Keep\n\nStays.");
    let old_staged = stage(input.path(), "old.md", "# Old\n\nGoes.");
    let (keep, old, keep_file, old_file) = {
        let state = AppState::start(DataDirs::ensure(&root), None, None);
        let core = state.core().unwrap();
        let mut made = Vec::new();
        for (staged, title) in [(&keep_staged, "Keep"), (&old_staged, "Old")] {
            let id = abiding_word_lib::db::new_id();
            let relative = library::managed_relative_path(&id, staged);
            let file = core.dirs.sources().join(&relative);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, b"copy").unwrap();
            core.db
                .with_conn(|conn| {
                    library::commit_import(
                        conn,
                        &id,
                        staged,
                        &CommitSourceImport {
                            preview_id: staged.id.clone(),
                            title: title.into(),
                            allow_duplicate: true,
                            ..Default::default()
                        },
                        &relative,
                    )
                })
                .unwrap();
            made.push((id, file));
        }
        core.db
            .with_conn(|conn| {
                library_manage::set_source_state(conn, &ids(&[&made[1].0]), ItemState::Trashed)?;
                conn.execute(
                    "UPDATE sources SET state_changed_at = '2020-01-01T00:00:00.000Z' WHERE id = ?1",
                    [&made[1].0],
                )?;
                Ok(())
            })
            .unwrap();
        // A copy whose record was never saved (the app stopped mid-import).
        let orphan = core.dirs.sources().join("notes/0000-orphan/left.md");
        fs::create_dir_all(orphan.parent().unwrap()).unwrap();
        fs::write(&orphan, b"left").unwrap();
        let (a, b) = (made.remove(0), made.remove(0));
        (a.0, b.0, a.1, b.1)
    };
    // Start again.
    let state = AppState::start(DataDirs::ensure(&root), None, None);
    let core = state.core().unwrap();
    assert!(keep_file.exists());
    assert!(!old_file.exists());
    assert!(!core.dirs.sources().join("notes/0000-orphan").exists());
    core.db
        .with_conn(|conn| {
            assert_eq!(library::get(conn, &keep)?.title, "Keep");
            assert_eq!(code(library::get(conn, &old)), "notFound");
            Ok(())
        })
        .unwrap();
    // The user's own files were never touched.
    assert!(input.path().join("keep.md").exists() && input.path().join("old.md").exists());
}
