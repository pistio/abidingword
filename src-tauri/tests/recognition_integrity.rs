//! Scripture integrity in Bible documents (v0.24.3): apparatus never read as
//! verses, running heads and raised verse numbers read from the layout, the
//! difference between finding a verse and verifying its text, and the
//! person's corrections kept across readings, with the database migration
//! that holds them.
//!
//! Every text here is invented for the tests; the books and their verse
//! counts are real, because the recogniser checks chapters against them.

use std::path::{Path, PathBuf};

use abiding_word_lib::db::bible_documents::{self as kept_documents, KeptFile};
use abiding_word_lib::db::bibles::{self, SourceRecord};
use abiding_word_lib::db::migrations::{self, Migration, MIGRATIONS};
use abiding_word_lib::db::Database;
use abiding_word_lib::scripture::canon;
use abiding_word_lib::scripture::documents::{Place, Unit};
use abiding_word_lib::scripture::import::{self, ImportOrigin, Severity, StagedImport};
use abiding_word_lib::scripture::reference::verse_key;
use abiding_word_lib::scripture::structure::{self, Corrections, Recognition};
use abiding_word_lib::scripture::CommitImport;
use rusqlite::Connection;

fn words(book: &str, chapter: u16, verse: u16) -> String {
    // (No "book chapter:verse" in it: Scripture never writes one, and a verse
    // that does is not verified.)
    format!("Invented words of {book} c{chapter}v{verse} for testing.")
}

fn line(text: &str, page: u32) -> Unit {
    Unit {
        text: text.to_string(),
        heading: false,
        place: Place {
            document: 0,
            page: Some(page),
            segment: Some(page),
            offset: None,
        },
        uncertain_order: false,
        note: false,
        large: false,
        styled: false,
        edge: false,
        centred: false,
        column: 0,
        gap_before: false,
        short: false,
        indented: false,
    }
}

fn recognise(units: &[Unit]) -> Recognition {
    structure::recognise(units, Some("en"), &Corrections::default())
}

fn verse(r: &Recognition, code: &str, chapter: u16, verse: u16) -> Option<String> {
    let book = canon::by_code(code).unwrap();
    r.content
        .books
        .get(&book.ordinal)?
        .chapters
        .get(&chapter)?
        .verses
        .get(&verse)
        .cloned()
}

fn errors(r: &Recognition) -> Vec<String> {
    r.content
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| format!("{:?}", i.code))
        .collect()
}

/// Philemon (one chapter, 25 verses) as PDF-like lines on pages of ten
/// verses, with whatever `extra` adds per verse.
fn philemon(extra: impl Fn(u16, u32, &mut Vec<Unit>)) -> Vec<Unit> {
    let mut units = vec![line("Philemon", 1)];
    for v in 1..=25u16 {
        let page = 1 + u32::from(v) / 10;
        units.push(line(&format!("{v} {}", words("Philemon", 1, v)), page));
        extra(v, page, &mut units);
    }
    units
}

#[test]
fn a_verse_printed_only_in_the_apparatus_has_no_text() {
    // "14 a": the verse's number and the label of a note naming this very
    // verse ("a 1:14 Some manuscripts add v. 14: …"): the verse is not in
    // this edition's text, and the note's words are never a verse.
    let mut units = philemon(|v, _, _| {
        let _ = v;
    });
    let at = units
        .iter()
        .position(|u| u.text.starts_with("14 "))
        .unwrap();
    units[at].text = "14 a".into();
    let page = units[at].place.page.unwrap();
    let mut note = line(
        "a 1:14 Some manuscripts add v. 14: Invented words of a variant.",
        page,
    );
    note.note = true;
    let last_on_page = units
        .iter()
        .rposition(|u| u.place.page == Some(page))
        .unwrap();
    units.insert(last_on_page + 1, note);
    let r = recognise(&units);
    assert_eq!(verse(&r, "PHM", 1, 14), None);
    assert_eq!(verse(&r, "PHM", 1, 15).unwrap(), words("Philemon", 1, 15));
    assert!(r
        .apparatus
        .iter()
        .any(|a| a.kind == "footnote" && a.text.contains("variant")));
    // The same label naming another verse is text, as in "il a": kept.
    let mut units = philemon(|_, _, _| {});
    let at = units
        .iter()
        .position(|u| u.text.starts_with("14 "))
        .unwrap();
    units[at].text = "14 a".into();
    let r = recognise(&units);
    assert_eq!(verse(&r, "PHM", 1, 14).as_deref(), Some("a"));
}

