//! Reads a JSON array of {"name", "text"} on standard input and writes, for
//! each, the E3 Rust reader's verses ([chapter, label, text, line]), its
//! problem codes, its verse-marker count and verses before any chapter.
use std::io::Read;

#[allow(dead_code)]
#[path = "../../../../../../../src-tauri/tests/eval_support/usfm_text.rs"]
mod usfm_text;

#[derive(serde::Deserialize)]
struct Input {
    name: String,
    text: String,
}

fn main() {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).expect("stdin");
    let inputs: Vec<Input> = serde_json::from_str(&s).expect("json");
    let mut out = serde_json::Map::new();
    for i in inputs {
        let b = usfm_text::read(&i.text);
        let verses: Vec<serde_json::Value> = b
            .verses
            .iter()
            .map(|v| serde_json::json!([v.chapter, v.label, v.text, v.line, v.alternate]))
            .collect();
        let problems: Vec<&str> = b.problems.iter().map(|p| p.code).collect();
        out.insert(
            i.name,
            serde_json::json!({"book": b.code, "verses": verses, "problems": problems,
                "verse_markers": b.verse_markers, "before": b.verses_before_chapter,
                "chapters": b.chapters.iter().map(|c| serde_json::json!([c.label, c.alternate, c.published])).collect::<Vec<_>>()}),
        );
    }
    println!("{}", serde_json::Value::Object(out));
}
