//! The owner's journey with a Bible read from a document (issue #60): an
//! invented EPUB, DOCX and text-layer PDF are each imported, corrected and
//! verified again, kept with their source evidence, reopened after a cold
//! restart, read, searched, cited in a Study, taught and exported — and the
//! one verse the gate withholds never becomes Scripture anywhere. An image
//! only PDF is refused, and the original files are never changed.
//!
//! Every word of Scripture here is invented. The books and verse counts are
//! real, because the recogniser checks chapters against the standard counts.

#[path = "support/library_fixtures.rs"]
mod fixtures;

use std::path::{Path, PathBuf};

use abiding_word_lib::commands::presentations::render;
use abiding_word_lib::db::bible_documents::{self as kept_documents, KeptFile};
use abiding_word_lib::db::bibles::{self, SourceRecord};
use abiding_word_lib::db::{
    annotations, presentations, projects, studies, teaching, workspaces, Database,
};
use abiding_word_lib::domain::{
    AddTeachingPassage, CreatePresentation, CreateProject, CreateStudy, CreateTeaching,
    ProposalLabels, StartingPoint, UpdatePresentation,
};
use abiding_word_lib::error::AppResult;
use abiding_word_lib::filesystem::DataDirs;
use abiding_word_lib::presentation::fonts::FontSet;
use abiding_word_lib::presentation::{document, package, ExportOptions, ImageStore};
use abiding_word_lib::scripture::concordance::{self, ConcordanceMode, ConcordanceQuery};
use abiding_word_lib::scripture::import::{self, ImportOrigin, StagedImport, SuggestedMetadata};
use abiding_word_lib::scripture::reference::ScriptureRef;
use abiding_word_lib::scripture::{
    AnchorEnd, AnchorInput, CommitImport, CreateAnnotation, SearchScripture,
};
use rusqlite::Connection;

/// Only in Ruth 1:1, which every format verifies.
const KEPT: &str = "crocus";
/// Only in Ruth 1:2, whose text the gate withholds ("Selah" inside a verse:
/// the boundary cannot be proven).
const GATED: &str = "hellebore";
const RUTH: [(u16, u16); 4] = [(1, 22), (2, 23), (3, 18), (4, 22)];

fn words(chapter: u16, verse: u16) -> String {
    match (chapter, verse) {
        (1, 1) => "Invented crocus grows beside the river.".into(),
        (1, 2) => "Invented hellebore Selah grows beside the water.".into(),
        _ => format!("Invented words of Ruth c{chapter}v{verse} for testing."),
    }
}

fn epub() -> Vec<u8> {
    let mut bodies = Vec::new();
    for (chapter, count) in RUTH {
        let mut body = String::new();
        if chapter == 1 {
            body.push_str("<h1>Ruth</h1>");
        }
        body.push_str(&format!("<h2>Chapter {chapter}</h2>"));
        for verse in 1..=count {
            let mut text = words(chapter, verse);
            if (chapter, verse) == (1, 1) {
                // A linked note against the verse's words: left out, recorded.
                text = text.replacen(
                    "crocus",
                    r##"crocus<a epub:type="noteref" href="#note-a">[a]</a>"##,
                    1,
                );
            }
            body.push_str(&format!("<p><sup class=\"v\">{verse}</sup>{text}</p>"));
        }
        if chapter == 1 {
            body.push_str(
                r##"<aside epub:type="footnote" id="note-a"><p>An invented footnote.</p></aside>"##,
            );
        }
        bodies.push(body);
    }
    let names = ["c1.xhtml", "c2.xhtml", "c3.xhtml", "c4.xhtml"];
    let chapters: Vec<_> = names
        .iter()
        .zip(&bodies)
        .map(|(file, body)| fixtures::EpubChapter {
            file,
            xhtml_body: body,
        })
        .collect();
    fixtures::epub("<dc:language>en</dc:language>", &chapters, &[0, 1, 2, 3])
}

