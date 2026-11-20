//! Independent invented PDF variants: no third-party document, title or page
//! map is embedded here. Exercise typography, labels, columns and absent text.
#[path = "support/library_fixtures.rs"]
mod fixtures;
use abiding_word_lib::{
    db::{library, pdf_structures, Database},
    library::{
        adapters::{pdf, pdf_layout::PageOrder},
        contents::ContentsTarget,
        import, CommitSourceImport,
    },
};
use lopdf::{dictionary, Document, Object};
use std::fs;

type Lines = Vec<(f64, f64, f64, String)>;
fn page(title: &str, body: f64) -> Lines {
    let mut lines = vec![(55.0, 730.0, body * 1.4, title.into())];
    for n in 0..22 {
        lines.push((
            55.0,
            670.0 - n as f64 * 24.0,
            body,
            format!("Invented explanation for section {n}."),
        ));
    }
    lines
}
fn toc(entries: &[(&str, &str)]) -> Lines {
    let mut lines = vec![(55.0, 750.0, 20.0, "Contents".into())];
    for (n, (title, label)) in entries.iter().enumerate() {
        lines.push((
            55.0,
            690.0 - n as f64 * 35.0,
            10.0,
            format!("{title} . . . {label}"),
        ));
    }
    lines
}
fn with_labels(bytes: &[u8], ranges: Vec<Object>) -> Vec<u8> {
    let mut doc = Document::load_mem(bytes).unwrap();
    let root = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
    let labels = doc.add_object(dictionary! {"Nums"=>ranges});
    doc.get_object_mut(root)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set("PageLabels", labels);
    let mut result = Vec::new();
    doc.save_to(&mut result).unwrap();
    result
}
fn imported(db: &Database, root: &std::path::Path, bytes: &[u8]) -> String {
    let path = root.join("invented-variant.pdf");
    fs::write(&path, bytes).unwrap();
    let staged = import::stage(&path).unwrap();
    let id = abiding_word_lib::db::new_id();
    let relative = library::managed_relative_path(&id, &staged);
    let target = root.join(&relative);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, bytes).unwrap();
    db.with_conn(|c| {
        library::commit_import(
            c,
            &id,
            &staged,
            &CommitSourceImport {
                preview_id: staged.id.clone(),
                title: "Invented PDF variant".into(),
                language: Some("English".into()),
                ..Default::default()
            },
            &relative,
        )
    })
    .unwrap();
    id
}

