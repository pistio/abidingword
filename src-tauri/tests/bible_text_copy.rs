//! Corrected USFM editions are explicit new imports. Existing wording and
//! phrase anchors must survive both a read-only comparison and a new copy.
use abiding_word_lib::db::{annotations, bibles, Database};
use abiding_word_lib::scripture::import::{self, ImportOrigin, StagedContent};
use abiding_word_lib::scripture::reference::verse_key;
use abiding_word_lib::scripture::{AnchorEnd, AnchorInput, CommitImport, CreateAnnotation};

fn metadata(short_name: &str) -> CommitImport {
    CommitImport {
        preview_id: "review".into(),
        name: "Invented source".into(),
        short_name: short_name.into(),
        language: "English".into(),
        language_code: Some("en".into()),
        publisher: Some("Fixture publisher".into()),
        copyright_notice: Some("Invented text".into()),
        license_type: None,
        words: Default::default(),
        as_document: false,
        confirm_edition: false,
    }
}

fn source(path: &str) -> bibles::SourceRecord {
    bibles::SourceRecord {
        origin: "managed_copy",
        path: Some(path.into()),
        original_file_name: Some("invented.usfm".into()),
        format: "usfm",
        is_builtin: false,
    }
}

#[test]
fn corrected_copy_preserves_existing_wording_source_and_phrase_anchor() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invented.usfm");
    let bytes = b"\\id JHN\n\\c 1\n\\v 1 Alpha\\f + \\ft An invented footnote.\\f*beta.\n";
    std::fs::write(&path, bytes).unwrap();
    let staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    assert!(!staged.content.has_errors());
    assert_eq!(
        staged.content.books[&43].chapters[&1].verses[&1],
        "Alphabeta."
    );

    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let original_id = bibles::new_translation_id();
        let original = bibles::commit_import(
            conn,
            &original_id,
            &staged.content,
            &metadata("OLD"),
            &source("sources/bibles/original"),
        )?;
        // This is what the old note-stripping reader persisted.
        conn.execute(
            "UPDATE bible_verse_texts SET text = 'Alpha beta.' WHERE translation_id = ?1",
            [&original_id],
        )?;
        let note = annotations::create(
            conn,
            &CreateAnnotation {
                reference: "JHN.1.1".into(),
                kind: "note".into(),
                content: Some("A note on the old wording".into()),
                anchor: Some(AnchorInput {
                    translation_id: original_id.clone(),
                    start: AnchorEnd {
                        verse: "JHN.1.1".into(),
                        offset: 6,
                    },
                    end: AnchorEnd {
                        verse: "JHN.1.1".into(),
                        offset: 11,
                    },
                    quote: "beta.".into(),
                }),
                ..Default::default()
            },
        )?;
        // get() intentionally has no displayed-translation context, unlike
        // create(); compare the same persisted view before and after the copy.
        let saved_note = annotations::get(conn, &note.id)?;
        let writes_before = conn.total_changes();
        let comparison = bibles::compare_source_text(conn, &original_id, &staged.content)?;
        assert_eq!(
            comparison,
            bibles::TextComparison {
                changed_verses: 1,
                ..Default::default()
            }
        );
        assert_eq!(
            conn.total_changes(),
            writes_before,
            "preview must be read-only"
        );
        let short_name = bibles::copy_short_name(conn, "OLD")?;
        assert_eq!(short_name, "OLD-2");
        let copy_id = bibles::new_translation_id();
        let copy = bibles::commit_import(
            conn,
            &copy_id,
            &staged.content,
            &metadata(&short_name),
            &source("sources/bibles/copy"),
        )?;
        assert_ne!(original.id, copy.id);
        assert_eq!(bibles::get(conn, &original_id)?, original);
        assert_eq!(annotations::get(conn, &note.id)?, saved_note);
        let texts = conn
            .prepare("SELECT translation_id, text FROM bible_verse_texts ORDER BY translation_id")?
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;
        assert_eq!(texts[&original_id], "Alpha beta.");
        assert_eq!(texts[&copy_id], "Alphabeta.");
        assert_eq!(
            bibles::compare_source_text(conn, &copy_id, &staged.content)?,
            Default::default()
        );
        Ok(())
    })
    .unwrap();
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn comparison_accounts_for_missing_new_bridged_verses_and_titles() {
    let mut old = StagedContent::default();
    let chapter = old
        .books
        .entry(19)
        .or_default()
        .chapters
        .entry(1)
        .or_default();
    chapter.verses.insert(1, "One".into());
    chapter.verses.insert(2, "Two".into());
    chapter.superscription = Some("Old title".into());
    let mut new = StagedContent::default();
    let chapter = new
        .books
        .entry(19)
        .or_default()
        .chapters
        .entry(1)
        .or_default();
    chapter.verses.insert(1, "One".into());
    chapter.bridges.insert(1, 2);
    chapter.verses.insert(3, "Three".into());

    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        bibles::commit_import(conn, "original", &old, &metadata("OLD"), &source("original"))?;
        assert_eq!(bibles::compare_source_text(conn, "original", &new)?, bibles::TextComparison {
            changed_verses: 1,
            added_verses: 1,
            removed_verses: 1,
            changed_superscriptions: 1,
        });
        assert_eq!(conn.query_row(
            "SELECT text FROM bible_verse_texts WHERE translation_id = 'original' AND verse_id = ?1",
            [verse_key(19, 1, 2)], |row| row.get::<_, String>(0),
        )?, "Two");
        Ok(())
    }).unwrap();
}

#[test]
fn copy_names_are_unicode_bounded_unique_and_rechecked_when_saved() {
    let mut content = StagedContent::default();
    content
        .books
        .entry(43)
        .or_default()
        .chapters
        .entry(1)
        .or_default()
        .verses
        .insert(1, "One".into());
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let name = "가".repeat(20);
        bibles::commit_import(
            conn,
            "original",
            &content,
            &metadata(&name),
            &source("original"),
        )?;
        let next = bibles::copy_short_name(conn, &name)?;
        assert_eq!(next.chars().count(), 20);
        assert!(next.ends_with("-2"));
        bibles::commit_import(conn, "copy", &content, &metadata(&next), &source("copy"))?;
        assert!(bibles::copy_short_name(conn, &name)?.ends_with("-3"));
        assert!(bibles::commit_import(
            conn,
            "collision",
            &content,
            &metadata(&next),
            &source("collision")
        )
        .is_err());
        assert!(bibles::get(conn, "collision").is_err());
        assert_eq!(bibles::list(conn)?.len(), 2);
        Ok(())
    })
    .unwrap();
}

#[test]
fn malformed_source_cannot_be_committed_as_a_corrected_copy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invented.usfm");
    std::fs::write(&path, "\\id JHN\n\\c 1\n\\v 1 One.\n").unwrap();
    let valid = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    std::fs::write(
        &path,
        "\\id JHN\n\\c 1\n\\v 1 One.\\f + unclosed\n\\v 2 Two.\n",
    )
    .unwrap();
    let invalid = import::stage(&[path], ImportOrigin::UserFiles).unwrap();
    assert!(invalid.content.has_errors());
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let original = bibles::commit_import(
            conn,
            "original",
            &valid.content,
            &metadata("OLD"),
            &source("original"),
        )?;
        assert!(bibles::commit_import(
            conn,
            "copy",
            &invalid.content,
            &metadata("OLD-2"),
            &source("copy")
        )
        .is_err());
        assert_eq!(bibles::list(conn)?, vec![original]);
        Ok(())
    })
    .unwrap();
}