fn docx() -> Vec<u8> {
    let heading = |level: u8, text: &str| {
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading{level}"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#
        )
    };
    let mut body = heading(1, "Ruth");
    for (chapter, count) in RUTH {
        body.push_str(&heading(2, &format!("Chapter {chapter}")));
        for verse in 1..=count {
            body.push_str(&format!(
                r#"<w:p><w:r><w:t xml:space="preserve">{verse} {}</w:t></w:r></w:p>"#,
                words(chapter, verse)
            ));
        }
    }
    let styles = br#"<?xml version="1.0"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/></w:style></w:styles>"#;
    let rels = r#"<Relationship Id="rS" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#;
    fixtures::docx(&body, rels, &[("word/styles.xml", styles)])
}

fn pdf() -> Vec<u8> {
    let mut lines = vec!["Ruth".to_string()];
    for (chapter, count) in RUTH {
        lines.push(format!("Chapter {chapter}"));
        for verse in 1..=count {
            lines.push(format!("{verse} {}", words(chapter, verse)));
        }
    }
    let pages: Vec<Vec<String>> = lines
        .chunks(9)
        .enumerate()
        .map(|(n, page)| {
            let mut page = page.to_vec();
            page.insert(0, "THE BOOK OF RUTH".into());
            page.push(format!("{}", n + 1));
            page
        })
        .collect();
    let refs: Vec<Vec<&str>> = pages
        .iter()
        .map(|p| p.iter().map(String::as_str).collect())
        .collect();
    let refs: Vec<&[&str]> = refs.iter().map(Vec::as_slice).collect();
    fixtures::pdf(&refs, Some("Ruth (invented test text)"), None)
}

/// As the app reads a document Bible: PDFs in the separate, limited worker
/// process (the app's own executable), the other formats in process.
fn stage(path: &Path) -> AppResult<StagedImport> {
    let paths = [path.to_path_buf()];
    if path.extension().is_some_and(|e| e == "pdf") {
        import::stage_document_recognition_read(
            &paths,
            ImportOrigin::UserFiles,
            &|_, _| {},
            &|| false,
            &|path, position| {
                abiding_word_lib::scripture::pdf_worker::read_with_executable(
                    Path::new(env!("CARGO_BIN_EXE_abiding-word")),
                    path,
                    position,
                    &|| false,
                )
            },
        )
    } else {
        import::stage(&paths, ImportOrigin::UserFiles)
    }
}

/// The app's commit of a previewed document Bible (`commit_documents`):
/// copy each reviewed file into the profile, check it is the reviewed file,
/// then save the Bible, its documents, verses and evidence in one go.
fn commit(
    conn: &Connection,
    profile: &DataDirs,
    staged: &StagedImport,
    short: &str,
    confirm_edition: bool,
) -> AppResult<String> {
    let id = bibles::new_translation_id();
    let stage = staged.documents.as_ref().unwrap();
    let dir = profile.bible_sources(&id);
    std::fs::create_dir_all(&dir).unwrap();
    let mut kept = Vec::new();
    for (index, file) in stage.files.iter().enumerate() {
        let name = format!("{}.{}", index + 1, file.extension);
        let target = dir.join(&name);
        std::fs::copy(&file.path, &target).unwrap();
        file.verify_source(&target)?;
        kept.push(KeptFile {
            id: abiding_word_lib::db::new_id(),
            file_name: name,
        });
    }
    kept_documents::commit(
        conn,
        &id,
        &staged.content,
        stage,
        &CommitImport {
            preview_id: staged.id.clone(),
            name: format!("Invented {short}"),
            short_name: short.into(),
            language: "English".into(),
            language_code: Some("en".into()),
            publisher: None,
            copyright_notice: None,
            license_type: None,
            words: Default::default(),
            as_document: false,
            confirm_edition,
        },
        &SourceRecord {
            origin: "managed_copy",
            path: Some(format!("sources/bibles/{id}")),
            original_file_name: Some(stage.files[0].file_name.clone()),
            format: staged.format,
            is_builtin: false,
        },
        &kept,
        true,
    )
    .map(|t| t.id)
}

