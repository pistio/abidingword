//! The evaluation's own USFM verse-text reader (candidate A1.5-E3,
//! `eval_support/usfm_text.rs`, rules `aw-usfm-text-1`) on the 13 real
//! references bundled in `resources/scripture/`, and on SYNTHETIC USFM
//! (invented sentences, labelled as such) for every construct the rules
//! name. Runs with `cargo test`.
//!
//! `AW_EVAL_TEXT_DUMP=<folder>` also writes, for every bundled reference,
//! what this reader read (one JSON file per archive) so that the Python
//! reader's reading can be compared with it verse by verse
//! (`docs/audit/candidate-a1.5/evaluation-e3/evidence/readers/`).

#[allow(dead_code)]
#[path = "eval_support/usfm_text.rs"]
mod usfm_text;

use std::io::Read;
use std::path::{Path, PathBuf};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel)
}

/// Every USFM member of a bundled archive: (member name, text).
fn members(edition: &str) -> Vec<(String, String)> {
    let dir = repo(&format!("resources/scripture/{edition}"));
    let path = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("_usfm.zip"))
        .expect("a bundled archive");
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut out = Vec::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).unwrap();
        let name = entry.name().to_string();
        let low = name.to_ascii_lowercase();
        if low.ends_with(".usfm") || low.ends_with(".sfm") {
            let mut text = String::new();
            entry.read_to_string(&mut text).unwrap();
            out.push((name, text));
        }
    }
    out
}

const BUNDLED: [&str; 13] = [
    "deu1912",
    "deu1951",
    "deuelo",
    "engbsb",
    "engwebp",
    "fraLSG",
    "grcbyz",
    "grcmt",
    "grcsbl",
    "grctr",
    "kor1961",
    "porbr2018",
    "spaRV1909",
];

#[test]
fn every_bundled_reference_reads_without_a_problem_and_loses_no_verse() {
    use sha2::{Digest, Sha256};
    let dump = std::env::var_os("AW_EVAL_TEXT_DUMP").map(PathBuf::from);
    // What both readers must read (`tests/evaluation/bundled-references.expected.json`).
    let expected: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo("tests/evaluation/bundled-references.expected.json"))
            .unwrap(),
    )
    .unwrap();
    // The 528 verses the independent E2 review found the app's reader
    // spacing wrongly where a note was removed: each with the edition's own
    // context and the app reader's.
    let review: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo(
            "docs/audit/candidate-a1.5/evaluation-e2-review/evidence/inventory/note-space-differences.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let mut total = 0usize;
    let mut artefacts_checked = 0usize;
    for edition in BUNDLED {
        let want = &expected["editions"][edition];
        let mut out = serde_json::Map::new();
        let mut verses = 0usize;
        let mut empty = 0usize;
        let mut before = 0usize;
        let mut digest = Sha256::new();
        let mut by_reference = std::collections::HashMap::new();
        for (name, text) in members(edition) {
            let book = usfm_text::read(&text);
            assert!(
                book.problems.is_empty(),
                "{edition} {name}: {:?}",
                book.problems
            );
            assert_eq!(
                book.verse_markers,
                book.verses.len() + book.verses_before_chapter,
                "{edition} {name}: every \\v is a verse"
            );
            verses += book.verses.len();
            before += book.verses_before_chapter;
            for v in &book.verses {
                empty += usize::from(v.text.is_empty());
                digest.update(
                    format!(
                        "{name}\t{}\t{}\t{}\t{}\n",
                        v.chapter,
                        v.label,
                        v.line,
                        usfm_text::text_sha256(&v.text)
                    )
                    .as_bytes(),
                );
                by_reference.insert(
                    format!(
                        "{} {}:{}",
                        book.code.as_deref().unwrap_or("?"),
                        v.chapter,
                        v.label
                    ),
                    v.text.clone(),
                );
            }
            if dump.is_some() {
                let rows: Vec<serde_json::Value> = book
                    .verses
                    .iter()
                    .map(|v| {
                        serde_json::json!([
                            v.chapter,
                            v.label,
                            v.line,
                            usfm_text::text_sha256(&v.text),
                            v.alternate
                        ])
                    })
                    .collect();
                out.insert(
                    name,
                    serde_json::json!({"book": book.code, "verses": rows,
                                       "before_chapter": book.verses_before_chapter}),
                );
            }
        }
        assert_eq!(verses as u64, want["verses"].as_u64().unwrap(), "{edition}");
        assert_eq!(
            empty as u64,
            want["numbered_but_empty"].as_u64().unwrap(),
            "{edition}"
        );
        assert_eq!(
            before as u64,
            want["verses_before_chapter"].as_u64().unwrap(),
            "{edition}"
        );
        assert_eq!(
            format!("{:x}", digest.finalize()),
            want["text_digest"].as_str().unwrap(),
            "{edition}: the Rust reader reads what the Python reader read"
        );
        if let Some(rows) = review[edition]["rows"].as_array() {
            for row in rows {
                let reference = row["reference"].as_str().unwrap();
                let text = &by_reference[reference];
                let edition_context = row["independent_context"].as_str().unwrap();
                let app_context = row["app_reader_context"].as_str().unwrap();
                assert!(
                    text.contains(edition_context.trim()),
                    "{edition} {reference}: «{text}» lacks «{edition_context}»"
                );
                assert!(
                    !text.contains(app_context.trim()),
                    "{edition} {reference}: «{text}» still holds «{app_context}»"
                );
                artefacts_checked += 1;
            }
        }
        if let Some(dir) = &dump {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(
                dir.join(format!("{edition}.rust.json")),
                serde_json::to_string(&out).unwrap(),
            )
            .unwrap();
        }
        total += verses;
    }
    // The review's count of verse labels in the 13 archives, and of verses
    // its reader found spaced wrongly.
    assert_eq!(total, 311_761);
    assert_eq!(artefacts_checked, 528);
}

