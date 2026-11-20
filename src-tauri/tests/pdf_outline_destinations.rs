//! Invented PDF outlines: physical destinations and bookmark labels are
//! independent of the text drawn on the page and its printed page number.
#[path = "support/library_fixtures.rs"]
mod fixtures;

use abiding_word_lib::{
    db::{library, pdf_structures, Database},
    library::{import, CommitSourceImport, GetSegments},
    scripture::reference::ScriptureRef,
};
use lopdf::{dictionary, Document, Object};
use std::fs;

fn page_only_commentary() -> Vec<u8> {
    let pages: Vec<_> = [
        "Invented handbook",
        "ROMANS",
        "Life in the Spirit",
        "Hope and patience",
    ]
    .iter()
    .enumerate()
    .map(|(page, title)| {
        let mut lines = vec![
            (9, 815, "SYNTHETIC HANDBOOK".into()),
            (22, 750, (*title).into()),
        ];
        for n in 0..18 {
            lines.push((
                11,
                690 - n * 22,
                if page == 0 && n == 1 {
                    "An incidental introductory mention of Romans 8:20.".into()
                } else {
                    format!("Invented paragraph {page}-{n} explains this part of the argument.")
                },
            ));
        }
        lines.push((9, 30, (100 + page).to_string()));
        lines
    })
    .collect();
    let mut doc = Document::load_mem(&fixtures::pdf_styled(&pages)).unwrap();
    let physical: Vec<_> = doc.get_pages().into_values().collect();
    let root = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
    let tree = doc.new_object_id();
    let book = doc.new_object_id();
    let chapter = doc.new_object_id();
    let verses = doc.new_object_id();
    doc.objects.insert(
        tree,
        Object::Dictionary(dictionary! {
            "Type" => "Outlines", "First" => book, "Last" => book, "Count" => 3,
        }),
    );
    doc.objects.insert(
        book,
        Object::Dictionary(dictionary! {
            "Title" => Object::string_literal("Romans"), "Parent" => tree,
            "First" => chapter, "Last" => chapter, "Count" => 2,
            "Dest" => vec![physical[1].into(), "Fit".into()],
        }),
    );
    doc.objects.insert(
        chapter,
        Object::Dictionary(dictionary! {
            "Title" => Object::string_literal("Chapter 8"), "Parent" => book,
            "First" => verses, "Last" => verses, "Count" => 1,
            "Dest" => Object::string_literal("chapter-eight"),
        }),
    );
    doc.objects.insert(
        verses,
        Object::Dictionary(dictionary! {
            "Title" => Object::string_literal("Verses 18-25"), "Parent" => chapter,
            "A" => dictionary! {"S" => "GoTo", "D" => Object::Name(b"hope".to_vec())},
        }),
    );
    // Exercise both the Names/Dests name tree and the older catalogue Dests
    // dictionary. XYZ with a null top is also a valid page-only destination.
    let names_leaf = doc.add_object(dictionary! {
        "Names" => vec![
            Object::string_literal("chapter-eight"),
            Object::Dictionary(dictionary! {"D" => vec![physical[2].into(), "Fit".into()]}),
        ],
    });
    let labels = doc.add_object(dictionary! {
        "Nums" => vec![0.into(), Object::Dictionary(dictionary! {"S" => "D", "St" => 100})],
    });
    let catalogue = doc.get_object_mut(root).unwrap().as_dict_mut().unwrap();
    catalogue.set("Outlines", tree);
    catalogue.set("PageLabels", labels);
    catalogue.set(
        "Names",
        dictionary! {"Dests" => dictionary! {"Kids" => vec![Object::Reference(names_leaf)]}},
    );
    catalogue.set(
        "Dests",
        dictionary! {"hope" => vec![
            physical[3].into(), "XYZ".into(), Object::Null, Object::Null, Object::Null,
        ]},
    );
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    bytes
}