fn count(conn: &Connection, sql: &str, id: &str) -> i64 {
    conn.query_row(sql, [id], |r| r.get(0)).unwrap()
}

fn store(dir: &Path) -> ImageStore {
    ImageStore {
        generated_dir: dir.join("teaching-images"),
        offline_dir: dir.join("presentation-images"),
        template_dir: dir.join("presentation-templates"),
    }
}

struct Journey {
    format: &'static str,
    file_name: &'static str,
    bytes: Vec<u8>,
    confirm: bool,
}

/// The whole journey for one document; returns what it observed, for the
/// test's record.
fn journey(j: &Journey) -> serde_json::Value {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join(j.file_name);
    std::fs::write(&original, &j.bytes).unwrap();

    // 1. Preview: verse 1:1 verified, 1:2 withheld with its reason.
    let mut staged = stage(&original).unwrap();
    let initial = staged.documents.as_ref().unwrap();
    assert!(
        initial.verified(None, (8, 1, 1)),
        "{}: 1:1 verified",
        j.format
    );
    let reason = *initial
        .unverified
        .get(&(8, 1, 2))
        .unwrap_or_else(|| panic!("{}: 1:2 withheld: {:?}", j.format, initial.unverified));
    let generation = initial.recognition.as_ref().map(|r| r.generation);
    let preview = staged.preview(SuggestedMetadata::default());
    let preview_json = serde_json::to_string(&preview).unwrap();

    // 2. A correction of the book heading: everything is recognised and
    //    gated again; the withheld verse stays withheld.
    let heading = initial
        .books
        .iter()
        .find(|b| b.book == Some("RUT"))
        .unwrap()
        .index;
    staged.correct(&[(heading, Some("RUT".into()))]).unwrap();
    let corrected = staged.documents.as_ref().unwrap();
    assert_eq!(
        corrected.recognition.as_ref().map(|r| r.generation),
        generation.map(|g| g + 1),
        "{}: a correction is a new gated pass",
        j.format
    );
    assert!(
        corrected.unverified.contains_key(&(8, 1, 2)),
        "{}",
        j.format
    );
    assert!(!corrected.verified(None, (8, 1, 2)));
    assert!(corrected.verified(None, (8, 1, 1)));

    // 3. Commit into a real profile, then a cold restart.
    let profile = DataDirs::ensure(temp.path().join("profile")).unwrap();
    let id = {
        let db =
            Database::open(&profile.database_file(), &profile.pre_migration_backups()).unwrap();
        db.with_conn(|conn| commit(conn, &profile, &staged, j.format, j.confirm))
            .unwrap()
    };
    drop(staged);
    let db = Database::open(&profile.database_file(), &profile.pre_migration_backups()).unwrap();
    let observed = db
        .with_conn(|conn| {
            let bible = bibles::get(conn, &id)?;
            assert_eq!(bible.source_type, j.format);
            assert_eq!(bible.edition_confirmed, Some(j.confirm), "{}", j.format);

            // Saved provenance: a run, the gate's evidence for every verse,
            // the source hash of the kept copy, the reason for 1:2.
            let source_sha: String = conn.query_row(
                "SELECT sha256 FROM bible_documents WHERE translation_id = ?1",
                [&id],
                |r| r.get(0),
            )?;
            let kept_file = profile.bible_sources(&id).join(format!("1.{}", j.format));
            assert_eq!(std::fs::read(&kept_file).unwrap(), j.bytes);
            assert_eq!(
                source_sha,
                abiding_word_lib::library::files::content_hash(&j.bytes)
            );
            let verified = count(
                conn,
                "SELECT COUNT(*) FROM bible_recognition_verse_evidence
                 WHERE translation_id = ?1 AND recognition_state = 'verifiedFromDocument'",
                &id,
            );
            let withheld: (String, String) = conn.query_row(
                "SELECT recognition_state, reason FROM bible_recognition_verse_evidence
                 WHERE translation_id = ?1 AND verse_id = 8001002",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            assert_eq!(withheld.0, "unverified");
            assert_eq!(withheld.1, reason);
            assert_eq!(
                count(
                    conn,
                    "SELECT COUNT(*) FROM bible_verse_texts WHERE translation_id = ?1 AND verse_id = 8001002",
                    &id
                ),
                0,
                "{}: the withheld verse's text is not stored as Scripture",
                j.format
            );
            let runs = count(
                conn,
                "SELECT COUNT(*) FROM bible_recognition_runs WHERE translation_id = ?1",
                &id,
            );
            assert_eq!(runs, 1);
            let recogniser_commit: String = conn.query_row(
                "SELECT recogniser_commit FROM bible_recognition_runs WHERE translation_id = ?1",
                [&id],
                |r| r.get(0),
            )?;
            assert_eq!(
                recogniser_commit,
                kept_documents::PRODUCT_RECOGNISER_COMMIT
            );
            assert_eq!(recogniser_commit, "fd2b1bec285b5122362c3cf95fc42c1ae6d3a486");
            assert_ne!(recogniser_commit, "08c6faac8c92c6c791ec1e7bbbe997362fa1059f");
            let origin = kept_documents::origin(conn, &id, 8_001_001)?.unwrap();
            assert_eq!(origin.source_sha256.as_deref(), Some(source_sha.as_str()));
            if j.format == "epub" {
                assert!(origin.entry_path.as_deref().unwrap().ends_with("c1.xhtml"));
                assert_eq!(origin.exclusions.as_ref().unwrap()[0]["text"], "[a]");
                assert!(origin.provenance.as_ref().unwrap()["spans"]
                    .as_array()
                    .is_some_and(|s| !s.is_empty()));
            } else {
                assert!(origin.entry_path.is_none());
            }
            if j.format == "pdf" {
                assert!(origin.first_page.is_some());
            }
            // The withheld verse is still inspectable at its source.
            let withheld_origin = kept_documents::origin(conn, &id, 8_001_002)?.unwrap();
            assert_eq!(withheld_origin.document_id, origin.document_id);
            let at_source = kept_documents::search(conn, &id, GATED)?;
            assert!(!at_source.is_empty(), "{}: findable in the document", j.format);

            // Reading by book and chapter.
            let chapter = bibles::chapter(conn, &id, "RUT", 1)?;
            assert_eq!(chapter.verses[0].text, words(1, 1));
            assert!(!chapter.verses.iter().any(|v| v.verse == 2));
            assert_eq!(chapter.unchecked[0].verse, 2);
            assert_eq!(chapter.verses.len(), 21);
            assert_eq!(bibles::chapter(conn, &id, "RUT", 4)?.verses.len(), 22);

            // Scripture search and the concordance: 1:1 found, 1:2 never.
            let search = |q: &str| {
                bibles::search(
                    conn,
                    &SearchScripture {
                        query: q.into(),
                        translation_id: Some(id.clone()),
                        book: None,
                    },
                )
            };
            assert!(!search(KEPT)?.hits.is_empty(), "{}", j.format);
            assert!(search(GATED)?.hits.is_empty(), "{}", j.format);
            let concord = |q: &str| {
                concordance::search(
                    conn,
                    &ConcordanceQuery {
                        translation_id: id.clone(),
                        query: q.into(),
                        mode: ConcordanceMode::Word,
                        book: None,
                    },
                )
            };
            assert!(concord(KEPT)?.verses > 0);
            assert_eq!(concord(GATED)?.verses, 0);

            // A passage (what Study, Teaching and the assistant quote).
            let passage = bibles::passage(conn, &id, &ScriptureRef::from_canonical("RUT.1.1-RUT.1.3")?)?;
            let quoted: Vec<u16> = passage.verses.iter().map(|v| v.verse).collect();
            assert_eq!(quoted, vec![1, 3]);
            assert!(!serde_json::to_string(&passage).unwrap().contains(GATED));

            // A Study, with a note anchored to words of the verified verse;
            // the withheld verse cannot be quoted.
            let workspace = workspaces::get_default(conn)?;
            let project = projects::create(
                conn,
                &CreateProject {
                    workspace_id: workspace.id,
                    title: "Ruth".into(),
                    description: None,
                },
            )?;
            let study = studies::create(
                conn,
                &CreateStudy {
                    project_id: project.id,
                    title: "Ruth 1".into(),
                    primary_passage: Some("RUT.1.1-RUT.1.3".into()),
                    ..Default::default()
                },
            )?
            .study
            .id;
            let note = annotations::create(
                conn,
                &CreateAnnotation {
                    reference: "RUT.1.1".into(),
                    kind: "note".into(),
                    content: Some("The crocus is the first sign.".into()),
                    translation_id: Some(id.clone()),
                    color: None,
                    anchor: Some(AnchorInput {
                        translation_id: id.clone(),
                        start: AnchorEnd {
                            verse: "RUT.1.1".into(),
                            offset: 9,
                        },
                        end: AnchorEnd {
                            verse: "RUT.1.1".into(),
                            offset: 15,
                        },
                        quote: KEPT.into(),
                    }),
                },
            )?;
            assert!(note.anchor.as_ref().is_some_and(|a| a.shown));
            let refused = annotations::create(
                conn,
                &CreateAnnotation {
                    reference: "RUT.1.2".into(),
                    kind: "note".into(),
                    content: Some("Not Scripture.".into()),
                    translation_id: Some(id.clone()),
                    color: None,
                    anchor: Some(AnchorInput {
                        translation_id: id.clone(),
                        start: AnchorEnd {
                            verse: "RUT.1.2".into(),
                            offset: 9,
                        },
                        end: AnchorEnd {
                            verse: "RUT.1.2".into(),
                            offset: 18,
                        },
                        quote: GATED.into(),
                    }),
                },
            );
            assert!(refused.is_err(), "{}: no anchor in withheld text", j.format);

            // Teaching from the Study, with the passage, presented from this
            // Bible and exported: the verified verses, never the withheld one.
            let prep = teaching::create(
                conn,
                &CreateTeaching {
                    project_id: None,
                    study_id: study,
                    title: "The first sign".into(),
                    kind: Some("sermon".into()),
                    points: vec![StartingPoint {
                        title: "Beside the river".into(),
                        kind: None,
                    }],
                    template_key: None,
                    template_version: None,
                    blocks: Vec::new(),
                },
            )?;
            teaching::add_passage(
                conn,
                &AddTeachingPassage {
                    preparation_id: prep.id.clone(),
                    point_id: None,
                    reference: "RUT.1.1-RUT.1.3".into(),
                    block_id: None,
                },
            )?;
            let presentation = presentations::create(
                conn,
                &CreatePresentation {
                    preparation_id: prep.id.clone(),
                    title: None,
                    propose: true,
                    labels: ProposalLabels {
                        introduction: "Introduction".into(),
                        conclusion: "Conclusion".into(),
                    },
                },
            )?;
            presentations::update(
                conn,
                &UpdatePresentation {
                    id: presentation.id.clone(),
                    translation_id: Some(id.clone()),
                    ..Default::default()
                },
            )?;
            let images = store(temp.path());
            let doc = document::build(conn, &presentation.id, &images)?;
            assert_eq!(doc.bible.as_ref().map(|b| b.id.as_str()), Some(id.as_str()));
            // The person preparing it is told that a verse is left out.
            assert!(
                doc.warnings.iter().any(|w| w.code == "scripturePartial"),
                "{}: {:?}",
                j.format,
                doc.warnings
            );
            let slides = serde_json::to_string(&doc).unwrap();
            assert!(slides.contains(KEPT), "{}", j.format);
            assert!(!slides.contains(GATED), "{}: presentation", j.format);
            let snapshot = package::build(conn, &presentation.id, &images)?;
            let mut exports = serde_json::Map::new();
            for format in ["markdown", "pptx", "pdf"] {
                let bytes = render(
                    &snapshot,
                    format,
                    &ExportOptions {
                        speaker_notes: true,
                        citations: true,
                        ..Default::default()
                    },
                    FontSet::system,
                )
                .unwrap();
                let text = exported_text(format, &bytes);
                assert!(!text.contains(GATED), "{}: {format} export", j.format);
                if format != "pdf" {
                    assert!(text.contains(KEPT), "{}: {format} export", j.format);
                }
                exports.insert(format.into(), bytes.len().into());
                if format == "markdown" {
                    exports.insert("markdown_text".into(), text.clone().into());
                }
            }

            // Materials (participant hand-outs) take their Scripture here.
            let materials = bibles::material_passages(
                conn,
                &id,
                &[ScriptureRef::from_canonical("RUT.1.1-RUT.1.3")?],
            )?;
            let materials_json = serde_json::to_string(&materials).unwrap();
            assert!(materials_json.contains(KEPT));
            assert!(!materials_json.contains(GATED), "{}: materials", j.format);

            Ok(serde_json::json!({
                "format": j.format,
                "file": j.file_name,
                "withheld_reason": reason,
                "verified_evidence_rows": verified,
                "edition_confirmed": bible.edition_confirmed,
                "preview_mentions_withheld_text": preview_json.contains(GATED),
                "exports_bytes": exports,
            }))
        })
        .unwrap();
    // The original file, untouched.
    assert_eq!(std::fs::read(&original).unwrap(), j.bytes);
    observed
}

/// The text a person sees in an export: Markdown as is; the words of every
/// slide of a PowerPoint file; the raw bytes of a PDF (fonts are embedded,
/// so only the absence of a word can be checked there).
fn exported_text(format: &str, bytes: &[u8]) -> String {
    use std::io::Read as _;
    match format {
        "markdown" => String::from_utf8(bytes.to_vec()).unwrap(),
        "pptx" => {
            let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
            let mut all = String::new();
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i).unwrap();
                if entry.name().ends_with(".xml") {
                    let mut s = String::new();
                    entry.read_to_string(&mut s).unwrap();
                    all.push_str(&s);
                }
            }
            all
        }
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

#[test]
fn epub_docx_and_pdf_bibles_are_imported_cited_taught_and_exported_without_withheld_text() {
    let journeys = [
        Journey {
            format: "epub",
            file_name: "룻기-джерело.epub",
            bytes: epub(),
            confirm: true,
        },
        Journey {
            format: "docx",
            file_name: "ruth invented.docx",
            bytes: docx(),
            confirm: false,
        },
        Journey {
            format: "pdf",
            file_name: "ruth-invented.pdf",
            bytes: pdf(),
            confirm: false,
        },
    ];
    let observed: Vec<_> = journeys.iter().map(journey).collect();
    if let Some(out) = std::env::var_os("AW_JOURNEY_OUT") {
        std::fs::write(
            PathBuf::from(out),
            serde_json::to_string_pretty(&observed).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn an_image_only_pdf_is_refused_as_needing_ocr_and_left_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let bytes = fixtures::pdf(&[&[], &[], &[]], Some("Scanned"), None);
    let path = temp.path().join("scan.pdf");
    std::fs::write(&path, &bytes).unwrap();
    let result = stage(&path);
    match &result {
        Ok(staged) => {
            // Read, but nothing in it can be Scripture.
            let stage = staged.documents.as_ref().unwrap();
            assert_eq!(stage.verified_count(&staged.content, None), 0);
            assert!(stage
                .files
                .iter()
                .all(|f| f.extraction_status.as_str() == "ocr_required"));
        }
        Err(error) => {
            let text = format!("{error:?}");
            assert!(
                text.contains("ocr") || text.contains("Ocr") || text.contains("scan"),
                "{text}"
            );
        }
    }
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

/// Every row of the Bible tables and the person's notes, in order.
fn rows(conn: &Connection, tables: &[String]) -> Vec<Vec<Vec<rusqlite::types::Value>>> {
    tables
        .iter()
        .map(|table| {
            let mut s = conn
                .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY 1, 2"))
                .unwrap();
            let n = s.column_count();
            s.query_map([], |r| {
                (0..n)
                    .map(|i| r.get(i))
                    .collect::<Result<Vec<rusqlite::types::Value>, _>>()
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
        })
        .collect()
}

/// A profile from before schema 28 with a document Bible (imported as the
/// latest code before 0028 stored one), a note on it and a USFM Bible:
/// opening it upgrades to 28 after a verified safety copy, changes no row,
/// keeps the document Bible's words out of Scripture until it is read again,
/// and reading it again restores them with evidence. Daily snapshots carry
/// the evidence, and a restore brings it back.
#[test]
fn schema_27_profile_upgrades_keeps_every_row_and_backups_carry_the_evidence() {
    use abiding_word_lib::db::backup;
    use abiding_word_lib::db::migrations::{self, MIGRATIONS};
    use abiding_word_lib::portability::snapshots;

    let temp = tempfile::tempdir().unwrap();
    let bytes = epub();
    let original = temp.path().join("old-ruth.epub");
    std::fs::write(&original, &bytes).unwrap();

    // The document Bible as the app stores it, made in a scratch profile.
    let scratch = DataDirs::ensure(temp.path().join("scratch")).unwrap();
    let id = {
        let db =
            Database::open(&scratch.database_file(), &scratch.pre_migration_backups()).unwrap();
        let staged = stage(&original).unwrap();
        db.with_conn(|conn| commit(conn, &scratch, &staged, "OLD", false))
            .unwrap()
    };

    // The person's profile at schema 27, holding that Bible's rows.
    let profile = DataDirs::ensure(temp.path().join("profile")).unwrap();
    let tables: Vec<String>;
    let before;
    {
        let mut c = Connection::open(profile.database_file()).unwrap();
        c.pragma_update(None, "foreign_keys", true).unwrap();
        migrations::run_with(&mut c, &MIGRATIONS[..27]).unwrap();
        c.execute(
            "ATTACH DATABASE ?1 AS scratch",
            [scratch.database_file().to_string_lossy().as_ref()],
        )
        .unwrap();
        tables = c
            .prepare(
                "SELECT name FROM main.sqlite_master WHERE type = 'table' AND name LIKE 'bible_%'
                 AND sql NOT LIKE 'CREATE VIRTUAL TABLE%'
                 AND name NOT LIKE '%\\_data' ESCAPE '\\' AND name NOT LIKE '%\\_idx' ESCAPE '\\'
                 AND name NOT LIKE '%\\_content' ESCAPE '\\' AND name NOT LIKE '%\\_docsize' ESCAPE '\\'
                 AND name NOT LIKE '%\\_config' ESCAPE '\\'
                 ORDER BY name",
            )
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        // Seeding only: the rows are copied table by table, so their order
        // is checked as a whole afterwards rather than row by row.
        c.pragma_update(None, "foreign_keys", false).unwrap();
        for table in &tables {
            c.execute_batch(&format!(
                "INSERT OR IGNORE INTO main.\"{table}\" SELECT * FROM scratch.\"{table}\";"
            ))
            .unwrap();
        }
        c.pragma_update(None, "foreign_keys", true).unwrap();
        assert!(!c
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap());
        c.execute_batch(&format!(
            "DETACH DATABASE scratch;
             INSERT INTO workspaces (id, name, is_default, created_at, updated_at)
                 VALUES ('w', 'Invented', 1, 'T', 'T');
             INSERT INTO annotations (id, workspace_id, passage, start_key, end_key, translation_id,
                                      kind, content, created_at, updated_at)
                 VALUES ('note-before-28', 'w', 'RUT.1.1', 8001001, 8001001, '{id}', 'note',
                         'Written before the upgrade.', 'T', 'T');"
        ))
        .unwrap();
        assert!(
            c.query_row(
                "SELECT COUNT(*) FROM bible_verse_texts WHERE translation_id = ?1",
                [&id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap()
                > 80
        );
        let mut all = tables.clone();
        all.push("annotations".into());
        before = rows(&c, &all);
        assert_eq!(migrations::current_version(&c).unwrap(), 27);
    }
    let kept = profile.bible_sources(&id);
    std::fs::create_dir_all(&kept).unwrap();
    std::fs::copy(&original, kept.join("1.epub")).unwrap();

    // Opening the profile upgrades it.
    let db = Database::open(&profile.database_file(), &profile.pre_migration_backups()).unwrap();
    let mut all = tables.clone();
    all.push("annotations".into());
    db.with_conn(|conn| {
        assert_eq!(migrations::current_version(conn)?, 28);
        assert_eq!(rows(conn, &all), before, "no row changed by the upgrade");
        assert!(!conn.prepare("PRAGMA foreign_key_check")?.exists([])?);
        let bible = bibles::get(conn, &id)?;
        assert_eq!(bible.kind, "document");
        assert_eq!(bible.edition_confirmed, Some(false));
        // Not Scripture until read again with evidence; the source stays.
        assert!(bibles::chapter(conn, &id, "RUT", 1)?.verses.is_empty());
        assert!(!kept_documents::search(conn, &id, KEPT)?.is_empty());
        let notes = annotations::list(conn, &ScriptureRef::from_canonical("RUT.1.1")?)?;
        assert!(notes.iter().any(|n| n.id == "note-before-28"));
        Ok(())
    })
    .unwrap();
    // The safety copy is the old database, exactly.
    let copies: Vec<PathBuf> = std::fs::read_dir(profile.pre_migration_backups())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(copies.len(), 1);
    let old = Connection::open_with_flags(&copies[0], rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .unwrap();
    assert_eq!(migrations::current_version(&old).unwrap(), 27);
    assert_eq!(rows(&old, &all), before);
    drop(old);

    // Read again from its kept file: verses with evidence, the withheld one
    // a place only, the note still there.
    db.with_conn(|conn| {
        let path = kept.join("1.epub");
        let mut staged = import::stage_document_recognition(
            &[path],
            ImportOrigin::Existing {
                translation_id: id.clone(),
            },
            &|_, _| {},
            &|| false,
        )?;
        staged.apply_saved(kept_documents::corrections(conn, &id)?, false)?;
        kept_documents::add_verses_with_confirmation(
            conn,
            &id,
            &staged.content,
            staged.documents.as_ref().unwrap(),
            false,
            true,
        )?;
        let chapter = bibles::chapter(conn, &id, "RUT", 1)?;
        assert_eq!(chapter.verses[0].text, words(1, 1));
        assert!(!chapter.verses.iter().any(|v| v.verse == 2));
        assert_eq!(bibles::get(conn, &id)?.edition_confirmed, Some(true));
        let notes = annotations::list(conn, &ScriptureRef::from_canonical("RUT.1.1")?)?;
        assert!(notes.iter().any(|n| n.id == "note-before-28"));
        Ok(())
    })
    .unwrap();

    // A daily snapshot carries the evidence; a restore brings it back after
    // the Bible was removed.
    let evidence = |conn: &Connection| -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM bible_recognition_verse_evidence WHERE translation_id = ?1",
            [&id],
            |r| r.get(0),
        )
        .unwrap()
    };
    let live = db.with_conn(|conn| Ok(evidence(conn))).unwrap();
    assert!(live > 80);
    let snapshot = db
        .with_conn(|conn| backup::daily_snapshot(conn, &profile.daily_backups()))
        .unwrap()
        .unwrap();
    {
        let copy =
            Connection::open_with_flags(&snapshot, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        assert_eq!(evidence(&copy), live);
    }
    db.with_conn(|conn| bibles::remove(conn, &id).map(|_| ()))
        .unwrap();
    assert_eq!(db.with_conn(|conn| Ok(evidence(conn))).unwrap(), 0);
    let listed = snapshots::list(&db, &profile).unwrap();
    let chosen = listed
        .iter()
        .find(|s| s.usable && s.schema_version == 28)
        .unwrap();
    snapshots::restore(&db, &profile, &chosen.id).unwrap();
    db.with_conn(|conn| {
        assert_eq!(evidence(conn), live);
        assert_eq!(
            bibles::chapter(conn, &id, "RUT", 1)?.verses[0].text,
            words(1, 1)
        );
        Ok(())
    })
    .unwrap();
}
