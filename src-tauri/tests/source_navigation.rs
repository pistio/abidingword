//! Invented source documents only. Source navigation does not import or verify
//! a Bible, rewrite quotation text, or infer section destinations from citations.
#[path = "support/library_fixtures.rs"]
mod fixtures;

use abiding_word_lib::{
    db::{library, pdf_structures, Database},
    library::{adapters::epub, import, pdf_structure, sections::LinkOrigin, CommitSourceImport},
    scripture::reference::ScriptureRef,
};
use rusqlite::Connection;

fn add(conn: &Connection, file: &str, bytes: &[u8]) -> (String, import::StagedSource) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(file);
    std::fs::write(&path, bytes).unwrap();
    let staged = import::stage(&path).unwrap();
    let id = abiding_word_lib::db::new_id();
    library::commit_import(
        conn,
        &id,
        &staged,
        &CommitSourceImport {
            preview_id: staged.id.clone(),
            title: "Invented navigation source".into(),
            language: Some("English".into()),
            ..Default::default()
        },
        &library::managed_relative_path(&id, &staged),
    )
    .unwrap();
    (id, staged)
}

fn target(conn: &Connection, source: &str, passage: &str, expected: &str, scope: &str) {
    let places = library::passage_source_locations(
        conn,
        &ScriptureRef::from_canonical(passage).unwrap(),
        source,
    )
    .unwrap();
    assert_eq!(places.len(), 1, "{passage}: {places:?}");
    let place = &places[0];
    assert_eq!(place.origin, LinkOrigin::Section, "{passage}: {places:?}");
    assert_eq!(place.structure_scope.as_deref(), Some(scope));
    let text: String = conn
        .query_row(
            "SELECT content FROM source_segments WHERE id=?1",
            [&place.segment_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(text.starts_with(expected), "{passage}: opened {text:?}");
}

const PARAGRAPHS: &[(&str, &str)] = &[
    (
        "",
        "Earlier incidental citations: Romans 8:11 and Romans 9:1. John 15:5 is also mentioned.",
    ),
    ("Romans", "BOOK OPENING. Invented introduction."),
    ("Chapter 8", "CHAPTER OPENING. Invented chapter discussion."),
    ("Verses 1-4", "VERSE OPENING. Invented section discussion."),
    ("Cross-references", "An unrelated mention of John 15:5."),
    (
        "Romans 8:11",
        "REFERENCE LIST ENTRY. Romans 8:11 is cited here.",
    ),
    ("Appendix", "APPENDIX OPENING. No Scripture section here."),
    (
        "Romans 8:11",
        "LATER SEPARATE HEADING. Outside the Romans book branch.",
    ),
];

fn structured(format: &str) -> Vec<u8> {
    heading_document(format, PARAGRAPHS, &[0, 1, 2, 3, 3, 4, 1, 2])
}

fn heading_document(format: &str, paragraphs: &[(&str, &str)], levels: &[usize]) -> Vec<u8> {
    match format {
        "md" => paragraphs
            .iter()
            .zip(levels.iter().copied())
            .map(|((heading, body), level)| {
                if level == 0 {
                    format!("{body}\n\n")
                } else {
                    format!("{} {heading}\n\n{body}\n\n", "#".repeat(level))
                }
            })
            .collect::<String>()
            .into_bytes(),
        "epub" => {
            let body = paragraphs
                .iter()
                .zip(levels.iter().copied())
                .map(|((heading, body), level)| {
                    if level == 0 {
                        format!("<p>{body}</p>")
                    } else {
                        format!("<h{level}>{heading}</h{level}><p>{body}</p>")
                    }
                })
                .collect::<String>();
            fixtures::epub(
                "<dc:language>en</dc:language>",
                &[fixtures::EpubChapter {
                    file: "body.xhtml",
                    xhtml_body: &body,
                }],
                &[0],
            )
        }
        "docx" => {
            let body = paragraphs.iter().zip(levels.iter().copied()).map(|((heading, body), level)| {
                let h = if level == 0 { String::new() } else { format!("<w:p><w:pPr><w:pStyle w:val=\"Heading{level}\"/></w:pPr><w:r><w:t>{heading}</w:t></w:r></w:p>") };
                format!("{h}<w:p><w:r><w:t>{body}</w:t></w:r></w:p>")
            }).collect::<String>();
            let styles = format!("<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">{}</w:styles>", (1..=6).map(|level| format!("<w:style w:type=\"paragraph\" w:styleId=\"Heading{level}\"><w:name w:val=\"heading {level}\"/><w:pPr><w:outlineLvl w:val=\"{}\"/></w:pPr></w:style>", level-1)).collect::<String>());
            fixtures::docx(&body, "<Relationship Id=\"styles\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>", &[("word/styles.xml", styles.as_bytes())])
        }
        _ => unreachable!(),
    }
}