#[test]
fn a_running_head_at_a_page_edge_is_furniture_whatever_its_wording() {
    // A head of more words than any earlier rule allowed, at the page's top
    // edge as the layout reads it.
    let units = philemon(|v, page, units| {
        if v % 10 == 9 {
            let mut head = line(
                &format!(
                    "The Letter of Paul to Philemon 1:{} {} The Letter of Paul to Philemon 1:{}",
                    v + 1,
                    300 + page,
                    (v + 10).min(25)
                ),
                page + 1,
            );
            head.edge = true;
            units.push(head);
        }
    });
    let r = recognise(&units);
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    for v in 1..=25 {
        assert_eq!(verse(&r, "PHM", 1, v).unwrap(), words("Philemon", 1, v));
    }
    assert!(r.apparatus.iter().any(|a| a.kind == "runningHead"));
    // A verse's line at a page's edge stays, numbers and all.
    let mut units = philemon(|_, _, _| {});
    let at = units
        .iter()
        .position(|u| u.text.starts_with("12 "))
        .unwrap();
    units[at].text = "12 They counted 1,254 men, 30 oxen.".into();
    units[at].edge = true;
    let r = recognise(&units);
    assert_eq!(
        verse(&r, "PHM", 1, 12).unwrap(),
        "They counted 1,254 men, 30 oxen."
    );
}

#[test]
fn raised_verse_numbers_against_the_next_word_are_verse_numbers() {
    // LibreOffice from USFM: "¹Im Anfang", the number raised, no space.
    let raised = |n: u16| -> String {
        n.to_string()
            .chars()
            .map(|c| {
                "⁰¹²³⁴⁵⁶⁷⁸⁹"
                    .chars()
                    .nth(c.to_digit(10).unwrap() as usize)
                    .unwrap()
            })
            .collect()
    };
    let mut text = String::new();
    for v in 1..=25u16 {
        text.push_str(&format!("{}{} ", raised(v), words("Philemon", 1, v)));
    }
    let r = recognise(&[line("Philemon", 1), line(text.trim(), 1)]);
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    for v in [1u16, 9, 10, 25] {
        assert_eq!(verse(&r, "PHM", 1, v).unwrap(), words("Philemon", 1, v));
    }
    // A plain digit against a word is still no verse number ("God1").
    let r = recognise(&[
        line("Philemon", 1),
        line(&format!("1 {} God2 and more", words("Philemon", 1, 1)), 1),
    ]);
    assert!(verse(&r, "PHM", 1, 1).unwrap().contains("God2"));
}

#[test]
fn a_centred_line_after_a_gap_is_a_heading_even_with_a_full_stop() {
    // "Das Paradies und der Sündenfall.": the body's own type, centred, a
    // block of its own after space, the next verse's number under it.
    let build = |centred: bool| {
        let mut units = philemon(|_, _, _| {});
        let at = units.iter().position(|u| u.text.starts_with("8 ")).unwrap();
        let mut heading = line("Paul's Plea for Onesimus.", units[at].place.page.unwrap());
        heading.place.segment = Some(100);
        heading.centred = centred;
        heading.gap_before = true;
        units.insert(at, heading);
        for u in units.iter_mut().skip(at + 1) {
            if let Some(s) = u.place.segment.as_mut() {
                *s += 200;
            }
        }
        recognise(&units)
    };
    let r = build(true);
    assert_eq!(verse(&r, "PHM", 1, 7).unwrap(), words("Philemon", 1, 7));
    assert!(r.apparatus.iter().any(|a| a.kind == "heading"));
    // Not centred: a poem's or column's first line can begin a block too;
    // it stays in its verse.
    let r = build(false);
    assert!(verse(&r, "PHM", 1, 7).unwrap().ends_with("Onesimus."));
}

#[test]
fn a_verse_whose_text_is_in_doubt_is_found_but_not_verified() {
    let mut units = philemon(|_, _, _| {});
    // A whole line that could be a heading in the body's type, closing
    // verse 7; a passage reference read into verse 12.
    let at = units.iter().position(|u| u.text.starts_with("8 ")).unwrap();
    let page = units[at].place.page.unwrap();
    units.insert(at, line("Paul Pleads for Onesimus", page));
    let at = units
        .iter()
        .position(|u| u.text.starts_with("12 "))
        .unwrap();
    // (A page's head joined into the middle of a verse running over it.)
    units[at].text = "12 Invented words Philemon 1:12 304 Philemon 1:19 and more.".into();
    let r = recognise(&units);
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    let key = |v: u16| (canon::by_code("PHM").unwrap().ordinal, 1u16, v);
    assert_eq!(r.unverified.get(&key(7)), Some(&"possibleHeading"));
    assert_eq!(r.unverified.get(&key(12)), Some(&"reference"));
    assert_eq!(r.unverified.len(), 2);
    // Their places are still known.
    assert!(r.origins.contains_key(&key(7)));
    // An ordinary verse's last line is not in doubt.
    let r = recognise(&philemon(|_, _, _| {}));
    assert!(r.unverified.is_empty());
}

