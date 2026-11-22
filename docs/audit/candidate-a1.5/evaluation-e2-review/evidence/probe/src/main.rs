//! Independent review probe (A1.5-E2). Drives the E2 harness's own rule
//! modules, included read-only from the repository, on the reviewer's own
//! invented inputs. Not part of the repository.
#[allow(dead_code)]
#[path = "/home/user/abidingword/src-tauri/tests/eval_support/fidelity.rs"]
mod fidelity;
#[allow(dead_code)]
#[path = "/home/user/abidingword/src-tauri/tests/eval_support/reference.rs"]
mod reference;

use std::collections::BTreeSet;
use std::io::{BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        // JSONL of {"id","edition","found","matched"} on stdin -> JSONL of class/kind.
        Some("fidelity") => {
            let stdin = std::io::stdin();
            let mut out = std::io::stdout().lock();
            for line in stdin.lock().lines() {
                let line = line.unwrap();
                if line.trim().is_empty() { continue; }
                let v: serde_json::Value = serde_json::from_str(&line).unwrap();
                let e = v["edition"].as_str().unwrap();
                let f = v["found"].as_str().unwrap();
                let m = v["matched"].as_bool().unwrap_or(false);
                let (c, k) = fidelity::judge(e, f, m);
                writeln!(out, "{}", serde_json::json!({"id": v["id"], "class": c.as_str(), "kind": k})).unwrap();
            }
        }
        // inventory <path> [BOOK,BOOK]
        Some("inventory") => {
            let path = std::path::Path::new(&args[2]);
            let declared: Option<BTreeSet<String>> = args.get(3).map(|s| s.split(',').map(str::to_string).collect());
            let inv = reference::read(path, declared.as_ref());
            let entries: Vec<serde_json::Value> = inv.entries.iter().map(|e| serde_json::json!({
                "label": e.label(), "file": e.file, "text": e.text,
                "status": match &e.status {
                    reference::Status::Scorable{..} => "scorable".to_string(),
                    reference::Status::Unsupported(c) => c.to_string(),
                    reference::Status::OutsideScope => "outside_declared_scope".to_string(),
                }})).collect();
            let report = serde_json::to_value(inv.report()).unwrap();
            println!("{}", serde_json::json!({"report": report, "entries": entries,
                "unscorable_keys": inv.unscorable_keys.len(), "outside_keys": inv.outside_keys.len()}));
        }
        // usfm <file>: the app's reader on one file
        Some("usfm") => {
            let text = std::fs::read_to_string(&args[2]).unwrap();
            let doc = abiding_word_lib::scripture::usfm::parse(&text);
            let verses: Vec<serde_json::Value> = doc.verses.iter().map(|v| serde_json::json!(
                {"c": v.chapter, "v": v.label, "text": v.text})).collect();
            println!("{}", serde_json::json!({"book": doc.book_code, "verses": verses,
                "outside": doc.verses_outside_chapter, "unknown": doc.unknown_markers}));
        }
        _ => eprintln!("usage: probe fidelity|inventory|usfm ..."),
    }
}