#[test]
fn real_heading_hierarchies_descend_and_stop_at_the_deepest_containing_ancestor() {
    for format in ["md", "epub", "docx"] {
        let db = Database::open_in_memory().unwrap();
        db.with_conn(|conn| {
            let (id, _) = add(conn, &format!("invented.{format}"), &structured(format));
            target(conn, &id, "ROM.9", "BOOK OPENING", "book");
            target(conn, &id, "ROM.8", "CHAPTER OPENING", "chapter");
            target(conn, &id, "ROM.8.11", "CHAPTER OPENING", "chapter");
            target(conn, &id, "ROM.8.2", "VERSE OPENING", "passage");
            // Unrelated structure is not a reason to hide literal choices.
            let mentions = library::passage_source_locations(
                conn,
                &ScriptureRef::from_canonical("JHN.15.5")?,
                &id,
            )?;
            assert_eq!(mentions.len(), 2, "{format}");
            assert!(mentions.iter().all(|p| p.origin == LinkOrigin::Explicit));
            assert_eq!(
                conn.query_row("SELECT COUNT(*) FROM bible_verses", [], |r| r
                    .get::<_, i64>(0))?,
                0
            );
            Ok(())
        })
        .unwrap();
    }
}

fn toc_epub(
    ncx: bool,
    targetless_book: bool,
    duplicate_book_anchor: bool,
    missing_book_anchor: bool,
) -> Vec<u8> {
    let duplicate = if duplicate_book_anchor {
        "<p id=\"book\">AMBIGUOUS SECOND OPENING.</p>"
    } else {
        ""
    };
    let book_anchor = if missing_book_anchor {
        "different-anchor"
    } else {
        "book"
    };
    let body = fixtures::xhtml(&format!("<p>FRONT MATTER. Romans 9:1 and Romans 8:11 are incidental.</p><p id=\"{book_anchor}\">BOOK OPENING. Invented introduction.</p>{duplicate}<p id=\"chapter\">CHAPTER OPENING. Invented exposition.</p><p id=\"verse\">VERSE OPENING. Invented detailed discussion.</p><p id=\"appendix\">APPENDIX. Romans 8:11 appears again.</p>"));
    let nav = if ncx {
        let book = if targetless_book {
            ""
        } else {
            "<content src=\"body.xhtml#book\"/>"
        };
        format!("<ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\"><navMap><navPoint id=\"book\"><navLabel><text>Romans</text></navLabel>{book}<navPoint id=\"chapter\"><navLabel><text>Chapter 8</text></navLabel><content src=\"body.xhtml#chapter\"/><navPoint id=\"verse\"><navLabel><text>Verses 1-4</text></navLabel><content src=\"body.xhtml#verse\"/></navPoint></navPoint></navPoint><navPoint id=\"appendix\"><navLabel><text>Appendix</text></navLabel><content src=\"body.xhtml#appendix\"/></navPoint></navMap></ncx>")
    } else {
        let book = if targetless_book {
            "<span>Romans</span>"
        } else {
            "<a href=\"body.xhtml#book\">Romans</a>"
        };
        fixtures::xhtml(&format!("<nav epub:type=\"toc\"><ol><li>{book}<ol><li><a href=\"body.xhtml#chapter\">Chapter 8</a><ol><li><a href=\"body.xhtml#verse\">Verses 1-4</a></li></ol></li></ol></li><li><a href=\"body.xhtml#appendix\">Appendix</a></li></ol></nav>"))
    };
    let (manifest, spine_attr, nav_name) = if ncx {
        ("<item id=\"body\" href=\"body.xhtml\" media-type=\"application/xhtml+xml\"/><item id=\"nav\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>", "toc=\"nav\"", "OEBPS/toc.ncx")
    } else {
        ("<item id=\"body\" href=\"body.xhtml\" media-type=\"application/xhtml+xml\"/><item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>", "", "OEBPS/nav.xhtml")
    };
    fixtures::epub_raw(
        manifest,
        spine_attr,
        "<itemref idref=\"body\"/>",
        &[("OEBPS/body.xhtml", &body), (nav_name, &nav)],
    )
}

