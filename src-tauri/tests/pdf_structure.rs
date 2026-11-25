//! Invented documents only. These regressions cannot accept the unavailable
//! owner PDF; the handover retains a separate real-Windows acceptance gate.
#[path = "support/library_fixtures.rs"]
mod fixtures;
use abiding_word_lib::{
    db::{library, pdf_structures, Database},
    filesystem::DataDirs,
    library::{
        import,
        pdf_structure::{self, Correction},
        CommitSourceImport, GetSegments,
    },
    portability::backup,
    scripture::reference::ScriptureRef,
};
use lopdf::{dictionary, Document, Object};
use rusqlite::Connection;
use std::fs;

fn commentary(outline: bool) -> Vec<u8> {
    let pages: Vec<_> = ["Apostelgeschichte", "Kapitel 6", "Kapitel 7"]
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let mut lines = vec![
                (9, 815, "SYNTHETIC COMMENTARY".into()),
                (if i == 0 { 24 } else { 19 }, 750, (*title).into()),
            ];
            for n in 0..18 {
                lines.push((
                    11,
                    690 - n * 22,
                    if n == 2 && i != 1 {
                        "Ein beiläufiger Verweis: Apg 6,1.".into()
                    } else {
                        format!(
                            "Erfundener Absatz {i}-{n} über Aufbau und Zusammenhang des Textes."
                        )
                    },
                ));
            }
            lines.push((8, 120, "Anmerkung: Apg 6,2; Joh 3,16.".into()));
            lines.push((9, 30, (100 + i).to_string()));
            lines
        })
        .collect();
    let bytes = fixtures::pdf_styled(&pages);
    let mut doc = Document::load_mem(&bytes).unwrap();
    let root = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
    let physical: Vec<_> = doc.get_pages().into_values().collect();
    let labels = doc.add_object(
        dictionary! {"Nums"=>vec![0.into(),Object::Dictionary(dictionary!{"S"=>"D","St"=>100})]},
    );
    doc.get_object_mut(root)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set("PageLabels", labels);
    if outline {
        let tree = doc.new_object_id();
        let book = doc.new_object_id();
        let six = doc.new_object_id();
        let seven = doc.new_object_id();
        doc.objects.insert(
            tree,
            Object::Dictionary(
                dictionary! {"Type"=>"Outlines","First"=>book,"Last"=>book,"Count"=>3},
            ),
        );
        let dest = |p: usize| {
            Object::Array(vec![
                physical[p].into(),
                "XYZ".into(),
                72.into(),
                750.into(),
                Object::Null,
            ])
        };
        doc.objects.insert(book,Object::Dictionary(dictionary!{"Title"=>Object::string_literal("Apostelgeschichte"),"Parent"=>tree,"First"=>six,"Last"=>seven,"Count"=>2,"Dest"=>dest(0)}));
        doc.objects.insert(six,Object::Dictionary(dictionary!{"Title"=>Object::string_literal("Kapitel 6"),"Parent"=>book,"Next"=>seven,"Dest"=>dest(1)}));
        doc.objects.insert(seven,Object::Dictionary(dictionary!{"Title"=>Object::string_literal("Kapitel 7"),"Parent"=>book,"Prev"=>six,"Dest"=>dest(2)}));
        doc.get_object_mut(root)
            .unwrap()
            .as_dict_mut()
            .unwrap()
            .set("Outlines", tree);
    }
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    out
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
            title: "Invented commentary".into(),
            language: Some("Deutsch".into()),
            ..Default::default()
        },
        &relative,
    )
    .unwrap();
    id
}
fn window(conn: &Connection, id: &str) -> abiding_word_lib::library::SegmentWindow {
    library::segments(
        conn,
        &GetSegments {
            source_id: id.into(),
            around_segment_id: None,
            around_page: None,
            offset: None,
            limit: Some(500),
        },
    )
    .unwrap()
}
#[test]
fn import_index_result_and_reader_target_prefer_the_chapter_not_earlier_mentions() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &commentary(true), temp.path());
        let view = pdf_structures::view(c, &id)?;
        let chapter = view
            .headings
            .iter()
            .find(|h| h.heading.title == "Kapitel 6")
            .unwrap();
        assert_eq!(chapter.canonical.as_deref(), Some("ACT.6"));
        assert_eq!(chapter.heading.basis, "outline");
        assert!(chapter.path.contains("Apostelgeschichte"));
        let places =
            library::passage_source_locations(c, &ScriptureRef::from_canonical("ACT.6")?, &id)?;
        assert_eq!(places[0].structure_scope.as_deref(), Some("chapter"));
        assert_eq!(places[0].structure_node_id.as_deref(),Some(chapter.heading.id.as_str()));
        let w = library::segments(
            c,
            &GetSegments {
                source_id: id.clone(),
                around_segment_id: Some(places[0].segment_id.clone()),
                around_page: None,
                offset: None,
                limit: Some(150),
            },
        )?;
        let target = w
            .segments
            .iter()
            .find(|s| s.id == places[0].segment_id)
            .unwrap();
        assert_eq!(
            target
                .text
                .encode_utf16()
                .skip(chapter.heading.start as usize)
                .take((chapter.heading.end - chapter.heading.start) as usize)
                .collect::<Vec<_>>(),
            "Kapitel 6".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(target.page_number, Some(2));
        let presentation = target.presentation.as_ref().unwrap();
        assert_eq!(presentation.printed_label.as_deref(), Some("101"));
        assert_eq!(presentation.headings[0].anchor.page, 2);
        assert!(presentation.headings[0].anchor.top_fraction() > 0.05);
        let fallback =
            library::passage_source_locations(c, &ScriptureRef::from_canonical("ACT.8")?, &id)?;
        assert_eq!(fallback[0].structure_scope.as_deref(), Some("book"));
        assert_eq!(fallback[0].matched_text, "Apostelgeschichte");
        if let Some(dir)=std::env::var_os("AW_PDF_REVIEW_DIR") {
            let dir=std::path::PathBuf::from(dir);fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("structure-review.pdf"),commentary(true)).unwrap();
            fs::write(dir.join("structure-review.json"),serde_json::to_vec_pretty(&serde_json::json!({"source":library::get(c,&id)?,"window":w,"structure":view,"locations":places,"contents":pdf_structures::contents(c,&id)?})).unwrap()).unwrap();
        }
        if let Some(profile)=std::env::var_os("AW_PDF_NATIVE_PROFILE") {
            let dirs=DataDirs::ensure(std::path::PathBuf::from(profile))?;
            let profile_db=Database::open(&dirs.database_file(),&dirs.pre_migration_backups())?;
            profile_db.with_conn(|conn| {
                assert_eq!(conn.query_row("SELECT count(*) FROM sources",[],|r|r.get::<_,i64>(0))?,0,"Native smoke fixture requires an empty disposable library");
                imported(conn,&commentary(true),&dirs.sources());Ok(())
            })?;
        }
        let contents = pdf_structures::contents(c, &id)?;
        assert!(!contents.is_empty());
        assert!(matches!(
            contents[0].target,
            Some(abiding_word_lib::library::contents::ContentsTarget::Segment { .. })
        ));
        Ok(())
    })
    .unwrap();
}
#[test]
fn layout_only_structure_keeps_evidence_and_uniform_prose_does_not_invent_a_section() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &commentary(false), temp.path());
        let view = pdf_structures::view(c, &id)?;
        assert!(view
            .headings
            .iter()
            .any(|h| h.canonical.as_deref() == Some("ACT.6") && h.heading.basis == "layout"));
        let bytes = fixtures::pdf(
            &[&[
                "This ordinary paragraph mentions Acts 6:1.",
                "A separate ordinary paragraph continues the discussion.",
            ]],
            None,
            None,
        );
        let id = imported(c, &bytes, temp.path());
        assert!(pdf_structures::view(c, &id)?.headings.is_empty());
        let places =
            library::passage_source_locations(c, &ScriptureRef::from_canonical("ACT.6")?, &id)?;
        assert!(!places.is_empty());
        assert!(places
            .iter()
            .all(|p| p.origin == abiding_word_lib::library::sections::LinkOrigin::Explicit));
        Ok(())
    })
    .unwrap();
}
fn patch(h: &pdf_structure::ResolvedHeading) -> Correction {
    Correction {
        id: h.heading.id.clone(),
        position: h.heading.segment_position,
        title: h.heading.title.clone(),
        level: h.heading.level,
        mapping: h.canonical.clone(),
        scope: Some(h.scope.clone()),
        disabled: false,
        anchor_id: h.heading.anchor.id(),
        start: h.heading.start,
        end: h.heading.end,
    }
}
#[test]
fn corrections_reindex_atomically_keep_text_ids_and_survive_full_backup_restore() {
    let a = tempfile::tempdir().unwrap();
    let dirs = DataDirs::ensure(a.path().join("data")).unwrap();
    let db = Database::open(&dirs.database_file(), &dirs.pre_migration_backups()).unwrap();
    let (id, hash, before, preserved) = db
        .with_conn(|c| {
            let id = imported(c, &commentary(true), &dirs.sources());
            let before = window(c, &id)
                .segments
                .iter()
                .map(|s| (s.id.clone(), s.text.clone(), s.locator.clone()))
                .collect::<Vec<_>>();
            let view = pdf_structures::view(c, &id)?;
            let hash = view.file_hash.unwrap();
            let h = view
                .headings
                .iter()
                .find(|h| h.heading.title == "Kapitel 6")
                .unwrap();
            use abiding_word_lib::{
                db::{findings, projects, source_annotations, studies, workspaces},
                domain::{CreateFinding, CreateStudy, FindingType},
                library::{
                    annotations::{CreateSourceAnnotation, TextPoint},
                    AddFindingSource,
                },
            };
            let project = projects::root(c, &workspaces::get_default(c)?.id, "Studies")?;
            let study = studies::create(
                c,
                &CreateStudy {
                    project_id: project.id,
                    title: "Invented provenance study".into(),
                    ..Default::default()
                },
            )?;
            let finding = findings::create(
                c,
                &CreateFinding {
                    study_id: study.study.id,
                    finding_type: FindingType::Quotation,
                    content: Some("An independently written note".into()),
                    section_id: None,
                    passage: None,
                },
            )?;
            let segment = &window(c, &id).segments[h.heading.segment_position as usize];
            findings::add_source(
                c,
                &AddFindingSource {
                    finding_id: finding.id.clone(),
                    segment_id: Some(segment.id.clone()),
                    cited: Default::default(),
                },
            )?;
            source_annotations::create(
                c,
                &CreateSourceAnnotation {
                    source_id: id.clone(),
                    kind: "highlight".into(),
                    color: Some("amber".into()),
                    start: Some(TextPoint {
                        segment_id: segment.id.clone(),
                        offset: h.heading.start,
                    }),
                    end: Some(TextPoint {
                        segment_id: segment.id.clone(),
                        offset: h.heading.end,
                    }),
                    quote: Some("Kapitel 6".into()),
                    ..Default::default()
                },
            )?;
            let preserved = (
                finding.id.clone(),
                serde_json::to_value(findings::get(c, &finding.id)?).unwrap(),
                serde_json::to_value(source_annotations::list_for_source(c, &id)?).unwrap(),
            );
            let mut p = patch(h);
            p.mapping = Some("ACT.5".into());
            p.title = "Corrected chapter".into();
            assert!(pdf_structures::correct(c, &id, "wrong hash", p.clone()).is_err());
            let mut invalid = p.clone();
            invalid.anchor_id = "wrong".into();
            assert!(pdf_structures::correct(c, &id, &hash, invalid).is_err());
            let corrected = pdf_structures::correct(c, &id, &hash, p)?;
            assert!(corrected.can_undo);
            assert_eq!(
                library::passage_source_locations(c, &ScriptureRef::from_canonical("ACT.5")?, &id)?
                    [0]
                .matched_text,
                "Corrected chapter"
            );
            library::reindex_passages(c, &id)?;
            Ok((id, hash, before, preserved))
        })
        .unwrap();
    let archive = a.path().join("structure.awbackup");
    backup::create(&db, &dirs, &archive).unwrap();
    let b = tempfile::tempdir().unwrap();
    let there = DataDirs::ensure(b.path().join("restored")).unwrap();
    let restored = Database::open(&there.database_file(), &there.pre_migration_backups()).unwrap();
    backup::restore(&restored, &there, &archive).unwrap();
    restored
        .with_conn(|c| {
            let view = pdf_structures::view(c, &id)?;
            assert_eq!(view.corrections.len(), 1);
            assert!(view.can_undo);
            assert_eq!(
                serde_json::to_value(abiding_word_lib::db::findings::get(c, &preserved.0)?)
                    .unwrap(),
                preserved.1
            );
            assert_eq!(
                serde_json::to_value(abiding_word_lib::db::source_annotations::list_for_source(
                    c, &id
                )?)
                .unwrap(),
                preserved.2
            );
            assert_eq!(
                window(c, &id)
                    .segments
                    .iter()
                    .map(|s| (s.id.clone(), s.text.clone(), s.locator.clone()))
                    .collect::<Vec<_>>(),
                before
            );
            let undone = pdf_structures::undo(c, &id, &hash, false)?;
            assert!(undone.corrections.is_empty());
            assert_eq!(
                library::passage_source_locations(c, &ScriptureRef::from_canonical("ACT.6")?, &id)?
                    [0]
                .matched_text,
                "Kapitel 6"
            );
            assert!(!c.prepare("PRAGMA foreign_key_check")?.exists([])?);
            Ok(())
        })
        .unwrap();
}
#[test]
fn clearing_a_user_mapping_is_not_silently_reinferred_and_reset_is_undoable() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &commentary(true), temp.path());
        let view = pdf_structures::view(c, &id)?;
        let hash = view.file_hash.unwrap();
        let mut p = patch(
            view.headings
                .iter()
                .find(|h| h.heading.title == "Kapitel 6")
                .unwrap(),
        );
        p.mapping = None;
        p.scope = None;
        let corrected = pdf_structures::correct(c, &id, &hash, p)?;
        assert!(corrected
            .headings
            .iter()
            .find(|h| h.heading.title == "Kapitel 6")
            .unwrap()
            .canonical
            .is_none());
        pdf_structures::undo(c, &id, &hash, true)?;
        assert!(pdf_structures::view(c, &id)?.corrections.is_empty());
        assert_eq!(
            pdf_structures::undo(c, &id, &hash, false)?
                .corrections
                .len(),
            1
        );
        Ok(())
    })
    .unwrap();
}

