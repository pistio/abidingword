//! Opt-in local diagnosis. No private document or extracted text is a fixture.
//! Set AW_PDF_STRUCTURE_FILE and AW_PDF_STRUCTURE_REPORT to local paths.
//! The report contains counts and canonical book/page metadata only.
use abiding_word_lib::{
    db::{library, pdf_structures, Database},
    library::{
        adapters::pdf,
        files::content_hash,
        import,
        pdf_build::{self, Job},
        CommitSourceImport,
    },
    scripture::reference::ScriptureRef,
};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::PathBuf, time::Instant};

#[test]
#[ignore = "requires an explicitly supplied local PDF; never runs in CI"]
fn real_file_import_and_rebuild_report() {
    let Some(path) = std::env::var_os("AW_PDF_STRUCTURE_FILE").map(PathBuf::from) else {
        return;
    };
    let output = PathBuf::from(
        std::env::var_os("AW_PDF_STRUCTURE_REPORT").expect("set a local report destination"),
    );
    let bytes = fs::read(&path).unwrap();
    let started = Instant::now();
    let staged = import::stage(&path).unwrap();
    let import_ms = started.elapsed().as_millis();
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let id = "local-pdf-diagnostic";
    let relative = library::managed_relative_path(id, &staged);
    let managed = dir.path().join(&relative);
    fs::create_dir_all(managed.parent().unwrap()).unwrap();
    fs::write(managed, &bytes).unwrap();
    db.with_conn(|conn| {
        library::commit_import(
            conn,
            id,
            &staged,
            &CommitSourceImport {
                preview_id: staged.id.clone(),
                title: "Local PDF diagnosis".into(),
                language: std::env::var("AW_PDF_STRUCTURE_LANGUAGE").ok(),
                ..Default::default()
            },
            &relative,
        )?;
        Ok(())
    })
    .unwrap();
    let segments = || {
        db.with_conn(|conn| {
            let mut statement = conn.prepare("SELECT id,position,page_number,content,char_start,char_end FROM source_segments WHERE source_id=?1 ORDER BY position")?;
            let rows = statement.query_map([id], |r| Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,Option<u32>>(2)?,r.get::<_,String>(3)?,r.get::<_,Option<u32>>(4)?,r.get::<_,Option<u32>>(5)?)))?.collect::<Result<Vec<_>,_>>()?;
            Ok((rows.len(),content_hash(&serde_json::to_vec(&rows).unwrap())))
        }).unwrap()
    };
    let segments_before = segments();
    let summarise = || {
        db.with_conn(|conn| {
            let view = pdf_structures::view(conn, id)?;
            let mut bases = BTreeMap::<String, usize>::new();
            for h in &view.headings {
                *bases.entry(h.heading.basis.clone()).or_default() += 1;
            }
            let queries = std::env::var("AW_PDF_STRUCTURE_QUERIES")
                .unwrap_or_default()
                .split(',')
                .filter(|q| !q.is_empty())
                .map(|query| {
                    let reference = ScriptureRef::from_canonical(query).unwrap();
                    let places = library::passage_source_locations(conn, &reference, id).unwrap();
                    let entries: Vec<_> = places.iter().map(|p| {
                        let page: Option<i64> = conn.query_row("SELECT page_number FROM source_segments WHERE id=?1", [&p.segment_id], |r| r.get(0)).unwrap();
                        json!({"page":page,"origin":p.origin,"scope":p.structure_scope,"basis":p.structure_basis})
                    }).collect();
                    json!({"canonical":query,"locations":entries})
                })
                .collect::<Vec<_>>();
            let mapped: Vec<_> = view.headings.iter().filter(|h| h.canonical.is_some() || !h.additional_canonicals.is_empty()).map(|h| json!({"canonical":h.canonical,"additionalCanonicals":h.additional_canonicals,"scope":h.scope,"page":h.heading.anchor.page,"basis":h.heading.basis,"level":h.heading.level})).collect();
            let book_coverage: Vec<_> = abiding_word_lib::scripture::canon::BOOKS.iter().map(|book| {
                let reference=ScriptureRef::from_canonical(&format!("{}.1",book.code)).unwrap();
                let places=library::passage_source_locations(conn,&reference,id).unwrap();
                let first=places.first().map(|p| {
                    let page:Option<i64>=conn.query_row("SELECT page_number FROM source_segments WHERE id=?1",[&p.segment_id],|r|r.get(0)).unwrap();
                    json!({"page":page,"origin":p.origin,"scope":p.structure_scope,"basis":p.structure_basis})
                });
                let pages:Vec<_>=view.headings.iter().filter(|h| h.scope=="book" && h.canonical.iter().chain(h.additional_canonicals.iter()).any(|c|c.starts_with(&format!("{}.",book.code)))).map(|h|h.heading.anchor.page).collect();
                json!({"book":book.code,"mappedBookPages":pages,"returnedCount":places.len(),"first":first})
            }).collect();
            let shared:Vec<_>=view.headings.iter().filter(|h|!h.additional_canonicals.is_empty()).map(|h| {
                let mappings:Vec<_>=h.additional_canonicals.iter().map(|canonical| {
                    let section:i64=conn.query_row("SELECT count(*) FROM source_passages WHERE source_id=?1 AND structure_node_id=?2 AND passage=?3 AND origin='section'",rusqlite::params![id,h.heading.id,canonical],|r|r.get(0)).unwrap();
                    let inherited:i64=conn.query_row("SELECT count(*) FROM source_passages WHERE source_id=?1 AND structure_node_id=?2 AND passage=?3 AND origin='inherited'",rusqlite::params![id,h.heading.id,canonical],|r|r.get(0)).unwrap();
                    json!({"canonical":canonical,"sectionRows":section,"inheritedRows":inherited})
                }).collect();
                json!({"page":h.heading.anchor.page,"mappings":mappings})
            }).collect();
            let record = pdf_structures::load(conn, id)?.unwrap();
            let printed: Vec<_> = record.document.printed_contents.iter().map(|d| json!({"page":d.page,"level":d.level,"anchored":d.anchor_id.as_ref().is_some_and(|id|view.headings.iter().any(|h| &h.heading.id==id))})).collect();
            Ok(json!({"analysis":view.analysis,"complete":view.complete,"headings":view.headings.len(),"bases":bases,"mapped":mapped,"queries":queries,"contentsRoots":pdf_structures::contents(conn,id)?.len(),"printedDestinations":printed,"bookCoverage":book_coverage,"sharedBookOpenings":shared}))
        }).unwrap()
    };
    let imported = summarise();
    let job = Job::default();
    let result = pdf_build::rebuild(&db, dir.path(), id, &job);
    job.finish(&result);
    result.unwrap();
    let rebuilt = summarise();
    let segments_after = segments();
    let mut report = json!({"bytes":bytes.len(),"inputSha256":content_hash(&bytes),"pages":staged.document.page_count,"segments":staged.document.segments.len(),"importMilliseconds":import_ms,"import":imported,"rebuild":rebuilt,"rebuildProgress":job.snapshot(),"storedSegmentsBefore":{"count":segments_before.0,"sha256":segments_before.1},"storedSegmentsAfter":{"count":segments_after.0,"sha256":segments_after.1},"storedSegmentsUnchanged":segments_before==segments_after});
    // Optional private scratch inspection; never print or store source text in
    // the repository/report. Only explicitly selected pages are written.
    if let (Some(trace), Ok(pages)) = (
        std::env::var_os("AW_PDF_STRUCTURE_PRIVATE_TRACE"),
        std::env::var("AW_PDF_STRUCTURE_INSPECT_PAGES"),
    ) {
        let pages: Vec<u32> = pages.split(',').filter_map(|p| p.parse().ok()).collect();
        let pdf = pdf::read_pages(&bytes, "local input").unwrap();
        let rows: Vec<_> = pdf.layouts.iter().filter(|p| pages.contains(&p.number)).map(|p| json!({"page":p.number,"order":format!("{:?}",p.order),"lines":p.lines.iter().map(|l| json!({"text":l.text,"x0":l.x0,"x1":l.x1,"y":l.y,"size":l.size,"role":format!("{:?}",l.role),"block":l.block})).collect::<Vec<_>>()})).collect();
        fs::write(trace, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
        report["failedPages"] = json!(pdf.failed_pages);
    }
    fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    // An optional independently reviewed map contains only canonical book
    // codes and expected physical opening pages, never document text.
    if let Some(expected) = std::env::var_os("AW_PDF_STRUCTURE_EXPECT_BOOK_OPENINGS") {
        let expected: BTreeMap<String, u32> =
            serde_json::from_slice(&fs::read(expected).unwrap()).unwrap();
        for stage in ["import", "rebuild"] {
            for (book, page) in &expected {
                let coverage = report[stage]["bookCoverage"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|b| b["book"] == *book)
                    .unwrap();
                assert!(
                    coverage["mappedBookPages"]
                        .as_array()
                        .unwrap()
                        .contains(&json!(page)),
                    "{stage}: missing independently reviewed book opening {book}"
                );
                assert_eq!(
                    coverage["first"]["page"],
                    json!(page),
                    "{stage}: wrong first destination for {book}"
                );
                assert_eq!(
                    coverage["first"]["scope"], "book",
                    "{stage}: invented narrower section for {book}"
                );
                assert_eq!(
                    coverage["returnedCount"], 1,
                    "{stage}: lower-priority places remain for {book}"
                );
            }
            for shared in report[stage]["sharedBookOpenings"].as_array().unwrap() {
                let mappings = shared["mappings"].as_array().unwrap();
                for mapping in mappings {
                    assert_eq!(
                        mapping["sectionRows"], 1,
                        "{stage}: shared book does not use exactly one real opener"
                    );
                    assert_eq!(
                        mapping["inheritedRows"], mappings[0]["inheritedRows"],
                        "{stage}: shared books inherit different body blocks"
                    );
                }
            }
        }
    }
    assert_eq!(fs::read(path).unwrap(), bytes, "input PDF was changed");
    assert_eq!(
        segments_before, segments_after,
        "Rebuild changed stored segment IDs or text"
    );
    assert_eq!(report["import"]["mapped"], report["rebuild"]["mapped"]);
}
