//! Metadata-only edits: names must be validated, and source/verses kept.
use abiding_word_lib::db::{bibles, Database};
use abiding_word_lib::error::{AppError, AppResult};
use abiding_word_lib::scripture::UpdateBibleDetails;
use rusqlite::{params, Connection};

fn seed(conn: &Connection, id: &str, short_name: &str, origin: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO bible_translations
         (id, name, short_name, language, language_code, publisher, copyright_notice,
          license_type, kind, source_type, source_origin, source_path, original_file_name,
          import_status, is_builtin, created_at, updated_at)
         VALUES (?1, 'Invented Ruth', ?2, 'English', 'en', 'Original publisher',
                 'Original source note', 'Source licence', 'document', 'docx',
                 ?3, 'sources/bibles/original.docx', 'original.docx',
                 'document_only', 0, '2026-11-26T00:00:00Z', '2026-11-26T00:00:00Z')",
        params![id, short_name, origin],
    )?;
    Ok(())
}

fn input(id: &str, short: &str) -> UpdateBibleDetails {
    UpdateBibleDetails {
        id: id.into(),
        name: "  Invented Ruth Updated  ".into(),
        short_name: short.into(),
        language: "Deutsch".into(),
        language_code: Some("DE".into()),
        publisher: Some("  Corrected publisher  ".into()),
        copyright_notice: Some("  Corrected reference note  ".into()),
    }
}

#[test]
fn edits_metadata_without_altering_source_identity_edition_or_verse_state() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        seed(conn, "doc", "IRD", "managed_copy")?;
        let before = bibles::get(conn, "doc")?;
        let after = bibles::update_details(conn, &input("doc", "IRD2"))?;
        assert_eq!(after.name, "Invented Ruth Updated");
        assert_eq!(after.short_name, "IRD2");
        assert_eq!(after.language, "Deutsch");
        assert_eq!(after.language_code.as_deref(), Some("de"));
        assert_eq!(after.publisher.as_deref(), Some("Corrected publisher"));
        assert_eq!(after.copyright_notice.as_deref(), Some("Corrected reference note"));
        assert_eq!(after.source_type, before.source_type);
        assert_eq!(after.source_origin, before.source_origin);
        assert_eq!(after.source_path, before.source_path);
        assert_eq!(after.original_file_name, before.original_file_name);
        assert_eq!(after.license_type, before.license_type);
        assert_eq!(after.edition_confirmed, before.edition_confirmed);
        assert_eq!(after.document_recognised, before.document_recognised);
        assert_eq!(after.kind, before.kind);
        assert_eq!(after.verse_count, before.verse_count);
        assert_eq!(after.book_count, before.book_count);
        assert_eq!(after.id, before.id);
        assert_eq!(after.created_at, before.created_at);
        assert_eq!(bibles::get(conn, "doc")?, after);
        Ok(())
    }).unwrap();
}

#[test]
fn duplicate_short_name_and_validation_fail_without_partial_edits() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        seed(conn, "doc1", "IRD", "managed_copy")?;
        seed(conn, "doc2", "WEB", "managed_copy")?;
        let unchanged = bibles::get(conn, "doc1")?;
        let duplicated = bibles::update_details(conn, &input("doc1", "web"));
        assert!(matches!(duplicated, Err(AppError::DuplicateTranslation(_))));
        let mut invalid = input("doc1", "X".repeat(21).as_str());
        assert!(matches!(bibles::update_details(conn, &invalid), Err(AppError::InvalidInput(_))));
        invalid = input("doc1", "IRD");
        invalid.name = " ".into();
        assert!(matches!(bibles::update_details(conn, &invalid), Err(AppError::InvalidInput(_))));
        invalid = input("doc1", "IRD");
        invalid.language_code = Some("no valid tag!".into());
        assert!(matches!(bibles::update_details(conn, &invalid), Err(AppError::InvalidInput(_))));
        invalid = input("doc1", "IRD");
        invalid.copyright_notice = Some("n".repeat(2_001));
        assert!(matches!(bibles::update_details(conn, &invalid), Err(AppError::InvalidInput(_))));
        assert_eq!(bibles::get(conn, "doc1")?, unchanged);
        Ok(())
    }).unwrap();
}

#[test]
fn bundled_source_attribution_is_immutable_even_when_display_name_changes() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        seed(conn, "included", "INC", "bundled")?;
        let prior = bibles::get(conn, "included")?;
        let mut edit = input("included", "NEW");
        assert!(matches!(bibles::update_details(conn, &edit), Err(AppError::InvalidInput(_))));
        assert_eq!(bibles::get(conn, "included")?, prior);
        edit.publisher = prior.publisher.clone();
        edit.copyright_notice = prior.copyright_notice.clone();
        let updated = bibles::update_details(conn, &edit)?;
        assert_eq!(updated.short_name, "NEW");
        assert_eq!(updated.publisher, prior.publisher);
        assert_eq!(updated.copyright_notice, prior.copyright_notice);
        Ok(())
    }).unwrap();
}