// --- the database: verified verses, places, corrections, migration ---------------

fn write(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

/// Ruth under a title no dictionary knows ("The Moabitess"), one verse a
/// line, so that the person must say which book it is.
fn moabitess(variant: &str) -> String {
    let mut text = String::from("The Moabitess\n\n");
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        text.push_str(&format!("Chapter {chapter}\n\n"));
        for v in 1..=count {
            text.push_str(&format!("{v} {}{variant}\n", words("Ruth", chapter, v)));
        }
        text.push('\n');
    }
    text
}

fn details(staged: &StagedImport) -> CommitImport {
    CommitImport {
        preview_id: staged.id.clone(),
        name: "Invented Moabitess".into(),
        short_name: format!("MOAB{}", &staged.id[..4]),
        language: "English".into(),
        language_code: Some("en".into()),
        publisher: None,
        copyright_notice: None,
        license_type: None,
        words: Default::default(),
        as_document: false,
    }
}

fn add(conn: &Connection, staged: &StagedImport, as_verses: bool) -> String {
    let id = bibles::new_translation_id();
    let stage = staged.documents.as_ref().unwrap();
    let kept: Vec<KeptFile> = stage
        .files
        .iter()
        .enumerate()
        .map(|(i, f)| KeptFile {
            id: format!("{id}-doc-{i}"),
            file_name: format!("{}.{}", i + 1, f.extension),
        })
        .collect();
    kept_documents::commit(
        conn,
        &id,
        &staged.content,
        stage,
        &details(staged),
        &SourceRecord {
            origin: "managed_copy",
            path: Some(format!("sources/bibles/{id}")),
            original_file_name: Some("moabitess.txt".into()),
            format: staged.format,
            is_builtin: false,
        },
        &kept,
        as_verses,
    )
    .unwrap()
    .id
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

fn again(path: &Path, id: &str) -> StagedImport {
    import::stage_documents(
        &[path.to_path_buf()],
        ImportOrigin::Existing {
            translation_id: id.into(),
        },
    )
    .unwrap()
}

#[test]
fn a_correction_survives_reading_again_and_conflicts_are_shown() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = write(temp.path(), "moabitess.txt", &moabitess(""));
    let mut staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    {
        let stage = staged.documents.as_ref().unwrap();
        assert_eq!(stage.books[0].heading, "The Moabitess");
        assert_eq!(stage.books[0].detected, None);
    }
    staged.correct(&[(0, Some("RUT".into()))]).unwrap();
    assert_eq!(staged.content.verse_count(), 85);
    db.with_conn(|conn| {
        let id = add(conn, &staged, true);
        // Kept: the person's choice beside what the recogniser found, and
        // every verse marked as found under a correction.
        assert_eq!(
            count(conn, "SELECT COUNT(*) FROM bible_recognition_corrections WHERE detected_book IS NULL AND chosen_book = 'RUT'"),
            1
        );
        assert_eq!(
            count(conn, "SELECT COUNT(*) FROM bible_verse_origins WHERE basis = 'corrected'"),
            85
        );
        let origin = kept_documents::origin(conn, &id, verse_key(8, 1, 1))?.unwrap();
        assert_eq!(origin.basis, "corrected");
        // Read again: the correction is applied, and says so.
        let mut read = again(&path, &id);
        read.apply_saved(kept_documents::corrections(conn, &id)?, false)?;
        assert_eq!(read.content.verse_count(), 85);
        let stage = read.documents.as_ref().unwrap();
        assert_eq!(stage.saved.len(), 1);
        assert_eq!(stage.saved[0].status, "applied");
        assert!(stage.books[0].corrected);
        assert_eq!(stage.books[0].book, Some("RUT"));
        // The recogniser now reads the heading otherwise than when the
        // correction was made: the person's choice stands, and is shown.
        conn.execute(
            "UPDATE bible_recognition_corrections SET detected_book = 'GEN'",
            [],
        )?;
        let mut read = again(&path, &id);
        read.apply_saved(kept_documents::corrections(conn, &id)?, false)?;
        assert_eq!(read.documents.as_ref().unwrap().saved[0].status, "detectionChanged");
        assert_eq!(read.content.verse_count(), 85);
        conn.execute(
            "UPDATE bible_recognition_corrections SET detected_book = NULL",
            [],
        )?;
        // The file changed (one word): applied where the heading is found
        // again, and the person is asked to check.
        let changed = write(temp.path(), "moabitess.txt", &moabitess(" Changed"));
        let mut read = again(&changed, &id);
        read.apply_saved(kept_documents::corrections(conn, &id)?, false)?;
        let stage = read.documents.as_ref().unwrap();
        assert_eq!(stage.saved[0].status, "documentChanged");
        assert_eq!(read.content.verse_count(), 85);
        // The heading is gone: nothing is applied, and nothing is dropped.
        let gone = write(
            temp.path(),
            "moabitess.txt",
            &moabitess("").replace("The Moabitess", "Another Title"),
        );
        let mut read = again(&gone, &id);
        read.apply_saved(kept_documents::corrections(conn, &id)?, false)?;
        let stage = read.documents.as_ref().unwrap();
        assert_eq!(stage.saved[0].status, "notFound");
        assert_eq!(read.content.verse_count(), 0);
        kept_documents::keep_corrections(conn, &id, stage)?;
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_recognition_corrections"), 1);
        // Set back to what was found: the correction is removed.
        let original = write(temp.path(), "moabitess.txt", &moabitess(""));
        let mut read = again(&original, &id);
        read.apply_saved(kept_documents::corrections(conn, &id)?, false)?;
        read.correct(&[])?;
        kept_documents::keep_corrections(conn, &id, read.documents.as_ref().unwrap())?;
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_recognition_corrections"), 0);
        Ok(())
    })
    .unwrap();
}

