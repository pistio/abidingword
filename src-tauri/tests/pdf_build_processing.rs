//! #35: invented owner-shaped input, never the owner's copyrighted PDF.
#[path = "support/library_fixtures.rs"]
mod fixtures;
use abiding_word_lib::{
    db::{library, pdf_structures, workspaces, Database},
    error::{AppError, AppResult, UserMessage},
    library::{
        files::content_hash,
        pdf_build::{self, Job, Jobs},
    },
};
use rusqlite::{params, Connection};
use std::{
    fs,
    time::{Duration, Instant},
};

type Lines = Vec<(f64, f64, f64, String)>;
struct Fixture {
    db: Database,
    dir: tempfile::TempDir,
    bytes: Vec<u8>,
}
fn fixture(pages: usize) -> Fixture {
    let db = Database::open_in_memory().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut layouts: Vec<Lines> = Vec::new();
    let mut segments = Vec::new();
    for page in 0..pages {
        let mut lines = Vec::new();
        if page >= 2 {
            let count = if page < 69 { 6 } else { 7 };
            for part in 0..count {
                let mut paragraph = Vec::new();
                for line in 0..3 {
                    let title = part == 0 && line == 0 && (page - 2) % 40 == 0;
                    let text = if title {
                        ["Johannes", "Apostelgeschichte", "Römer"][((page - 2) / 40) % 3]
                            .to_string()
                    } else {
                        format!("Erfundene Untersuchung {page}-{part}-{line}: Apg 6,1 und Röm 8,1 im Text.")
                    };
                    lines.push((
                        60.0,
                        700.0 - part as f64 * 85.0 - line as f64 * 15.0,
                        if title { 24.0 } else { 11.0 },
                        text.clone(),
                    ));
                    paragraph.push(text);
                }
                segments.push((page + 1, paragraph.join(" ")));
            }
        }
        layouts.push(lines);
    }
    let bytes = fixtures::pdf_sized(&layouts);
    let mut doc = lopdf::Document::load_mem(&bytes).unwrap();
    // Legal unused stream approximates the owner's file-size/hash workload.
    doc.add_object(lopdf::Stream::new(
        lopdf::Dictionary::new(),
        vec![b' '; 2_191_290usize.saturating_sub(bytes.len())],
    ));
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    fs::create_dir_all(dir.path().join("book/source")).unwrap();
    fs::write(dir.path().join("book/source/input.pdf"), &bytes).unwrap();
    db.with_conn(|conn| {
        let workspace = workspaces::get_default(conn)?;
        conn.execute("INSERT INTO sources(id,workspace_id,source_type,title,language,content_hash,created_at,updated_at) VALUES('source',?1,'commentary','Invented handbook','Deutsch',?2,'T','T')", params![workspace.id,content_hash(&bytes)])?;
        conn.execute("INSERT INTO source_files(id,source_id,relative_path,format,sha256,size_bytes,page_count,created_at,updated_at) VALUES('file','source','book/source/input.pdf','pdf',?1,?2,?3,'T','T')", params![content_hash(&bytes),bytes.len(),pages])?;
        for (i,(page,text)) in segments.iter().enumerate() {
            conn.execute("INSERT INTO source_segments(id,source_id,source_file_id,position,page_number,content,created_at,updated_at) VALUES(?1,'source','file',?2,?3,?4,'T','T')", params![format!("segment-{i}"),i,page,text])?;
        }
        let reference = abiding_word_lib::scripture::reference::ScriptureRef::from_canonical("ACT.6.1")?;
        let count = if pages == 400 {4112} else {20};
        for i in 0..count {
            conn.execute("INSERT INTO source_passages(id,source_id,segment_id,passage,start_key,end_key,matched_text,origin,created_at) VALUES(?1,'source',?2,'ACT.6.1',?3,?4,'Apg 6,1','explicit','BEFORE')",params![format!("literal-{i}"),format!("segment-{}",i%segments.len()),reference.start_key(),reference.end_key()])?;
        }
        Ok(())
    }).unwrap();
    Fixture { db, dir, bytes }
}
fn fingerprint(conn: &Connection) -> AppResult<String> {
    let mut data = String::new();
    for table in [
        "sources",
        "source_files",
        "source_segments",
        "source_pdf_structures",
        "source_passages",
    ] {
        let mut stmt = conn.prepare(&format!(
            "SELECT * FROM {table} ORDER BY {}",
            if table == "source_pdf_structures" {
                "source_id"
            } else {
                "id"
            }
        ))?;
        let count = stmt.column_count();
        for row in stmt.query_map([], |r| {
            (0..count)
                .map(|i| r.get::<_, rusqlite::types::Value>(i))
                .collect::<Result<Vec<_>, _>>()
        })? {
            data.push_str(&format!("{table}{:?}", row?));
        }
    }
    Ok(content_hash(data.as_bytes()))
}
#[test]
fn four_hundred_pages_finish_with_measured_stages_and_keep_literal_provenance() {
    let f = fixture(400);
    f.db.with_conn(|conn| {
        let (segments, pages): (usize, usize) = conn.query_row(
            "SELECT COUNT(*),COUNT(DISTINCT page_number) FROM source_segments",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert_eq!((segments, pages), (2719, 398));
        Ok(())
    })
    .unwrap();
    let job = Job::default();
    let start = Instant::now();
    let result = pdf_build::rebuild(&f.db, f.dir.path(), "source", &job);
    job.finish(&result);
    let view = result.unwrap();
    assert_eq!(view.status, "ready");
    assert!(!view.headings.is_empty());
    assert!(
        start.elapsed() < Duration::from_secs(60),
        "400-page rebuild exceeded 60s: {:?}",
        start.elapsed()
    );
    let progress = job.snapshot();
    // A coarse regression ceiling for this entire isolated Linux test process,
    // including concurrent fixture tests; not a Windows memory acceptance claim.
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        let kib = status
            .lines()
            .find(|l| l.starts_with("VmHWM:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap();
        eprintln!("owner-shaped process peak_rss_kib={kib}");
        assert!(
            kib < 768 * 1024,
            "processing fixture exceeded 768 MiB peak RSS"
        );
    }
    assert_eq!(progress.status, "succeeded");
    for stage in [
        "file",
        "segments",
        "pdf",
        "native",
        "headings",
        "alignment",
        "store",
        "index",
        "commit",
    ] {
        assert!(
            progress.timings.iter().any(|t| t.stage == stage),
            "missing {stage}"
        );
    }
    f.db.with_conn(|conn| {
        let literals: usize = conn.query_row("SELECT COUNT(*) FROM source_passages WHERE origin='explicit' AND id LIKE 'literal-%' AND created_at='BEFORE'",[],|r|r.get(0))?;
        assert_eq!(literals,4112,"all literal reference IDs and timestamps survive");
        assert!(!pdf_structures::contents(conn,"source")?.is_empty());
        Ok(())
    }).unwrap();
    assert_eq!(
        fs::read(f.dir.path().join("book/source/input.pdf")).unwrap(),
        f.bytes
    );
    if std::env::var_os("AW_PDF_BASELINE_PROFILE").is_some() {
        let baseline = Instant::now();
        f.db.with_conn(|conn| library::reindex_passages(conn, "source"))
            .unwrap();
        eprintln!(
            "previous whole-text reindex alone: {} ms",
            baseline.elapsed().as_millis()
        );
    }

    eprintln!(
        "owner-shaped rebuild bytes={} elapsed_ms={} stages={}",
        f.bytes.len(),
        progress.elapsed_ms,
        serde_json::to_string(&progress).unwrap()
    );
}
#[test]
fn cancellation_and_sql_failure_during_reindex_restore_previous_structure_and_links() {
    let f = fixture(85);
    let first = Job::default();
    let result = pdf_build::rebuild(&f.db, f.dir.path(), "source", &first);
    first.finish(&result);
    result.unwrap();
    f.db.with_conn(|conn| {
        let before = fingerprint(conn)?;
        let document = &pdf_structures::load(conn,"source")?.unwrap().document;
        for fail_stage in ["store","indexRead","index","indexRoles","commit"] {
            let result=pdf_structures::replace_observed(conn,"source",&content_hash(&f.bytes),document, |stage,_,_| {
                if stage == fail_stage { Err(AppError::InvalidInput(UserMessage::new("library.structure.cancelled","injected cancellation"))) } else {Ok(())}
            });
            assert!(result.is_err()); assert!(conn.is_autocommit());
            assert_eq!(fingerprint(conn)?,before,"cancellation at {fail_stage}");
        }
        conn.execute_batch("CREATE TEMP TRIGGER fail_index AFTER INSERT ON source_passages WHEN NEW.origin='section' BEGIN SELECT RAISE(FAIL,'injected reindex failure'); END;")?;
        assert!(pdf_structures::replace(conn,"source",&content_hash(&f.bytes),document).is_err());
        assert_eq!(fingerprint(conn)?,before);
        conn.execute_batch("DROP TRIGGER fail_index")?;
        let mut changed = document.clone();
        for block in &mut changed.blocks { block.role = "note".into(); }
        conn.execute_batch("CREATE TEMP TRIGGER fail_role BEFORE UPDATE OF structure_role ON source_passages BEGIN SELECT RAISE(FAIL,'injected role failure'); END;")?;
        assert!(pdf_structures::replace(conn,"source",&content_hash(&f.bytes),&changed).is_err());
        assert_eq!(fingerprint(conn)?,before,"role write failure must roll back metadata and links");
        conn.execute_batch("DROP TRIGGER fail_role")?;
        Ok(())
    }).unwrap();
}
#[test]
fn duplicate_jobs_cancel_retry_and_failure_status_are_retained() {
    let jobs = Jobs::default();
    let job = jobs.start("source").unwrap();
    assert!(jobs.start("source").is_err());
    assert!(jobs.start("another").is_err());
    assert!(job.cancel());
    assert!(job.check().is_err());
    let result = Err(job.check().unwrap_err());
    job.finish(&result);
    assert_eq!(jobs.get("source").unwrap().snapshot().status, "cancelled");
    let next = jobs.start("source").unwrap();
    next.begin_commit().unwrap();
    assert!(
        !next.cancel(),
        "commit is the cancellation linearization boundary"
    );
    next.finish(&Err(AppError::Unexpected(
        "not logged raw path SECRET".into(),
    )));
    let json = serde_json::to_string(&next.snapshot()).unwrap();
    assert!(!json.contains("SECRET"));
    assert_eq!(next.snapshot().status, "failed");
    assert!(jobs.start("source").is_ok());
}
#[test]
fn cancellation_stops_at_pdf_page_boundary_without_touching_database() {
    let f = fixture(5);
    let before = f.db.with_conn(fingerprint).unwrap();
    let mut seen = vec![];
    let result = abiding_word_lib::library::adapters::pdf::read_pages_with_progress(
        &f.bytes,
        |done, total| {
            seen.push((done, total));
            if done == 2 {
                Err(AppError::InvalidInput(UserMessage::new(
                    "library.structure.cancelled",
                    "cancelled",
                )))
            } else {
                Ok(())
            }
        },
    );
    assert!(result.is_err());
    assert_eq!(seen, vec![(0, 5), (1, 5), (2, 5)]);
    assert_eq!(f.db.with_conn(fingerprint).unwrap(), before);
}

#[test]
fn reading_and_autosave_database_access_remain_available_during_pdf_work() {
    let f = fixture(400);
    let job = Job::default();
    std::thread::scope(|scope| {
        let work = scope.spawn(|| pdf_build::rebuild(&f.db, f.dir.path(), "source", &job));
        let deadline = Instant::now() + Duration::from_secs(5);
        while job.snapshot().stage != "pdf" {
            assert!(Instant::now() < deadline, "worker never reached PDF stage");
            std::thread::yield_now();
        }
        let began = Instant::now();
        f.db.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM source_segments", [], |r| {
                r.get::<_, usize>(0)
            })?;
            assert_eq!(conn.execute("INSERT INTO settings(key,value,created_at,updated_at) VALUES('analysis-concurrent-write','saved','T','T')", [])?, 1);
            Ok(())
        })
        .unwrap();
        assert!(
            began.elapsed() < Duration::from_secs(1),
            "PDF work held the database lock"
        );
        assert!(job.cancel());
        let result = work.join().unwrap();
        job.finish(&result);
        assert!(result.is_err());
        assert_eq!(job.snapshot().status, "cancelled");
        assert!(f
            .db
            .with_conn(|conn| pdf_structures::load(conn, "source"))
            .unwrap()
            .is_none());
    });
}

