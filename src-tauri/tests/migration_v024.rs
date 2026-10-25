//! v0.24 migration evidence: a 0.5.0-dev.43 database (schema 19) upgrades to
//! the v0.24 schema (20: archive/Trash/collections/tags, 21: marked places in
//! document Bibles, 22: import jobs) with a checked copy made first, and
//! keeps every source, Bible, mark and study as it was: everything is
//! *in use*, nothing is organised, nothing is in Trash. All data invented.

use std::fs;

use abiding_word_lib::db::library_manage::{self, ManageQuery};
use abiding_word_lib::db::migrations::{self, MIGRATIONS};
use abiding_word_lib::db::{bibles, import_jobs, library, Database};
use abiding_word_lib::library::import;
use abiding_word_lib::library::CommitSourceImport;
use rusqlite::Connection;

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn a_dev43_database_upgrades_to_v024_and_keeps_everything_in_use() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("abiding-word.db");
    let backups = temp.path().join("pre-migration");
    let input = tempfile::tempdir().unwrap();
    let file = input.path().join("romans.md");
    fs::write(&file, "# Romans\n\nJustified by faith (Romans 5:1).").unwrap();
    let staged = import::stage(&file).unwrap();
    let source_id;
    {
        let mut conn = Connection::open(&path).unwrap();
        // dev.43 is schema 19 (migrations 0001–0019).
        migrations::run_with(&mut conn, &MIGRATIONS[..19]).unwrap();
        assert_eq!(migrations::current_version(&conn).unwrap(), 19);
        abiding_word_lib::db::workspaces::ensure_default(&conn).unwrap();
        source_id = abiding_word_lib::db::new_id();
        let relative = library::managed_relative_path(&source_id, &staged);
        library::commit_import(
            &conn,
            &source_id,
            &staged,
            &CommitSourceImport {
                preview_id: staged.id.clone(),
                title: "Romans notes".into(),
                ..Default::default()
            },
            &relative,
        )
        .unwrap();
        let t = "2026-10-20T10:00:00.000Z";
        conn.execute_batch(&format!(
            "INSERT INTO bible_translations (id, name, short_name, language, kind, source_type, source_origin,
                 import_status, book_count, verse_count, created_at, updated_at)
             VALUES ('tr-1', 'Invented', 'INV', 'English', 'structured', 'usfm', 'managed_copy', 'ready', 1, 1, '{t}', '{t}');
             INSERT OR IGNORE INTO bible_verses (id, book_id, chapter, verse) VALUES (8001001, 8, 1, 1);
             INSERT INTO bible_verse_texts (translation_id, verse_id, text) VALUES ('tr-1', 8001001, 'Invented words.');"
        ))
        .unwrap();
    }
    let db = Database::open(&path, &backups).unwrap();
    // The checked copy of schema 19 made before migrating.
    let names: Vec<String> = fs::read_dir(&backups)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        names.iter().any(|n| n.starts_with("abiding-word-v19-")),
        "{names:?}"
    );
    db.with_conn(|conn| {
        assert_eq!(
            migrations::current_version(conn)?,
            MIGRATIONS.last().unwrap().version
        );
        assert_eq!(MIGRATIONS.last().unwrap().version, 22);
        // Sources: in use, listed, searchable as before.
        let page = library_manage::list(conn, &ManageQuery::default())?;
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].title, "Romans notes");
        assert_eq!((page.active, page.archived, page.trashed), (1, 0, 0));
        assert_eq!(
            count(conn, "SELECT COUNT(*) FROM sources WHERE state = 'active'"),
            1
        );
        assert_eq!(library::list(conn, &Default::default())?.len(), 1);
        // Bibles: in use, text unchanged.
        assert_eq!(bibles::list(conn)?.len(), 1);
        assert_eq!(
            bibles::chapter(conn, "tr-1", "RUT", 1)?.verses[0].text,
            "Invented words."
        );
        // New structures start empty.
        for table in [
            "source_collections",
            "source_collection_items",
            "source_tags",
            "source_tag_items",
            "bible_passage_locations",
            "import_jobs",
            "import_job_items",
        ] {
            assert_eq!(
                count(conn, &format!("SELECT COUNT(*) FROM {table}")),
                0,
                "{table}"
            );
        }
        assert!(import_jobs::open_jobs(conn)?.is_empty());
        Ok(())
    })
    .unwrap();
    assert!(source_id.len() > 10);
}
