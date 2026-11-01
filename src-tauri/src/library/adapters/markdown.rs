//! Markdown (.md, .markdown): headings build a section path; paragraphs,
//! list items, quotations and code keep their order. Embedded HTML is
//! dropped, never interpreted or executed.
//!
//! Footnotes in the common extended syntax (pandoc, GitHub: `text[^1]` and a
//! definition `[^1]: note`) are what the document declares them to be
//! (v0.24.6): a caller is marked for the Scripture recogniser as a note's
//! caller (as an EPUB's declared noteref is), and a definition is kept as a
//! note, never as running text.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::pdf_layout::{NOTE_CLOSE, NOTE_OPEN};

use super::text::decode_utf8;
use super::{
    collapse, split_long, ExtractionStatus, FoundMetadata, ImportedDocument, NewSegment,
    SourceImportAdapter,
};
use crate::error::AppResult;

pub struct MarkdownImporter;

const MAX_SEGMENT: usize = 1_500;

fn level(heading: HeadingLevel) -> usize {
    match heading {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

impl SourceImportAdapter for MarkdownImporter {
    fn format(&self) -> &'static str {
        "markdown"
    }

    fn accepts(&self, extension: &str) -> bool {
        matches!(extension, "md" | "markdown")
    }

    fn extract(&self, bytes: &[u8], file_name: &str) -> AppResult<ImportedDocument> {
        let source = decode_utf8(bytes, file_name)?;
        let mut segments: Vec<NewSegment> = Vec::new();
        // headings[0] = H1 text, headings[1] = H2 …
        let mut headings: Vec<Option<String>> = vec![None; 6];
        let mut title: Option<String> = None;
        let mut top_level = 0u32;
        let mut buffer = String::new();
        let mut in_heading: Option<usize> = None;
        let mut block_start = 0usize;
        let mut paragraph_in_section = 0u32;
        // Emphasis open now, and whether every piece of the block's text so
        // far stood inside it: a paragraph written wholly in italics or bold
        // ("*TEMA: …*", "*Mateus 1:1–16*") is set apart by the document's own
        // markup, as an EPUB's styled class is (v0.24.5).
        let mut emphasis = 0usize;
        let mut all_emphasised = true;
        // The block's text with its footnote callers marked (`NOTE_OPEN` label
        // `NOTE_CLOSE`), and whether it has any; inside a footnote definition.
        let mut marked = String::new();
        let mut has_callers = false;
        let mut in_footnote = 0usize;
        let mut html = String::new();

        let path = |headings: &[Option<String>]| -> Option<String> {
            let parts: Vec<&str> = headings.iter().flatten().map(String::as_str).collect();
            (!parts.is_empty()).then(|| parts.join(" › "))
        };

        for (event, range) in Parser::new_ext(&source, Options::ENABLE_FOOTNOTES).into_offset_iter()
        {
            match event {
                Event::Start(Tag::Heading { level: l, .. }) => {
                    in_heading = Some(level(l));
                    buffer.clear();
                    marked.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    let text = collapse(&buffer);
                    buffer.clear();
                    if let Some(l) = in_heading.take() {
                        if l == 1 && title.is_none() {
                            title = Some(text.clone());
                        }
                        if l <= 2 {
                            top_level += 1;
                        }
                        headings[l - 1] = Some(text);
                        for deeper in headings.iter_mut().skip(l) {
                            *deeper = None;
                        }
                        paragraph_in_section = 0;
                    }
                }
                Event::Start(Tag::Paragraph | Tag::CodeBlock(_) | Tag::Item) => {
                    buffer.clear();
                    marked.clear();
                    has_callers = false;
                    block_start = range.start;
                    all_emphasised = true;
                }
                Event::Start(Tag::FootnoteDefinition(_)) => in_footnote += 1,
                Event::End(TagEnd::FootnoteDefinition) => {
                    in_footnote = in_footnote.saturating_sub(1);
                }
                // A footnote's caller: kept in the text as written, marked
                // for the recogniser as a declared caller.
                Event::FootnoteReference(label) => {
                    buffer.push_str(&format!("[^{label}]"));
                    marked.push(NOTE_OPEN);
                    marked.push_str(&label);
                    marked.push(NOTE_CLOSE);
                    has_callers = true;
                    all_emphasised = false;
                }
                Event::Start(Tag::Emphasis | Tag::Strong) => emphasis += 1,
                Event::End(TagEnd::Emphasis | TagEnd::Strong) => {
                    emphasis = emphasis.saturating_sub(1);
                }
                // A tight list item has text but no paragraph of its own.
                // A hard line break ends a line of its own ("1:1 …  " / "1:2
                // …", a verse per line, v0.24.3).
                Event::End(TagEnd::Paragraph | TagEnd::CodeBlock | TagEnd::Item)
                | Event::HardBreak => {
                    let hard = matches!(event, Event::HardBreak);
                    if hard && in_heading.is_some() {
                        buffer.push(' ');
                        continue;
                    }
                    let text = collapse(&buffer);
                    buffer.clear();
                    let marked_text = collapse(&std::mem::take(&mut marked));
                    let callers = std::mem::replace(&mut has_callers, false);
                    let styled =
                        std::mem::replace(&mut all_emphasised, true) && in_heading.is_none();
                    if text.is_empty() {
                        continue;
                    }
                    let section_path = path(&headings);
                    let heading = headings.iter().rev().flatten().next().cloned();
                    let parts = split_long(&text, MAX_SEGMENT);
                    // (The marked text split the same way; where the two do
                    // not split alike, no marks: the callers then stay as
                    // written, and a verse holding one is not verified.)
                    let marked_parts = if callers {
                        let m = split_long(&marked_text, MAX_SEGMENT);
                        (m.len() == parts.len()).then_some(m)
                    } else {
                        None
                    };
                    for (n, part) in parts.into_iter().enumerate() {
                        segments.push(NewSegment {
                            note: in_footnote > 0,
                            marked: marked_parts.as_ref().map(|m| m[n].clone()),
                            text: part,
                            chapter: headings[0].clone(),
                            heading: heading.clone(),
                            section_path: section_path.clone(),
                            paragraph_index: Some(paragraph_in_section),
                            char_start: Some(block_start as u32),
                            char_end: Some(range.end as u32),
                            styled,
                            ..Default::default()
                        });
                        paragraph_in_section += 1;
                    }
                    if hard {
                        block_start = range.end;
                    }
                }
                Event::Text(t) | Event::Code(t) => {
                    if emphasis == 0 && !t.trim().is_empty() {
                        all_emphasised = false;
                    }
                    buffer.push_str(&t);
                    marked.push_str(&t);
                }
                Event::SoftBreak => {
                    buffer.push(' ');
                    marked.push(' ');
                }
                // Raw HTML is ignored: never rendered, never executed.
                // A block of raw HTML: never rendered or executed, but its
                // text is the document's (a verse's closing "Amen." in a
                // centred <div>): kept as a paragraph, read as plain text
                // with every tag taken out and script, style and comments
                // dropped whole; set apart when it uses anything beyond
                // simple text formatting (v0.24.7).
                Event::Start(Tag::HtmlBlock) => html.clear(),
                Event::Html(t) => html.push_str(&t),
                Event::End(TagEnd::HtmlBlock) => {
                    if let Some((text, set_apart)) = html_block_text(&html) {
                        let section_path = path(&headings);
                        let heading = headings.iter().rev().flatten().next().cloned();
                        for part in split_long(&text, MAX_SEGMENT) {
                            segments.push(NewSegment {
                                note: in_footnote > 0,
                                text: part,
                                chapter: headings[0].clone(),
                                heading: heading.clone(),
                                section_path: section_path.clone(),
                                paragraph_index: Some(paragraph_in_section),
                                char_start: Some(range.start as u32),
                                char_end: Some(range.end as u32),
                                styled: set_apart,
                                ..Default::default()
                            });
                            paragraph_in_section += 1;
                        }
                    }
                    html.clear();
                }
                Event::InlineHtml(_) => {}
                _ => {}
            }
        }

        Ok(ImportedDocument {
            format: "markdown",
            metadata: FoundMetadata {
                title,
                ..Default::default()
            },
            segments,
            extraction_status: ExtractionStatus::DigitalText,
            page_count: None,
            chapter_count: Some(top_level),
        })
    }
}

/// The plain text of a block of raw HTML, and whether it is set apart (it
/// uses more than simple text formatting: headings, tables, lists, links,
/// unknown elements). Script, style, template and iframe elements and
/// comments are dropped with their content; every other tag is taken out;
/// the common character references are decoded. Nothing is interpreted or
/// executed. None when no text is left.
fn html_block_text(html: &str) -> Option<(String, bool)> {
    const SIMPLE: &[&str] = &[
        "div",
        "p",
        "span",
        "center",
        "b",
        "i",
        "em",
        "strong",
        "sup",
        "sub",
        "br",
        "small",
        "big",
        "u",
        "font",
        "blockquote",
    ];
    // (ASCII lower case only: byte positions must stay those of `html`.)
    let lower = html.to_ascii_lowercase();
    let mut text = String::new();
    let mut set_apart = false;
    let mut at = 0usize;
    let bytes = html.as_bytes();
    while at < html.len() {
        if bytes[at] == b'<' {
            // A comment, or an element dropped with its content.
            if lower[at..].starts_with("<!--") {
                at = lower[at..].find("-->").map_or(html.len(), |e| at + e + 3);
                continue;
            }
            let name: String = lower[at + 1..]
                .trim_start_matches('/')
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect();
            if ["script", "style", "template", "iframe"].contains(&name.as_str())
                && !lower[at + 1..].starts_with('/')
            {
                let close = format!("</{name}");
                at = match lower[at..].find(&close) {
                    Some(e) => lower[at + e..]
                        .find('>')
                        .map_or(html.len(), |g| at + e + g + 1),
                    None => html.len(),
                };
                continue;
            }
            if !name.is_empty() && !SIMPLE.contains(&name.as_str()) {
                set_apart = true;
            }
            match html[at..].find('>') {
                Some(g) => {
                    text.push(' ');
                    at += g + 1;
                }
                None => break,
            }
            continue;
        }
        let next = html[at..].find('<').map_or(html.len(), |n| at + n);
        text.push_str(&html[at..next]);
        at = next;
    }
    let decoded = text
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&");
    let text = collapse(&decoded);
    (!text.is_empty()).then_some((text, set_apart))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_heading_hierarchy_and_paragraph_order() {
        let md = "# Romans Notes\n\nIntro line.\n\n## Justification\n\nDeclared righteous.\n\n- by faith\n\n### Imputation\n\nCounted, not infused.\n\n## Adoption\n\n<script>alert(1)</script>\n\nSons and heirs.";
        let doc = MarkdownImporter
            .extract(md.as_bytes(), "romans.md")
            .unwrap();
        assert_eq!(doc.metadata.title.as_deref(), Some("Romans Notes"));
        let seen: Vec<_> = doc
            .segments
            .iter()
            .map(|s| (s.text.as_str(), s.section_path.as_deref().unwrap_or("")))
            .collect();
        assert_eq!(
            seen,
            [
                ("Intro line.", "Romans Notes"),
                ("Declared righteous.", "Romans Notes › Justification"),
                ("by faith", "Romans Notes › Justification"),
                (
                    "Counted, not infused.",
                    "Romans Notes › Justification › Imputation"
                ),
                ("Sons and heirs.", "Romans Notes › Adoption"),
            ]
        );
        assert!(doc
            .segments
            .iter()
            .all(|s| !s.text.contains("script") && !s.text.contains("alert")));
        assert_eq!(doc.segments[3].heading.as_deref(), Some("Imputation"));
        assert!(doc.segments.iter().all(|s| s.page_number.is_none()));
    }

    #[test]
    fn a_paragraph_wholly_in_emphasis_is_set_apart() {
        // "*TEMA: …*" between "**1:1** …" verse lines: the whole paragraph
        // in italics is styled; a verse line with a bold number is not.
        let md = "## Filemom\n\n**1:1** Paulo, prisioneiro.\n\n*TEMA: Sempre dou graças.*\n\n**1:2** E à irmã *Áfia*.\n";
        let doc = MarkdownImporter.extract(md.as_bytes(), "f.md").unwrap();
        let styled: Vec<(&str, bool)> = doc
            .segments
            .iter()
            .map(|s| (s.text.as_str(), s.styled))
            .collect();
        assert_eq!(
            styled,
            [
                ("1:1 Paulo, prisioneiro.", false),
                ("TEMA: Sempre dou graças.", true),
                ("1:2 E à irmã Áfia.", false),
            ]
        );
    }

    #[test]
    fn footnote_callers_are_marked_and_definitions_are_notes() {
        // Pandoc/GitHub footnotes: the caller stays in the text as written
        // and is marked for the recogniser; the definition, even when it
        // opens with a bold chapter and verse, is a note, never text.
        use super::super::pdf_layout::{NOTE_CLOSE, NOTE_OPEN};
        let md = "## Lucas 17\n\n**35** Duas mulheres estarão moendo.[^luk-17-1]\n\n**37** Eles perguntaram.\n\n[^luk-17-1]: **17:36** Alguns manuscritos acrescentam: dois estarão no campo.\n";
        let doc = MarkdownImporter.extract(md.as_bytes(), "l.md").unwrap();
        let verse = doc
            .segments
            .iter()
            .find(|s| s.text.starts_with("35"))
            .unwrap();
        assert_eq!(verse.text, "35 Duas mulheres estarão moendo.[^luk-17-1]");
        assert_eq!(
            verse.marked.as_deref(),
            Some(
                format!("35 Duas mulheres estarão moendo.{NOTE_OPEN}luk-17-1{NOTE_CLOSE}").as_str()
            )
        );
        assert!(!verse.note);
        let note = doc
            .segments
            .iter()
            .find(|s| s.text.contains("manuscritos"))
            .unwrap();
        assert!(note.note, "{note:?}");
        assert!(doc
            .segments
            .iter()
            .filter(|s| !s.note)
            .all(|s| !s.text.contains("manuscritos")));
    }
}
