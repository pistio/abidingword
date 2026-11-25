//! What a whole source is about, independently of any reader destination.
//! Coverage never creates a paragraph link or participates in reference detection.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use unicode_normalization::UnicodeNormalization;

use crate::error::{AppError, AppResult, UserMessage};
use crate::scripture::{
    language::{fold, ReferenceLanguage},
    reference::{ScriptureRef, ScriptureRefDto},
    resolve,
};

pub const MAX_PASSAGES: usize = 8;
const DATA_KEY: &str = "sourceCoverage";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageMode {
    #[default]
    Automatic,
    Manual,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageBasis {
    Manual,
    Metadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageScope {
    Book,
    Chapter,
    Passage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCoverage {
    pub reference: ScriptureRefDto,
    pub scope: CoverageScope,
    pub basis: CoverageBasis,
}

impl SourceCoverage {
    pub fn new(reference: ScriptureRef, basis: CoverageBasis) -> Self {
        let scope = if reference.start.chapter == 1
            && reference.end.chapter == reference.book.chapter_count
            && reference.start.verse.is_none()
            && reference.end.verse.is_none()
        {
            CoverageScope::Book
        } else if reference.start.chapter == reference.end.chapter
            && reference.start.verse.is_none()
            && reference.end.verse.is_none()
        {
            CoverageScope::Chapter
        } else {
            CoverageScope::Passage
        };
        Self {
            reference: reference.to_dto(),
            scope,
            basis,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageUpdate {
    pub mode: CoverageMode,
    #[serde(default)]
    pub passages: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Stored {
    version: u8,
    mode: CoverageMode,
    #[serde(default)]
    passages: Vec<String>,
}

fn invalid(passage: &str) -> AppError {
    AppError::InvalidInput(
        UserMessage::new(
            "library.coverage.invalidPassage",
            "Enter a Bible book, chapter or passage for whole-source coverage.",
        )
        .with("passage", passage),
    )
}

/// Typed scope is intentional. Bare book names mean the whole book here,
/// unlike the Bible reader's typed-input shortcut to a book's first chapter.
pub(crate) fn parse(passage: &str, language: ReferenceLanguage) -> AppResult<ScriptureRef> {
    let text = passage.trim();
    if text.is_empty() || text.chars().count() > resolve::MAX_TYPED {
        return Err(invalid(text));
    }
    if ![" — ", " – ", " - ", ": "]
        .iter()
        .any(|separator| text.contains(separator))
    {
        let books = super::pdf_structure::book_references(text, language);
        if books.len() == 1 {
            return Ok(books[0]);
        }
        if books.len() > 1 {
            return Err(invalid(text));
        }
    }
    let reference = ScriptureRef::from_canonical(text)
        .or_else(|_| resolve::resolve(text, resolve::Context::new(None, Some(language))))
        .or_else(|_| {
            super::pdf_structure::source_reference(text, language)
                .map(|(reference, _)| reference)
                .ok_or_else(|| invalid(text))
        })
        .map_err(|_| invalid(text))?;
    if reference.canon_verse_error(&|_, _, _| false).is_some() {
        return Err(invalid(text));
    }
    Ok(reference)
}

/// Only this namespaced key is changed. Invalid unrelated metadata is never
/// replaced by a freshly invented object during a user metadata edit.
pub(crate) fn merged_data(
    data: Option<&str>,
    update: &CoverageUpdate,
    language: ReferenceLanguage,
) -> AppResult<String> {
    let mut object = match data.filter(|data| !data.trim().is_empty()) {
        None => serde_json::Map::new(),
        Some(data) => serde_json::from_str::<Value>(data)
            .ok()
            .and_then(|value| value.as_object().cloned())
            .ok_or_else(|| {
                AppError::InvalidInput(UserMessage::new(
                    "library.coverage.metadataUnreadable",
                    "The existing source metadata could not be safely updated.",
                ))
            })?,
    };
    match update.mode {
        CoverageMode::Automatic => {
            object.remove(DATA_KEY);
        }
        CoverageMode::None => {
            object.insert(
                DATA_KEY.into(),
                serde_json::json!({"version":1,"mode":"none","passages":[]}),
            );
        }
        CoverageMode::Manual => {
            if update.passages.len() > MAX_PASSAGES {
                return Err(AppError::InvalidInput(
                    UserMessage::new(
                        "library.coverage.tooMany",
                        "Use at most eight whole-source coverage references.",
                    )
                    .with("max", MAX_PASSAGES),
                ));
            }
            if update.passages.is_empty() {
                return Err(AppError::InvalidInput(UserMessage::new(
                    "library.coverage.empty",
                    "Enter at least one Bible book, chapter or passage.",
                )));
            }
            let mut passages = Vec::new();
            for text in &update.passages {
                let canonical = parse(text, language)?.canonical();
                if !passages.contains(&canonical) {
                    passages.push(canonical);
                }
            }
            object.insert(
                DATA_KEY.into(),
                serde_json::json!({"version":1,"mode":"manual","passages":passages}),
            );
        }
    }
    Ok(Value::Object(object).to_string())
}

/// Bad or future coverage data cannot silently turn into an inferred claim.
/// The rest of the source remains readable and its stored metadata stays intact.
pub(crate) fn stored(data: Option<&str>) -> (CoverageMode, Vec<SourceCoverage>) {
    let Some(data) = data.filter(|data| !data.trim().is_empty()) else {
        return (CoverageMode::Automatic, Vec::new());
    };
    let Ok(value) = serde_json::from_str::<Value>(data) else {
        return (CoverageMode::None, Vec::new());
    };
    let Some(object) = value.as_object() else {
        return (CoverageMode::None, Vec::new());
    };
    let Some(value) = object.get(DATA_KEY) else {
        return (CoverageMode::Automatic, Vec::new());
    };
    let Ok(saved) = serde_json::from_value::<Stored>(value.clone()) else {
        return (CoverageMode::None, Vec::new());
    };
    if saved.version != 1 || saved.passages.len() > MAX_PASSAGES {
        return (CoverageMode::None, Vec::new());
    }
    if saved.mode != CoverageMode::Manual {
        return (saved.mode, Vec::new());
    }
    let ranges = saved
        .passages
        .iter()
        .map(|text| {
            ScriptureRef::from_canonical(text)
                .map(|reference| SourceCoverage::new(reference, CoverageBasis::Manual))
        })
        .collect::<AppResult<Vec<_>>>();
    match ranges {
        Ok(ranges) if !ranges.is_empty() => (CoverageMode::Manual, ranges),
        _ => (CoverageMode::None, Vec::new()),
    }
}

pub(crate) fn contains(outer: &ScriptureRef, inner: &ScriptureRef) -> bool {
    outer.start_key() <= inner.start_key() && outer.end_key() >= inner.end_key()
}
pub(crate) fn overlaps(a: &ScriptureRef, b: &ScriptureRef) -> bool {
    a.start_key() <= b.end_key() && b.start_key() <= a.end_key()
}

fn context_guard(title: &str, subtitle: Option<&str>, kind: &str) -> bool {
    if matches!(kind, "commentary" | "sermon" | "lecture") {
        return true;
    }
    let text = fold(&format!("{title} {}", subtitle.unwrap_or_default()));
    text.split(|c: char| !c.is_alphabetic()).any(|word| {
        [
            "commentary",
            "exposition",
            "sermon",
            "lecture",
            "biblical",
            "bible",
            "bibel",
            "bibelstudie",
            "kommentar",
            "auslegung",
            "predigt",
            "biblisch",
            "commentaire",
            "biblique",
            "comentario",
            "bíblico",
            "bíblica",
            "comentário",
            "성경",
            "주석",
            "강해",
        ]
        .into_iter()
        .any(|guard| fold(guard) == word)
    })
}

// Compare normalized text, but remove original characters at UTF-8 boundaries.
// Accents may change a byte length during folding (e.g. "étude", "über").
fn strip_title_prefix<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let boundary = text
        .char_indices()
        .nth(prefix.chars().count())
        .map_or(text.len(), |(at, _)| at);
    (fold(&text[..boundary]) == fold(prefix)).then_some(&text[boundary..])
}

fn strip_title_suffix<'a>(text: &'a str, suffix: &str) -> Option<&'a str> {
    let remaining = text.chars().count().checked_sub(suffix.chars().count())?;
    let boundary = text
        .char_indices()
        .nth(remaining)
        .map_or(text.len(), |(at, _)| at);
    (fold(&text[boundary..]) == fold(suffix)).then_some(&text[..boundary])
}

fn title_piece(text: &str) -> &str {
    let mut text = text.trim();
    for _ in 0..2 {
        let prefix = [
            "a commentary on ",
            "commentary on ",
            "an exposition of ",
            "exposition of ",
            "a sermon on ",
            "sermon on ",
            "lecture on ",
            "bible study on ",
            "commentary: ",
            "kommentar zum ",
            "kommentar zu ",
            "auslegung von ",
            "bibelstudie zu ",
            "predigt über ",
            "commentaire sur ",
            "étude biblique sur ",
            "comentario sobre ",
            "estudio bíblico de ",
            "comentário sobre ",
            "estudo bíblico de ",
        ]
        .into_iter()
        .find_map(|prefix| strip_title_prefix(text, prefix));
        if let Some(rest) = prefix {
            text = rest.trim();
            continue;
        }
        let suffix = [
            " commentary",
            " exposition",
            " bible study",
            " kommentar",
            " auslegung",
            " 주석",
            " 강해",
            " 성경 공부",
        ]
        .into_iter()
        .find_map(|suffix| strip_title_suffix(text, suffix));
        if let Some(rest) = suffix {
            text = rest.trim();
            continue;
        }
        break;
    }
    // A descriptor such as "A Commentary" must not leave a bare article to
    // be interpreted as a short Bible-book abbreviation in another language.
    if [
        "a", "an", "the", "der", "die", "das", "le", "la", "les", "un", "une", "el", "los", "las",
        "una", "o", "os", "as", "um", "uma",
    ]
    .contains(&fold(text).as_str())
    {
        ""
    } else {
        text
    }
}

/// Metadata proposes a scope; actual structure must corroborate it separately.
/// No body text, citation count, publisher, filename or series field is read.
pub(crate) fn candidate(
    title: &str,
    subtitle: Option<&str>,
    kind: &str,
    language: ReferenceLanguage,
) -> Option<ScriptureRef> {
    // Metadata from PDF/EPUB producers may use decomposed accents. Keep this
    // normalization local to whole-source titles, outside reference detection.
    let title = title.nfc().collect::<String>();
    let subtitle = subtitle.map(|text| text.nfc().collect::<String>());
    let title = title.as_str();
    let subtitle = subtitle.as_deref();
    if !context_guard(title, subtitle, kind) {
        return None;
    }
    let mut candidates = Vec::new();
    for field in [Some(title), subtitle].into_iter().flatten() {
        if field.chars().count() > 300 {
            return None;
        }
        if super::pdf_structure::book_references(title_piece(field), language).len() > 1
            || crate::scripture::recognise::Recogniser::for_source(language)
                .detect(field)
                .len()
                > 1
        {
            return None;
        }
        if let Ok(reference) = parse(title_piece(field), language) {
            candidates.push(reference);
        }
        let mut pieces = vec![field];
        for separator in [" — ", " – ", " - ", ": "] {
            pieces = pieces
                .into_iter()
                .flat_map(|piece| piece.split(separator))
                .collect();
        }
        for piece in pieces {
            if let Ok(reference) = parse(title_piece(piece), language) {
                if !candidates.contains(&reference) {
                    candidates.push(reference);
                }
            }
        }
    }
    let mut candidate = *candidates.first()?;
    for reference in candidates.iter().skip(1) {
        if contains(&candidate, reference) {
            candidate = *reference;
        } else if !contains(reference, &candidate) {
            return None;
        }
    }
    Some(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_scope_accepts_explicit_bibliographic_forms_but_not_mixed_or_generic_titles() {
        for title in [
            "Romans 8 — A Commentary",
            "Romans 8: A Commentary",
            "A Commentary on Romans 8",
            "Romans 8 Commentary",
        ] {
            assert_eq!(
                candidate(title, None, "commentary", ReferenceLanguage::En)
                    .map(|r| r.canonical())
                    .as_deref(),
                Some("ROM.8"),
                "{title}: {:?}",
                crate::scripture::recognise::Recogniser::for_source(ReferenceLanguage::En)
                    .detect(title)
            );
        }
        for title in [
            "Romans and Galatians: A Commentary",
            "General Essays",
            "Romans Bible Commentary Series",
            "A Commentary",
        ] {
            assert!(
                candidate(title, None, "commentary", ReferenceLanguage::En).is_none(),
                "{title}"
            );
        }
    }
}
