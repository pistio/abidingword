//! Whole-work relevance is not a fabricated reader destination. Invented texts only.
use abiding_word_lib::{
    db::{library, library_manage, Database},
    library::{
        coverage::{CoverageBasis, CoverageMode, CoverageScope},
        import, CommitSourceImport, SourceType,
    },
    scripture::reference::ScriptureRef,
};
use rusqlite::Connection;
use serde_json::{json, Value};

fn add(conn: &Connection, file: &str, text: &str, title: &str, kind: SourceType) -> String {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join(file);
    std::fs::write(&path, format!("{text}\n\nFixture identity: {file}.\n")).unwrap();
    let staged = import::stage(&path).unwrap();
    let id = abiding_word_lib::db::new_id();
    library::commit_import(
        conn,
        &id,
        &staged,
        &CommitSourceImport {
            preview_id: staged.id.clone(),
            title: title.into(),
            language: Some("English".into()),
            source_type: Some(kind),
            ..Default::default()
        },
        &library::managed_relative_path(&id, &staged),
    )
    .unwrap();
    id
}
fn input(conn: &Connection, id: &str, coverage: Option<Value>) -> library_manage::UpdateSource {
    let mut value = serde_json::to_value(library::get(conn, id).unwrap()).unwrap();
    value.as_object_mut().unwrap().remove("coverage");
    if let Some(coverage) = coverage {
        value["coverage"] = coverage;
    }
    serde_json::from_value(value).unwrap()
}
fn set(conn: &Connection, id: &str, mode: &str, passages: &[&str]) {
    library_manage::update_source(
        conn,
        &input(conn, id, Some(json!({"mode":mode,"passages":passages}))),
    )
    .unwrap();
}
fn groups(conn: &Connection, query: &str) -> Vec<abiding_word_lib::library::PassageSourceGroup> {
    library::passage_sources(conn, &ScriptureRef::from_canonical(query).unwrap()).unwrap()
}
fn row(conn: &Connection, id: &str) -> (String, Option<String>, String, i64) {
    conn.query_row(
        "SELECT title,data,updated_at,revision FROM sources WHERE id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )
    .unwrap()
}

#[test]
fn confirmed_whole_work_matches_without_any_invented_paragraph_page_or_reference() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let id = add(
            conn,
            "monograph.txt",
            "An introduction with no explicit biblical citation or heading.",
            "A monograph",
            SourceType::Book,
        );
        let before = library::get(conn, &id)?;
        set(conn, &id, "manual", &["Romans"]);
        let saved = library::get(conn, &id)?;
        assert_eq!(saved.coverage_mode, CoverageMode::Manual);
        assert_eq!(saved.coverage[0].scope, CoverageScope::Book);
        assert_eq!(saved.coverage[0].basis, CoverageBasis::Manual);
        assert_eq!(saved.content_hash, before.content_hash);
        assert_eq!(saved.segment_count, before.segment_count);
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM source_passages WHERE source_id=?1",
                [&id],
                |r| r.get::<_, i64>(0)
            )?,
            0
        );
        let found = groups(conn, "ROM.8.1");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].source_id, id);
        assert!(found[0].locations.is_empty());
        assert_eq!(found[0].more_locations, 0);
        assert_eq!(found[0].coverage[0].reference.canonical, "ROM.1-ROM.16");
        assert!(library::passage_source_locations(
            conn,
            &ScriptureRef::from_canonical("ROM.8.1")?,
            &id
        )?
        .is_empty());
        assert!(groups(conn, "GAL.1.1").is_empty());
        conn.execute("UPDATE sources SET state='archived' WHERE id=?1", [&id])?;
        assert!(groups(conn, "ROM.8.1").is_empty());
        assert_eq!(
            library::get(conn, &id)?.coverage.len(),
            1,
            "explicitly opened archived source keeps its own metadata"
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn manual_ranges_accept_interface_languages_preserve_other_metadata_and_match_only_their_scope() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let id = add(
            conn,
            "scope.txt",
            "Invented text.",
            "Scoped work",
            SourceType::Book,
        );
        conn.execute(
            "UPDATE sources SET data=?2 WHERE id=?1",
            rusqlite::params![id, r#"{"other":{"keep":[1,"value"]}}"#],
        )?;
        for name in [
            "Romans",
            "Römer",
            "로마서",
            "Romains",
            "Romanos",
            "ROM.1-ROM.16",
        ] {
            set(conn, &id, "manual", &[name]);
            assert_eq!(
                library::get(conn, &id)?.coverage[0].reference.canonical,
                "ROM.1-ROM.16",
                "{name}"
            );
        }
        set(conn, &id, "manual", &["Römer 8", "요한복음 3:16", "ROM.8"]);
        let saved = library::get(conn, &id)?;
        assert_eq!(
            saved.coverage.len(),
            2,
            "duplicates are not additional evidence"
        );
        let found = groups(conn, "ROM.8.4");
        assert_eq!(found[0].coverage.len(), 1);
        assert_eq!(found[0].coverage[0].scope, CoverageScope::Chapter);
        assert_eq!(
            groups(conn, "JHN.3.16")[0].coverage[0].scope,
            CoverageScope::Passage
        );
        assert!(groups(conn, "ROM.9").is_empty());
        let mut edit = input(conn, &id, None);
        edit.title = "A corrected title unrelated to the scope".into();
        let after = library_manage::update_source(conn, &edit)?;
        assert_eq!(after.coverage, saved.coverage);
        assert_eq!(after.coverage_mode, CoverageMode::Manual);
        let data: Value = serde_json::from_str(row(conn, &id).1.as_deref().unwrap()).unwrap();
        assert_eq!(data["other"], json!({"keep":[1,"value"]}));
        set(conn, &id, "none", &[]);
        assert!(groups(conn, "ROM.8").is_empty());
        assert_eq!(library::get(conn, &id)?.coverage_mode, CoverageMode::None);
        set(conn, &id, "automatic", &[]);
        assert_eq!(
            library::get(conn, &id)?.coverage_mode,
            CoverageMode::Automatic
        );
        assert_eq!(
            serde_json::from_str::<Value>(row(conn, &id).1.as_deref().unwrap()).unwrap(),
            json!({"other":{"keep":[1,"value"]}})
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn source_coverage_uses_the_same_german_spelling_variants_for_books_and_passages() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let id = add(
            conn,
            "aliases.txt",
            "An invented whole-work discussion.",
            "Alias scope",
            SourceType::Book,
        );
        for (written, expected) in [
            ("Sprüche", "PRO.1-PRO.31"),
            ("Sprichwörter", "PRO.1-PRO.31"),
            ("Sprueche", "PRO.1-PRO.31"),
            ("Sprichwoerter", "PRO.1-PRO.31"),
            ("Sprueche 3:5-6", "PRO.3.5-PRO.3.6"),
            ("Sprichwoerter 3,5", "PRO.3.5"),
            ("Kohelet", "ECC.1-ECC.12"),
            ("Prediger", "ECC.1-ECC.12"),
            ("1. Mose", "GEN.1-GEN.50"),
            ("Genesis", "GEN.1-GEN.50"),
        ] {
            set(conn, &id, "manual", &[written]);
            assert_eq!(
                library::get(conn, &id)?.coverage[0].reference.canonical,
                expected,
                "{written}"
            );
        }
        // A typed scope must still be one reference, not a prose sentence or
        // an unsupported passage rescued by a smaller prefix match.
        for invalid in [
            "Read Sprueche 3:5",
            "Sprueche 3:999",
            "Sprueche 3:5 and more",
            "Sprueche 3:5 — unrelated prose",
            "Jude 1절2절",
            "Jude 1節2節",
        ] {
            let before = row(conn, &id);
            assert!(
                library_manage::update_source(
                    conn,
                    &input(
                        conn,
                        &id,
                        Some(json!({"mode":"manual","passages":[invalid]}))
                    )
                )
                .is_err(),
                "{invalid}"
            );
            assert_eq!(row(conn, &id), before);
        }
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM source_passages WHERE source_id=?1",
                [&id],
                |row| row.get::<_, i64>(0)
            )?,
            0
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn coverage_errors_are_atomic_and_unrelated_edits_preserve_malformed_legacy_data() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let id = add(
            conn,
            "atomic.txt",
            "Original text.",
            "Original title",
            SourceType::Book,
        );
        set(conn, &id, "manual", &["Romans"]);
        for coverage in [
            json!({"mode":"manual","passages":["Romans 8","not a passage"]}),
            json!({"mode":"manual","passages":[]}),
            json!({"mode":"manual","passages":vec!["ROM.8";9]}),
        ] {
            let before = row(conn, &id);
            let mut edit = input(conn, &id, Some(coverage));
            edit.title = "Must not save".into();
            assert!(library_manage::update_source(conn, &edit).is_err());
            assert_eq!(row(conn, &id), before);
        }
        for raw in ["{broken", "[1,2,3]"] {
            conn.execute(
                "UPDATE sources SET data=?2 WHERE id=?1",
                rusqlite::params![id, raw],
            )?;
            let before = row(conn, &id);
            let mut edit = input(
                conn,
                &id,
                Some(json!({"mode":"manual","passages":["ROM.8"]})),
            );
            edit.title = "Must not save".into();
            assert!(library_manage::update_source(conn, &edit).is_err());
            assert_eq!(row(conn, &id), before);
            let mut ordinary = input(conn, &id, None);
            ordinary.title = "Regular metadata correction".into();
            assert_eq!(
                library_manage::update_source(conn, &ordinary)?.title,
                "Regular metadata correction"
            );
            assert_eq!(row(conn, &id).1.as_deref(), Some(raw));
        }
        Ok(())
    })
    .unwrap();
}