fn tagged() -> Vec<u8> {
    use lopdf::{
        content::{Content, Operation},
        Stream,
    };
    let mut doc = Document::load_mem(&commentary(false)).unwrap();
    let tree = doc.new_object_id();
    let mut nodes = Vec::new();
    for (page_no, page_id) in doc.get_pages() {
        let mut content = Content::decode(&doc.get_page_content(page_id).unwrap()).unwrap();
        let mut operations = Vec::new();
        let mut mcid = 0;
        for op in content.operations.drain(..) {
            if op.operator == "BT" {
                operations.push(Operation::new(
                    "BDC",
                    vec![
                        "Span".into(),
                        Object::Dictionary(dictionary! {"MCID"=>mcid}),
                    ],
                ));
            }
            let end = op.operator == "ET";
            operations.push(op);
            if end {
                operations.push(Operation::new("EMC", vec![]));
                let role = if mcid == 0 {
                    "Artifact"
                } else if mcid == 1 {
                    if page_no == 1 {
                        "MainTitle"
                    } else {
                        "H2"
                    }
                } else if mcid == 20 {
                    "Note"
                } else if mcid == 3 {
                    "BlockQuote"
                } else if mcid == 4 {
                    "L"
                } else {
                    "P"
                };
                let id = doc.add_object(
                    dictionary! {"Type"=>"StructElem","S"=>role,"P"=>tree,"Pg"=>page_id,"K"=>mcid},
                );
                nodes.push(id.into());
                mcid += 1;
            }
        }
        let stream = doc.add_object(Stream::new(
            dictionary! {},
            Content { operations }.encode().unwrap(),
        ));
        doc.get_object_mut(page_id)
            .unwrap()
            .as_dict_mut()
            .unwrap()
            .set("Contents", stream);
    }
    let first = *doc.get_pages().values().next().unwrap();
    let unmatched =
        doc.add_object(dictionary! {"Type"=>"StructElem","S"=>"H2","P"=>tree,"Pg"=>first,"K"=>999});
    nodes.push(unmatched.into());
    doc.objects.insert(tree,Object::Dictionary(dictionary!{"Type"=>"StructTreeRoot","K"=>nodes,"RoleMap"=>dictionary!{"MainTitle"=>"H1"}}));
    let root = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
    doc.get_object_mut(root)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set("StructTreeRoot", tree);
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    bytes
}
#[test]
fn tagged_headings_roles_and_unaligned_tags_keep_their_own_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &tagged(), temp.path());
        let view = pdf_structures::view(c, &id)?;
        assert!(view.tagged);
        assert!(view.unaligned_tags > 0);
        assert!(view
            .headings
            .iter()
            .any(|h| h.heading.basis == "tagged" && h.canonical.as_deref() == Some("ACT.6")));
        let record = pdf_structures::load(c, &id)?.unwrap();
        assert!(record
            .document
            .blocks
            .iter()
            .any(|b| b.role == "quote" || b.runs.iter().any(|r| r.role == "quote")));
        assert!(record
            .document
            .blocks
            .iter()
            .any(|b| b.role == "list" || b.runs.iter().any(|r| r.role == "list")));
        assert!(record.document.blocks.iter().any(|b| b
            .runs
            .iter()
            .any(|r| matches!(r.role.as_str(), "header" | "artifact"))));
        Ok(())
    })
    .unwrap();
}

