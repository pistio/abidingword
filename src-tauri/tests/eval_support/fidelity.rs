//! Strict text fidelity, rules `aw-fidelity-2` (evaluation candidate A1.5-E2).
//!
//! The harness's implementation of the rule the scorer applies again on its
//! own (`scripts/evaluation/fidelity.py`). The two are written apart and must
//! agree on the golden vectors in `tests/evaluation/fidelity-vectors.json`
//! (`tests/evaluation_rules.rs`); the scorer re-classifies every non-exact
//! verse the harness reports and refuses to call a run a protocol run when
//! they disagree.
//!
//! A verse found in a document (`found`) is compared with the edition's own
//! text (`edition`). Its class is the first that holds:
//!
//! * `exact` — the same code points, leading and trailing white space aside;
//! * `canonical` — the same after Unicode NFC (canonical equivalence only);
//! * `layout` — the same after NFC and white-space-run normalisation: every
//!   maximal run of white space becomes one U+0020 and the ends are trimmed.
//!   A run is never removed and none is added, so word boundaries stay;
//! * `mismatch` — anything else.
//!
//! Only the first three are faithful. Every mismatch is wrong text under the
//! zero-wrong-text criterion; its kind (`kind`) is a diagnostic label only.
//!
//! White space is the Unicode White_Space property, listed explicitly so that
//! both implementations use the same 25 code points.

use regex::Regex;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

pub const RULES: &str = "aw-fidelity-2";

pub fn is_ws(c: char) -> bool {
    matches!(
        c,
        '\u{09}'..='\u{0D}'
            | '\u{20}'
            | '\u{85}'
            | '\u{A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Class {
    Exact,
    Canonical,
    Layout,
    Mismatch,
}

impl Class {
    pub fn as_str(self) -> &'static str {
        match self {
            Class::Exact => "exact",
            Class::Canonical => "canonical",
            Class::Layout => "layout",
            Class::Mismatch => "mismatch",
        }
    }

    pub fn faithful(self) -> bool {
        self != Class::Mismatch
    }
}

pub fn trim(t: &str) -> &str {
    t.trim_matches(is_ws)
}

/// Each maximal run of white space as one U+0020, the ends trimmed.
pub fn runs(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    let mut gap = false;
    for c in trim(t).chars() {
        if is_ws(c) {
            gap = true;
            continue;
        }
        if gap {
            out.push(' ');
            gap = false;
        }
        out.push(c);
    }
    out
}

pub fn nfc(t: &str) -> String {
    t.nfc().collect()
}

fn dense(t: &str) -> String {
    t.chars().filter(|c| !is_ws(*c)).collect()
}

fn categories() -> &'static (Regex, Regex, Regex) {
    static RE: OnceLock<(Regex, Regex, Regex)> = OnceLock::new();
    RE.get_or_init(|| {
        (
            Regex::new(r"[^\p{L}\p{N}]").unwrap(),
            Regex::new(r"[^\p{L}\p{M}\p{N}]").unwrap(),
            Regex::new(r"[\p{Cf}\p{Cc}]").unwrap(),
        )
    })
}

/// Letters and digits only, compatibility-decomposed, lower case, marks dropped.
pub fn base(t: &str) -> String {
    let lower: String = t.to_lowercase().nfkd().collect();
    categories().0.replace_all(&lower, "").into_owned()
}

/// Letters, digits and combining marks, compatibility-decomposed, lower case.
fn marked(t: &str) -> String {
    let lower: String = t.to_lowercase().nfkd().collect();
    categories().1.replace_all(&lower, "").into_owned()
}

/// The white-space-separated words, each as `marked`, empty ones dropped.
fn words(t: &str) -> Vec<String> {
    runs(t)
        .split(' ')
        .map(marked)
        .filter(|w| !w.is_empty())
        .collect()
}

fn without_format(t: &str) -> String {
    t.chars()
        .filter(|c| {
            let mut buf = [0u8; 4];
            is_ws(*c) || !categories().2.is_match(c.encode_utf8(&mut buf))
        })
        .collect()
}

pub fn classify(edition: &str, found: &str) -> Class {
    if trim(found) == trim(edition) {
        Class::Exact
    } else if nfc(trim(found)) == nfc(trim(edition)) {
        Class::Canonical
    } else if runs(&nfc(found)) == runs(&nfc(edition)) {
        Class::Layout
    } else {
        Class::Mismatch
    }
}

/// The diagnostic kind of a mismatch. `matched`: the found text's letters
/// are those of another verse of the reference (looked up by the caller in
/// the whole reference inventory).
pub fn kind(edition: &str, found: &str, matched: bool) -> &'static str {
    let (e, f) = (nfc(trim(edition)), nfc(trim(found)));
    let (be, bf) = (base(&e), base(&f));
    if be != bf {
        return if !be.is_empty() && bf.contains(&be) {
            "extra"
        } else if be.contains(&bf) {
            "missing"
        } else if matched {
            "wrong_reference"
        } else {
            "changed"
        };
    }
    if runs(&e).to_lowercase() == runs(&f).to_lowercase() {
        return "case";
    }
    if dense(&e) == dense(&f) {
        return if words(&e) != words(&f) {
            "word_boundary"
        } else {
            "punctuation_spacing"
        };
    }
    if runs(&without_format(&e)) == runs(&without_format(&f)) {
        return "format_characters";
    }
    if runs(&e).nfkc().collect::<String>() == runs(&f).nfkc().collect::<String>() {
        return "compatibility";
    }
    if marked(&e) != marked(&f) {
        return "marks";
    }
    if words(&e) != words(&f) {
        return "word_boundary";
    }
    let signs = |t: &str| t.chars().filter(|c| "*†‡§¶".contains(*c)).count();
    if signs(&f) > signs(&e) {
        return "sign";
    }
    "punctuation"
}

/// (class, kind); the kind only for a mismatch.
pub fn judge(edition: &str, found: &str, matched: bool) -> (Class, Option<&'static str>) {
    let class = classify(edition, found);
    let kind = (class == Class::Mismatch).then(|| kind(edition, found, matched));
    (class, kind)
}
