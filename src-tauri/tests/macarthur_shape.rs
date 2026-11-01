//! A.4.1: invented documents shaped like a German Bible handbook (front matter,
//! a Contents page listing biblical books, one introduction per book with
//! running heads, page numbers and incidental references). They reproduce
//! failure *hypotheses* for the owner's MacArthur report; they cannot accept
//! the real PDF, which was never available to development.
#[path = "support/library_fixtures.rs"]
mod fixtures;
use abiding_word_lib::{
    db::{library, pdf_diagnostic, pdf_structures, Database},
    library::{import, pdf_structure, CommitSourceImport, PassageLocation},
    scripture::reference::ScriptureRef,
};
use rusqlite::Connection;
use std::fs;

type Line = (f64, f64, f64, String);

fn body(page: &mut Vec<Line>, from: f64, count: usize, tag: &str) {
    for n in 0..count {
        page.push((
            72.0,
            from - n as f64 * 16.0,
            11.0,
            format!("Erfundener Absatz {tag}-{n} zu Aufbau, Hintergrund und Zusammenhang."),
        ));
    }
}

/// The shape options let each hypothesis be switched on separately.
#[derive(Clone, Copy, Default, Debug)]
pub struct Shape {
    /// A Contents page lists bare book titles in a heading-like size.
    contents_titles: bool,
    /// A book's opening title sits in the top margin with the very same text as
    /// its running head on following pages.
    title_in_margin: bool,
    /// Running heads carry the chapter: "Apostelgeschichte 7".
    chapter_running_heads: bool,
}

fn book(pages: &mut Vec<Vec<Line>>, title: &str, shape: Shape, extra: &[&str], folio: &mut u32) {
    let title_y = if shape.title_in_margin { 800.0 } else { 640.0 };
    let mut first = vec![(72.0, title_y, 24.0, title.to_string())];
    first.push((
        72.0,
        title_y - 50.0,
        15.0,
        "Autor und Abfassungszeit".into(),
    ));
    body(&mut first, title_y - 80.0, 8, &format!("{title}-a"));
    first.push((72.0, 260.0, 15.0, "Hintergrund".into()));
    body(&mut first, 230.0, 6, &format!("{title}-b"));
    first.push((290.0, 40.0, 9.0, folio.to_string()));
    *folio += 1;
    pages.push(first);
    for (index, chapter) in [6, 7, 8].iter().enumerate() {
        let head = if shape.chapter_running_heads {
            format!("{title} {chapter}")
        } else {
            title.to_string()
        };
        let mut page = vec![(72.0, 810.0, 9.0, head)];
        page.push((
            72.0,
            740.0,
            15.0,
            if index == 0 {
                "Gliederung".into()
            } else {
                format!("Themen {index}")
            },
        ));
        body(&mut page, 710.0, 20, &format!("{title}-{index}"));
        if let Some(text) = extra.get(index) {
            page.push((72.0, 360.0, 11.0, (*text).into()));
        }
        page.push((290.0, 40.0, 9.0, folio.to_string()));
        *folio += 1;
        pages.push(page);
    }
}

pub fn handbook(shape: Shape) -> Vec<u8> {
    let mut pages: Vec<Vec<Line>> = Vec::new();
    // 1 title page, 2 imprint with a dedication verse, 3 Contents.
    pages.push(vec![
        (72.0, 600.0, 28.0, "Erfundene Basisinformationen".into()),
        (72.0, 560.0, 18.0, "zur Bibel".into()),
    ]);
    let mut imprint = vec![
        (
            72.0,
            700.0,
            9.0,
            "Alle Rechte vorbehalten. Erfundener Verlag.".into(),
        ),
        (
            72.0,
            680.0,
            9.0,
            "Bibelzitate nach einer erfundenen Ausgabe.".into(),
        ),
    ];
    imprint.push((72.0, 500.0, 11.0, "Leitvers dieses Buches: Apg 6,4.".into()));
    pages.push(imprint);
    let mut contents = vec![(72.0, 760.0, 18.0, "Inhalt".into())];
    for (n, name) in ["Johannes", "Apostelgeschichte", "Römer"]
        .iter()
        .enumerate()
    {
        if shape.contents_titles {
            contents.push((72.0, 700.0 - n as f64 * 40.0, 18.0, (*name).into()));
        } else {
            contents.push((
                72.0,
                700.0 - n as f64 * 40.0,
                11.0,
                format!("{name} . . . . {}", 10 + n * 4),
            ));
        }
    }
    pages.push(contents);
    let mut folio = 10;
    book(
        &mut pages,
        "Johannes",
        shape,
        &["Vgl. Apg 6,1 im Zusammenhang."],
        &mut folio,
    );
    book(
        &mut pages,
        "Apostelgeschichte",
        shape,
        &[
            "Die Wahl der Sieben (6,1-7) ist erfunden kommentiert.",
            "Ein beiläufiger Verweis: Apg 7,58.",
        ],
        &mut folio,
    );
    book(
        &mut pages,
        "Römer",
        shape,
        &["Siehe Apg 6,7 und Röm 1,16."],
        &mut folio,
    );
    fixtures::pdf_sized(&pages)
}