const ROMANS_EIGHT:&str="# Romans\n\nBook introduction.\n\n## Chapter 8\n\nA real chapter discussion.\n\n### Verses 1-4\n\nA narrower exposition.\n";

#[test]
fn localized_bibliographic_titles_corroborate_scope_without_accent_or_byte_boundary_loss() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        for (i, title) in [
            "Étude biblique sur Romains 8",
            "E\u{0301}tude biblique sur Romains 8",
            "ÉTUDE BIBLIQUE SUR ROMAINS 8",
            "Comentário sobre Romanos 8",
            "Comenta\u{0301}rio sobre Romanos 8",
            "Estudo bíblico de Romanos 8",
            "Estudio bíblico de Romanos 8",
            "Predigt über Römer 8",
            "Predigt uber Römer 8",
        ]
        .iter()
        .enumerate()
        {
            let id = add(
                conn,
                &format!("localized{i}.md"),
                ROMANS_EIGHT,
                title,
                SourceType::Book,
            );
            let source = library::get(conn, &id)?;
            assert_eq!(source.coverage.len(), 1, "{title}");
            assert_eq!(source.coverage[0].reference.canonical, "ROM.8", "{title}");
            assert_eq!(source.coverage[0].basis, CoverageBasis::Metadata);
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn automatic_scope_requires_metadata_and_real_structure_and_distinguishes_parent_from_competing_content(
) {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let chapter = add(
            conn,
            "chapter.md",
            ROMANS_EIGHT,
            "Romans 8 — A Commentary",
            SourceType::Commentary,
        );
        let source = library::get(conn, &chapter)?;
        assert_eq!(source.coverage_mode, CoverageMode::Automatic);
        let evidence: Vec<String> = conn.prepare("SELECT passage || '|' || COALESCE(structure_scope,'NULL') || '|' || COALESCE(structure_node_id,'NULL') || '|' || COALESCE(section_heading,'NULL') || '|' || COALESCE(section_path,'NULL') || '|' || position FROM source_passages p JOIN source_segments g ON g.id=p.segment_id WHERE p.source_id=?1 AND origin='section' ORDER BY position")?.query_map([&chapter],|r|r.get(0))?.collect::<Result<_,_>>()?;
        assert_eq!(
            source.coverage.len(),
            1,
            "book ancestor is not competing chapter content: {evidence:?}"
        );
        assert_eq!(source.coverage[0].reference.canonical, "ROM.8");
        assert_eq!(source.coverage[0].basis, CoverageBasis::Metadata);
        let book = add(
            conn,
            "book.md",
            ROMANS_EIGHT,
            "Romans: A Commentary",
            SourceType::Book,
        );
        assert_eq!(
            library::get(conn, &book)?.coverage[0].scope,
            CoverageScope::Book
        );
        let sibling = add(
            conn,
            "sibling.md",
            &format!("{ROMANS_EIGHT}\n## Chapter 9\n\nAnother chapter's discussion."),
            "Romans 8 — A Commentary",
            SourceType::Commentary,
        );
        assert!(library::get(conn, &sibling)?.coverage.is_empty());
        let other = add(
            conn,
            "other.md",
            &format!("{ROMANS_EIGHT}\n# Galatians\n\nAnother book's introduction."),
            "Romans — A Commentary",
            SourceType::Commentary,
        );
        assert!(library::get(conn, &other)?.coverage.is_empty());
        let separate = add(
            conn,
            "separate.md",
            "# Romans\n\nIntroduction.\n\n# Appendix\n\n## Romans 8\n\nIndependent section.",
            "Romans 8 Commentary",
            SourceType::Commentary,
        );
        assert!(
            library::get(conn, &separate)?.coverage.is_empty(),
            "a broader sibling is not a proven parent"
        );
        set(conn, &sibling, "manual", &["ROM.8"]);
        assert_eq!(
            library::get(conn, &sibling)?.coverage[0].basis,
            CoverageBasis::Manual
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn homonyms_multi_book_titles_publisher_noise_and_frequent_mentions_never_infer_coverage() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn|{
        for (i,title) in ["Job","Acts of Kindness","John and His Family","John"].iter().enumerate() {
            let id=add(conn,&format!("name{i}.txt"),"A discussion of employment, actions and a person's life.",title,SourceType::Book);
            assert!(library::get(conn,&id)?.coverage.is_empty(),"{title}");
        }
        let mixed=add(conn,"mixed.md",ROMANS_EIGHT,"Romans and Galatians: A Commentary",SourceType::Commentary);
        assert!(library::get(conn,&mixed)?.coverage.is_empty());
        let citations=add(conn,"citations.txt",&"Romans 8:1 is cited here.\n\n".repeat(80),"Romans 8 Commentary",SourceType::Commentary);
        assert!(library::get(conn,&citations)?.coverage.is_empty());
        let noise=add(conn,"noise.md",ROMANS_EIGHT,"General Essays",SourceType::Book);
        conn.execute("UPDATE sources SET publisher='Romans 8 Bible Commentary',subtitle='Romans Bible Commentary Series' WHERE id=?1",[&noise])?;
        assert!(library::get(conn,&noise)?.coverage.is_empty());
        Ok(())
    }).unwrap();
}

#[test]
fn focused_monograph_precedes_broad_book_intro_but_real_chapter_destination_still_leads() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let intro = add(
            conn,
            "intro.md",
            "# Romans\n\nA general book introduction.",
            "Book introductions",
            SourceType::Book,
        );
        let monograph = add(
            conn,
            "mono.txt",
            "A focused whole-work discussion with no target.",
            "Focused monograph",
            SourceType::Book,
        );
        set(conn, &monograph, "manual", &["Romans 8"]);
        let precise = add(
            conn,
            "precise.md",
            ROMANS_EIGHT,
            "Chapter discussions",
            SourceType::Book,
        );
        let mention = add(
            conn,
            "mention.txt",
            "An incidental citation: Romans 8:11.",
            "Mention only",
            SourceType::Book,
        );
        let found = groups(conn, "ROM.8.11");
        assert_eq!(
            found
                .iter()
                .map(|g| g.source_id.as_str())
                .collect::<Vec<_>>(),
            [
                precise.as_str(),
                monograph.as_str(),
                intro.as_str(),
                mention.as_str()
            ]
        );
        assert!(found[1].locations.is_empty());
        assert_eq!(found[1].coverage[0].reference.canonical, "ROM.8");
        let before = library::passage_source_locations(
            conn,
            &ScriptureRef::from_canonical("ROM.8.11")?,
            &precise,
        )?;
        set(conn, &precise, "manual", &["Romans"]);
        assert_eq!(
            library::passage_source_locations(
                conn,
                &ScriptureRef::from_canonical("ROM.8.11")?,
                &precise
            )?,
            before
        );
        // A narrow whole work can be relevant to a chapter reading without
        // pretending its scope or a navigable target covers the whole chapter.
        set(conn, &monograph, "manual", &["Romans 8:1-4"]);
        let partial = groups(conn, "ROM.8")
            .into_iter()
            .find(|g| g.source_id == monograph)
            .unwrap();
        assert_eq!(partial.coverage[0].reference.canonical, "ROM.8.1-ROM.8.4");
        assert_eq!(partial.coverage[0].scope, CoverageScope::Passage);
        assert!(partial.locations.is_empty());
        Ok(())
    })
    .unwrap();
}