#[test]
fn the_same_documents_added_again_keep_the_earlier_corrections() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = write(temp.path(), "moabitess.txt", &moabitess(""));
    let mut staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    staged.correct(&[(0, Some("RUT".into()))]).unwrap();
    db.with_conn(|conn| {
        add(conn, &staged, false);
        // Added as a document only: the correction is kept all the same.
        assert_eq!(
            count(conn, "SELECT COUNT(*) FROM bible_recognition_corrections"),
            1
        );
        let mut second =
            import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
        let hashes: Vec<String> = second
            .documents
            .as_ref()
            .unwrap()
            .files
            .iter()
            .map(|f| f.hash.clone())
            .collect();
        second.apply_saved(
            kept_documents::corrections_for_documents(conn, &hashes)?,
            true,
        )?;
        assert_eq!(
            second.documents.as_ref().unwrap().saved[0].status,
            "earlier"
        );
        assert_eq!(second.content.verse_count(), 85);
        let preview = second.preview(Default::default());
        assert_eq!(preview.document.unwrap().saved.len(), 1);
        Ok(())
    })
    .unwrap();
}

#[test]
fn unverified_verses_keep_their_place_but_never_become_the_bible_s_text() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    // Ruth 1:7 closes with a line that could be a heading.
    let text = moabitess("").replace("The Moabitess", "Ruth").replacen(
        &format!("7 {}\n", words("Ruth", 1, 7)),
        &format!("7 {}\nNaomi Returns Home\n", words("Ruth", 1, 7)),
        1,
    );
    let path = write(temp.path(), "ruth.txt", &text);
    let staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    let preview = staged.preview(Default::default());
    let document = preview.document.as_ref().unwrap();
    assert!(document.verses_usable);
    assert_eq!(document.unverified_verses, 1);
    assert_eq!(document.unverified_reasons.get("possibleHeading"), Some(&1));
    db.with_conn(|conn| {
        let id = add(conn, &staged, true);
        let translation = bibles::get(conn, &id)?;
        // Read by its verses, with the one verse in doubt left out of its
        // text and shown as found but unchecked.
        assert_eq!(translation.kind, "structured");
        assert_eq!(translation.verse_count, 84);
        let chapter = bibles::chapter(conn, &id, "RUT", 1)?;
        assert_eq!(chapter.verses.len(), 21);
        assert!(chapter.verses.iter().all(|v| v.verse != 7));
        assert_eq!(chapter.unchecked.len(), 1);
        assert_eq!(chapter.unchecked[0].verse, 7);
        assert_eq!(chapter.unchecked[0].reason, "possibleHeading");
        assert_eq!(
            count(
                conn,
                &format!(
                    "SELECT COUNT(*) FROM bible_verse_texts WHERE verse_id = {}",
                    verse_key(8, 1, 7)
                )
            ),
            0
        );
        let reference = abiding_word_lib::scripture::reference::ScriptureRef::from_canonical(
            "RUT.1.6-RUT.1.8",
        )?;
        let passage = kept_documents::locate(conn, &id, &reference)?;
        assert_eq!(passage.verses.len(), 2);
        assert!(!passage.complete);
        assert_eq!(passage.places.len(), 1);
        assert_eq!(passage.places[0].reference, "RUT.1.7");
        assert_eq!(passage.places[0].reason, "possibleHeading");
        let coverage = kept_documents::coverage(conn, &id)?;
        assert_eq!((coverage.verses, coverage.unverified), (84, 1));
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_v022_database_upgrades_and_keeps_its_verse_origins_as_automatic() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("abiding-word.db");
    let backups = temp.path().join("pre-migration");
    {
        let mut conn = Connection::open(&path).unwrap();
        migrations::run_with(&mut conn, &MIGRATIONS[..22]).unwrap();
        let t = "2026-10-20T10:00:00.000Z";
        conn.execute_batch(&format!(
            "INSERT INTO bible_translations (id, name, short_name, language, kind, source_type, source_origin,
                 import_status, book_count, verse_count, created_at, updated_at)
             VALUES ('tr-1', 'Invented', 'INV', 'English', 'structured', 'pdf', 'managed_copy', 'ready', 1, 1, '{t}', '{t}');
             INSERT INTO bible_documents (id, translation_id, position, format, file_name, original_file_name,
                 sha256, size_bytes, extraction_status, created_at)
             VALUES ('doc-1', 'tr-1', 0, 'pdf', '1.pdf', 'ruth.pdf', 'abc', 10, 'extracted_text', '{t}');
             INSERT OR IGNORE INTO bible_verses (id, book_id, chapter, verse) VALUES (8001001, 8, 1, 1);
             INSERT INTO bible_verse_texts (translation_id, verse_id, text) VALUES ('tr-1', 8001001, 'Invented words.');
             INSERT INTO bible_verse_origins (translation_id, verse_id, document_id, first_segment, first_page)
             VALUES ('tr-1', 8001001, 'doc-1', 3, 1);"
        ))
        .unwrap();
    }
    let db = Database::open(&path, &backups).unwrap();
    db.with_conn(|conn| {
        assert_eq!(migrations::current_version(conn)?, 23);
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_verse_texts"), 1);
        let origin = kept_documents::origin(conn, "tr-1", 8_001_001)?.unwrap();
        assert_eq!(origin.basis, "automatic");
        assert_eq!(origin.first_segment, Some(3));
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_recognition_corrections"), 0);
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_verse_places"), 0);
        // Removing the Bible removes its corrections, places and apparatus.
        conn.execute_batch(
            "INSERT INTO bible_recognition_corrections (id, translation_id, document_sha256, document_position,
                 kind, heading, chosen_book, created_at, updated_at)
             VALUES ('c-1', 'tr-1', 'abc', 0, 'book', 'Ruth', 'RUT', 'x', 'x');
             INSERT INTO bible_document_apparatus (translation_id, document_id, kind, content)
             VALUES ('tr-1', 'doc-1', 'footnote', 'a 1:1 invented note');",
        )?;
        bibles::remove(conn, "tr-1")?;
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_recognition_corrections"), 0);
        assert_eq!(count(conn, "SELECT COUNT(*) FROM bible_document_apparatus"), 0);
        Ok(())
    })
    .unwrap();
    // A checked copy of schema 22 was made first.
    let names: Vec<String> = std::fs::read_dir(&backups)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        names.iter().any(|n| n.starts_with("abiding-word-v22-")),
        "{names:?}"
    );
}