/// Small self-contained Unicode CMap; invented content, no embedded third-party
/// font. Two columns exercise extraction order independently of Bible imports.
fn korean_columns() -> Vec<u8> {
    use lopdf::{
        content::{Content, Operation},
        Stream,
    };
    let mut doc = Document::with_version("1.7");
    let pages = doc.new_object_id();
    let cmap=b"/CIDInit /ProcSet findresource begin 12 dict begin begincmap /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def /CMapName /SyntheticUnicode def /CMapType 2 def 1 begincodespacerange <0000> <FFFF> endcodespacerange 1 beginbfrange <0000> <FFFF> <0000> endbfrange endcmap CMapName currentdict /CMap defineresource pop end end";
    let unicode = doc.add_object(Stream::new(dictionary! {}, cmap.to_vec()));
    let descriptor=doc.add_object(dictionary!{"Type"=>"FontDescriptor","FontName"=>"SyntheticUnicode","Flags"=>4,"FontBBox"=>vec![0.into(),(-200).into(),1000.into(),1000.into()],"ItalicAngle"=>0,"Ascent"=>880,"Descent"=>-120,"CapHeight"=>700,"StemV"=>80});
    let cid=doc.add_object(dictionary!{"Type"=>"Font","Subtype"=>"CIDFontType2","BaseFont"=>"SyntheticUnicode","CIDSystemInfo"=>dictionary!{"Registry"=>Object::string_literal("Adobe"),"Ordering"=>Object::string_literal("Identity"),"Supplement"=>0},"FontDescriptor"=>descriptor,"DW"=>500});
    let font=doc.add_object(dictionary!{"Type"=>"Font","Subtype"=>"Type0","BaseFont"=>"SyntheticUnicode","Encoding"=>"Identity-H","DescendantFonts"=>vec![cid.into()],"ToUnicode"=>unicode});
    let lines = [
        (60, 750, 24, "사도행전"),
        (60, 700, 20, "6장"),
        (60, 650, 16, "6:1–7"),
    ];
    let mut operations = Vec::new();
    let mut line = |x: i64, y: i64, size: i64, text: &str| {
        operations.extend([
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
        ]);
    };
    for (x, y, size, text) in lines {
        line(x, y, size, text);
    }
    for x in [60, 325] {
        for n in 0..18 {
            line(
                x,
                600 - n * 22,
                11,
                if x == 60 {
                    "관찰과 문맥에 관한 짧은 예시 문단입니다."
                } else {
                    "본문의 구조를 살펴보는 다른 예시 문단입니다."
                },
            );
        }
    }
    line(60, 95, 8, "각주: 행 7:1, 요 3:16을 참조하세요.");
    let stream = doc.add_object(Stream::new(
        dictionary! {},
        Content { operations }.encode().unwrap(),
    ));
    let page=doc.add_object(dictionary!{"Type"=>"Page","Parent"=>pages,"Contents"=>stream,"MediaBox"=>vec![0.into(),0.into(),600.into(),842.into()],"Resources"=>dictionary!{"Font"=>dictionary!{"F1"=>font}}});
    doc.objects.insert(
        pages,
        Object::Dictionary(dictionary! {"Type"=>"Pages","Kids"=>vec![page.into()],"Count"=>1}),
    );
    let outlines = doc.new_object_id();
    let mut parent = outlines;
    let mut nodes = Vec::new();
    for (_, y, _, title) in lines {
        let node = doc.new_object_id();
        nodes.push(node);
        doc.objects.insert(node,Object::Dictionary(dictionary!{"Title"=>Object::String([vec![0xfe,0xff],title.encode_utf16().flat_map(u16::to_be_bytes).collect()].concat(),lopdf::StringFormat::Hexadecimal),"Parent"=>parent,"Dest"=>vec![page.into(),"XYZ".into(),60.into(),y.into(),Object::Null]}));
        if parent != outlines {
            let d = doc.get_object_mut(parent).unwrap().as_dict_mut().unwrap();
            d.set("First", node);
            d.set("Last", node);
            d.set("Count", 1);
        }
        parent = node;
    }
    doc.objects.insert(
        outlines,
        Object::Dictionary(
            dictionary! {"Type"=>"Outlines","First"=>nodes[0],"Last"=>nodes[0],"Count"=>3},
        ),
    );
    let root = doc.add_object(dictionary! {"Type"=>"Catalog","Pages"=>pages,"Outlines"=>outlines});
    doc.trailer.set("Root", root);
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    bytes
}
#[test]
fn korean_unicode_book_chapter_and_verse_headings_keep_coordinates_in_a_multicolumn_pdf() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &korean_columns(), temp.path());
        let view = pdf_structures::view(c, &id)?;
        assert!(view
            .headings
            .iter()
            .any(|h| h.heading.title == "사도행전" && h.scope == "book"));
        assert!(view
            .headings
            .iter()
            .any(|h| h.heading.title == "6장" && h.canonical.as_deref() == Some("ACT.6")));
        assert!(view.headings.iter().any(
            |h| h.heading.title == "6:1–7" && h.canonical.as_deref() == Some("ACT.6.1-ACT.6.7")
        ));
        let record = pdf_structures::load(c, &id)?.unwrap();
        assert!(record
            .document
            .blocks
            .iter()
            .flat_map(|b| &b.anchors)
            .any(|a| a.x0 > 300.0));
        let places =
            library::passage_source_locations(c, &ScriptureRef::from_canonical("ACT.6.2")?, &id)?;
        assert_eq!(places[0].structure_scope.as_deref(), Some("passage"));
        Ok(())
    })
    .unwrap();
}

