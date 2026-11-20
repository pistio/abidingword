//! Invented publications only: printed TOC destinations are physical pages,
//! never body citations or a guessed biblical book order.
#[path = "support/library_fixtures.rs"]
mod fixtures;
use abiding_word_lib::{
    db::{library, pdf_structures, Database},
    library::{
        contents::ContentsTarget,
        import,
        pdf_build::{self, Job},
        pdf_structure::{Correction, PdfStructure},
        CommitSourceImport,
    },
};
use std::fs;

type Lines = Vec<(f64, f64, f64, String)>;
fn page(title: &str) -> Lines {
    let mut out = vec![(60.0, 730.0, 14.0, title.into())];
    for n in 0..22 {
        out.push((
            60.0,
            670.0 - n as f64 * 24.0,
            10.0,
            format!("Invented paragraph {n} supplies ordinary context for this discussion."),
        ));
    }
    out
}
fn cover() -> Lines {
    vec![
        (60.0, 730.0, 40.0, "Invented Handbook".into()),
        (60.0, 600.0, 30.0, "Research and Study".into()),
        (60.0, 470.0, 22.0, "A made-up edition".into()),
    ]
}
fn contents() -> Lines {
    let mut out = vec![(60.0, 740.0, 20.0, "Contents".into())];
    for (n, title) in [
        "Romans",
        "Background and approach",
        "Acts",
        "Matthew",
        "John",
        "Final observations",
        "Diagram appendix",
    ]
    .into_iter()
    .enumerate()
    {
        out.push((
            60.0,
            680.0 - n as f64 * 35.0,
            10.0,
            format!("{title} . . . {}", 10 + n),
        ));
    }
    out
}
fn imported(db: &Database, root: &std::path::Path, bytes: &[u8]) -> String {
    let path = root.join("invented.pdf");
    fs::write(&path, bytes).unwrap();
    let staged = import::stage(&path).unwrap();
    let id = abiding_word_lib::db::new_id();
    let relative = library::managed_relative_path(&id, &staged);
    let target = root.join(&relative);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, bytes).unwrap();
    db.with_conn(|conn| {
        library::commit_import(
            conn,
            &id,
            &staged,
            &CommitSourceImport {
                preview_id: staged.id.clone(),
                title: "Invented handbook".into(),
                language: Some("English".into()),
                ..Default::default()
            },
            &relative,
        )
    })
    .unwrap();
    id
}
fn publication() -> Vec<u8> {
    let mut romans = page("A letter addressed to the Roman church");
    romans.insert(
        1,
        (
            60.0,
            716.0,
            12.0,
            "A short subtitle immediately under the title".into(),
        ),
    );
    romans.retain(|(_, y, _, _)| !(*y < 620.0 && *y > 580.0));
    romans.push((60.0, 600.0, 12.0, "Historical setting".into()));
    let mut chart = page("Ordinary chart labels have no heading");
    chart[0].2 = 10.0;
    fixtures::pdf_sized(&[
        cover(),
        contents(),
        romans,
        page("Background and approach"),
        page("THE ACCTS OF THE APOSTLES"),
        page("Matthew"),
        page("John"),
        page("Final observations"),
        chart,
    ])
}
#[test]
fn oversized_cover_type_does_not_hide_smaller_book_headings_without_a_toc() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let id = imported(
        &db,
        root.path(),
        &fixtures::pdf_sized(&[cover(), page("Romans"), page("Acts")]),
    );
    db.with_conn(|c| {
        let view = pdf_structures::view(c, &id)?;
        assert!(view.headings.iter().any(|h| h.heading.title == "Romans"
            && h.scope == "book"
            && h.heading.anchor.page == 2));
        assert!(view
            .headings
            .iter()
            .any(|h| h.heading.title == "Acts" && h.scope == "book" && h.heading.anchor.page == 3));
        assert!(view.headings.iter().all(|h| h.heading.basis == "layout"));
        Ok(())
    })
    .unwrap();
}
#[test]
fn printed_titles_use_corroborated_offsets_and_real_openings_even_when_drawn_titles_differ() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let id = imported(&db, root.path(), &publication());
    db.with_conn(|c| {
        let view = pdf_structures::view(c, &id)?;
        for (title, page, original) in [
            ("Romans", 3, "A letter addressed to the Roman church"),
            ("Acts", 5, "THE ACCTS OF THE APOSTLES"),
        ] {
            let h = view
                .headings
                .iter()
                .find(|h| h.heading.title == title)
                .unwrap();
            assert_eq!(h.heading.basis, "printed");
            assert_eq!(h.heading.anchor.page, page);
            assert_eq!(h.scope, "book");
            assert_eq!(h.heading.anchor.original_text, original);
            let text: String = c.query_row(
                "SELECT content FROM source_segments WHERE source_id=?1 AND position=?2",
                rusqlite::params![id, h.heading.segment_position],
                |r| r.get(0),
            )?;
            assert_eq!(
                text.encode_utf16()
                    .skip(h.heading.start as usize)
                    .take((h.heading.end - h.heading.start) as usize)
                    .collect::<Vec<_>>(),
                original.encode_utf16().collect::<Vec<_>>()
            );
        }
        assert!(view.headings.iter().all(|h| h.heading.anchor.page > 2));
        assert!(!view
            .headings
            .iter()
            .any(|h| h.heading.title == "A short subtitle immediately under the title"));
        let tree = pdf_structures::contents(c, &id)?;
        assert_eq!(tree.len(), 7);
        let roman = tree.iter().find(|h| h.title == "Romans").unwrap();
        assert!(roman
            .children
            .iter()
            .any(|h| h.title == "Historical setting"));
        let chart = tree.iter().find(|h| h.title == "Diagram appendix").unwrap();
        assert_eq!(chart.target, Some(ContentsTarget::Page { page: 9 }));
        assert!(!view
            .headings
            .iter()
            .any(|h| h.heading.title == "Diagram appendix"));
        let doc = &pdf_structures::load(c, &id)?.unwrap().document;
        assert_eq!(doc.printed_contents.len(), 7);
        assert_eq!(
            doc.printed_contents
                .iter()
                .filter(|d| d.anchor_id.is_some())
                .count(),
            6
        );
        Ok(())
    })
    .unwrap();
}
#[test]
fn multilingual_contents_aliases_recover_real_destinations_when_old_analysis_is_rebuilt() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let expected = [
        ("Sprueche", "Sprichwörter", 3, "PRO.1-PRO.31"),
        ("Roemer", "Römer", 4, "ROM.1-ROM.16"),
        ("Matthaeus", "Matthäus", 5, "MAT.1-MAT.28"),
    ];
    let mut toc = vec![(60.0, 740.0, 20.0, "Inhaltsverzeichnis".into())];
    for (n, (title, _, _, _)) in expected.iter().enumerate() {
        toc.push((
            60.0,
            680.0 - n as f64 * 35.0,
            10.0,
            format!("{title} . . . {}", 10 + n),
        ));
    }
    // No PDF page labels or literal title matches: all three independent book
    // aliases must corroborate the printed-to-physical page offset.
    let bytes = fixtures::pdf_sized(&[
        cover(),
        toc,
        page("Sprichwörter"),
        page("Römer"),
        page("Matthäus"),
    ]);
    let id = imported(&db, root.path(), &bytes);
    let hash = db
        .with_conn(|c| {
            let record = pdf_structures::load(c, &id)?.unwrap();
            assert_eq!(record.document.printed_contents.len(), 3);
            assert!(record
                .document
                .blocks
                .iter()
                .all(|b| b.printed_label.is_none()));
            // Simulate saved analysis 4 metadata whose extraction omitted
            // these destinations. Merely resolving its remaining headings
            // cannot recreate the missing printed Contents entries.
            let mut old = record.document.clone();
            old.analysis = 4;
            old.printed_contents.clear();
            old.headings.retain(|h| h.basis != "printed");
            pdf_structures::store(c, &id, &record.hash, &old)?;
            let view = pdf_structures::view(c, &id)?;
            assert_eq!(view.analysis, 4);
            assert!(view.rebuild_recommended);
            assert!(pdf_structures::load(c, &id)?
                .unwrap()
                .document
                .printed_contents
                .is_empty());
            Ok(record.hash.clone())
        })
        .unwrap();

    let rebuilt = pdf_build::rebuild(&db, root.path(), &id, &Job::default()).unwrap();
    assert_eq!(rebuilt.analysis, 5);
    assert!(!rebuilt.rebuild_recommended);
    assert_eq!(rebuilt.file_hash.as_deref(), Some(hash.as_str()));
    db.with_conn(|c| {
        let record = pdf_structures::load(c, &id)?.unwrap();
        assert_eq!(record.document.printed_contents.len(), 3);
        for (title, original, page, canonical) in expected {
            let destination = record
                .document
                .printed_contents
                .iter()
                .find(|d| d.title == title)
                .unwrap();
            let h = rebuilt
                .headings
                .iter()
                .find(|h| h.heading.title == title)
                .unwrap();
            assert_eq!(destination.page, page);
            assert_eq!(
                destination.anchor_id.as_deref(),
                Some(h.heading.anchor.id().as_str())
            );
            assert_eq!(h.heading.anchor.page, page);
            assert_eq!(h.heading.anchor.original_text, original);
            assert_eq!(h.heading.basis, "printed");
            assert_eq!(h.scope, "book");
            assert_eq!(h.canonical.as_deref(), Some(canonical));
            let text: String = c.query_row(
                "SELECT content FROM source_segments WHERE source_id=?1 AND position=?2",
                rusqlite::params![id, h.heading.segment_position],
                |r| r.get(0),
            )?;
            assert_eq!(
                text.encode_utf16()
                    .skip(h.heading.start as usize)
                    .take((h.heading.end - h.heading.start) as usize)
                    .collect::<Vec<_>>(),
                original.encode_utf16().collect::<Vec<_>>()
            );
        }
        Ok(())
    })
    .unwrap();
}
#[test]
fn rebuild_keeps_user_corrections_and_old_documents_default_to_no_printed_page_destinations() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let id = imported(&db, root.path(), &publication());
    db.with_conn(|c| {
        let view = pdf_structures::view(c, &id)?;
        let h = view
            .headings
            .iter()
            .find(|h| h.heading.title == "Romans")
            .unwrap();
        pdf_structures::correct(
            c,
            &id,
            view.file_hash.as_deref().unwrap(),
            Correction {
                id: h.heading.id.clone(),
                position: h.heading.segment_position,
                title: "My Romans introduction".into(),
                level: 1,
                mapping: None,
                disabled: false,
                anchor_id: h.heading.anchor.id(),
                start: h.heading.start,
                end: h.heading.end,
                scope: None,
            },
        )?;
        let mut legacy =
            serde_json::to_value(&pdf_structures::load(c, &id)?.unwrap().document).unwrap();
        legacy.as_object_mut().unwrap().remove("printedContents");
        let old: PdfStructure = serde_json::from_value(legacy).unwrap();
        assert!(old.printed_contents.is_empty());
        Ok(())
    })
    .unwrap();
    let job = Job::default();
    let rebuilt = pdf_build::rebuild(&db, root.path(), &id, &job).unwrap();
    assert_eq!(rebuilt.corrections.len(), 1);
    db.with_conn(|c| {
        let tree = pdf_structures::contents(c, &id)?;
        assert!(tree.iter().any(|i| i.title == "My Romans introduction"));
        assert!(!tree.iter().any(|i| i.title == "Romans"));
        let mut disabled = rebuilt.corrections[0].clone();
        disabled.disabled = true;
        pdf_structures::correct(c, &id, rebuilt.file_hash.as_deref().unwrap(), disabled)?;
        let tree = pdf_structures::contents(c, &id)?;
        assert!(
            tree.iter()
                .all(|i| i.title != "Romans" && i.title != "My Romans introduction"),
            "a disabled anchored entry must not return as a page-only TOC item"
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn repeated_overprinting_does_not_turn_a_real_book_opening_into_a_body_mention() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let mut overprinted = page("A long introductory title addressed to this particular community");
    overprinted.insert(1, overprinted[0].clone());
    overprinted.insert(2, overprinted[0].clone());
    let bytes = fixtures::pdf_sized(&[
        cover(),
        contents(),
        overprinted,
        page("Background and approach"),
        page("Acts"),
        page("Matthew"),
        page("John"),
        page("Final observations"),
        page("Diagram appendix"),
    ]);
    let id = imported(&db, root.path(), &bytes);
    db.with_conn(|c| {
        let view = pdf_structures::view(c, &id)?;
        let h = view
            .headings
            .iter()
            .find(|h| h.heading.title == "Romans")
            .expect("printed book anchored despite original overprint");
        assert_eq!(h.heading.anchor.page, 3);
        assert_eq!(h.scope, "book");
        assert_eq!(h.heading.basis, "printed");
        assert!(
            h.heading
                .anchor
                .original_text
                .matches("A long introductory title")
                .count()
                >= 2,
            "original overprint stays untouched"
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_numbered_outline_inside_a_book_is_not_another_printed_page_toc() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let mut romans = page("A letter addressed to the Roman church");
    for (n, title) in ["Chapter ten", "Chapter eleven", "Chapter twelve"]
        .into_iter()
        .enumerate()
    {
        romans[n + 3].3 = format!("{title} . . . {}", 10 + n);
    }
    let bytes = fixtures::pdf_sized(&[
        cover(),
        contents(),
        romans,
        page("Background and approach"),
        page("Acts"),
        page("Matthew"),
        page("John"),
        page("Final observations"),
        page("Diagram appendix"),
    ]);
    let id = imported(&db, root.path(), &bytes);
    db.with_conn(|c| {
        let record = pdf_structures::load(c, &id)?.unwrap();
        assert_eq!(record.document.printed_contents.len(), 7);
        assert!(record
            .document
            .printed_contents
            .iter()
            .all(|d| !d.title.starts_with("Chapter ")));
        assert!(record
            .headings
            .iter()
            .all(|h| h.scope != "chapter" && h.scope != "passage"));
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_title_mismatch_without_a_corroborated_page_offset_never_guesses_a_book_destination() {
    let root = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    let toc = vec![
        (60.0, 740.0, 20.0, "Contents".into()),
        (60.0, 680.0, 10.0, "Romans . . . 1".into()),
        (60.0, 640.0, 10.0, "Acts . . . 2".into()),
        (60.0, 600.0, 10.0, "Matthew . . . 3".into()),
    ];
    let bytes = fixtures::pdf_sized(&[
        toc,
        page("Unrelated first opening"),
        page("Unrelated second opening"),
        page("Matthew"),
    ]);
    let id = imported(&db, root.path(), &bytes);
    db.with_conn(|c| {
        let record = pdf_structures::load(c, &id)?.unwrap();
        assert!(record.document.printed_contents.is_empty());
        assert!(record.headings.iter().all(|h| h.heading.basis != "printed"));
        Ok(())
    })
    .unwrap();
}
