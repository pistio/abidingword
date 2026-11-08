//! Source navigation metadata only; invented PDFs, no verified Bible import.
#[path = "support/library_fixtures.rs"]
mod fixtures;
use abiding_word_lib::{
    db::{library, pdf_diagnostic, pdf_structures, Database},
    library::{import, CommitSourceImport, GetSegments},
    scripture::reference::ScriptureRef,
};
use lopdf::{
    content::{Content, Operation},
    dictionary, Document, Object, Stream,
};
use std::fs;
fn imported(conn: &rusqlite::Connection, bytes: &[u8], root: &std::path::Path) -> String {
    let staging = tempfile::tempdir().unwrap();
    let path = staging.path().join("invented.pdf");
    fs::write(&path, bytes).unwrap();
    let staged = import::stage(&path).unwrap();
    let id = abiding_word_lib::db::new_id();
    let relative = library::managed_relative_path(&id, &staged);
    let target = root.join(&relative);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, bytes).unwrap();
    library::commit_import(
        conn,
        &id,
        &staged,
        &CommitSourceImport {
            preview_id: staged.id.clone(),
            title: "Invented Source".into(),
            language: Some("Deutsch".into()),
            ..Default::default()
        },
        &relative,
    )
    .unwrap();
    id
}
fn page(title: &str) -> Vec<(f64, f64, f64, String)> {
    let mut lines = vec![(60.0, 720.0, 24.0, title.into())];
    for n in 0..18 {
        lines.push((
            60.0,
            660.0 - n as f64 * 25.0,
            11.0,
            format!("Erfundener Absatz {n} zur Beobachtung. Apg 6,1 ist ein Verweis."),
        ));
    }
    lines
}
fn assert_target(
    conn: &rusqlite::Connection,
    id: &str,
    reference: &str,
    page: i64,
    basis: &str,
    scope: &str,
) {
    let places = library::passage_source_locations(
        conn,
        &ScriptureRef::from_canonical(reference).unwrap(),
        id,
    )
    .unwrap();
    let first = &places[0];
    assert_eq!(first.structure_basis.as_deref(), Some(basis));
    assert_eq!(first.structure_scope.as_deref(), Some(scope));
    let window = library::segments(
        conn,
        &GetSegments {
            source_id: id.into(),
            around_segment_id: Some(first.segment_id.clone()),
            around_page: None,
            offset: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(
        window
            .segments
            .iter()
            .find(|s| s.id == first.segment_id)
            .unwrap()
            .page_number,
        Some(page)
    );
    let contents = pdf_structures::contents(conn, id).unwrap();
    assert!(!contents.is_empty());
    let report =
        serde_json::to_value(pdf_diagnostic::diagnostic(conn, id, Some(reference)).unwrap())
            .unwrap();
    assert_eq!(report["query"]["readerTarget"]["physicalPage"], page);
}
#[test]
fn printed_index_uses_actual_openings_in_the_editions_order_never_the_listing_page() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let toc = vec![
        (60.0, 740.0, 20.0, "Inhalt".into()),
        (60.0, 680.0, 11.0, "Römer . . . 1".into()),
        (60.0, 640.0, 11.0, "Johannes . . . 2".into()),
        (60.0, 600.0, 11.0, "Apostelgeschichte . . . 3".into()),
    ];
    let bytes = fixtures::pdf_sized(&[
        toc,
        page("Römer"),
        page("Johannes"),
        page("Apostelgeschichte"),
    ]);
    db.with_conn(|c| {
        let id = imported(c, &bytes, root.path());
        assert_target(c, &id, "ACT.6", 4, "printed", "book");
        let view = pdf_structures::view(c, &id)?;
        assert_eq!(
            view.headings
                .iter()
                .filter(|h| h.heading.basis == "printed")
                .count(),
            3
        );
        assert!(!view
            .headings
            .iter()
            .any(|h| h.heading.anchor.page == 1 && h.canonical.is_some()));
        Ok(())
    })
    .unwrap();
}
#[test]
fn toc_book_destination_precedes_a_more_specific_layout_heading_and_body_mentions() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let mut bytes = fixtures::pdf_sized(&[
        page("Apostelgeschichte"),
        page("Kapitel 6"),
        page("Kapitel 7"),
    ]);
    let mut doc = Document::load_mem(&bytes).unwrap();
    let physical = doc.get_pages()[&1];
    let tree = doc.new_object_id();
    let entry=doc.add_object(dictionary!{"Title"=>Object::string_literal("Apostelgeschichte"),"Parent"=>tree,"Dest"=>vec![physical.into(),"XYZ".into(),60.into(),720.into(),Object::Null]});
    doc.objects.insert(
        tree,
        Object::Dictionary(
            dictionary! {"Type"=>"Outlines","First"=>entry,"Last"=>entry,"Count"=>1},
        ),
    );
    let catalog = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
    doc.get_object_mut(catalog)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set("Outlines", tree);
    bytes.clear();
    doc.save_to(&mut bytes).unwrap();
    db.with_conn(|c| {
        let id = imported(c, &bytes, root.path());
        assert_target(c, &id, "ACT.6", 1, "outline", "book");
        let report =
            serde_json::to_value(pdf_diagnostic::diagnostic(c, &id, Some("Apg 6"))?).unwrap();
        assert_eq!(report["query"]["candidates"][0]["evidenceTier"], 1);
        Ok(())
    })
    .unwrap();
}
fn unicode_pages(titles: &[&str]) -> Vec<u8> {
    let mut doc = Document::with_version("1.7");
    let pages = doc.new_object_id();
    let cmap=b"/CIDInit /ProcSet findresource begin 12 dict begin begincmap /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def /CMapName /SyntheticUnicode def /CMapType 2 def 1 begincodespacerange <0000> <FFFF> endcodespacerange 1 beginbfrange <0000> <FFFF> <0000> endbfrange endcmap CMapName currentdict /CMap defineresource pop end end";
    let unicode = doc.add_object(Stream::new(dictionary! {}, cmap.to_vec()));
    let descriptor=doc.add_object(dictionary!{"Type"=>"FontDescriptor","FontName"=>"SyntheticUnicode","Flags"=>4,"FontBBox"=>vec![0.into(),(-200).into(),1000.into(),1000.into()],"ItalicAngle"=>0,"Ascent"=>880,"Descent"=>-120,"CapHeight"=>700,"StemV"=>80});
    let cid=doc.add_object(dictionary!{"Type"=>"Font","Subtype"=>"CIDFontType2","BaseFont"=>"SyntheticUnicode","CIDSystemInfo"=>dictionary!{"Registry"=>Object::string_literal("Adobe"),"Ordering"=>Object::string_literal("Identity"),"Supplement"=>0},"FontDescriptor"=>descriptor,"DW"=>500});
    let font=doc.add_object(dictionary!{"Type"=>"Font","Subtype"=>"Type0","BaseFont"=>"SyntheticUnicode","Encoding"=>"Identity-H","DescendantFonts"=>vec![cid.into()],"ToUnicode"=>unicode});
    let mut kids = vec![];
    for title in titles {
        let operations: Vec<_> = page(title)
            .into_iter()
            .flat_map(|(x, y, size, text)| {
                [
                    Operation::new("BT", vec![]),
                    Operation::new("Tf", vec!["F1".into(), size.into()]),
                    Operation::new("Td", vec![x.into(), y.into()]),
                    Operation::new(
                        "Tj",
                        vec![Object::String(
                            text.encode_utf16().flat_map(u16::to_be_bytes).collect(),
                            lopdf::StringFormat::Hexadecimal,
                        )],
                    ),
                    Operation::new("ET", vec![]),
                ]
            })
            .collect();
        let stream = doc.add_object(Stream::new(
            dictionary! {},
            Content { operations }.encode().unwrap(),
        ));
        let p=doc.add_object(dictionary!{"Type"=>"Page","Parent"=>pages,"Contents"=>stream,"MediaBox"=>vec![0.into(),0.into(),600.into(),842.into()],"Resources"=>dictionary!{"Font"=>dictionary!{"F1"=>font}}});
        kids.push(p.into());
    }
    doc.objects.insert(
        pages,
        Object::Dictionary(dictionary! {"Type"=>"Pages","Count"=>titles.len() as i64,"Kids"=>kids}),
    );
    let catalog = doc.add_object(dictionary! {"Type"=>"Catalog","Pages"=>pages});
    doc.trailer.set("Root", catalog);
    let mut bytes = vec![];
    doc.save_to(&mut bytes).unwrap();
    bytes
}
#[test]
fn greek_new_testament_book_titles_work_only_as_source_metadata_offline() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let bytes = unicode_pages(&["ΚΑΤᾺ ΜΑΤΘΑῖΟΝ", "ΠΡΆΞΕΙΣ ΤῶΝ ἈΠΟΣΤΌΛΩΝ", "ΠΡῸΣ ῬΩΜΑΊΟΥΣ"]);
    db.with_conn(|c| {
        let id = imported(c, &bytes, root.path());
        assert_target(c, &id, "ACT.6", 2, "layout", "book");
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM bible_verses", [], |r| r
                .get::<_, i64>(0))?,
            0,
            "no Bible text imported or verified"
        );
        Ok(())
    })
    .unwrap();
}
#[test]
fn unsupported_or_ambiguous_traditions_remain_unmapped_and_keep_their_headings() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let bytes = unicode_pages(&[
        "Jud.",
        "1 Esdras",
        "Judith",
        "Greek Esther 6",
        "Apostelgeschichte",
    ]);
    db.with_conn(|c| {
        let id = imported(c, &bytes, root.path());
        let view = pdf_structures::view(c, &id)?;
        for title in ["Jud.", "1 Esdras", "Judith", "Greek Esther 6"] {
            let h = view
                .headings
                .iter()
                .find(|h| h.heading.title == title)
                .expect("original heading retained");
            assert!(h.canonical.is_none(), "ambiguous title mapped: {title}");
        }
        assert_target(c, &id, "ACT.6", 5, "layout", "book");
        Ok(())
    })
    .unwrap();
}