#[test]
fn named_page_destinations_use_physical_pages_and_keep_original_text_alignment() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("invented-outlines.pdf");
    fs::write(&input, page_only_commentary()).unwrap();
    let staged = import::stage(&input).unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let id = "synthetic-page-only-outline";
        let relative = library::managed_relative_path(id, &staged);
        library::commit_import(
            conn,
            id,
            &staged,
            &CommitSourceImport {
                preview_id: staged.id.clone(),
                title: "Invented outline destination test".into(),
                language: Some("English".into()),
                ..Default::default()
            },
            &relative,
        )?;
        let view = pdf_structures::view(conn, id)?;
        for (title, original, canonical, page) in [
            ("Romans", "ROMANS", "ROM.1-ROM.16", 2),
            ("Chapter 8", "Life in the Spirit", "ROM.8", 3),
            ("Verses 18-25", "Hope and patience", "ROM.8.18-ROM.8.25", 4),
        ] {
            let heading = view
                .headings
                .iter()
                .find(|h| h.heading.title == title)
                .unwrap_or_else(|| panic!("missing supplied outline: {title}"));
            assert_eq!(heading.heading.basis, "outline");
            assert_eq!(heading.canonical.as_deref(), Some(canonical));
            assert_eq!(heading.heading.anchor.page, page);
            assert_eq!(heading.heading.anchor.original_text, original);
            let window = library::segments(
                conn,
                &GetSegments {
                    source_id: id.into(),
                    around_segment_id: None,
                    around_page: None,
                    offset: Some(heading.heading.segment_position),
                    limit: Some(1),
                },
            )?;
            let segment = &window.segments[0];
            assert_eq!(segment.page_number, Some(i64::from(page)));
            assert_eq!(
                segment
                    .text
                    .encode_utf16()
                    .skip(heading.heading.start as usize)
                    .take((heading.heading.end - heading.heading.start) as usize)
                    .collect::<Vec<_>>(),
                original.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                segment.presentation.as_ref().unwrap().printed_label,
                Some((99 + page).to_string())
            );
        }
        for (reference, page, scope) in [
            ("ROM.8.20", 4, "passage"),
            ("ROM.8.1", 3, "chapter"),
            ("ROM.9", 2, "book"),
        ] {
            let locations = library::passage_source_locations(
                conn,
                &ScriptureRef::from_canonical(reference)?,
                id,
            )?;
            assert!(
                !locations.is_empty(),
                "no structural target for {reference}"
            );
            assert_eq!(locations[0].structure_basis.as_deref(), Some("outline"));
            assert_eq!(locations[0].structure_scope.as_deref(), Some(scope));
            let physical: i64 = conn.query_row(
                "SELECT page_number FROM source_segments WHERE id=?1",
                [&locations[0].segment_id],
                |row| row.get(0),
            )?;
            assert_eq!(physical, page);
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_page_only_bookmark_does_not_choose_between_parallel_column_headings() {
    use lopdf::{
        content::{Content, Operation},
        Stream,
    };
    let mut doc = Document::load_mem(&page_only_commentary()).unwrap();
    let page = doc.get_pages()[&3];
    let mut operations = Vec::new();
    for (x, title) in [(72, "Left discussion"), (330, "Right discussion")] {
        for n in 0..16 {
            let (size, y, text) = if n == 0 {
                (22, if x == 72 { 750 } else { 738 }, title.to_string())
            } else {
                (11, 700 - n * 24, format!("Invented paragraph {n}."))
            };
            operations.extend([
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), size.into()]),
                Operation::new("Td", vec![x.into(), y.into()]),
                Operation::new("Tj", vec![Object::string_literal(text)]),
                Operation::new("ET", vec![]),
            ]);
        }
    }
    let stream = doc.add_object(Stream::new(
        dictionary! {},
        Content { operations }.encode().unwrap(),
    ));
    doc.get_object_mut(page)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set("Contents", stream);
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("parallel-columns.pdf");
    fs::write(&path, bytes).unwrap();
    let staged = import::stage(&path).unwrap();
    assert!(!staged
        .pdf_structure
        .unwrap()
        .headings
        .iter()
        .any(|heading| heading.title == "Chapter 8"));
}