#[test]
fn epub_nav_and_ncx_use_actual_anchors_and_never_front_matter_mentions() {
    for ncx in [false, true] {
        let db = Database::open_in_memory().unwrap();
        db.with_conn(|conn| {
            let (id, _) = add(conn, "invented.epub", &toc_epub(ncx, false, false, false));
            target(conn, &id, "ROM.9", "BOOK OPENING", "book");
            target(conn, &id, "ROM.8", "CHAPTER OPENING", "chapter");
            target(conn, &id, "ROM.8.11", "CHAPTER OPENING", "chapter");
            target(conn, &id, "ROM.8.2", "VERSE OPENING", "passage");
            Ok(())
        })
        .unwrap();
    }
}

#[test]
fn targetless_or_ambiguous_book_entries_never_invent_a_book_opening() {
    for (ncx, targetless, duplicate, missing) in [
        (false, true, false, false),
        (true, true, false, false),
        (false, false, true, false),
        (true, false, false, true),
        (false, false, false, true),
    ] {
        let db = Database::open_in_memory().unwrap();
        db.with_conn(|conn| {
            let bytes = toc_epub(ncx, targetless, duplicate, missing);
            let (id, staged) = add(conn, "invented.epub", &bytes);
            target(conn, &id, "ROM.8.11", "CHAPTER OPENING", "chapter");
            let places = library::passage_source_locations(
                conn,
                &ScriptureRef::from_canonical("ROM.9")?,
                &id,
            )?;
            assert!(!places.is_empty());
            assert!(places.iter().all(|p| p.origin == LinkOrigin::Explicit));
            // Old files get the same negative result without reimport or
            // loss of their original literal-reference identities.
            let before = preserved(conn, &id);
            conn.execute(
                "UPDATE sources SET passage_index_version=4 WHERE id=?1",
                [&id],
            )?;
            library::ensure_passage_index(conn)?;
            let (document, contents) = epub::extract_with_contents(&bytes, "invented.epub")?;
            assert!(library::refresh_contents_links(
                conn,
                &id,
                &staged.hash,
                &document,
                &contents
            )?);
            assert_eq!(preserved(conn, &id), before);
            assert!(library::passage_source_locations(
                conn,
                &ScriptureRef::from_canonical("ROM.9")?,
                &id
            )?
            .iter()
            .all(|p| p.origin == LinkOrigin::Explicit));
            Ok(())
        })
        .unwrap();
    }
}

