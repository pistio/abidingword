//! Upgrading a schema-24 database to teaching blocks (0025).
//!
//! The database is written with plain SQL after migrations 1–24, so the
//! upgrade is the only thing that can touch it. A sermon, its points, a
//! passage, kept material, a presentation item and a version-1 teaching
//! template stay byte for byte. No blocks are invented. All text is invented.

use std::fs;
use std::path::Path;

use abiding_word_lib::db::migrations::{self, MIGRATIONS};
use abiding_word_lib::db::{teaching, teaching_templates, Database};
use rusqlite::Connection;

const T: &str = "2026-10-28T12:00:00.000Z";

fn database_v24(path: &Path) {
    let mut conn = Connection::open(path).unwrap();
    migrations::run_with(&mut conn, &MIGRATIONS[..24]).unwrap();
    conn.execute_batch(&format!(
        "INSERT INTO workspaces (id, name, is_default, created_at, updated_at) VALUES ('w', 'Home', 1, '{T}', '{T}');
         INSERT INTO projects (id, workspace_id, title, created_at, updated_at) VALUES ('p', 'w', 'John', '{T}', '{T}');
         INSERT INTO studies (id, project_id, title, study_type, created_at, updated_at) VALUES ('s', 'p', 'John 15', 'passage', '{T}', '{T}');
         INSERT INTO teaching_preparations (id, study_id, title, kind, audience, purpose, central_message, introduction, conclusion, notes, position, created_at, updated_at)
           VALUES ('tp', 's', 'Sunday', 'sermon', 'The church', 'Abide', 'Remain', 'Last week', 'Go in peace', 'Ask Anna', 0, '{T}', '{T}');
         INSERT INTO teaching_points (id, preparation_id, position, kind, title, explanation, application, illustration, created_at, updated_at)
           VALUES ('pt', 'tp', 0, NULL, 'The vine', 'He is the vine.', 'Abide daily.', 'A pruned rose.', '{T}', '{T}');
         INSERT INTO teaching_passages (id, preparation_id, point_id, book, start_chapter, start_verse, end_chapter, end_verse, position, created_at)
           VALUES ('ps', 'tp', 'pt', 'JHN', 15, 1, 15, 8, 0, '{T}');
         INSERT INTO teaching_materials (id, preparation_id, point_id, kind, standing, title, content, position, created_at)
           VALUES ('tm', 'tp', 'pt', 'illustration', 'hypothetical', 'The rose', 'Invented.', 0, '{T}');
         INSERT INTO presentations (id, preparation_id, title, theme, position, created_at, updated_at)
           VALUES ('pr', 'tp', 'Sunday slides', 'light', 0, '{T}', '{T}');
         INSERT INTO presentation_items (id, presentation_id, position, kind, source_kind, source_id, source_field, heading, notes, created_at, updated_at)
           VALUES ('pi', 'pr', 0, 'heading', 'point', 'pt', NULL, 'The vine', 'Speaker only', '{T}', '{T}');
         INSERT INTO teaching_templates (id, name, kind, definition, created_at, updated_at)
           VALUES ('tt', 'My sermon', 'sermon', '{{\"points\":[{{\"title\":\"Kept\"}}]}}', '{T}', '{T}');"
    ))
    .unwrap();
}

#[test]
fn a_schema_24_teaching_is_kept_and_gains_no_blocks() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("abiding-word.db");
    let backups = temp.path().join("pre-migration");
    database_v24(&path);

    let db = Database::open(&path, &backups).unwrap();
    let files: Vec<_> = fs::read_dir(&backups)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1);
    assert!(files[0]
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("abiding-word-v24-"));

    db.with_conn(|conn| {
        assert_eq!(migrations::current_version(conn)?, 25);
        let prep = teaching::get(conn, "tp")?;
        assert_eq!(prep.template_key, None);
        assert_eq!(prep.template_version, None);
        assert!(prep.blocks.is_empty());
        assert_eq!(prep.title, "Sunday");
        assert_eq!(prep.kind.as_deref(), Some("sermon"));
        assert_eq!(prep.audience.as_deref(), Some("The church"));
        assert_eq!(prep.purpose.as_deref(), Some("Abide"));
        assert_eq!(prep.central_message.as_deref(), Some("Remain"));
        assert_eq!(prep.introduction, "Last week");
        assert_eq!(prep.conclusion, "Go in peace");
        assert_eq!(prep.notes, "Ask Anna");
        assert_eq!(prep.points.len(), 1);
        assert_eq!(prep.points[0].id, "pt");
        assert_eq!(prep.points[0].title, "The vine");
        assert_eq!(prep.points[0].explanation, "He is the vine.");
        assert_eq!(prep.points[0].application, "Abide daily.");
        assert_eq!(prep.points[0].illustration, "A pruned rose.");
        assert_eq!(prep.passages.len(), 1);
        assert_eq!(prep.passages[0].id, "ps");
        assert_eq!(prep.passages[0].point_id.as_deref(), Some("pt"));
        assert_eq!(prep.passages[0].block_id, None);
        assert_eq!(prep.passages[0].reference.canonical, "JHN.15.1-JHN.15.8");
        assert_eq!(prep.materials[0].title, "The rose");
        assert_eq!(prep.materials[0].content, "Invented.");
        let slide: (String, String, String) = conn.query_row(
            "SELECT kind, source_kind, source_id FROM presentation_items WHERE id = 'pi'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        assert_eq!(slide, ("heading".into(), "point".into(), "pt".into()));
        let template = teaching_templates::get(conn, "tt")?;
        assert_eq!(template.version, 1);
        assert!(template.blocks.is_empty());
        assert_eq!(template.points[0].title, "Kept");
        let blocks: i64 =
            conn.query_row("SELECT COUNT(*) FROM teaching_blocks", [], |r| r.get(0))?;
        assert_eq!(blocks, 0);
        Ok(())
    })
    .unwrap();

    let db = Database::open(&path, &backups).unwrap();
    assert_eq!(fs::read_dir(&backups).unwrap().count(), 1);
    db.with_conn(|conn| {
        assert_eq!(teaching::get(conn, "tp")?.introduction, "Last week");
        Ok(())
    })
    .unwrap();
}