#[test]
fn a_failing_migration_after_0023_rolls_back_and_leaves_0023_intact() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrations::run_with(&mut conn, MIGRATIONS).unwrap();
    let mut list: Vec<Migration> = MIGRATIONS
        .iter()
        .map(|m| Migration {
            version: m.version,
            name: m.name,
            sql: m.sql,
        })
        .collect();
    list.push(Migration {
        version: 24,
        name: "broken",
        sql: "CREATE TABLE half_done (x INTEGER);
              INSERT INTO bible_recognition_corrections (id) VALUES ('not enough columns');",
    });
    assert!(migrations::run_with(&mut conn, &list).is_err());
    assert_eq!(migrations::current_version(&conn).unwrap(), 23);
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM sqlite_master WHERE name = 'half_done'"
        ),
        0
    );
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM sqlite_master WHERE name = 'bible_verse_places'"
        ),
        1
    );
}

#[test]
fn headings_set_apart_are_kept_with_the_passage_they_stand_in() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    // A Markdown Ruth with a section heading of its own between 1:7 and 1:8.
    let mut text = String::from("# Ruth\n\n");
    for (chapter, count) in [(1u16, 22u16), (2, 23), (3, 18), (4, 22)] {
        text.push_str(&format!("## Chapter {chapter}\n\n"));
        for v in 1..=count {
            if (chapter, v) == (1, 8) {
                text.push_str("### Naomi Turns Homeward\n\n");
            }
            text.push_str(&format!("{v} {}\n\n", words("Ruth", chapter, v)));
        }
    }
    let path = write(temp.path(), "ruth.md", &text);
    let staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    db.with_conn(|conn| {
        let id = add(conn, &staged, true);
        let passage = kept_documents::locate(
            conn,
            &id,
            &abiding_word_lib::scripture::reference::ScriptureRef::from_canonical(
                "RUT.1.6-RUT.1.8",
            )?,
        )?;
        assert_eq!(passage.verses.len(), 3);
        // The heading is in no verse, and is kept beside them.
        assert!(passage.verses.iter().all(|v| !v.text.contains("Homeward")));
        assert!(passage
            .apparatus
            .iter()
            .any(|a| a.kind == "heading" && a.text == "Naomi Turns Homeward"));
        Ok(())
    })
    .unwrap();
}