fn preserved(conn: &Connection, id: &str) -> (Vec<String>, Vec<String>) {
    let segments = conn.prepare("SELECT id || '|' || position || '|' || content || '|' || created_at || '|' || updated_at FROM source_segments WHERE source_id=?1 ORDER BY position").unwrap().query_map([id], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
    let links = conn.prepare("SELECT id || '|' || origin || '|' || segment_id || '|' || passage || '|' || matched_text || '|' || created_at FROM source_passages WHERE source_id=?1 AND origin IN ('explicit','manual') ORDER BY id").unwrap().query_map([id], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
    (segments, links)
}

#[test]
fn existing_epub_upgrade_preserves_text_literal_and_manual_identities_without_reimport() {
    for previous_version in [4, 5] {
        let bytes = toc_epub(false, false, false, false);
        let db = Database::open_in_memory().unwrap();
        db.with_conn(|conn| {
        let (id, staged) = add(conn, "invented.epub", &bytes);
        conn.execute("DELETE FROM source_passages WHERE source_id=?1 AND origin IN ('section','inherited')", [&id])?;
        conn.execute("UPDATE sources SET passage_index_version=?2 WHERE id=?1", rusqlite::params![&id, previous_version])?;
        // A user decision remains authoritative even after the TOC is rebuilt.
        conn.execute("INSERT INTO source_passages(id,source_id,segment_id,passage,start_key,end_key,matched_text,created_at,origin) SELECT 'intentional-link',source_id,segment_id,passage,start_key,end_key,matched_text,'original-manual-time','manual' FROM source_passages WHERE source_id=?1 AND passage='ROM.9.1' LIMIT 1", [&id])?;
        let before = preserved(conn, &id);
        assert_eq!(library::ensure_passage_index(conn)?.rebuilt, 1);
        let (document, contents) = epub::extract_with_contents(&bytes, "invented.epub")?;
        assert!(library::refresh_contents_links(conn, &id, &staged.hash, &document, &contents)?);
        assert_eq!(preserved(conn, &id), before);
        target(conn, &id, "ROM.8.11", "CHAPTER OPENING", "chapter");
        let manual = library::passage_source_locations(conn, &ScriptureRef::from_canonical("ROM.9.1")?, &id)?;
        assert!(manual.iter().all(|p| p.origin == LinkOrigin::Manual));
        // A second read is an exact metadata no-op, and a mismatching file is rejected.
        let ids_before: Vec<String> = conn.prepare("SELECT id FROM source_passages WHERE source_id=?1 ORDER BY id")?.query_map([&id], |r| r.get(0))?.collect::<Result<_, _>>()?;
        assert!(library::refresh_contents_links(conn, &id, &staged.hash, &document, &contents)?);
        assert!(!library::refresh_contents_links(conn, &id, "different-file", &document, &contents)?);
        let ids_after: Vec<String> = conn.prepare("SELECT id FROM source_passages WHERE source_id=?1 ORDER BY id")?.query_map([&id], |r| r.get(0))?.collect::<Result<_, _>>()?;
        assert_eq!(ids_after, ids_before);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM bible_verses", [], |r| r.get::<_, i64>(0))?, 0);
        Ok(())
    }).unwrap();
    }
}

#[test]
fn combined_book_openings_cover_both_books_without_borrowing_one_for_relative_chapters() {
    let paragraphs = [
        ("", "FRONT MATTER. 1 Samuel 8:1 and 2 Samuel 8:1 are cited."),
        (
            "1 + 2 Samuel",
            "COMBINED OPENING. Both books are discussed together.",
        ),
        ("Chapter 8", "AMBIGUOUS CHAPTER. No book is specified."),
        ("2 Samuel 9", "NAMED CHAPTER. The book is explicit here."),
    ];
    for format in ["md", "epub", "docx"] {
        let db = Database::open_in_memory().unwrap();
        db.with_conn(|conn| {
            let (id, _) = add(conn, &format!("combined.{format}"), &heading_document(format, &paragraphs, &[0,1,2,2]));
            for passage in ["1SA.8", "2SA.8", "1SA.8.1", "2SA.8.1"] {
                target(conn, &id, passage, "COMBINED OPENING", "book");
            }
            target(conn, &id, "2SA.9.1", "NAMED CHAPTER", "chapter");
            assert_eq!(conn.query_row("SELECT COUNT(*) FROM source_passages WHERE source_id=?1 AND origin='section' AND section_heading='Chapter 8'", [&id], |r| r.get::<_, i64>(0))?, 0);
            Ok(())
        }).unwrap();
    }
}

#[test]
fn old_epub_pre_heading_paths_are_corrected_without_changing_text_or_reference_identities() {
    let bytes = structured("epub");
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let (id, staged) = add(conn, "invented.epub", &bytes);
        // Simulate the former importer assigning a later first heading to
        // front matter. Its segment identity and literal citations are real.
        conn.execute(
            "UPDATE source_segments SET section_path='Romans' WHERE source_id=?1 AND position=0",
            [&id],
        )?;
        conn.execute(
            "UPDATE sources SET passage_index_version=4 WHERE id=?1",
            [&id],
        )?;
        let before = preserved(conn, &id);
        library::ensure_passage_index(conn)?;
        let (document, contents) = epub::extract_with_contents(&bytes, "invented.epub")?;
        assert!(library::refresh_contents_links(
            conn,
            &id,
            &staged.hash,
            &document,
            &contents
        )?);
        assert_eq!(preserved(conn, &id), before);
        assert_eq!(
            conn.query_row(
                "SELECT section_path FROM source_segments WHERE source_id=?1 AND position=0",
                [&id],
                |r| r.get::<_, Option<String>>(0)
            )?,
            None
        );
        target(conn, &id, "ROM.9", "BOOK OPENING", "book");
        target(conn, &id, "ROM.8.11", "CHAPTER OPENING", "chapter");
        Ok(())
    })
    .unwrap();
}

