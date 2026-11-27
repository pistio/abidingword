//! Remove non-text USFM constructs without guessing where malformed spans end.
//!
//! This pass is deliberately separate from word-annotation placement. A note
//! contributes no characters at all: spaces outside its markers remain exactly
//! where the publisher put them. Structural markers always end a broken note,
//! so one missing closing marker can never consume the following verses.

const MAX_PROBLEMS: usize = 32;
const MAX_OPEN: usize = 64;

pub(super) fn problem(problems: &mut Vec<String>, code: &str, marker: &str) {
    if problems.len() < MAX_PROBLEMS {
        problems.push(format!("{code}: \\{marker}"));
    }
}

fn is_note(name: &str) -> bool {
    matches!(name, "f" | "fe" | "ef" | "x" | "ex")
}

fn is_note_part(name: &str) -> bool {
    matches!(
        name,
        "fr" | "ft"
            | "fk"
            | "fq"
            | "fqa"
            | "fl"
            | "fw"
            | "fp"
            | "fv"
            | "fdc"
            | "fm"
            | "xo"
            | "xk"
            | "xq"
            | "xt"
            | "xta"
            | "xop"
            | "xot"
            | "xnt"
            | "xdc"
    )
}

fn boundary(name: &str) -> bool {
    matches!(name, "id" | "c" | "v" | "d")
        || super::is_paragraph(name)
        || super::is_skipped_paragraph(name)
}

struct Marker<'a> {
    name: &'a str,
    plus: bool,
    closing: bool,
    end: usize,
}

fn marker(source: &str, at: usize) -> Marker<'_> {
    let bytes = source.as_bytes();
    let mut i = at + 1;
    let plus = bytes.get(i) == Some(&b'+');
    if plus {
        i += 1;
    }
    let start = i;
    while bytes.get(i).is_some_and(u8::is_ascii_alphanumeric) {
        i += 1;
    }
    if bytes.get(i) == Some(&b'-') {
        i += 1;
        while bytes.get(i).is_some_and(u8::is_ascii_alphanumeric) {
            i += 1;
        }
    }
    let name = &source[start..i];
    let closing = bytes.get(i) == Some(&b'*');
    if closing
        || bytes
            .get(i)
            .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n'))
    {
        // Exactly one delimiter belongs to an opening marker. In particular,
        // do not consume the first source space after a closing marker.
        i += 1;
    }
    Marker {
        name,
        plus,
        closing,
        end: i,
    }
}

/// Keeps source text and ordinary markers; removes notes and milestone syntax.
/// Problems make this book unusable in the import adapter, not silently repaired.
pub(super) fn prepare(source: &str, problems: &mut Vec<String>) -> String {
    let source = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = String::with_capacity(source.len());
    let mut notes: Vec<&str> = Vec::new();
    let mut milestones: Vec<&str> = Vec::new();
    let mut position = 0;
    while let Some(at) = source[position..].find('\\').map(|i| position + i) {
        if notes.is_empty() {
            out.push_str(&source[position..at]);
        }
        let token = marker(&source, at);
        if token.name.is_empty() {
            if token.closing {
                problem(problems, "stray_milestone_end", "*");
            } else if notes.is_empty() {
                // Attribute lists can escape quotes/backslashes. Their own
                // parser, not this pass, determines whether the escape is valid.
                out.push_str(&source[at..token.end]);
            }
            position = token.end;
            continue;
        }

        if !token.closing && !token.plus && boundary(token.name) && !notes.is_empty() {
            for name in notes.drain(..) {
                problem(problems, "note_unclosed", name);
            }
        }
        if is_note(token.name) {
            if token.plus {
                problem(problems, "note_nested_marker", token.name);
            }
            if token.closing {
                if notes.last() == Some(&token.name) {
                    notes.pop();
                } else {
                    problem(problems, "note_close_mismatch", token.name);
                }
            } else {
                if !notes.is_empty() {
                    problem(problems, "note_nested", token.name);
                }
                if notes.len() < MAX_OPEN {
                    notes.push(token.name);
                } else {
                    problem(problems, "note_nesting_limit", token.name);
                }
            }
            position = token.end;
            continue;
        }
        if !notes.is_empty() {
            position = token.end;
            continue;
        }
        if is_note_part(token.name) {
            problem(problems, "note_part_outside_note", token.name);
        }

        let milestone =
            token.name.ends_with("-s") || token.name.ends_with("-e") || token.name == "ts";
        let next = source[token.end..].find('\\').map(|i| token.end + i);
        let terminated = next.filter(|&stop| {
            source[stop..].starts_with("\\*")
                && (source[token.end..stop]
                    .trim_matches(super::is_usfm_space)
                    .is_empty()
                    || source[token.end..stop]
                        .trim_start_matches(super::is_usfm_space)
                        .starts_with('|'))
        });
        if milestone || terminated.is_some() {
            if !milestone || token.closing || token.plus {
                problem(problems, "invalid_milestone_marker", token.name);
            }
            if let Some(stop) = terminated {
                if let Some(base) = token.name.strip_suffix("-s") {
                    if milestones.len() < MAX_OPEN {
                        milestones.push(base);
                    } else {
                        problem(problems, "milestone_nesting_limit", token.name);
                    }
                } else if let Some(base) = token.name.strip_suffix("-e") {
                    if let Some(index) = milestones.iter().rposition(|name| *name == base) {
                        milestones.remove(index);
                    } else {
                        problem(problems, "milestone_close_stray", token.name);
                    }
                }
                position = stop + 2;
                continue;
            }
            problem(problems, "milestone_unterminated", token.name);
            // Do not scan forward over another marker to recover a missing
            // terminator; the next verse/chapter must remain visible.
        }
        out.push_str(&source[at..token.end]);
        position = token.end;
    }
    if notes.is_empty() {
        out.push_str(&source[position..]);
    }
    for name in notes {
        problem(problems, "note_unclosed", name);
    }
    for name in milestones {
        problem(problems, "milestone_unpaired", name);
    }
    out
}