#[test]
fn english_headings_map_but_orphan_numeric_titles_do_not_borrow_context_from_body_citations() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        for (titles,expected) in [(["Acts","Chapter 6","Acts 6:1-7"],true),(["6","7","8"],false)] {
            let pages=titles.iter().enumerate().map(|(i,title)| {
                let mut lines=vec![(if i==0 {24}else{18},750,(*title).into())];
                for n in 0..20 {lines.push((11,690-n*22,format!("Invented paragraph {n} mentions Acts 6:1 without establishing section context.")));}lines
            }).collect::<Vec<_>>();
            let id=imported(c,&fixtures::pdf_styled(&pages),temp.path());
            let view=pdf_structures::view(c,&id)?;
            if expected {assert!(!view.headings.is_empty(),"English heading evidence");assert!(view.headings.iter().any(|h|h.canonical.as_deref()==Some("ACT.6")));}
            else {assert!(view.headings.is_empty(),"Bare numbers are ambiguous pagination, not established sections");}
        }
        Ok(())
    }).unwrap();
}
#[test]
fn disabled_parent_does_not_resurrect_from_raw_outline_titles_and_removed_sources_drop_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &commentary(true), temp.path());
        let view = pdf_structures::view(c, &id)?;
        let hash = view.file_hash.unwrap();
        let mut p = patch(
            view.headings
                .iter()
                .find(|h| h.heading.title == "Apostelgeschichte")
                .unwrap(),
        );
        p.disabled = true;
        let corrected = pdf_structures::correct(c, &id, &hash, p)?;
        assert!(corrected.headings.iter().all(|h| h.canonical.is_none()));
        pdf_structures::undo(c, &id, &hash, false)?;
        c.execute(
            "UPDATE source_pdf_structures SET document='{}',etag='corrupt-test' WHERE source_id=?1",
            [&id],
        )?;
        assert!(pdf_structures::view(c, &id).is_err());
        assert!(
            !window(c, &id).segments.is_empty(),
            "Bad optional metadata must not prevent ordinary Reading"
        );
        library::remove(c, &id)?;
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM source_pdf_structures WHERE source_id=?1",
                [&id],
                |r| r.get::<_, i64>(0)
            )?,
            0
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_new_outline_book_group_without_a_destination_does_not_inherit_the_previous_book() {
    let mut doc = Document::load_mem(&commentary(true)).unwrap();
    let root = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
    let tree = doc
        .get_object(root)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"Outlines")
        .unwrap()
        .as_reference()
        .unwrap();
    let book = doc
        .get_object(tree)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"First")
        .unwrap()
        .as_reference()
        .unwrap();
    let chapter = doc
        .get_object(book)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"First")
        .unwrap()
        .as_reference()
        .unwrap();
    let seven = doc
        .get_object(chapter)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"Next")
        .unwrap()
        .as_reference()
        .unwrap();
    let group=doc.add_object(dictionary!{"Title"=>Object::string_literal("Matthew"),"Parent"=>tree,"Prev"=>book,"First"=>seven,"Last"=>seven,"Count"=>1});
    {
        let b = doc.get_object_mut(book).unwrap().as_dict_mut().unwrap();
        b.set("Next", group);
        b.set("Last", chapter);
        b.set("Count", 1);
    }
    doc.get_object_mut(chapter)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .remove(b"Next");
    {
        let s = doc.get_object_mut(seven).unwrap().as_dict_mut().unwrap();
        s.set("Parent", group);
        s.remove(b"Prev");
    }
    doc.get_object_mut(tree)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set("Last", group);
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &bytes, temp.path());
        let view = pdf_structures::view(c, &id)?;
        assert_eq!(
            view.headings
                .iter()
                .find(|h| h.heading.title == "Kapitel 7")
                .unwrap()
                .canonical
                .as_deref(),
            Some("MAT.7")
        );
        let contents = pdf_structures::contents(c, &id)?;
        assert!(contents.iter().any(|i| i.title == "Matthew"
            && i.target.is_none()
            && i.children.iter().any(|c| c.title == "Kapitel 7")));
        Ok(())
    })
    .unwrap();
}

