//! v0.24 migration evidence: a 0.5.0-dev.43 database (schema 19) upgrades to
//! the v0.24 schema (20: archive/Trash/collections/tags, 21: marked places in
//! document Bibles, 22: import jobs) with a checked copy made first, and
//! keeps every source, Bible, mark and study as it was: everything is
//! *in use*, nothing is organised, nothing is in Trash. All data invented.

use std::fs;

use abiding_word_lib::db::library_manage::{self, ManageQuery};
use abiding_word_lib::db::migrations::{self, MIGRATIONS};
use abiding_word_lib::db::{bibles, import_jobs, library, Database};
use rusqlite::{params, Connection};

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn a_dev43_database_upgrades_to_v024_and_keeps_everything_in_use() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("abiding-word.db");
    let backups = temp.path().join("pre-migration");
    let text = "Justified by faith (Romans 5:1).";
    let original = format!("# Romans\n\n{text}");
    let hash = abiding_word_lib::library::files::content_hash(original.as_bytes());
    let t = "2026-10-20T10:00:00.000Z";
    let source_id;
    {
        let mut conn = Connection::open(&path).unwrap();
        // dev.43 is schema 19 (migrations 0001–0019).
        migrations::run_with(&mut conn, &MIGRATIONS[..19]).unwrap();
        assert_eq!(migrations::current_version(&conn).unwrap(), 19);
        let workspace = abiding_word_lib::db::workspaces::ensure_default(&conn).unwrap();
        source_id = abiding_word_lib::db::new_id();
        // Seed exactly the columns available in schema 19. Today's import API
        // runs only after migration and must not be used to invent historical
        // rows requiring structure metadata that did not exist then.
        conn.execute("INSERT INTO sources (id,workspace_id,source_type,title,language,content_hash,passage_index_version,created_at,updated_at) VALUES (?1,?2,'notes','Romans notes','English',?3,4,?4,?4)", params![source_id,workspace.id,hash,t]).unwrap();
        conn.execute("INSERT INTO source_files (id,source_id,relative_path,format,original_filename,sha256,size_bytes,extraction_status,created_at,updated_at) VALUES ('old-file',?1,?2,'markdown','romans.md',?3,?4,'digital_text',?5,?5)", params![source_id,format!("books/{source_id}/romans.md"),hash,original.len() as i64,t]).unwrap();
        conn.execute("INSERT INTO source_segments (id,source_id,source_file_id,position,content,heading,section_path,paragraph_index,locator,extraction_status,created_at,updated_at) VALUES ('old-segment',?1,'old-file',0,?2,'Romans','Romans',0,'Romans · ¶ 1','digital_text',?3,?3)", params![source_id,text,t]).unwrap();
        let offset = text.find("Romans 5:1").unwrap() as i64;
        conn.execute("INSERT INTO source_passages (id,source_id,segment_id,passage,start_key,end_key,matched_text,start_offset,end_offset,created_at,origin) VALUES ('old-literal',?1,'old-segment','ROM.5.1',45005001,45005001,'Romans 5:1',?2,?3,?4,'explicit')", params![source_id,offset,offset+10,t]).unwrap();
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
        assert!(MIGRATIONS.last().unwrap().version >= 22);
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
        let original_segment: (String, String, String) = conn.query_row(
            "SELECT content,created_at,updated_at FROM source_segments WHERE id='old-segment' AND source_id=?1", [&source_id],
            |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        assert_eq!(original_segment, (text.into(), t.into(), t.into()));
        let original_link: (String, String, i64, i64, String) = conn.query_row(
            "SELECT segment_id,passage,start_offset,end_offset,created_at FROM source_passages WHERE id='old-literal' AND source_id=?1", [&source_id],
            |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
        let offset = text.find("Romans 5:1").unwrap() as i64;
        assert_eq!(original_link, ("old-segment".into(), "ROM.5.1".into(), offset, offset+10, t.into()));
        assert_eq!(conn.query_row("SELECT sha256 FROM source_files WHERE id='old-file'", [], |r| r.get::<_,String>(0))?, hash);
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