#[test]
fn printed_roman_front_matter_and_restarted_decimal_labels_use_their_real_physical_pages() {
    let bytes = fixtures::pdf_sized(&[
        toc(&[("Opening note", "iii"), ("Romans", "1"), ("Acts", "2")]),
        page("Other preliminary material", 10.0),
        page("Opening note", 10.0),
        page("An argument about grace", 10.0),
        page("The earliest witnesses", 10.0),
    ]);
    let bytes = with_labels(
        &bytes,
        vec![
            0.into(),
            Object::Dictionary(dictionary! {"S"=>"r","St"=>1}),
            3.into(),
            Object::Dictionary(dictionary! {"S"=>"D","St"=>1}),
        ],
    );
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let id = imported(&db, temp.path(), &bytes);
    db.with_conn(|c| {
        let record = pdf_structures::load(c, &id)?.unwrap();
        assert_eq!(record.document.printed_contents.len(), 3);
        for (title, page) in [("Opening note", 3), ("Romans", 4), ("Acts", 5)] {
            let destination = record
                .document
                .printed_contents
                .iter()
                .find(|d| d.title == title)
                .unwrap();
            assert_eq!(destination.page, page);
            assert!(destination.anchor_id.is_some());
            let heading = record
                .headings
                .iter()
                .find(|h| h.heading.title == title)
                .unwrap();
            assert_eq!(heading.heading.anchor.page, page);
            assert_eq!(heading.heading.basis, "printed");
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn duplicate_printed_labels_never_choose_the_first_matching_page() {
    let bytes = fixtures::pdf_sized(&[
        toc(&[("Romans", "1"), ("Acts", "2"), ("Matthew", "3")]),
        page("Romans", 10.0),
        page("A different opening", 10.0),
        page("Acts", 10.0),
        page("Matthew", 10.0),
    ]);
    let bytes = with_labels(
        &bytes,
        vec![
            0.into(),
            Object::Dictionary(dictionary! {"P"=>Object::string_literal("front")}),
            1.into(),
            Object::Dictionary(dictionary! {"S"=>"D","St"=>1}),
            2.into(),
            Object::Dictionary(dictionary! {"S"=>"D","St"=>1}),
        ],
    );
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let id = imported(&db, temp.path(), &bytes);
    db.with_conn(|c| {
        let record = pdf_structures::load(c, &id)?.unwrap();
        assert_eq!(record.document.printed_contents.len(), 2);
        assert!(record
            .document
            .printed_contents
            .iter()
            .all(|d| d.title != "Romans"));
        let independent = record
            .headings
            .iter()
            .find(|h| h.heading.title == "Romans")
            .expect("the independent layout heading remains usable");
        assert_eq!(independent.heading.anchor.page, 2);
        assert_ne!(
            independent.heading.basis, "printed",
            "independent layout evidence must not be mislabeled as a resolved printed page"
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn two_column_printed_contents_preserve_each_column_order_and_do_not_fake_indentation() {
    let titles = [
        "Romans",
        "Acts",
        "Matthew",
        "Mark",
        "Luke",
        "John",
        "Genesis",
        "Exodus",
        "Leviticus",
        "Numbers",
        "Deuteronomy",
        "Joshua",
    ];
    let mut contents = vec![(55.0, 760.0, 20.0, "Contents".into())];
    // Intentionally draw rows across the two columns, not in reading order.
    for row in 0..6 {
        for column in 0..2 {
            let index = row + column * 6;
            contents.push((
                55.0 + column as f64 * 275.0,
                710.0 - row as f64 * 30.0,
                10.0,
                format!("{} .... {}", titles[index], index + 1),
            ));
        }
    }
    let mut pages = vec![contents];
    pages.extend(titles.iter().map(|title| page(title, 10.0)));
    let bytes = fixtures::pdf_sized(&pages);
    let text = pdf::read_pages(&bytes, "invented columns").unwrap();
    assert_eq!(
        text.orders[0],
        PageOrder::Columns(2),
        "the fixture must exercise actual multicolumn layout"
    );
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let id = imported(&db, temp.path(), &bytes);
    db.with_conn(|c| {
        let record = pdf_structures::load(c, &id)?.unwrap();
        assert_eq!(record.document.printed_contents.len(), titles.len());
        for (index, destination) in record.document.printed_contents.iter().enumerate() {
            assert_eq!(destination.title, titles[index]);
            assert_eq!(destination.page, index as u32 + 2);
            assert_eq!(
                destination.level, 1,
                "a right-hand column is not a nested section"
            );
        }
        let tree = pdf_structures::contents(c, &id)?;
        assert_eq!(tree.len(), titles.len());
        Ok(())
    })
    .unwrap();
}

#[test]
fn body_relative_heading_detection_survives_different_font_families_and_scales() {
    for (family, body) in [("Times-Roman", 8.5), ("Helvetica", 11.0), ("Courier", 14.0)] {
        let cover = vec![
            (55.0, 730.0, body * 4.0, "Field notes".into()),
            (55.0, 600.0, body * 3.0, "Reader companion".into()),
            (55.0, 470.0, body * 2.5, "Study edition".into()),
        ];
        let bytes = fixtures::pdf_sized(&[cover, page("Romans", body), page("Acts", body)]);
        let mut doc = Document::load_mem(&bytes).unwrap();
        for object in doc.objects.values_mut() {
            if let Ok(d) = object.as_dict_mut() {
                if d.get(b"Type").and_then(Object::as_name).ok() == Some(b"Font") {
                    d.set("BaseFont", Object::Name(family.as_bytes().to_vec()));
                }
            }
        }
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).unwrap();
        let temp = tempfile::tempdir().unwrap();
        let db = Database::open_in_memory().unwrap();
        let id = imported(&db, temp.path(), &bytes);
        db.with_conn(|c| {
            let view = pdf_structures::view(c, &id)?;
            for (title, page) in [("Romans", 2), ("Acts", 3)] {
                assert!(
                    view.headings.iter().any(|h| h.heading.title == title
                        && h.heading.anchor.page == page
                        && h.scope == "book"),
                    "lost genuine heading for {family} at {body}pt body size"
                );
            }
            Ok(())
        })
        .unwrap();
    }
}

#[test]
fn a_validated_contents_destination_without_any_text_is_page_only_and_not_an_indexed_heading() {
    let bytes = fixtures::pdf_sized(&[
        toc(&[
            ("Romans", "10"),
            ("Acts", "11"),
            ("Matthew", "12"),
            ("Diagram appendix", "13"),
        ]),
        page("Romans", 10.0),
        page("Acts", 10.0),
        page("Matthew", 10.0),
        Vec::new(),
    ]);
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let id = imported(&db, temp.path(), &bytes);
    db.with_conn(|c| {
        let record = pdf_structures::load(c, &id)?.unwrap();
        let destination = record
            .document
            .printed_contents
            .iter()
            .find(|d| d.title == "Diagram appendix")
            .unwrap();
        assert_eq!(destination.page, 5);
        assert!(destination.anchor_id.is_none());
        assert!(record.headings.iter().all(|h| h.heading.anchor.page != 5));
        let tree = pdf_structures::contents(c, &id)?;
        assert_eq!(
            tree.iter()
                .find(|i| i.title == "Diagram appendix")
                .unwrap()
                .target,
            Some(ContentsTarget::Page { page: 5 })
        );
        let stored: i64 = c.query_row(
            "SELECT count(*) FROM source_segments WHERE source_id=?1 AND page_number=5",
            [&id],
            |r| r.get(0),
        )?;
        assert_eq!(stored, 0, "a page target must not fabricate extracted text");
        Ok(())
    })
    .unwrap();
}