#[test]
fn an_ambiguous_outline_coordinate_cannot_invent_a_section_between_two_columns() {
    let mut doc = Document::load_mem(&korean_columns()).unwrap();
    let root = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
    let tree = doc
        .get_object(root)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"Outlines")
        .unwrap()
        .as_reference()
        .unwrap();
    let book = doc
        .get_object(tree)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"First")
        .unwrap()
        .as_reference()
        .unwrap();
    let chapter = doc
        .get_object(book)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"First")
        .unwrap()
        .as_reference()
        .unwrap();
    let leaf = doc
        .get_object(chapter)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"First")
        .unwrap()
        .as_reference()
        .unwrap();
    let page = *doc.get_pages().values().next().unwrap();
    let node = doc.get_object_mut(leaf).unwrap().as_dict_mut().unwrap();
    node.set("Title", Object::string_literal("Acts 6:20"));
    node.set(
        "Dest",
        vec![
            page.into(),
            "XYZ".into(),
            Object::Null,
            600.into(),
            Object::Null,
        ],
    );
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|c| {
        let id = imported(c, &bytes, temp.path());
        let view = pdf_structures::view(c, &id)?;
        assert!(!view.headings.iter().any(|h| h.heading.title == "Acts 6:20"));
        Ok(())
    })
    .unwrap();
}

#[test]
fn a_pdf_changed_after_preview_cannot_commit_anchors_for_different_managed_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("source.pdf");
    let managed = temp.path().join("managed.pdf");
    fs::write(&input, commentary(true)).unwrap();
    let staged = import::stage(&input).unwrap();
    fs::write(&input, commentary(false)).unwrap();
    let error = abiding_word_lib::library::files::copy_verified(
        &input,
        &managed,
        &staged.hash,
        &staged.file_name,
    )
    .unwrap_err();
    assert!(
        matches!(error,abiding_word_lib::error::AppError::Import(ref message) if message.code==Some("batch.changed"))
    );
    assert!(!managed.exists());
    fs::write(&input, commentary(true)).unwrap();
    abiding_word_lib::library::files::copy_verified(
        &input,
        &managed,
        &staged.hash,
        &staged.file_name,
    )
    .unwrap();
    assert_eq!(
        abiding_word_lib::library::files::content_hash(&fs::read(managed).unwrap()),
        staged.hash
    );
}