// --- the independent review's cases (v0.24.3) -----------------------------------

fn key(v: u16) -> (u8, u16, u16) {
    (canon::by_code("PHM").unwrap().ordinal, 1, v)
}

#[test]
fn a_heading_in_the_middle_of_a_verse_leaves_it_unverified() {
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("7 ")).unwrap();
    let page = units[at].place.page.unwrap();
    units[at].text = "7 Invented words, first part.".into();
    let mut heading = line("Paul Pleads for Onesimus", page);
    heading.short = true;
    units.insert(at + 1, heading);
    units.insert(at + 2, line("and the rest of verse seven.", page));
    let r = recognise(&units);
    assert!(verse(&r, "PHM", 1, 7).unwrap().contains("Onesimus"));
    assert_eq!(r.unverified.get(&key(7)), Some(&"possibleHeading"));
}

#[test]
fn the_verse_before_a_missing_number_is_not_verified() {
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("8 ")).unwrap();
    units[at].text = units[at].text.trim_start_matches("8 ").to_string();
    let r = recognise(&units);
    assert_eq!(verse(&r, "PHM", 1, 8), None);
    assert_eq!(r.unverified.get(&key(7)), Some(&"boundary"));
}

#[test]
fn a_raised_number_against_a_word_does_not_make_a_verified_boundary() {
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("8 ")).unwrap();
    units.remove(at);
    let at = units.iter().position(|u| u.text.starts_with("7 ")).unwrap();
    units[at].text = "7 Invented words⁸ of verse seven continues.".into();
    let r = recognise(&units);
    // Read as verse 8 (the next verse expected), but never as verified.
    assert!(r.unverified.contains_key(&key(7)));
    assert!(r.unverified.contains_key(&key(8)));
    // A raised number left in a verse is a note's: not verified either.
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("7 ")).unwrap();
    units[at].text = "7 Invented words⁴ of verse seven.".into();
    let r = recognise(&units);
    assert!(verse(&r, "PHM", 1, 7).unwrap().contains('⁴'));
    assert_eq!(r.unverified.get(&key(7)), Some(&"noteSign"));
}

#[test]
fn references_with_a_full_stop_or_comma_are_not_verified_text() {
    for head in [
        "Philemon 1.12 304 Philemon 1.19",
        "Philemon 1,12 304 Philemon 1,19",
    ] {
        let mut units = philemon(|_, _, _| {});
        let at = units
            .iter()
            .position(|u| u.text.starts_with("12 "))
            .unwrap();
        units[at].text = format!("12 Invented words {head} and more.");
        let r = recognise(&units);
        assert_eq!(r.unverified.get(&key(12)), Some(&"reference"), "{head}");
    }
    // A number such as "1.000" is not a reference.
    let mut units = philemon(|_, _, _| {});
    let at = units
        .iter()
        .position(|u| u.text.starts_with("12 "))
        .unwrap();
    units[at].text = "12 There were de 1.000 men, and 2,500 more.".into();
    let r = recognise(&units);
    assert_eq!(r.unverified.get(&key(12)), None);
}