/// The shared vectors (`tests/evaluation/usfm-text-vectors.json`), which the
/// Python reader must also reproduce exactly.
#[test]
fn every_reader_vector_is_read_as_specified() {
    let text = std::fs::read_to_string(repo("tests/evaluation/usfm-text-vectors.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(doc["rules"], usfm_text::RULES);
    let vectors = doc["vectors"].as_array().unwrap();
    assert!(vectors.len() >= 50, "the vectors are all there");
    let mut wrong = Vec::new();
    for v in vectors {
        let book = usfm_text::read(v["usfm"].as_str().unwrap());
        let verses: Vec<serde_json::Value> = book
            .verses
            .iter()
            .map(|x| serde_json::json!([x.chapter, x.label, x.text]))
            .collect();
        let mut codes: Vec<&str> = book.problems.iter().map(|p| p.code).collect();
        codes.sort_unstable();
        codes.dedup();
        let want_codes: Vec<&str> = v["problems"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect();
        let mut ok = serde_json::Value::from(verses.clone()) == v["verses"]
            && codes == want_codes
            && book.verses_before_chapter as u64 == v["before_chapter"].as_u64().unwrap_or(0);
        if let Some(chapters) = v["chapters"].as_array() {
            let got: Vec<serde_json::Value> = book
                .chapters
                .iter()
                .map(|c| serde_json::json!({"label": c.label, "alternate": c.alternate, "published": c.published}))
                .collect();
            ok &= &got == chapters;
        }
        if !ok {
            wrong.push(format!(
                "{}: got {verses:?} {codes:?} before {}",
                v["id"], book.verses_before_chapter
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// For the cross-implementation evidence only: reads every USFM text of the
/// JSON list in `AW_EVAL_TEXT_INPUT` and writes what this reader read to
/// `AW_EVAL_TEXT_OUTPUT` (`readers/fuzz_readers.py` compares it with the
/// Python reader). Does nothing when they are not set.
#[test]
#[ignore]
fn read_texts_for_cross_checking() {
    let (Some(input), Some(output)) = (
        std::env::var_os("AW_EVAL_TEXT_INPUT"),
        std::env::var_os("AW_EVAL_TEXT_OUTPUT"),
    ) else {
        return;
    };
    let inputs: Vec<String> =
        serde_json::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();
    let out: Vec<serde_json::Value> = inputs
        .iter()
        .map(|text| {
            let book = usfm_text::read(text);
            let mut codes: Vec<&str> = book.problems.iter().map(|p| p.code).collect();
            codes.sort_unstable();
            codes.dedup();
            serde_json::json!({
                "book": book.code,
                "verses": book.verses.iter().map(|v| serde_json::json!([v.chapter, v.label, v.text, v.line])).collect::<Vec<_>>(),
                "before_chapter": book.verses_before_chapter,
                "verse_markers": book.verse_markers,
                "problems": codes,
            })
        })
        .collect();
    std::fs::write(output, serde_json::to_string(&out).unwrap()).unwrap();
}

/// `\v` markers counted byte by byte, apart from the reader's own count: a
/// backslash, `v`, then white space, a backslash or the end.
fn plain_verse_count(text: &str) -> usize {
    let b = text.as_bytes();
    (0..b.len().saturating_sub(1))
        .filter(|&i| {
            b[i] == b'\\'
                && b[i + 1] == b'v'
                && matches!(
                    b.get(i + 2),
                    None | Some(b' ' | b'\t' | b'\n' | b'\r' | b'\\')
                )
        })
        .count()
}

/// No reading loses a verse silently: on thousands of mutations of the
/// vectors (markers and their ends removed, inserted or cut), every `\v` is
/// a verse unless the reader reports a problem.
#[test]
fn no_mutation_loses_a_verse_silently() {
    let text = std::fs::read_to_string(repo("tests/evaluation/usfm-text-vectors.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let seeds: Vec<String> = doc["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["usfm"].as_str().unwrap().to_string())
        .collect();
    let pieces = [
        "\\f + ",
        "\\f*",
        "\\x - ",
        "\\x*",
        "\\wj ",
        "\\wj*",
        "\\+w ",
        "\\+w*",
        "|",
        "\\va ",
        "\\va*",
        "\\fig ",
        "\\fig*",
        "\\qt-s |who=\"x\"\\*",
        "\\qt-e\\*",
        "\\*",
        "\\",
        "\\v ",
        "\\v 9 ",
        "\\c ",
        "\\c 7\n",
        "\\p\n",
        "\\esb\n",
        "\\esbe\n",
        "\\add ",
        "\\nd*",
        "\\zz ",
        "\n",
    ];
    let mut state: u64 = 20_261_122;
    let mut pick = |n: usize| -> usize {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((state >> 33) as usize) % n.max(1)
    };
    let (mut refused, mut clean) = (0usize, 0usize);
    for _ in 0..4000 {
        let mut s = seeds[pick(seeds.len())].clone();
        for _ in 0..=pick(4) {
            let mut at = pick(s.len() + 1);
            while !s.is_char_boundary(at) {
                at -= 1;
            }
            match pick(3) {
                0 => s.insert_str(at, pieces[pick(pieces.len())]),
                1 => {
                    if let Some(k) = s[at..].find('\\').map(|k| at + k) {
                        let end = s[k + 1..]
                            .find(|c: char| !(c.is_ascii_alphanumeric() || "+*-".contains(c)))
                            .map_or(s.len(), |e| k + 1 + e);
                        s.replace_range(k..end, "");
                    }
                }
                _ => s.truncate(at),
            }
        }
        let book = usfm_text::read(&s);
        let normalised = s
            .trim_start_matches('\u{feff}')
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        if book.problems.is_empty() {
            clean += 1;
            assert_eq!(
                book.verses.len() + book.verses_before_chapter,
                plain_verse_count(&normalised),
                "a verse lost silently in {s:?}"
            );
        } else {
            refused += 1;
        }
    }
    assert!(
        refused > 1000 && clean > 500,
        "{refused} refused, {clean} clean"
    );
}

/// The marker table of `docs/evaluation/usfm-text-rules.md` is the one this
/// reader uses (the Python reader is held to the same table).
#[test]
fn the_marker_table_is_the_rules_documents() {
    let rules = std::fs::read_to_string(repo("docs/evaluation/usfm-text-rules.md")).unwrap();
    let block = rules
        .split("```aw-usfm-text-1 markers\n")
        .nth(1)
        .and_then(|rest| rest.split("```").next())
        .expect("the marker table");
    let mut seen = 0;
    for line in block.lines() {
        let (role, markers) = line.split_once(": ").expect("role: markers");
        for marker in markers.split_whitespace() {
            assert_eq!(usfm_text::role_name(marker), role, "\\{marker}");
            seen += 1;
        }
    }
    assert!(seen > 120, "{seen} markers");
    for marker in ["v", "c", "id", "zfoo", "qqq", "esb"] {
        assert_eq!(usfm_text::role_name(marker), "unknown", "\\{marker}");
    }
}