#[test]
fn supplied_epub_combined_book_node_keeps_both_destinations_at_the_same_real_anchor() {
    let body = fixtures::xhtml("<p>Front matter cites 1 Samuel 8:1 and 2 Samuel 8:1.</p><p id=\"book\">COMBINED OPENING. Shared introduction.</p><p id=\"chapter\">AMBIGUOUS CHAPTER. Book not named.</p>");
    let nav = fixtures::xhtml("<nav epub:type=\"toc\"><ol><li><a href=\"body.xhtml#book\">1 + 2 Samuel</a><ol><li><a href=\"body.xhtml#chapter\">Chapter 8</a></li></ol></li></ol></nav>");
    let bytes = fixtures::epub_raw("<item id=\"body\" href=\"body.xhtml\" media-type=\"application/xhtml+xml\"/><item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>", "", "<itemref idref=\"body\"/>", &[("OEBPS/body.xhtml",&body),("OEBPS/nav.xhtml",&nav)]);
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let (id, _) = add(conn, "combined.epub", &bytes);
        target(conn, &id, "1SA.8.1", "COMBINED OPENING", "book");
        target(conn, &id, "2SA.8.1", "COMBINED OPENING", "book");
        Ok(())
    })
    .unwrap();
}

#[test]
fn an_unanchored_printed_next_book_closes_previous_book_context_and_inheritance() {
    let bytes = fixtures::pdf_styled(&[
        vec![
            (24, 740, "Romans".into()),
            (11, 680, "Invented introduction.".into()),
        ],
        vec![(
            11,
            680,
            "NEXT BOOK BODY. Its opening is not aligned.".into(),
        )],
        vec![
            (18, 740, "Chapter 8".into()),
            (11, 680, "This belongs to the next unnamed book.".into()),
        ],
    ]);
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let (id, staged) = add(conn, "bounded.pdf", &bytes);
        let before = preserved(conn, &id);
        let mut document = staged.pdf_structure.unwrap();
        document.printed_contents.push(pdf_structure::PrintedDestination {
            id: "unreadable-book-opening".into(), title: "Unaligned book".into(),
            level: 1, page: 2, anchor_id: None,
        });
        let view = pdf_structures::replace(conn, &id, &staged.hash, &document)?;
        let book = view.headings.iter().find(|h| h.heading.title == "Romans").unwrap();
        let boundary = document.blocks.iter().find(|b| b.page == 2).unwrap().position;
        assert_eq!(book.end_position, boundary);
        let chapter = view.headings.iter().find(|h| h.heading.title == "Chapter 8").unwrap();
        assert!(chapter.canonical.is_none(), "later relative chapter borrowed Romans context");
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM source_passages p JOIN source_segments g ON g.id=p.segment_id WHERE p.source_id=?1 AND p.origin='inherited' AND g.page_number>=2", [&id], |r| r.get::<_, i64>(0))?, 0);
        assert_eq!(preserved(conn, &id), before);
        Ok(())
    }).unwrap();
}

