//! Bible documents (v0.23): the files a Bible was added from when they are
//! PDF, EPUB, Word, Markdown or text documents; their text for reading and
//! searching; and where each recognised verse came from.
//!
//! A document Bible is kept as its own documents, never as library sources:
//! it has no citations, marks or passage links of the library's. Removing
//! the Bible removes all of it (`ON DELETE CASCADE`), and the command layer
//! deletes the managed copies.

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::db::bibles::{self, Shape, SourceRecord};
use crate::db::now;
use crate::error::{AppError, AppResult, UserMessage};
use crate::scripture::import::{DocumentStage, SavedCorrection, StagedContent};
use crate::scripture::reference::verse_key;
use crate::scripture::{BibleTranslation, CommitImport};

/// A kept document file: its row id and its managed copy's name.
pub struct KeptFile {
    pub id: String,
    pub file_name: String,
}

/// Adds a Bible read from documents in one transaction: by its verses when
/// `as_verses` (they must be free of problems), otherwise as a document to
/// read. The documents and their text are kept either way.
#[allow(clippy::too_many_arguments)]
pub fn commit(
    conn: &Connection,
    id: &str,
    content: &StagedContent,
    stage: &DocumentStage,
    meta: &CommitImport,
    source: &SourceRecord,
    kept: &[KeptFile],
    as_verses: bool,
) -> AppResult<BibleTranslation> {
    if as_verses && (content.has_errors() || content.verse_count() == 0) {
        return Err(AppError::Import(UserMessage::new(
            "bibles.hasProblems",
            "This Bible has problems that must be resolved before it can be added.",
        )));
    }
    if kept.len() != stage.files.len() {
        return Err(AppError::Unexpected(
            "document files and records differ".into(),
        ));
    }
    let checked = bibles::check_details(conn, meta)?;
    let now = now();
    let tx = conn.unchecked_transaction()?;
    // Every verse found with its text verified (v0.24.3): a Bible read by its
    // verses only when there is no other; otherwise it stays a document whose
    // verified passages can be found, the rest by their places.
    let whole = as_verses && stage.unverified.is_empty();
    let shape = if whole {
        Shape::Structured(content)
    } else {
        Shape::Document
    };
    bibles::insert_translation(&tx, id, &checked, meta, source, shape, &now)?;
    insert_documents(&tx, id, stage, kept, &now)?;
    insert_apparatus(&tx, id, stage, kept)?;
    save_corrections(&tx, id, stage, &stage_corrections(stage), &now)?;
    if as_verses {
        store_verses(&tx, id, content, stage, kept, None)?;
        if !whole {
            update_counts(&tx, id, false)?;
        }
    }
    tx.commit()?;
    log::info!(
        "Bible added from {} document(s): {} ({})",
        kept.len(),
        checked.short_name,
        if as_verses {
            format!("{} verses", content.verse_count())
        } else {
            "read as a document".into()
        }
    );
    bibles::get(conn, id)
}