fn imported(conn: &Connection, bytes: &[u8], sources: &std::path::Path) -> String {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("invented.pdf");
    fs::write(&path, bytes).unwrap();
    let staged = import::stage(&path).unwrap();
    let id = abiding_word_lib::db::new_id();
    let relative = library::managed_relative_path(&id, &staged);
    let file = sources.join(&relative);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, bytes).unwrap();
    library::commit_import(
        conn,
        &id,
        &staged,
        &CommitSourceImport {
            preview_id: staged.id.clone(),
            title: "Invented handbook".into(),
            language: Some("Deutsch".into()),
            ..Default::default()
        },
        &relative,
    )
    .unwrap();
    id
}

fn page_of(conn: &Connection, place: &PassageLocation) -> i64 {
    conn.query_row(
        "SELECT page_number FROM source_segments WHERE id=?1",
        [&place.segment_id],
        |r| r.get(0),
    )
    .unwrap()
}

struct Outcome {
    page: i64,
    first: PassageLocation,
    places: Vec<PassageLocation>,
    view: pdf_structures::View,
    diagnostic: serde_json::Value,
}

fn outcome(shape: Shape, passage: &str) -> Outcome {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &handbook(shape), temp.path());
        let view = pdf_structures::view(c, &id)?;
        let reference = ScriptureRef::from_canonical(passage)?;
        let places = library::passage_source_locations(c, &reference, &id)?;
        let first = places.first().cloned().expect("a place for the passage");
        let diagnostic =
            serde_json::to_value(pdf_diagnostic::diagnostic(c, &id, Some(passage))?).unwrap();
        Ok(Outcome {
            page: page_of(c, &first),
            first,
            places,
            view,
            diagnostic,
        })
    })
    .unwrap()
}

const SHAPES: [(&str, Shape); 5] = [
    (
        "plain",
        Shape {
            contents_titles: false,
            title_in_margin: false,
            chapter_running_heads: false,
        },
    ),
    (
        "contents titles",
        Shape {
            contents_titles: true,
            title_in_margin: false,
            chapter_running_heads: false,
        },
    ),
    (
        "title in margin",
        Shape {
            contents_titles: false,
            title_in_margin: true,
            chapter_running_heads: false,
        },
    ),
    (
        "chapter running heads",
        Shape {
            contents_titles: false,
            title_in_margin: false,
            chapter_running_heads: true,
        },
    ),
    (
        "all",
        Shape {
            contents_titles: true,
            title_in_margin: true,
            chapter_running_heads: true,
        },
    ),
];

/// Physical page of the Acts introduction in the invented handbook.
const ACTS_PAGE: i64 = 8;