#[test]
fn a_verse_per_line_psalter_stays_a_bible_read_by_its_verses() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let mut text = String::new();
    for (psalm, count) in [(1u16, 6u16), (2, 12), (3, 8)] {
        for v in 1..=count {
            text.push_str(&format!("PSA {psalm}:{v} {}\n", words("Psalm", psalm, v)));
        }
    }
    let path = write(temp.path(), "psalms.txt", &text);
    let staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    let stage = staged.documents.as_ref().unwrap();
    // Only Psalm 3 has a title in the Hebrew text: only its verse 1 is in doubt.
    assert_eq!(stage.unverified.len(), 1);
    assert_eq!(
        stage
            .unverified
            .get(&(canon::by_code("PSA").unwrap().ordinal, 3, 1)),
        Some(&"psalmTitle")
    );
    db.with_conn(|conn| {
        let id = add(conn, &staged, true);
        let bible = bibles::get(conn, &id)?;
        assert_eq!(bible.kind, "structured");
        assert_eq!(bible.verse_count, 25);
        let chapter = bibles::chapter(conn, &id, "PSA", 3)?;
        assert_eq!(chapter.unchecked.len(), 1);
        assert_eq!(chapter.unchecked[0].verse, 1);
        Ok(())
    })
    .unwrap();
}

#[test]
fn an_undone_correction_found_elsewhere_now_does_not_come_back() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = write(temp.path(), "moabitess.txt", &moabitess(""));
    let mut staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    staged.correct(&[(0, Some("RUT".into()))]).unwrap();
    db.with_conn(|conn| {
        let id = add(conn, &staged, true);
        // The kept correction's place no longer matches (as when segments are
        // numbered otherwise by a newer version): found by its wording.
        conn.execute("UPDATE bible_recognition_corrections SET segment = 999", [])?;
        let mut read = again(&path, &id);
        read.apply_saved(kept_documents::corrections(conn, &id)?, false)?;
        assert_eq!(read.documents.as_ref().unwrap().saved[0].status, "applied");
        // The person sets it back to what was found.
        read.correct(&[])?;
        kept_documents::keep_corrections(conn, &id, read.documents.as_ref().unwrap())?;
        assert_eq!(
            count(conn, "SELECT COUNT(*) FROM bible_recognition_corrections"),
            0
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_correction_the_recogniser_now_agrees_with_is_kept() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = write(
        temp.path(),
        "ruth.txt",
        &moabitess("").replace("The Moabitess", "Ruth"),
    );
    let staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    db.with_conn(|conn| {
        let id = add(conn, &staged, true);
        // A correction made when the heading was read as no book.
        let doc: String = conn.query_row("SELECT sha256 FROM bible_documents", [], |r| r.get(0))?;
        conn.execute(
            "INSERT INTO bible_recognition_corrections (id, translation_id, document_sha256, document_position,
                 kind, heading, page, segment, detected_book, chosen_book, created_at, updated_at)
             VALUES ('c-1', ?1, ?2, 0, 'book', 'Ruth', NULL, 0, NULL, 'RUT', 'x', 'x')",
            rusqlite::params![id, doc],
        )?;
        let mut read = again(&path, &id);
        read.apply_saved(kept_documents::corrections(conn, &id)?, false)?;
        assert_eq!(read.documents.as_ref().unwrap().saved[0].status, "detectionChanged");
        // The review sends no correction for it (book = detected), and
        // another change is made elsewhere: the person's choice stays kept.
        read.correct(&[])?;
        kept_documents::keep_corrections(conn, &id, read.documents.as_ref().unwrap())?;
        assert_eq!(
            count(conn, "SELECT COUNT(*) FROM bible_recognition_corrections WHERE chosen_book = 'RUT'"),
            1
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn corrections_that_fit_nothing_are_carried_to_the_bible_added_again() {
    let db = Database::open_in_memory().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = write(temp.path(), "moabitess.txt", &moabitess(""));
    let mut staged = import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
    staged.correct(&[(0, Some("RUT".into()))]).unwrap();
    db.with_conn(|conn| {
        let first = add(conn, &staged, true);
        // A second correction, for a heading not in the document.
        let doc: String = conn.query_row("SELECT sha256 FROM bible_documents", [], |r| r.get(0))?;
        conn.execute(
            "INSERT INTO bible_recognition_corrections (id, translation_id, document_sha256, document_position,
                 kind, heading, page, segment, detected_book, chosen_book, created_at, updated_at)
             VALUES ('c-2', ?1, ?2, 0, 'book', 'Elsewhere', NULL, 5, NULL, 'GEN', 'x', 'x')",
            rusqlite::params![first, doc],
        )?;
        let mut second =
            import::stage(std::slice::from_ref(&path), ImportOrigin::UserFiles).unwrap();
        let hashes = vec![doc.clone()];
        second.apply_saved(kept_documents::corrections_for_documents(conn, &hashes)?, true)?;
        let id = add(conn, &second, true);
        assert_eq!(
            count(conn, &format!("SELECT COUNT(*) FROM bible_recognition_corrections WHERE translation_id = '{id}'")),
            2
        );
        Ok(())
    })
    .unwrap();
}

// --- heading and numbering evidence added after the first audit (v0.24.3) --------

#[test]
fn a_heading_over_two_lines_at_a_verse_s_end_leaves_it_unverified() {
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("7 ")).unwrap();
    let page = units[at].place.page.unwrap();
    units.insert(
        at + 1,
        line("Paul Asks a Favour for Onesimus, His Son in the", page),
    );
    units.insert(at + 2, line("faith, Now Useful to Him", page));
    let r = recognise(&units);
    assert_eq!(r.unverified.get(&key(7)), Some(&"possibleHeading"));
    // A wrapped line of prose beginning a sentence is no heading when the
    // verse's sentence ends after it.
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("7 ")).unwrap();
    units[at].text = "7 Invented words, first part.".into();
    units.insert(
        at + 1,
        line("The next sentence of verse seven goes on", page),
    );
    units.insert(at + 2, line("and ends here.", page));
    let r = recognise(&units);
    assert_eq!(r.unverified.get(&key(7)), None);
}

#[test]
fn the_line_before_a_line_of_references_is_not_verified_text() {
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("8 ")).unwrap();
    let page = units[at].place.page.unwrap();
    units.insert(at, line("Who Is the Greatest?", page));
    units.insert(at + 1, line("V. 8-14: cf. (Col 4:9. Col 3:22.)", page));
    let r = recognise(&units);
    assert!(!verse(&r, "PHM", 1, 7).unwrap().contains("V. 8"));
    assert_eq!(r.unverified.get(&key(7)), Some(&"possibleHeading"));
}

#[test]
fn a_number_after_a_verse_abbreviation_or_in_a_book_name_is_no_verse() {
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("8 ")).unwrap();
    units[at].text = format!("8 {} As in V. 9 and v. 10 below.", words("Philemon", 1, 8));
    let r = recognise(&units);
    assert!(errors(&r).is_empty(), "{:?}", errors(&r));
    assert!(verse(&r, "PHM", 1, 8).unwrap().contains("V. 9"));
}