fn insert_documents(
    tx: &Connection,
    id: &str,
    stage: &DocumentStage,
    kept: &[KeptFile],
    now: &str,
) -> AppResult<()> {
    let mut add_document = tx.prepare(
        "INSERT INTO bible_documents
             (id, translation_id, position, format, file_name, original_file_name, sha256,
              size_bytes, extraction_status, page_count, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )?;
    let mut add_segment = tx.prepare(
        "INSERT INTO bible_document_segments (document_id, position, content, page_number, section_path)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for (position, (file, record)) in stage.files.iter().zip(kept).enumerate() {
        add_document.execute(params![
            record.id,
            id,
            position as i64,
            file.format,
            record.file_name,
            file.file_name,
            file.hash,
            file.size as i64,
            file.extraction_status.as_str(),
            file.page_count,
            now
        ])?;
        for (n, segment) in file.segments.iter().enumerate() {
            add_segment.execute(params![
                record.id,
                n as i64,
                segment.text,
                segment.page_number,
                segment.section_path
            ])?;
        }
    }
    Ok(())
}

/// The verses of a reading (v0.24.3), in the chapters `usable` holds (None:
/// all): those whose text is verified become the Bible's verses, with where
/// they came from; the others keep only their place in the document
/// (`bible_verse_places`), with why their text was not verified. A verse
/// under a book the person chose is marked as corrected.
fn store_verses(
    tx: &Connection,
    id: &str,
    content: &StagedContent,
    stage: &DocumentStage,
    kept: &[KeptFile],
    usable: Option<&std::collections::BTreeSet<(u8, u16)>>,
) -> AppResult<()> {
    let corrected: std::collections::HashSet<u8> = stage
        .books
        .iter()
        .filter(|b| b.corrected && !b.left_out)
        .filter_map(|b| b.book.and_then(crate::scripture::canon::by_code))
        .map(|b| b.ordinal)
        .collect();
    let in_chapter =
        |ordinal: u8, chapter: u16| usable.is_none_or(|u| u.contains(&(ordinal, chapter)));
    bibles::insert_verses_matching(tx, id, content, |o, c, v| {
        in_chapter(o, c) && stage.verified(None, (o, c, v))
    })?;
    let mut add_origin = tx.prepare(
        "INSERT INTO bible_verse_origins
             (translation_id, verse_id, document_id, first_segment, last_segment, first_page, last_page,
              first_offset, basis)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    let mut add_place = tx.prepare(
        "INSERT INTO bible_verse_places
             (translation_id, verse_id, document_id, first_segment, last_segment, first_page, last_page,
              first_offset, basis, reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?;
    let mut add_verse = tx.prepare(
        "INSERT OR IGNORE INTO bible_verses (id, book_id, chapter, verse) VALUES (?1, ?2, ?3, ?4)",
    )?;
    for (&(ordinal, chapter, verse), origin) in &stage.origins {
        if !in_chapter(ordinal, chapter) {
            continue;
        }
        let Some(document) = kept.get(usize::from(origin.start.document)) else {
            continue;
        };
        // A verse that runs into the next file keeps only where it starts.
        let same = origin.end.document == origin.start.document;
        let key = verse_key(ordinal, chapter, verse);
        let basis = if corrected.contains(&ordinal) {
            "corrected"
        } else {
            "automatic"
        };
        let place = params![
            id,
            key,
            document.id,
            origin.start.segment,
            if same { origin.end.segment } else { None },
            origin.start.page,
            if same { origin.end.page } else { None },
            origin.start.offset,
            basis
        ];
        match stage.unverified.get(&(ordinal, chapter, verse)) {
            None => {
                add_origin.execute(place)?;
            }
            Some(reason) => {
                add_verse.execute(params![key, ordinal, chapter, verse])?;
                add_place.execute(params![
                    id,
                    key,
                    document.id,
                    origin.start.segment,
                    if same { origin.end.segment } else { None },
                    origin.start.page,
                    if same { origin.end.page } else { None },
                    origin.start.offset,
                    basis,
                    reason
                ])?;
            }
        }
    }
    Ok(())
}

/// The Bible's counts from its stored verses, and its kind: read by verses
/// (`whole`), or a document with some passages found.
fn update_counts(tx: &Connection, id: &str, whole: bool) -> AppResult<(i64, i64)> {
    let verses: i64 = tx.query_row(
        "SELECT COUNT(*) FROM bible_verse_texts WHERE translation_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    let books: i64 = tx.query_row(
        "SELECT COUNT(DISTINCT v.book_id) FROM bible_verse_texts t JOIN bible_verses v ON v.id = t.verse_id
         WHERE t.translation_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    let (kind, status) = if whole {
        ("structured", "ready")
    } else {
        ("document", "document_only")
    };
    tx.execute(
        "UPDATE bible_translations
         SET kind = ?2, import_status = ?3, book_count = ?4, verse_count = ?5,
             updated_at = ?6, revision = revision + 1
         WHERE id = ?1",
        params![id, kind, status, books, verses, now()],
    )?;
    Ok((books, verses))
}

/// What the recogniser set apart from the verses, kept with its places
/// (v0.24.3), replacing what an earlier reading kept.
fn insert_apparatus(
    tx: &Connection,
    id: &str,
    stage: &DocumentStage,
    kept: &[KeptFile],
) -> AppResult<()> {
    tx.execute(
        "DELETE FROM bible_document_apparatus WHERE translation_id = ?1",
        [id],
    )?;
    let mut add = tx.prepare(
        "INSERT INTO bible_document_apparatus
             (translation_id, document_id, kind, page, segment, verse_id, content)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for piece in &stage.apparatus {
        let Some(document) = kept.get(usize::from(piece.place.document)) else {
            continue;
        };
        add.execute(params![
            id,
            document.id,
            piece.kind,
            piece.place.page,
            piece.place.segment,
            piece.verse.map(|(o, c, v)| verse_key(o, c, v)),
            piece.text
        ])?;
    }
    Ok(())
}

/// The corrections in force in a stage (see `StagedImport::corrections_to_keep`).
fn stage_corrections(stage: &DocumentStage) -> Vec<SavedCorrection> {
    stage
        .books
        .iter()
        .filter(|b| b.corrected)
        .filter_map(|b| {
            let file = stage.files.get(usize::from(b.place.document))?;
            Some(SavedCorrection {
                id: String::new(),
                document_position: b.place.document,
                document_sha256: file.hash.clone(),
                heading: b.heading.clone(),
                page: b.place.page,
                segment: b.place.segment,
                detected_book: b.detected.map(str::to_string),
                chosen_book: if b.left_out {
                    None
                } else {
                    b.book.map(str::to_string)
                },
            })
        })
        .collect()
}

/// Keeps the person's corrections for a Bible (v0.24.3). Each found heading
/// stands for at most one: the ones given replace any kept for the same
/// heading, and a kept correction for a heading found again but no longer
/// corrected (set back to what was found) is removed. Kept corrections that
/// fit no heading of this reading stay: they are the person's, and are
/// shown again the next time.
pub fn save_corrections(
    tx: &Connection,
    id: &str,
    stage: &DocumentStage,
    corrections: &[SavedCorrection],
    now: &str,
) -> AppResult<()> {
    let mut remove = tx.prepare(
        "DELETE FROM bible_recognition_corrections
         WHERE translation_id = ?1 AND document_position = ?2 AND heading = ?3
           AND IFNULL(page, -1) = IFNULL(?4, -1) AND IFNULL(segment, -1) = IFNULL(?5, -1)",
    )?;
    for book in &stage.books {
        remove.execute(params![
            id,
            book.place.document,
            book.heading,
            book.place.page,
            book.place.segment
        ])?;
    }
    // A kept correction applied to a heading found elsewhere now (the
    // document changed) is replaced by the one for its new place.
    for state in &stage.saved {
        if state.index.is_some() && !state.correction.id.is_empty() {
            tx.execute(
                "DELETE FROM bible_recognition_corrections WHERE id = ?1 AND translation_id = ?2",
                params![state.correction.id, id],
            )?;
        }
    }
    let mut add = tx.prepare(
        "INSERT INTO bible_recognition_corrections
             (id, translation_id, document_sha256, document_position, kind, heading, page, segment,
              detected_book, chosen_book, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'book', ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
    )?;
    for c in corrections {
        add.execute(params![
            crate::db::new_id(),
            id,
            c.document_sha256,
            c.document_position,
            c.heading,
            c.page,
            c.segment,
            c.detected_book,
            c.chosen_book,
            now
        ])?;
    }
    Ok(())
}

/// Keeps the corrections in force in a stage for a Bible already added
/// (v0.24.3), as soon as the person makes them: they are the person's work
/// whether or not the verses are used now.
pub fn keep_corrections(conn: &Connection, id: &str, stage: &DocumentStage) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    save_corrections(&tx, id, stage, &stage_corrections(stage), &now())?;
    tx.commit()?;
    Ok(())
}

fn correction_from_row(r: &rusqlite::Row) -> rusqlite::Result<SavedCorrection> {
    Ok(SavedCorrection {
        id: r.get(0)?,
        document_position: r.get(1)?,
        document_sha256: r.get(2)?,
        heading: r.get(3)?,
        page: r.get(4)?,
        segment: r.get(5)?,
        detected_book: r.get(6)?,
        chosen_book: r.get(7)?,
    })
}

const CORRECTION_SELECT: &str =
    "SELECT id, document_position, document_sha256, heading, page, segment,
        detected_book, chosen_book FROM bible_recognition_corrections";

/// The corrections kept for a Bible, oldest first.
pub fn corrections(conn: &Connection, translation_id: &str) -> AppResult<Vec<SavedCorrection>> {
    let mut stmt = conn.prepare(&format!(
        "{CORRECTION_SELECT} WHERE translation_id = ?1 ORDER BY created_at, rowid"
    ))?;
    let rows = stmt.query_map([translation_id], correction_from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Corrections kept for the same documents (by content) when they were
/// added as another Bible (v0.24.3): a document added again keeps the
/// person's work. The most recent Bible that has any, only when all its
/// documents are the ones chosen now, in the same order.
pub fn corrections_for_documents(
    conn: &Connection,
    hashes: &[String],
) -> AppResult<Vec<SavedCorrection>> {
    let mut stmt = conn.prepare(
        "SELECT c.translation_id FROM bible_recognition_corrections c
         GROUP BY c.translation_id ORDER BY MAX(c.updated_at) DESC",
    )?;
    let candidates: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    for translation in candidates {
        let mut docs = conn.prepare(
            "SELECT sha256 FROM bible_documents WHERE translation_id = ?1 ORDER BY position",
        )?;
        let theirs: Vec<String> = docs
            .query_map([&translation], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        if theirs == hashes {
            return corrections(conn, &translation);
        }
    }
    Ok(Vec::new())
}

/// Gives a Bible read as a document the verses found in it later (v0.23):
/// the documents and their text stay as they are. With `partial` (v0.24),
/// when some chapters are doubtful, only the chapters recognised without any
/// doubt are kept, and the Bible stays a document whose other places are not
/// yet identified. Verses found before are replaced by this reading.
pub fn add_verses(
    conn: &Connection,
    id: &str,
    content: &StagedContent,
    stage: &DocumentStage,
    partial: bool,
) -> AppResult<BibleTranslation> {
    let translation = bibles::get(conn, id)?;
    if translation.kind != "document" {
        return Err(AppError::InvalidInput(UserMessage::new(
            "bibles.alreadyVerses",
            "This Bible can already be read by book, chapter and verse.",
        )));
    }
    let sound = !content.has_errors() && content.verse_count() > 0;
    let usable = if sound {
        None
    } else if partial {
        match content.usable_chapters() {
            Some(chapters) if !chapters.is_empty() => Some(chapters),
            _ => return Err(problems()),
        }
    } else {
        return Err(problems());
    };
    // Read by its verses only when every verse found has verified text.
    let whole = sound && stage.unverified.is_empty();
    let kept: Vec<KeptFile> = list(conn, id)?
        .into_iter()
        .map(|d| KeptFile {
            id: d.id,
            file_name: String::new(),
        })
        .collect();
    if kept.len() != stage.files.len() {
        return Err(AppError::Unexpected(
            "document files and records differ".into(),
        ));
    }
    let tx = conn.unchecked_transaction()?;
    // A new reading replaces an earlier one.
    for table in [
        "bible_verse_origins",
        "bible_verse_places",
        "bible_verse_texts",
        "bible_superscriptions",
    ] {
        tx.execute(
            &format!("DELETE FROM {table} WHERE translation_id = ?1"),
            [id],
        )?;
    }
    store_verses(&tx, id, content, stage, &kept, usable.as_ref())?;
    insert_apparatus(&tx, id, stage, &kept)?;
    save_corrections(&tx, id, stage, &stage_corrections(stage), &now())?;
    let (_, verses) = update_counts(&tx, id, whole)?;
    tx.commit()?;
    log::info!(
        "Bible document given verses: {} ({verses} verses{})",
        translation.short_name,
        if whole { "" } else { ", partly" }
    );
    bibles::get(conn, id)
}

fn problems() -> AppError {
    AppError::Import(UserMessage::new(
        "bibles.hasProblems",
        "This Bible has problems that must be resolved before it can be added.",
    ))
}

/// One of a Bible's documents, as the reader sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BibleDocument {
    pub id: String,
    pub position: i64,
    pub format: String,
    /// The file's name as the person had it.
    pub name: String,
    pub page_count: Option<i64>,
    pub extraction_status: String,
    /// Paragraphs of text kept (0 for scanned pages).
    pub segment_count: i64,
    /// How the original is shown: "pdf", "epub" or "docx" (its layout), or
    /// None (Markdown and text are shown as their text).
    pub view: Option<&'static str>,
}

pub fn list(conn: &Connection, translation_id: &str) -> AppResult<Vec<BibleDocument>> {
    let mut stmt = conn.prepare(
        "SELECT d.id, d.position, d.format, d.original_file_name, d.page_count, d.extraction_status,
                (SELECT COUNT(*) FROM bible_document_segments s WHERE s.document_id = d.id)
         FROM bible_documents d WHERE d.translation_id = ?1 ORDER BY d.position",
    )?;
    let rows = stmt.query_map([translation_id], |r| {
        let format: String = r.get(2)?;
        Ok(BibleDocument {
            id: r.get(0)?,
            position: r.get(1)?,
            view: crate::library::original::source_view_for(&format).map(|v| v.kind),
            format,
            name: r.get(3)?,
            page_count: r.get(4)?,
            extraction_status: r.get(5)?,
            segment_count: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Where a document's managed copy is: its Bible, its file name, its format.
pub struct DocumentRecord {
    pub translation_id: String,
    pub file_name: String,
    pub format: String,
    pub segment_count: i64,
}

pub fn record(conn: &Connection, document_id: &str) -> AppResult<DocumentRecord> {
    conn.query_row(
        "SELECT d.translation_id, d.file_name, d.format,
                (SELECT COUNT(*) FROM bible_document_segments s WHERE s.document_id = d.id)
         FROM bible_documents d WHERE d.id = ?1",
        [document_id],
        |r| {
            Ok(DocumentRecord {
                translation_id: r.get(0)?,
                file_name: r.get(1)?,
                format: r.get(2)?,
                segment_count: r.get(3)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::not_found("Bible document", document_id))
}

/// One paragraph of a document's text, with its place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSegment {
    pub position: i64,
    pub text: String,
    pub page_number: Option<i64>,
    pub section_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentWindow {
    pub total: i64,
    pub offset: i64,
    pub segments: Vec<DocumentSegment>,
}

/// Most paragraphs one request returns.
const WINDOW_MAX: i64 = 400;

/// Paragraphs of a document from `offset` (or those of a page, `around_page`).
pub fn segments(
    conn: &Connection,
    document_id: &str,
    offset: i64,
    limit: i64,
    around_page: Option<i64>,
) -> AppResult<DocumentWindow> {
    let record = record(conn, document_id)?;
    let limit = limit.clamp(1, WINDOW_MAX);
    let offset = match around_page {
        Some(page) => conn
            .query_row(
                "SELECT MIN(position) FROM bible_document_segments
                 WHERE document_id = ?1 AND page_number >= ?2",
                params![document_id, page],
                |r| r.get::<_, Option<i64>>(0),
            )?
            .unwrap_or(0),
        None => offset.max(0),
    };
    let mut stmt = conn.prepare(
        "SELECT position, content, page_number, section_path FROM bible_document_segments
         WHERE document_id = ?1 AND position >= ?2 ORDER BY position LIMIT ?3",
    )?;
    let rows = stmt.query_map(params![document_id, offset, limit], |r| {
        Ok(DocumentSegment {
            position: r.get(0)?,
            text: r.get(1)?,
            page_number: r.get(2)?,
            section_path: r.get(3)?,
        })
    })?;
    Ok(DocumentWindow {
        total: record.segment_count,
        offset,
        segments: rows.collect::<Result<_, _>>()?,
    })
}

/// A place in a document Bible's text where the words searched for occur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentHit {
    pub document_id: String,
    pub position: i64,
    pub page_number: Option<i64>,
    pub section_path: Option<String>,
    pub text: String,
}

const SEARCH_LIMIT: i64 = 200;

/// Searches the text of one document Bible.
pub fn search(conn: &Connection, translation_id: &str, query: &str) -> AppResult<Vec<DocumentHit>> {
    let Some(query) = crate::db::fts::query(query) else {
        return Err(AppError::InvalidInput(UserMessage::new(
            "search.empty",
            "Type a word or phrase to search for.",
        )));
    };
    let mut stmt = conn.prepare(
        "SELECT s.document_id, s.position, s.page_number, s.section_path, s.content
         FROM bible_document_search f
         JOIN bible_document_segments s ON s.seq = f.rowid
         JOIN bible_documents d ON d.id = s.document_id
         WHERE bible_document_search MATCH ?1 AND d.translation_id = ?2
         ORDER BY d.position, s.position
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(params![query, translation_id, SEARCH_LIMIT], |r| {
        Ok(DocumentHit {
            document_id: r.get(0)?,
            position: r.get(1)?,
            page_number: r.get(2)?,
            section_path: r.get(3)?,
            text: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Where a verse came from in its Bible's documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerseOrigin {
    pub document_id: String,
    pub document_name: String,
    pub first_segment: Option<i64>,
    pub last_segment: Option<i64>,
    pub first_page: Option<i64>,
    pub last_page: Option<i64>,
    /// "automatic", or "corrected": found under the person's correction of
    /// the book (v0.24.3).
    pub basis: String,
    /// Where in its first paragraph the verse begins (characters).
    pub first_offset: Option<i64>,
}

pub fn origin(
    conn: &Connection,
    translation_id: &str,
    verse_id: i64,
) -> AppResult<Option<VerseOrigin>> {
    Ok(conn
        .query_row(
            "SELECT o.document_id, d.original_file_name, o.first_segment, o.last_segment,
                    o.first_page, o.last_page, o.basis, o.first_offset
             FROM bible_verse_origins o JOIN bible_documents d ON d.id = o.document_id
             WHERE o.translation_id = ?1 AND o.verse_id = ?2",
            params![translation_id, verse_id],
            |r| {
                Ok(VerseOrigin {
                    document_id: r.get(0)?,
                    document_name: r.get(1)?,
                    first_segment: r.get(2)?,
                    last_segment: r.get(3)?,
                    first_page: r.get(4)?,
                    last_page: r.get(5)?,
                    basis: r.get(6)?,
                    first_offset: r.get(7)?,
                })
            },
        )
        .optional()?)
}

/// A verse whose place in a document was found but whose text was not
/// verified (v0.24.3): it can be opened in the document, never read as the
/// Bible's text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersePlace {
    /// Canonical, e.g. "MAT.23.14".
    pub reference: String,
    pub document_id: String,
    pub document_name: String,
    pub first_segment: Option<i64>,
    pub first_page: Option<i64>,
    pub first_offset: Option<i64>,
    pub basis: String,
    /// Why its text was not verified: "possibleHeading", "reference",
    /// "noteSign", "unnumbered".
    pub reason: String,
}

/// The unverified places of a passage, in order.
pub fn places(
    conn: &Connection,
    translation_id: &str,
    start_key: i64,
    end_key: i64,
) -> AppResult<Vec<VersePlace>> {
    let mut stmt = conn.prepare(
        "SELECT p.verse_id, p.document_id, d.original_file_name, p.first_segment, p.first_page,
                p.first_offset, p.basis, p.reason
         FROM bible_verse_places p JOIN bible_documents d ON d.id = p.document_id
         WHERE p.translation_id = ?1 AND p.verse_id BETWEEN ?2 AND ?3
         ORDER BY p.verse_id LIMIT 2000",
    )?;
    let rows = stmt.query_map(params![translation_id, start_key, end_key], |r| {
        let key: i64 = r.get(0)?;
        Ok(VersePlace {
            reference: range_canonical(key, key),
            document_id: r.get(1)?,
            document_name: r.get(2)?,
            first_segment: r.get(3)?,
            first_page: r.get(4)?,
            first_offset: r.get(5)?,
            basis: r.get(6)?,
            reason: r.get(7)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

// --- passages in a document Bible (v0.24) -------------------------------------------

/// A place marked by the person: a passage and where it is in a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassageLocation {
    pub id: String,
    /// Canonical, e.g. "LUK.19.1-LUK.19.10".
    pub reference: String,
    pub document_id: String,
    pub document_name: String,
    pub page: Option<i64>,
    pub segment: Option<i64>,
    pub item: Option<String>,
    pub quote: Option<String>,
    pub created_at: String,
}

/// What is known of a passage in a document Bible: its verses where they
/// were recognised and confirmed (with where they begin in the document),
/// and the places marked by hand. `complete` says every verse of the passage
/// has its text; otherwise the rest has not been identified yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentPassage {
    pub reference: String,
    pub verses: Vec<crate::scripture::VerseText>,
    pub complete: bool,
    /// Where the first verse found begins in the document.
    pub origin: Option<VerseOrigin>,
    pub locations: Vec<PassageLocation>,
    /// Verses of the passage found in the document whose text was not
    /// verified (v0.24.3): their places only.
    pub places: Vec<VersePlace>,
    /// What the recogniser set apart from these verses (notes, headings,
    /// cross-references), with where it stands (v0.24.3).
    pub apparatus: Vec<ApparatusPiece>,
}

/// A piece of a document's apparatus kept beside its verses (v0.24.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApparatusPiece {
    /// "footnote", "crossReference", "heading", "subscription", "notes",
    /// "introduction", "runningHead", "frontMatter", "otherBook".
    pub kind: String,
    pub text: String,
    pub document_id: String,
    pub page: Option<i64>,
    pub segment: Option<i64>,
    /// The verse it stood with (canonical), when known.
    pub reference: Option<String>,
}

/// The apparatus kept with a passage's verses: notes, headings and
/// cross-references, at most 100.
pub fn apparatus(
    conn: &Connection,
    translation_id: &str,
    start_key: i64,
    end_key: i64,
) -> AppResult<Vec<ApparatusPiece>> {
    let mut stmt = conn.prepare(
        "SELECT kind, content, document_id, page, segment, verse_id FROM bible_document_apparatus
         WHERE translation_id = ?1 AND verse_id BETWEEN ?2 AND ?3
           AND kind IN ('footnote', 'crossReference', 'heading', 'subscription', 'notes')
         ORDER BY verse_id, id LIMIT 100",
    )?;
    let rows = stmt.query_map(params![translation_id, start_key, end_key], |r| {
        let verse: Option<i64> = r.get(5)?;
        Ok(ApparatusPiece {
            kind: r.get(0)?,
            text: r.get(1)?,
            document_id: r.get(2)?,
            page: r.get(3)?,
            segment: r.get(4)?,
            reference: verse.map(|k| range_canonical(k, k)),
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

const LOCATION_SELECT: &str =
    "SELECT l.id, l.start_key, l.end_key, l.document_id, d.original_file_name,
        l.page, l.segment, l.item, l.quote, l.created_at
     FROM bible_passage_locations l JOIN bible_documents d ON d.id = l.document_id";

fn location_from_row(r: &rusqlite::Row) -> rusqlite::Result<PassageLocation> {
    let start: i64 = r.get(1)?;
    let end: i64 = r.get(2)?;
    Ok(PassageLocation {
        id: r.get(0)?,
        reference: range_canonical(start, end),
        document_id: r.get(3)?,
        document_name: r.get(4)?,
        page: r.get(5)?,
        segment: r.get(6)?,
        item: r.get(7)?,
        quote: r.get(8)?,
        created_at: r.get(9)?,
    })
}

fn range_canonical(start: i64, end: i64) -> String {
    use crate::scripture::reference::split_key;
    let code = |k: i64| {
        let (o, c, v) = split_key(k);
        let book = crate::scripture::canon::by_ordinal(o).map_or("", |b| b.code);
        format!("{book}.{c}.{v}")
    };
    if start == end {
        code(start)
    } else {
        format!("{}-{}", code(start), code(end))
    }
}

/// A passage in a document Bible: verses found, and places marked.
pub fn locate(
    conn: &Connection,
    translation_id: &str,
    reference: &crate::scripture::reference::ScriptureRef,
) -> AppResult<DocumentPassage> {
    let text = bibles::passage(conn, translation_id, reference)?;
    let complete = covers_every_verse(reference, &text.verses);
    let origin = match text.verses.first() {
        Some(first) => {
            let key = crate::scripture::reference::ScriptureRef::from_canonical(&first.canonical)?
                .start_key();
            origin(conn, translation_id, key)?
        }
        None => None,
    };
    let mut stmt = conn.prepare(&format!(
        "{LOCATION_SELECT} WHERE l.translation_id = ?1 AND l.start_key <= ?3 AND l.end_key >= ?2
         ORDER BY l.start_key, l.created_at"
    ))?;
    let locations = stmt
        .query_map(
            params![translation_id, reference.start_key(), reference.end_key()],
            location_from_row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    let places = places(
        conn,
        translation_id,
        reference.start_key(),
        reference.end_key(),
    )?;
    let apparatus = apparatus(
        conn,
        translation_id,
        reference.start_key(),
        reference.end_key(),
    )?;
    Ok(DocumentPassage {
        reference: reference.canonical(),
        verses: text.verses,
        complete,
        origin,
        locations,
        places,
        apparatus,
    })
}

/// Whether every verse the standard count gives the passage has its text:
/// each one by its own number, a bridged verse ("4-5") counting for each
/// verse it holds. False when the count is unknown.
fn covers_every_verse(
    reference: &crate::scripture::reference::ScriptureRef,
    verses: &[crate::scripture::VerseText],
) -> bool {
    use crate::scripture::canon;
    use crate::scripture::reference::{split_key, verse_key, ScriptureRef};
    let mut found = std::collections::HashSet::new();
    for verse in verses {
        let Ok(r) = ScriptureRef::from_canonical(&verse.canonical) else {
            continue;
        };
        let (start, end) = (r.start_key(), r.end_key());
        // Within one chapter (verse keys are consecutive there).
        if end >= start && end - start < 1000 {
            found.extend(start..=end);
        }
    }
    let (o, c1, v1) = split_key(reference.start_key());
    let (_, c2, v2) = split_key(reference.end_key());
    let Some(book) = canon::by_ordinal(o) else {
        return false;
    };
    let mut any = false;
    for chapter in c1..=c2 {
        let Some(count) = canon::verse_count(book, chapter) else {
            return false;
        };
        let from = if chapter == c1 { v1.max(1) } else { 1 };
        let to = if chapter == c2 { v2.min(count) } else { count };
        for verse in from..=to {
            any = true;
            let key = verse_key(o, chapter, verse);
            if !found.contains(&key) {
                return false;
            }
        }
    }
    any
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddLocation {
    pub translation_id: String,
    /// Canonical, e.g. "LUK.19.1-LUK.19.10".
    pub reference: String,
    pub document_id: String,
    pub page: Option<i64>,
    pub segment: Option<i64>,
    pub item: Option<String>,
    pub quote: Option<String>,
}

/// Marks where a passage is in one of a Bible's documents. Nothing else
/// changes: notes and references keep their canonical verses.
pub fn add_location(conn: &Connection, input: &AddLocation) -> AppResult<PassageLocation> {
    let reference = crate::scripture::reference::ScriptureRef::from_canonical(&input.reference)?;
    let document = record(conn, &input.document_id)?;
    if document.translation_id != input.translation_id {
        return Err(AppError::not_found("document", &input.document_id));
    }
    if input.page.is_none() && input.segment.is_none() && input.item.is_none() {
        return Err(AppError::InvalidInput(UserMessage::new(
            "bibles.locationPlace",
            "Choose the place in the document first.",
        )));
    }
    if let Some(page) = input.page {
        let pages: Option<i64> = conn.query_row(
            "SELECT page_count FROM bible_documents WHERE id = ?1",
            [&input.document_id],
            |r| r.get(0),
        )?;
        // A document without pages (EPUB, Word) has no page to mark.
        if page < 1 || pages.is_none_or(|p| page > p) {
            return Err(AppError::InvalidInput(UserMessage::new(
                "bibles.locationPage",
                "That page is not in the document.",
            )));
        }
    }
    if let Some(segment) = input.segment {
        let known: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM bible_document_segments WHERE document_id = ?1 AND position = ?2)",
            params![input.document_id, segment],
            |r| r.get(0),
        )?;
        if !known {
            return Err(AppError::not_found("paragraph", segment.to_string()));
        }
    }
    let quote = crate::domain::validate::optional_text(input.quote.as_deref(), 500, "selection")?;
    let item = crate::domain::validate::optional_text(input.item.as_deref(), 300, "part")?;
    let id = crate::db::new_id();
    conn.execute(
        "INSERT INTO bible_passage_locations
             (id, translation_id, document_id, start_key, end_key, origin, page, segment, item, quote, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'manual', ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            input.translation_id,
            input.document_id,
            reference.start_key(),
            reference.end_key(),
            input.page,
            input.segment,
            item,
            quote,
            now()
        ],
    )?;
    conn.query_row(
        &format!("{LOCATION_SELECT} WHERE l.id = ?1"),
        [&id],
        location_from_row,
    )
    .map_err(Into::into)
}

pub fn remove_location(conn: &Connection, id: &str) -> AppResult<()> {
    if conn.execute("DELETE FROM bible_passage_locations WHERE id = ?1", [id])? == 0 {
        return Err(AppError::not_found("place", id));
    }
    Ok(())
}

pub fn locations(conn: &Connection, translation_id: &str) -> AppResult<Vec<PassageLocation>> {
    let mut stmt = conn.prepare(&format!(
        "{LOCATION_SELECT} WHERE l.translation_id = ?1 ORDER BY l.start_key, l.created_at LIMIT 5000"
    ))?;
    let rows = stmt.query_map([translation_id], location_from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// What of a document Bible can be found by passage (v0.24).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Coverage {
    /// Verses recognised and confirmed.
    pub verses: i64,
    /// Books with verses, with how many of their chapters have them.
    pub books: Vec<BookCoverage>,
    /// Places marked by hand.
    pub marked: i64,
    /// Verses found whose text was not verified: their places only (v0.24.3).
    pub unverified: i64,
    /// Verses found under a book the person chose (v0.24.3).
    pub corrected: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookCoverage {
    pub book: &'static str,
    pub chapters: i64,
    pub of: i64,
}

pub fn coverage(conn: &Connection, translation_id: &str) -> AppResult<Coverage> {
    let verses: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bible_verse_texts WHERE translation_id = ?1",
        [translation_id],
        |r| r.get(0),
    )?;
    let mut stmt = conn.prepare(
        "SELECT v.book_id, COUNT(DISTINCT v.chapter) FROM bible_verse_texts t
         JOIN bible_verses v ON v.id = t.verse_id
         WHERE t.translation_id = ?1 GROUP BY v.book_id ORDER BY v.book_id",
    )?;
    let books = stmt
        .query_map([translation_id], |r| {
            Ok((r.get::<_, u8>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter_map(|(ordinal, chapters)| {
            let book = crate::scripture::canon::by_ordinal(ordinal)?;
            Some(BookCoverage {
                book: book.code,
                chapters,
                of: i64::from(book.chapter_count),
            })
        })
        .collect();
    let marked: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bible_passage_locations WHERE translation_id = ?1",
        [translation_id],
        |r| r.get(0),
    )?;
    let unverified: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bible_verse_places WHERE translation_id = ?1",
        [translation_id],
        |r| r.get(0),
    )?;
    let corrected: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bible_verse_origins WHERE translation_id = ?1 AND basis = 'corrected'",
        [translation_id],
        |r| r.get(0),
    )?;
    Ok(Coverage {
        verses,
        books,
        marked,
        unverified,
        corrected,
    })
}