#[test]
fn acts_6_opens_the_book_section_not_front_matter_in_every_handbook_shape() {
    for (name, shape) in SHAPES {
        let o = outcome(shape, "ACT.6");
        assert_eq!(
            o.page, ACTS_PAGE,
            "{name}: the Acts opening heading, not front matter"
        );
        assert_eq!(o.first.matched_text, "Apostelgeschichte", "{name}");
        assert_eq!(
            o.first.structure_scope.as_deref(),
            Some("book"),
            "{name}: honestly broader"
        );
        let node = o
            .first
            .structure_node_id
            .clone()
            .expect("exact heading identity");
        let heading = o
            .view
            .headings
            .iter()
            .find(|h| h.heading.id == node)
            .unwrap();
        assert_eq!(heading.heading.anchor.page as i64, ACTS_PAGE, "{name}");
        assert_eq!(heading.canonical.as_deref(), Some("ACT.1-ACT.28"), "{name}");
        // The front-matter dedication and the Acts 7 body mention stay literal
        // mentions after the grounded section; nothing is dropped.
        let mention = o
            .places
            .iter()
            .position(|p| p.matched_text == "Apg 6,4")
            .expect("front matter mention kept");
        assert!(mention > 0, "{name}");
        assert!(
            o.places.iter().all(|p| p.matched_text != "Apg 7,58"),
            "{name}: shares no verse with Acts 6"
        );
        // the diagnostic reports exactly that target
        let target = &o.diagnostic["query"]["readerTarget"];
        assert_eq!(target["physicalPage"], ACTS_PAGE, "{name}");
        assert_eq!(target["structureNodeId"], node.as_str(), "{name}");
        assert_eq!(
            o.diagnostic["query"]["candidates"][0]["evidence"], "Book section (broader)",
            "{name}"
        );
    }
}

#[test]
fn a_contents_page_listing_books_keeps_its_headings_but_maps_no_passage() {
    let o = outcome(SHAPES[4].1, "ACT.6");
    let listed: Vec<_> = o
        .view
        .headings
        .iter()
        .filter(|h| {
            h.heading.anchor.page == 3 && h.heading.level > 1 && h.heading.title != "Inhalt"
        })
        .collect();
    assert_eq!(
        listed.len(),
        3,
        "the Contents lines remain navigable headings"
    );
    assert!(listed
        .iter()
        .all(|h| h.scope == "contents" && h.canonical.is_none()));
    assert_eq!(
        o.diagnostic["structure"]["listingPages"],
        serde_json::json!([3])
    );
    // Each book still has exactly its own grounded section.
    for (title, page) in [("Johannes", 4), ("Apostelgeschichte", 8), ("Römer", 12)] {
        let books: Vec<_> = o
            .view
            .headings
            .iter()
            .filter(|h| h.heading.title == title && h.scope == "book")
            .collect();
        assert_eq!(books.len(), 1, "{title}");
        assert_eq!(books[0].heading.anchor.page, page, "{title}");
    }
}

#[test]
fn the_running_head_never_becomes_a_section_and_the_margin_title_survives() {
    for (name, shape) in SHAPES {
        let o = outcome(shape, "ACT.6");
        let acts: Vec<_> = o
            .view
            .headings
            .iter()
            .filter(|h| h.heading.title.starts_with("Apostelgeschichte"))
            .collect();
        let grounded: Vec<_> = acts.iter().filter(|h| h.scope == "book").collect();
        assert_eq!(grounded.len(), 1, "{name}: one Acts book section");
        assert!(
            acts.iter().all(|h| h.heading.anchor.size > 9.0),
            "{name}: no 9-pt running head"
        );
    }
}

#[test]
fn results_and_diagnostic_are_deterministic() {
    let a = outcome(SHAPES[4].1, "ACT.6");
    let b = outcome(SHAPES[4].1, "ACT.6");
    let strip = |mut v: serde_json::Value| {
        v["generatedAt"] = serde_json::Value::Null;
        v["source"]["id"] = serde_json::Value::Null;
        v["query"]["readerTarget"]["segmentId"] = serde_json::Value::Null;
        v
    };
    assert_eq!(a.page, b.page);
    assert_eq!(a.first.matched_text, b.first.matched_text);
    let (x, y) = (strip(a.diagnostic), strip(b.diagnostic));
    assert_eq!(x["headings"], y["headings"]);
    assert_eq!(
        x["query"]["candidates"].as_array().unwrap().len(),
        y["query"]["candidates"].as_array().unwrap().len()
    );
}

#[test]
fn the_diagnostic_is_metadata_only() {
    let o = outcome(SHAPES[4].1, "ACT.6");
    let text = serde_json::to_string(&o.diagnostic).unwrap();
    assert!(!text.contains("Erfundener Absatz"), "no paragraph text");
    assert!(!text.contains("Alle Rechte"), "no imprint text");
    assert!(
        !text.contains("invented.pdf") && !text.contains("sources/"),
        "no file names or paths"
    );
    assert_eq!(o.diagnostic["schema"], pdf_diagnostic::SCHEMA);
    assert_eq!(
        o.diagnostic["source"]["fileSha256"].as_str().map(str::len),
        Some(64)
    );
    assert_eq!(o.diagnostic["structure"]["status"], "ready");
    assert_eq!(
        o.diagnostic["structure"]["analysis"],
        pdf_structure::ANALYSIS
    );
    assert_eq!(o.diagnostic["structure"]["rebuildRecommended"], false);
    assert_eq!(o.diagnostic["structure"]["pageCount"], 15);
}