#[test]
fn incidental_mentions_do_not_promote_or_demote_equivalent_whole_source_coverage() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let with_mention=add(conn,"with-mention.txt","A brief incidental comparison cites Romans 8:11.","Alpha work",SourceType::Book);
        let without=add(conn,"without-mention.txt","A whole-work discussion with no exact paragraph target.","Beta work",SourceType::Book);
        for id in [&with_mention,&without] { set(conn,id,"manual",&["Romans 8"]); }
        let found=groups(conn,"ROM.8.11");
        assert_eq!(found.iter().map(|group|group.source_id.as_str()).collect::<Vec<_>>(),[with_mention.as_str(),without.as_str()],"equal whole-work scopes retain the title tie-break, independently of an incidental mention");
        assert_eq!(found[0].coverage,found[1].coverage);
        assert_eq!(found[0].locations.len(),1);
        assert_eq!(found[0].locations[0].origin,abiding_word_lib::library::sections::LinkOrigin::Explicit);
        assert!(found[1].locations.is_empty());
        Ok(())
    }).unwrap();
}

#[test]
fn coverage_edits_preserve_stored_links_and_user_intent_survives_rebuild() {
    let db = Database::open_in_memory().unwrap();
    db.with_conn(|conn| {
        let id=add(conn,"rebuild.md",&format!("{ROMANS_EIGHT}\nAn incidental comparison cites Galatians 2:20."),"Romans 8 — A Commentary",SourceType::Commentary);
        conn.execute("INSERT INTO source_passages(id,source_id,segment_id,passage,start_key,end_key,matched_text,created_at,origin) SELECT 'coverage-manual-link',source_id,segment_id,passage,start_key,end_key,matched_text,'original-manual-time','manual' FROM source_passages WHERE source_id=?1 AND passage='GAL.2.20' LIMIT 1",[&id])?;
        let links=|conn:&Connection|conn.prepare("SELECT id || '|' || origin || '|' || segment_id || '|' || passage || '|' || matched_text || '|' || created_at FROM source_passages WHERE source_id=?1 ORDER BY id").unwrap().query_map([&id],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        let text=|conn:&Connection|conn.prepare("SELECT id || '|' || content || '|' || COALESCE(section_path,'') || '|' || created_at || '|' || updated_at FROM source_segments WHERE source_id=?1 ORDER BY position").unwrap().query_map([&id],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        let original_links=links(conn);let original_text=text(conn);
        assert!(!original_links.is_empty());assert!(original_links.iter().any(|link|link.starts_with("coverage-manual-link|")));
        for (mode,passages) in [("manual",vec!["Romans 8:1-4"]),("none",vec![])] {
            set(conn,&id,mode,&passages);
            assert_eq!(links(conn),original_links,"changing source metadata never edits reference/link identities");
            assert_eq!(text(conn),original_text);
        }
        for (mode,passages) in [("manual",vec!["Romans 8:1-4"]),("none",vec![])] {
            set(conn,&id,mode,&passages);
            let before=library::get(conn,&id)?;let data=row(conn,&id).1;
            let manual:Vec<_>=links(conn).into_iter().filter(|link|link.starts_with("coverage-manual-link|")).collect();
            conn.execute("UPDATE sources SET passage_index_version=4 WHERE id=?1",[&id])?;
            assert_eq!(library::ensure_passage_index(conn)?.rebuilt,1);
            assert_eq!(row(conn,&id).1,data);
            library::reindex_passages(conn,&id)?;
            let after=library::get(conn,&id)?;
            assert_eq!(after.coverage_mode,before.coverage_mode);assert_eq!(after.coverage,before.coverage);
            assert_eq!(row(conn,&id).1,data);assert_eq!(text(conn),original_text);
            assert_eq!(links(conn).into_iter().filter(|link|link.starts_with("coverage-manual-link|")).collect::<Vec<_>>(),manual);
            assert!(!library::passage_source_locations(conn,&ScriptureRef::from_canonical("ROM.8.11")?,&id)?.is_empty(),"opting out of whole-work coverage never removes an actual chapter destination");
        }
        Ok(())
    }).unwrap();
}