#[test]
fn automatic_contents_persist_the_same_structure_and_never_reparse_saved_results() {
    let fixture = fixture(400);
    let jobs = Jobs::default();
    let items =
        pdf_build::ensure_contents(&fixture.db, fixture.dir.path(), "source", &jobs).unwrap();
    assert!(!items.is_empty());
    let job = jobs.get("source").unwrap().snapshot();
    assert_eq!(job.status, "succeeded");
    assert!(job.heading_count.unwrap() > 0);
    let before = fixture.db.with_conn(fingerprint).unwrap();
    fs::remove_file(fixture.dir.path().join("book/source/input.pdf")).unwrap();
    let fresh_session = Jobs::default();
    let again =
        pdf_build::ensure_contents(&fixture.db, fixture.dir.path(), "source", &fresh_session)
            .unwrap();
    assert_eq!(
        serde_json::to_value(items).unwrap(),
        serde_json::to_value(again).unwrap()
    );
    assert!(fresh_session.get("source").is_none());
    assert_eq!(fixture.db.with_conn(fingerprint).unwrap(), before);
}

#[test]
fn a_saved_empty_analysis_is_not_an_invisible_second_full_pdf_parse() {
    let fixture = fixture(8);
    pdf_build::ensure_contents(&fixture.db, fixture.dir.path(), "source", &Jobs::default())
        .unwrap();
    fixture
        .db
        .with_conn(|conn| {
            let record = pdf_structures::load(conn, "source")?.unwrap();
            let mut document = record.document.clone();
            document.headings.clear();
            pdf_structures::replace(conn, "source", &record.hash, &document)?;
            Ok(())
        })
        .unwrap();
    fs::remove_file(fixture.dir.path().join("book/source/input.pdf")).unwrap();
    let jobs = Jobs::default();
    assert!(
        pdf_build::ensure_contents(&fixture.db, fixture.dir.path(), "source", &jobs)
            .unwrap()
            .is_empty()
    );
    assert!(jobs.get("source").is_none());
}