#[test]
fn german_abbreviation_and_english_passage_input_resolve_in_the_diagnostic() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &handbook(SHAPES[0].1), temp.path());
        for input in ["Apg 6", "Apostelgeschichte 6", "Acts 6", "ACT.6"] {
            let d = pdf_diagnostic::diagnostic(c, &id, Some(input))?;
            let q = d.query.unwrap();
            assert_eq!(q.canonical.as_deref(), Some("ACT.6"), "{input}");
            assert_eq!(
                q.reader_target.unwrap().physical_page,
                Some(ACTS_PAGE),
                "{input}"
            );
        }
        let none = pdf_diagnostic::diagnostic(c, &id, Some("kein Bibeltext"))?
            .query
            .unwrap();
        assert!(none.canonical.is_none() && none.candidates.is_empty());
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_structure_built_before_a4_still_loads_and_asks_for_a_rebuild() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &handbook(SHAPES[0].1), temp.path());
        // An A.3 document has no `analysis` member at all.
        let json: String = c.query_row(
            "SELECT document FROM source_pdf_structures WHERE source_id=?1",
            [&id],
            |r| r.get(0),
        )?;
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        value.as_object_mut().unwrap().remove("analysis");
        c.execute(
            "UPDATE source_pdf_structures SET document=?2, etag='a3' WHERE source_id=?1",
            rusqlite::params![id, value.to_string()],
        )?;
        let view = pdf_structures::view(c, &id)?;
        assert_eq!(view.status, "ready");
        assert_eq!(view.analysis, 0);
        assert!(view.rebuild_recommended);
        let d = pdf_diagnostic::diagnostic(c, &id, None)?;
        assert!(d.structure.rebuild_recommended);
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_chapter_heading_outranks_the_book_section_when_the_book_has_one() {
    let mut pages: Vec<Vec<Line>> = Vec::new();
    for n in 0..2 {
        pages.push(vec![(
            72.0,
            700.0,
            11.0,
            format!("Erfundene Einleitung {n}."),
        )]);
    }
    let mut acts = vec![(72.0, 640.0, 24.0, "Apostelgeschichte".to_string())];
    body(&mut acts, 600.0, 10, "a");
    pages.push(acts);
    for chapter in [5, 6, 7] {
        let mut page = vec![
            (72.0, 810.0, 9.0, "Apostelgeschichte".to_string()),
            (72.0, 740.0, 18.0, format!("Kapitel {chapter}")),
        ];
        body(&mut page, 700.0, 20, &chapter.to_string());
        pages.push(page);
    }
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &fixtures::pdf_sized(&pages), temp.path());
        let places =
            library::passage_source_locations(c, &ScriptureRef::from_canonical("ACT.6")?, &id)?;
        assert_eq!(places[0].structure_scope.as_deref(), Some("chapter"));
        assert_eq!(places[0].matched_text, "Kapitel 6");
        assert_eq!(page_of(c, &places[0]), 5);
        Ok(())
    })
    .unwrap();
}

#[test]
fn uniform_prose_without_headings_is_reported_as_no_section_not_guessed() {
    let mut pages: Vec<Vec<Line>> = Vec::new();
    for n in 0..4 {
        let mut page = Vec::new();
        body(&mut page, 760.0, 30, &n.to_string());
        if n == 2 {
            page.push((
                72.0,
                200.0,
                11.0,
                "Ein erfundener Verweis auf Apg 6,3.".into(),
            ));
        }
        pages.push(page);
    }
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &fixtures::pdf_sized(&pages), temp.path());
        let d = pdf_diagnostic::diagnostic(c, &id, Some("Apg 6"))?;
        let q = d.query.unwrap();
        assert_eq!(q.candidates[0].origin, "explicit");
        assert_eq!(q.candidates[0].evidence, "Mentions the passage");
        assert!(q.verdict.starts_with("No matching section identified"));
        Ok(())
    })
    .unwrap();
}