#[test]
fn combined_pdf_heading_maps_both_books_without_arbitrary_relative_child_context() {
    let bytes = fixtures::pdf_styled(&[
        vec![
            (24, 740, "1 + 2 Samuel".into()),
            (11, 680, "Invented combined introduction.".into()),
        ],
        vec![
            (18, 740, "Chapter 8".into()),
            (11, 680, "No individual book is named here.".into()),
        ],
    ]);
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let (id, _) = add(conn, "combined.pdf", &bytes);
        let view = pdf_structures::view(conn, &id)?;
        let book = view
            .headings
            .iter()
            .find(|h| h.heading.title == "1 + 2 Samuel")
            .unwrap();
        assert_eq!(book.additional_canonicals, ["1SA.1-1SA.31", "2SA.1-2SA.24"]);
        assert!(book.canonical.is_none());
        assert!(view
            .headings
            .iter()
            .find(|h| h.heading.title == "Chapter 8")
            .unwrap()
            .canonical
            .is_none());
        let places: Vec<_> = ["1SA.8.1", "2SA.8.1"]
            .iter()
            .map(|query| {
                library::passage_source_locations(
                    conn,
                    &ScriptureRef::from_canonical(query).unwrap(),
                    &id,
                )
                .unwrap()
            })
            .collect();
        assert!(places.iter().all(|places| places.len() == 1
            && places[0].origin == LinkOrigin::Section
            && places[0].structure_scope.as_deref() == Some("book")));
        assert_eq!(places[0][0].segment_id, places[1][0].segment_id);
        assert_eq!(
            places[0][0].structure_node_id,
            places[1][0].structure_node_id
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn empty_spine_files_and_anchors_after_the_final_paragraph_do_not_borrow_another_opening() {
    let body = fixtures::xhtml(
        "<p>Only an incidental mention of Romans 9:1.</p><span id=\"after\"></span>",
    );
    let empty = fixtures::xhtml("<h1 id=\"book\">Romans</h1>");
    for href in ["empty.xhtml#book", "empty.xhtml", "body.xhtml#after"] {
        let nav = fixtures::xhtml(&format!(
            "<nav epub:type=\"toc\"><ol><li><a href=\"{href}\">Romans</a></li></ol></nav>"
        ));
        let bytes = fixtures::epub_raw("<item id=\"empty\" href=\"empty.xhtml\" media-type=\"application/xhtml+xml\"/><item id=\"body\" href=\"body.xhtml\" media-type=\"application/xhtml+xml\"/><item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>", "", "<itemref idref=\"empty\"/><itemref idref=\"body\"/>", &[("OEBPS/empty.xhtml", &empty),("OEBPS/body.xhtml", &body),("OEBPS/nav.xhtml", &nav)]);
        let db = Database::open_in_memory().unwrap();
        db.with_conn(|conn| {
            let (id, staged) = add(conn, "empty-destination.epub", &bytes);
            assert!(staged.contents.is_empty(), "unusable destination {href}");
            let places = library::passage_source_locations(
                conn,
                &ScriptureRef::from_canonical("ROM.9")?,
                &id,
            )?;
            assert_eq!(places.len(), 1);
            assert_eq!(places[0].origin, LinkOrigin::Explicit, "{href}");
            Ok(())
        })
        .unwrap();
    }
}