#[test]
fn failed_automatic_analysis_is_retained_until_an_explicit_retry() {
    let fixture = fixture(8);
    let path = fixture.dir.path().join("book/source/input.pdf");
    fs::remove_file(&path).unwrap();
    let jobs = Jobs::default();
    let before = fixture.db.with_conn(fingerprint).unwrap();
    assert!(pdf_build::ensure_contents(&fixture.db, fixture.dir.path(), "source", &jobs).is_err());
    assert_eq!(jobs.get("source").unwrap().snapshot().status, "failed");
    fs::write(&path, &fixture.bytes).unwrap();
    assert!(pdf_build::ensure_contents(&fixture.db, fixture.dir.path(), "source", &jobs).is_err());
    assert_eq!(fixture.db.with_conn(fingerprint).unwrap(), before);
    let retry = jobs.start("source").unwrap();
    let result = pdf_build::rebuild(&fixture.db, fixture.dir.path(), "source", &retry);
    retry.finish(&result);
    assert!(result.is_ok());
    pdf_build::ensure_contents(&fixture.db, fixture.dir.path(), "source", &jobs).unwrap();
    assert!(fixture
        .db
        .with_conn(|conn| Ok(pdf_structures::load(conn, "source")?.is_some()))
        .unwrap());
}

#[test]
fn unchanged_literal_roles_are_not_written_again() {
    let f = fixture(8);
    let result = pdf_build::rebuild(&f.db, f.dir.path(), "source", &Job::default());
    result.unwrap();
    f.db.with_conn(|conn| {
        let record = pdf_structures::load(conn, "source")?.unwrap();
        conn.execute_batch("CREATE TEMP TRIGGER no_redundant_roles BEFORE UPDATE OF structure_role ON source_passages WHEN OLD.origin='explicit' BEGIN SELECT RAISE(FAIL,'redundant literal role update'); END;")?;
        pdf_structures::replace(conn, "source", &record.hash, &record.document)?;
        Ok(())
    }).unwrap();
}