#[test]
fn a_verse_ending_with_capitals_or_a_book_s_name_is_not_verified() {
    for tail in ["THE LETTER TO PHILEMON", "Isaiah", "30."] {
        let mut units = philemon(|_, _, _| {});
        let at = units.iter().position(|u| u.text.starts_with("9 ")).unwrap();
        units[at].text = format!("9 {} {tail}", words("Philemon", 1, 9));
        let r = recognise(&units);
        assert!(r.unverified.contains_key(&key(9)), "{tail}");
    }
}

#[test]
fn a_verb_that_is_also_an_abbreviation_still_ends_a_sentence() {
    // "ver" (Spanish, Portuguese "to see") before the next verse's number.
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("8 ")).unwrap();
    units.remove(at);
    let at = units.iter().position(|u| u.text.starts_with("7 ")).unwrap();
    units[at].text = format!("7 Mirad para ver. 8 {}", words("Philemon", 1, 8));
    let r = recognise(&units);
    assert!(verse(&r, "PHM", 1, 8).is_some(), "{:?}", errors(&r));
}

#[test]
fn a_styled_heading_after_an_ellipsis_is_not_verified_text() {
    let mut units = philemon(|_, _, _| {});
    let at = units.iter().position(|u| u.text.starts_with("8 ")).unwrap();
    let page = units[at].place.page.unwrap();
    units[at - 1].text = "7 Then he said to me: Go, I will send you far away…".into();
    let mut heading = line(
        "New Tumult. About to Be Flogged, Paul Claims to Be a Roman",
        page,
    );
    heading.styled = true;
    units.insert(at, heading);
    units.insert(at + 1, line("V. 8-14: cf. Col 4:9, 10.", page));
    let r = recognise(&units);
    assert_eq!(
        r.unverified.get(&key(7)),
        Some(&"possibleHeading"),
        "{:?}",
        verse(&r, "PHM", 1, 7)
    );
}
