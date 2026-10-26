//! A PDF page's text in reading order, from where the text is drawn (v0.24.1).
//!
//! The text layer of a PDF is a list of drawing operations: the order in which
//! a typesetting program happened to draw the characters, not the order in
//! which they are read. On a page set in two columns that order often runs
//! across both columns line by line, so the lines of one column end up between
//! the lines of the other. This module reads the **position** of every
//! character instead (through `pdf_extract`'s output device: the same parser
//! the library has always used) and puts the page back together:
//!
//! 1. characters drawn one after another on the same baseline become a
//!    **span** (a run of words), never crossing a wide gap;
//! 2. a **running head** or a **page number** standing apart at the top or the
//!    bottom of the page is taken first or last;
//! 3. a **gutter** — a vertical strip that no span crosses, with columns of
//!    text on both sides — divides the page into columns. Lines that cross the
//!    gutter (a book title over both columns, footnotes across the page) cut
//!    the page into bands: each band is read left column, then right column;
//! 4. within a column, spans on the same baseline (raised verse numbers
//!    included) make a **line**, read left to right; lines are read top to
//!    bottom; a large initial (a chapter number set over two lines) opens the
//!    first line beside it.
//!
//! Nothing is guessed. When the page cannot be put in order with confidence
//! (text set at an angle, a line crossing the gutter in the middle of a
//! column), the page is marked **uncertain**: its text is kept in the order it
//! was drawn, as before, and readers that depend on the order (the Scripture
//! recogniser) do not use it. No character is added, dropped or repeated: each
//! one drawn is in exactly one span (rotated text is kept at the end of its
//! page), and spans keep the index of their first character in drawing order
//! as their provenance.

use std::cmp::Ordering;
use std::collections::HashMap;

/// More characters than this in one file are not examined (a very long
/// document is read as far as this; the rest is reported, not guessed).
pub const MAX_CHARS: usize = 40_000_000;
/// Pieces of text on one page beyond which it is not put in order.
const MAX_SPANS: usize = 50_000;

/// How a page's reading order was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageOrder {
    /// No text on the page.
    Empty,
    /// One column, read top to bottom.
    Single,
    /// Several columns (with bands where lines cross them).
    Columns(u8),
    /// The order could not be established; the text is in drawing order.
    Uncertain,
}

/// A run of characters drawn one after another on one baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    /// Left and right edges, in PDF units from the page's left edge.
    pub x0: f64,
    pub x1: f64,
    /// The baseline, in PDF units from the page's top edge (down is more).
    pub y: f64,
    /// The text size (font size scaled by the text matrix).
    pub size: f64,
    /// The index of its first character in the page's drawing order.
    pub first_char: usize,
    /// Of its characters whose usual width on the page is known, how many
    /// are drawn with another width (`styled`): set in another typeface
    /// (bold, italic) than the page's running text.
    pub compared: u32,
    pub styled: u32,
}

impl Span {
    fn top(&self) -> f64 {
        self.y - 0.8 * self.size
    }

    fn bottom(&self) -> f64 {
        self.y + 0.25 * self.size
    }
}

/// What a line is on its page, from where it stands and how it is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineRole {
    /// The page's running text.
    Text,
    /// A line standing apart at the top of the page (a running head).
    Head,
    /// A line standing apart at the bottom of the page (a page number).
    Foot,
    /// Notes: smaller text standing below all of the page's running text.
    Note,
    /// Text set at an angle, kept after the rest of the page.
    Rotated,
    /// The page's text in drawing order (its reading order is uncertain).
    Drawn,
}

/// One line of a page, in reading order.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub text: String,
    pub x0: f64,
    pub x1: f64,
    /// Baseline of the line (its main text), from the page's top edge.
    pub y: f64,
    /// Main text size of the line.
    pub size: f64,
    /// Which column it is in (0 = left or only; lines over several columns are 0).
    pub column: u8,
    /// Which block of text it belongs to (a new block starts a paragraph).
    pub block: u32,
    /// Drawing-order index of its first character (provenance).
    pub first_char: usize,
    pub role: LineRole,
    /// The line's text with raised small letters (a cross-reference's letter
    /// set against a word: "ᶜEt") between [`RAISED_OPEN`] and
    /// [`RAISED_CLOSE`]; None when it has none. For readers that need to know
    /// (the Scripture recogniser); the stored text is `text`.
    pub marked: Option<String>,
    /// Most of the line is set in another typeface than the page's running
    /// text (bold or italic: a heading, a label), judged from the widths
    /// its font gives its letters.
    pub styled: bool,
}

/// Around raised small letters in [`Line::marked`] (private-use characters,
/// never in a document's text).
pub const RAISED_OPEN: char = '\u{E000}';
pub const RAISED_CLOSE: char = '\u{E001}';

/// One or two lower-case letters set smaller and raised above the line's
/// baseline: a note's or cross-reference's letter.
fn raised_letters(s: &Span, main: &Span) -> bool {
    let n = s.text.chars().count();
    (1..=2).contains(&n)
        && s.text
            .chars()
            .all(|c| c.is_alphabetic() && c.is_lowercase())
        && {
            let lift = main.y - s.y;
            // Smaller and raised (a superscript letter), or as large as the
            // text but clearly lifted off its baseline (some editions set
            // their reference letters so). Never so high it is another line.
            (s.size < 0.85 * main.size && lift > 0.2 * main.size)
                || (lift > 0.12 * main.size && lift < 0.7 * main.size)
        }
}

/// A page put in reading order.
#[derive(Debug, Clone, PartialEq)]
pub struct PageLayout {
    /// 1 = the first page.
    pub number: u32,
    pub width: f64,
    pub height: f64,
    pub order: PageOrder,
    pub lines: Vec<Line>,
    /// Characters drawn at an angle (kept at the end of the page).
    pub rotated_chars: usize,
    /// Characters drawn on the page.
    pub chars: usize,
}

impl PageLayout {
    /// A page whose text could not be read.
    pub fn empty(number: u32) -> Self {
        PageLayout {
            number,
            width: 0.0,
            height: 0.0,
            order: PageOrder::Uncertain,
            lines: Vec::new(),
            rotated_chars: 0,
            chars: 0,
        }
    }

    /// The size most of the page's running text is set in (by characters).
    pub fn body_size(&self) -> f64 {
        let mut sizes: Vec<(f64, usize)> = Vec::new();
        for l in self.lines.iter().filter(|l| l.role == LineRole::Text) {
            let n = l.text.chars().filter(|c| !c.is_whitespace()).count();
            let key = (l.size * 2.0).round() / 2.0;
            match sizes.iter_mut().find(|(k, _)| (*k - key).abs() < 1e-6) {
                Some((_, count)) => *count += n,
                None => sizes.push((key, n)),
            }
        }
        sizes
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .map_or(10.0, |(k, _)| k)
    }

    /// The page's text: its lines in reading order, a blank line between
    /// blocks (paragraphs).
    pub fn text(&self) -> String {
        let mut out = String::new();
        let mut block = None;
        for line in &self.lines {
            if block.is_some() && block != Some(line.block) {
                out.push('\n');
            }
            block = Some(line.block);
            out.push_str(&line.text);
            out.push('\n');
        }
        out
    }
}

// --- collecting characters ----------------------------------------------------

/// A glyph's text without a heap allocation in the usual case (one character,
/// or a short ligature): a whole Bible is millions of glyphs.
#[derive(Debug, Clone)]
enum GlyphText {
    Inline(u8, [u8; 14]),
    Heap(Box<str>),
}

impl From<&str> for GlyphText {
    fn from(text: &str) -> Self {
        if text.len() <= 14 {
            let mut bytes = [0u8; 14];
            bytes[..text.len()].copy_from_slice(text.as_bytes());
            GlyphText::Inline(text.len() as u8, bytes)
        } else {
            GlyphText::Heap(text.into())
        }
    }
}

impl std::ops::Deref for GlyphText {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            // Always whole UTF-8 copied from a &str.
            GlyphText::Inline(len, bytes) => {
                std::str::from_utf8(&bytes[..*len as usize]).unwrap_or("")
            }
            GlyphText::Heap(text) => text,
        }
    }
}

/// One character as drawn. Positions are kept in single precision (exact to
/// well under a thousandth of a point at page scale): millions are held.
#[derive(Debug, Clone)]
struct Glyph {
    text: GlyphText,
    x: f32,
    /// Baseline from the top of the page.
    y: f32,
    end: f32,
    size: f32,
    /// The width the font gives the glyph (thousandths of the text size):
    /// the same letter has another width in another typeface.
    width: u16,
    upright: bool,
}

#[derive(Default)]
struct PageGlyphs {
    number: u32,
    width: f64,
    height: f64,
    glyphs: Vec<Glyph>,
}

/// `pdf_extract`'s output device: every character drawn, with its place.
#[derive(Default)]
pub struct Collector {
    pages: Vec<PageGlyphs>,
    /// Each page's own gutter, and its layout when that gutter made it
    /// final at the page's end (its characters are then let go: a whole
    /// Bible is millions of them).
    found: Vec<(Option<Gutter>, Option<PageLayout>)>,
    top: f64,
    left: f64,
    chars: usize,
    /// Characters past [`MAX_CHARS`] (not examined).
    pub skipped: usize,
    /// The first page (1-based, in drawing order) whose text was cut off at
    /// [`MAX_CHARS`]: it and every page after it are incomplete.
    pub truncated_from: Option<u32>,
}

impl Collector {
    /// Pages laid out, in the order they were drawn.
    pub fn layouts(mut self) -> Vec<PageLayout> {
        // Pages whose end was never reported (an error part-way): their
        // gutter is found now.
        while self.found.len() < self.pages.len() {
            let gutter = body_gutter(&self.pages[self.found.len()]);
            self.found.push((gutter, None));
        }
        // The gutter the document's pages share (odd and even pages apart: a
        // book's margins alternate). A page with too little text to show its
        // own gutter (a last page whose right column holds one line) is read
        // with it, when no text of its own crosses it.
        let found: Vec<(u32, Option<Gutter>)> = self
            .pages
            .iter()
            .zip(&self.found)
            .map(|(p, (g, _))| (p.number, *g))
            .collect();
        let shared = |parity: u32| -> Option<Gutter> {
            let mut from: Vec<f64> = Vec::new();
            let mut to: Vec<f64> = Vec::new();
            for (n, g) in &found {
                if let (true, Some(g)) = (n % 2 == parity, g) {
                    from.push(g.from);
                    to.push(g.to);
                }
            }
            if from.len() < 5 {
                return None;
            }
            from.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            to.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            // The strip most pages leave free: from the latest start to the
            // earliest end among the middle half of the pages.
            let q = from.len() / 4;
            let (from, to) = (from[from.len() - 1 - q], to[q]);
            (to > from).then_some(Gutter { from, to })
        };
        let hints = [shared(0), shared(1)];
        self.pages
            .into_iter()
            .zip(self.found)
            .map(|(p, (_, done))| match done {
                Some(layout) => layout,
                None => {
                    let hint = hints[(p.number % 2) as usize];
                    layout_page(p, hint)
                }
            })
            .collect()
    }
}

impl pdf_extract::OutputDev for Collector {
    fn begin_page(
        &mut self,
        page_num: u32,
        media_box: &pdf_extract::MediaBox,
        _art_box: Option<(f64, f64, f64, f64)>,
    ) -> Result<(), pdf_extract::OutputError> {
        self.top = media_box.ury.max(media_box.lly);
        self.left = media_box.llx.min(media_box.urx);
        self.pages.push(PageGlyphs {
            number: page_num,
            width: (media_box.urx - media_box.llx).abs(),
            height: (media_box.ury - media_box.lly).abs(),
            glyphs: Vec::new(),
        });
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), pdf_extract::OutputError> {
        let Some(page) = self.pages.last_mut() else {
            return Ok(());
        };
        // A page that shows its own gutter is laid out the same whatever the
        // other pages share (the shared gutter only serves pages without
        // one): it is laid out now and its characters let go.
        let gutter = body_gutter(page);
        let done = gutter.map(|_| {
            let glyphs = std::mem::take(&mut page.glyphs);
            layout_page(
                PageGlyphs {
                    number: page.number,
                    width: page.width,
                    height: page.height,
                    glyphs,
                },
                None,
            )
        });
        self.found.push((gutter, done));
        Ok(())
    }

    fn output_character(
        &mut self,
        trm: &pdf_extract::Transform,
        width: f64,
        spacing: f64,
        font_size: f64,
        char: &str,
    ) -> Result<(), pdf_extract::OutputError> {
        if self.chars >= MAX_CHARS {
            self.skipped += char.chars().count();
            if self.truncated_from.is_none() {
                self.truncated_from = Some(self.pages.len().max(1) as u32);
            }
            return Ok(());
        }
        self.chars += char.chars().count();
        let Some(page) = self.pages.last_mut() else {
            return Ok(());
        };
        let upright =
            trm.m12.abs() <= 1e-3 && trm.m21.abs() <= 1e-3 && trm.m11 > 0.0 && trm.m22 > 0.0;
        let size = if upright {
            font_size * (trm.m11 * trm.m22).sqrt()
        } else {
            font_size * (trm.m11 * trm.m22 - trm.m12 * trm.m21).abs().sqrt()
        };
        let size = if size.is_finite() && size > 0.0 {
            size
        } else {
            0.0
        };
        let x = trm.m31 - self.left;
        let y = self.top - trm.m32;
        let advance = (width * font_size + spacing) * trm.m11;
        let end = x + if advance.is_finite() { advance } else { 0.0 };
        let glyph_width = if width.is_finite() {
            (width * 1000.0).round().clamp(0.0, 60000.0) as u16
        } else {
            0
        };
        page.glyphs.push(Glyph {
            text: GlyphText::from(char),
            x: x as f32,
            y: y as f32,
            end: end as f32,
            size: size as f32,
            width: glyph_width,
            upright: upright && size > 0.0 && x.is_finite() && y.is_finite(),
        });
        Ok(())
    }

    fn begin_word(&mut self) -> Result<(), pdf_extract::OutputError> {
        Ok(())
    }

    fn end_word(&mut self) -> Result<(), pdf_extract::OutputError> {
        Ok(())
    }

    fn end_line(&mut self) -> Result<(), pdf_extract::OutputError> {
        Ok(())
    }
}

// --- spans -------------------------------------------------------------------

/// Characters drawn one after another on one baseline, with no wide gap,
/// become a span. A space is put where the gap between two characters is
/// wider than a narrow space.
fn spans(glyphs: &[Glyph]) -> Vec<Span> {
    // The usual width of each letter on the page (its running text's
    // typeface): the width most of its occurrences have.
    let mut widths: HashMap<char, HashMap<u16, u32>> = HashMap::new();
    for g in glyphs.iter().filter(|g| g.upright && g.width > 0) {
        let mut chars = g.text.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            if c.is_alphabetic() {
                *widths.entry(c).or_default().entry(g.width).or_default() += 1;
            }
        }
    }
    let usual: HashMap<char, u16> = widths
        .into_iter()
        .filter_map(|(c, counts)| {
            let total: u32 = counts.values().sum();
            let (w, n) = counts.into_iter().max_by_key(|(_, n)| *n)?;
            // Only letters seen often enough for "usual" to mean something.
            (total >= 8 && n * 2 > total).then_some((c, w))
        })
        .collect();
    let compare = |g: &Glyph| -> Option<bool> {
        let mut chars = g.text.chars();
        let (Some(c), None) = (chars.next(), chars.next()) else {
            return None;
        };
        let w = *usual.get(&c)?;
        // Another width by more than 3 % (rounding aside).
        Some((i32::from(g.width) - i32::from(w)).abs() * 100 > i32::from(w) * 3 + 100)
    };
    let mut out: Vec<Span> = Vec::new();
    let mut current: Option<(Span, f64)> = None; // span, x where the next character would go
    let finish = |out: &mut Vec<Span>, span: Option<(Span, f64)>| {
        if let Some((mut span, _)) = span {
            let trimmed = span.text.trim();
            if !trimmed.is_empty() {
                if trimmed.len() != span.text.len() {
                    span.text = trimmed.to_string();
                }
                out.push(span);
            }
        }
    };
    for (index, g) in glyphs.iter().enumerate() {
        if !g.upright {
            continue;
        }
        let blank = g.text.trim().is_empty();
        let continues = current.as_ref().is_some_and(|(span, next)| {
            let ratio = f64::from(g.size) / span.size;
            (f64::from(g.y) - span.y).abs() <= 0.15 * span.size.max(f64::from(g.size))
                && (0.8..=1.25).contains(&ratio)
                && f64::from(g.x) >= *next - 0.6 * span.size
                && f64::from(g.x) - *next <= 0.9 * span.size
        });
        if !continues {
            finish(&mut out, current.take());
            if blank {
                continue;
            }
            current = Some((
                Span {
                    text: String::new(),
                    x0: f64::from(g.x),
                    x1: f64::from(g.x),
                    y: f64::from(g.y),
                    size: f64::from(g.size),
                    first_char: index,
                    compared: 0,
                    styled: 0,
                },
                f64::from(g.x),
            ));
        }
        let (span, next) = current.as_mut().expect("a span is being built");
        if blank {
            if !span.text.ends_with(' ') {
                span.text.push(' ');
            }
        } else {
            if f64::from(g.x) - *next > 0.18 * span.size && !span.text.ends_with(' ') {
                span.text.push(' ');
            }
            span.text.push_str(&g.text);
            span.x1 = span.x1.max(f64::from(g.end).max(f64::from(g.x)));
            if let Some(other) = compare(g) {
                span.compared += 1;
                span.styled += u32::from(other);
            }
        }
        *next = f64::from(g.end).max(f64::from(g.x));
    }
    finish(&mut out, current.take());
    out
}

// --- lines -------------------------------------------------------------------

/// The size most of the text is set in (by characters).
fn body_size(spans: &[&Span]) -> f64 {
    let mut sizes: Vec<(f64, usize)> = Vec::new();
    for s in spans {
        let n = s.text.chars().filter(|c| !c.is_whitespace()).count();
        let key = (s.size * 2.0).round() / 2.0;
        match sizes.iter_mut().find(|(k, _)| (*k - key).abs() < 1e-6) {
            Some((_, count)) => *count += n,
            None => sizes.push((key, n)),
        }
    }
    sizes
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map_or(10.0, |(k, _)| k)
}

/// The page's running-text size. Usually its most common size; but on a page
/// dense with notes the notes' smaller size can be the most common. When a
/// larger size makes up a good part of the page and stands wholly above the
/// most common size's text, the larger is the running text and the smaller
/// the notes below it.
fn page_body_size(spans: &[&Span]) -> f64 {
    let common = body_size(spans);
    let letters = |s: &Span| s.text.chars().filter(|c| !c.is_whitespace()).count();
    let total: usize = spans.iter().map(|s| letters(s)).sum();
    let larger: Vec<&Span> = spans
        .iter()
        .copied()
        .filter(|s| s.size >= 1.1 * common && s.size < 1.6 * common && s.text.chars().count() > 2)
        .collect();
    let larger_letters: usize = larger.iter().map(|s| letters(s)).sum();
    if larger.is_empty() || larger_letters < 200.max(total / 10) {
        return common;
    }
    let larger_bottom = larger
        .iter()
        .map(|s| s.bottom())
        .fold(f64::NEG_INFINITY, f64::max);
    let common_top = spans
        .iter()
        .filter(|s| (s.size - common).abs() <= 0.1 * common)
        .map(|s| s.top())
        .fold(f64::INFINITY, f64::min);
    if common_top > larger_bottom {
        body_size(&larger)
    } else {
        common
    }
}

/// Spans of one column put into lines, top to bottom, each left to right. A
/// large initial (a chapter number set over two lines or more) opens the
/// first line it stands beside.
fn lines_of(spans: &[&Span], body: f64, column: u8, block: &mut u32) -> Vec<Line> {
    let (large, normal): (Vec<&Span>, Vec<&Span>) = spans
        .iter()
        .copied()
        .partition(|s| s.size >= 1.6 * body && s.text.chars().count() <= 4);
    let mut sorted = normal;
    sorted.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal));
    // Group by baseline: a raised or lowered verse number stays in its line.
    let mut groups: Vec<Vec<&Span>> = Vec::new();
    for span in sorted {
        let joins = groups
            .last()
            .is_some_and(|g| main_span(g).is_some_and(|main| same_line(main, span)));
        if joins {
            groups.last_mut().expect("a group").push(span);
        } else {
            groups.push(vec![span]);
        }
    }
    for initial in large {
        // The first line whose text is beside it (vertically within it).
        let place = groups.iter().position(|g| {
            let y = main_baseline(g);
            y >= initial.top() && y <= initial.bottom() + 0.5 * body
        });
        match place {
            Some(i) => groups[i].push(initial),
            None => {
                let at = groups
                    .iter()
                    .position(|g| main_baseline(g) > initial.y)
                    .unwrap_or(groups.len());
                groups.insert(at, vec![initial]);
            }
        }
    }
    let mut out = Vec::new();
    let mut previous: Option<(f64, f64)> = None; // baseline, size
    for mut group in groups {
        group.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap_or(Ordering::Equal));
        let y = main_baseline(&group);
        // The line's size is its main text's (most characters), not that of
        // a larger chapter number or initial beside it.
        let size = group
            .iter()
            .copied()
            .filter(|s| s.size < 1.6 * body)
            .max_by_key(|s| s.text.chars().count())
            .map_or(group[0].size, |s| s.size);
        // A wide space above the line starts a new block (paragraph).
        if let Some((py, ps)) = previous {
            if y - py > 1.9 * ps.max(size) {
                *block += 1;
            }
        }
        previous = Some((y, size));
        let mut text = String::new();
        let mut marked = String::new();
        let mut any_raised = false;
        let mut end: Option<(f64, f64)> = None;
        let main = main_span(&group);
        for s in &group {
            if let Some((x1, sz)) = end {
                if s.x0 - x1 > 0.18 * sz.min(s.size) && !text.ends_with(' ') {
                    text.push(' ');
                    marked.push(' ');
                }
            }
            text.push_str(&s.text);
            if main.is_some_and(|m| raised_letters(s, m)) {
                any_raised = true;
                marked.push(RAISED_OPEN);
                marked.push_str(&s.text);
                marked.push(RAISED_CLOSE);
            } else {
                marked.push_str(&s.text);
            }
            end = Some((s.x1, s.size));
        }
        out.push(Line {
            marked: any_raised.then_some(marked),
            text,
            x0: group.iter().map(|s| s.x0).fold(f64::INFINITY, f64::min),
            x1: group.iter().map(|s| s.x1).fold(f64::NEG_INFINITY, f64::max),
            y,
            size,
            column,
            block: *block,
            first_char: group.iter().map(|s| s.first_char).min().unwrap_or(0),
            role: LineRole::Text,
            styled: {
                let compared: u32 = group.iter().map(|s| s.compared).sum();
                let styled: u32 = group.iter().map(|s| s.styled).sum();
                // Lines of running text reach a third or so (accented
                // letters, a word in italics); a heading line set in another
                // face, well over half.
                compared >= 6 && styled * 100 >= compared * 55
            },
        });
    }
    out
}

/// The span of a line with the most characters (its main text).
fn main_span<'a>(group: &[&'a Span]) -> Option<&'a Span> {
    group.iter().copied().max_by_key(|s| s.text.chars().count())
}

/// The baseline most of a group's characters stand on.
fn main_baseline(group: &[&Span]) -> f64 {
    main_span(group).map_or(0.0, |s| s.y)
}

/// Whether a span stands on a line's baseline: on the same baseline, or a
/// smaller raised or lowered piece (a verse number, a note sign) within the
/// height of the line's text. A larger number set above the line (a chapter
/// number centred over its first verse) is a line of its own.
fn same_line(main: &Span, s: &Span) -> bool {
    if (s.y - main.y).abs() <= 0.2 * main.size.min(s.size) {
        return true;
    }
    // Set in nearly the same size, a little raised (a note's sign): most of
    // its height overlaps the line's.
    let overlap = s.bottom().min(main.bottom()) - s.top().max(main.top());
    let height = (s.bottom() - s.top()).min(main.bottom() - main.top());
    if (s.size - main.size).abs() <= 0.1 * main.size && overlap >= 0.6 * height {
        return true;
    }
    // A sign or two set in another size within the line's height (the signs
    // of notes printed larger than the notes).
    let short = |x: &Span| x.text.chars().filter(|c| !c.is_whitespace()).count() <= 3;
    if (short(s) || short(main)) && overlap >= 0.6 * height {
        return true;
    }
    let (small, big) = if s.size < main.size {
        (s, main)
    } else {
        (main, s)
    };
    small.size < 0.9 * big.size
        && small.top() >= big.top() - 0.15 * big.size
        && small.bottom() <= big.bottom() + 0.15 * big.size
}

// --- columns -----------------------------------------------------------------

/// A strip no span (or only a few, crossing it) runs through, with text on
/// both sides: the left edge, the right edge, and how many spans cross it.
#[derive(Debug, Clone, Copy)]
struct Gutter {
    from: f64,
    to: f64,
}

/// Finds the gutter between two columns, if the spans show one: at least six
/// spans entirely on each side, the strip at least half a character wide,
/// each column at least a fifth of the text's width, and few spans (lines
/// over both columns) crossing it.
fn find_gutter(spans: &[&Span], body: f64) -> Option<Gutter> {
    if spans.len() < 12 {
        return None;
    }
    let left_edge = spans.iter().map(|s| s.x0).fold(f64::INFINITY, f64::min);
    let right_edge = spans.iter().map(|s| s.x1).fold(f64::NEG_INFINITY, f64::max);
    let width = right_edge - left_edge;
    if width <= 0.0 {
        return None;
    }
    // Coverage of each x by spans, as a sweep over their edges.
    let mut events: Vec<(f64, i32)> = Vec::with_capacity(spans.len() * 2);
    for s in spans {
        events.push((s.x0, 1));
        events.push((s.x1, -1));
    }
    events.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(Ordering::Equal)
            .then(a.1.cmp(&b.1))
    });
    let mut segments: Vec<(f64, f64, i32)> = Vec::new();
    let mut depth = 0;
    for pair in events.windows(2) {
        depth += pair[0].1;
        if pair[1].0 > pair[0].0 {
            segments.push((pair[0].0, pair[1].0, depth));
        }
    }
    let mut ends: Vec<f64> = spans.iter().map(|s| s.x1).collect();
    let mut starts: Vec<f64> = spans.iter().map(|s| s.x0).collect();
    ends.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    starts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let left_of = |x: f64| ends.partition_point(|e| *e <= x + 1e-6);
    let right_of = |x: f64| spans.len() - starts.partition_point(|s| *s < x - 1e-6);
    let mut best: Option<(usize, f64, Gutter)> = None;
    for allowed in 0..=3 {
        let mut i = 0;
        while i < segments.len() {
            if segments[i].2 > allowed {
                i += 1;
                continue;
            }
            let from = segments[i].0;
            let mut to = segments[i].1;
            let mut j = i + 1;
            while j < segments.len() && segments[j].2 <= allowed {
                to = segments[j].1;
                j += 1;
            }
            i = j;
            if to - from < 0.5 * body
                || from - left_edge < 0.2 * width
                || right_edge - to < 0.2 * width
            {
                continue;
            }
            let (left, right) = (left_of(from), right_of(to));
            let fewer = left.min(right);
            let crossing = spans.len() - left - right;
            // A short column (three spans or more) beside a full one counts
            // only when nothing at all crosses a strip at least a character wide.
            let short_column =
                fewer >= 3 && left.max(right) >= 6 && crossing == 0 && to - from >= body;
            if (fewer < 6 && !short_column) || crossing * 6 > fewer {
                continue;
            }
            let better = best
                .as_ref()
                .is_none_or(|(n, w, _)| fewer > *n || (fewer == *n && to - from > *w));
            if better {
                best = Some((fewer, to - from, Gutter { from, to }));
            }
        }
    }
    best.map(|(_, _, g)| g)
}

/// Where a page is divided: lines in reading order, or None when the order is
/// uncertain.
/// [`arrange`] for a page's running text, with the gutter the document's
/// other pages share as a fall-back: used when the page shows none of its
/// own but has text on both sides of it and none across it (other than
/// lines standing clear of the columns).
fn arrange_page(
    spans: &[&Span],
    body: f64,
    block: &mut u32,
    hint: Option<Gutter>,
) -> Option<(Vec<Line>, u8)> {
    if find_gutter(spans, body).is_none() {
        if let Some(g) = hint {
            let left = spans.iter().any(|s| s.x1 <= g.from + 1e-6);
            let right = spans.iter().any(|s| s.x0 >= g.to - 1e-6);
            if left && right {
                return split(spans, body, 0, block, g);
            }
        }
    }
    arrange(spans, body, 0, block)
}

/// The widest space between two pieces of one line, in text sizes.
fn widest_gap(line: &[&Span]) -> f64 {
    let mut sorted: Vec<&Span> = line.to_vec();
    sorted.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap_or(Ordering::Equal));
    sorted
        .windows(2)
        .map(|w| (w[1].x0 - w[0].x1) / w[0].size.max(w[1].size).max(1.0))
        .fold(0.0, f64::max)
}

/// Lines of one column, or None when a line has a space inside it wider
/// than any word space: two columns run together (the order is uncertain).
fn column_lines(spans: &[&Span], body: f64, column: u8, block: &mut u32) -> Option<Vec<Line>> {
    let lines = lines_of(spans, body, column, block);
    for line in &lines {
        let pieces: Vec<&Span> = spans
            .iter()
            .copied()
            .filter(|s| {
                (s.y - line.y).abs() <= 0.5 * line.size
                    && s.x0 >= line.x0 - 1e-6
                    && s.x1 <= line.x1 + 1e-6
            })
            .filter(|s| s.size < 1.6 * body)
            .collect();
        if widest_gap(&pieces) > 2.5 {
            return None;
        }
    }
    Some(lines)
}

fn arrange(spans: &[&Span], body: f64, depth: u8, block: &mut u32) -> Option<(Vec<Line>, u8)> {
    let Some(gutter) = (depth < 2).then(|| find_gutter(spans, body)).flatten() else {
        return Some((column_lines(spans, body, 0, block)?, 1));
    };
    split(spans, body, depth, block, gutter)
}

/// Lines in reading order on both sides of a gutter (see [`arrange`]).
fn split(
    spans: &[&Span],
    body: f64,
    depth: u8,
    block: &mut u32,
    gutter: Gutter,
) -> Option<(Vec<Line>, u8)> {
    let middle = (gutter.from + gutter.to) / 2.0;
    let (mut crossing, mut inside): (Vec<&Span>, Vec<&Span>) = spans
        .iter()
        .copied()
        .partition(|s| s.x0 < gutter.from - 1e-6 && s.x1 > gutter.to + 1e-6);
    // A crossing line's other pieces on its own baseline belong to it (the
    // sign before a footnote, a second footnote on the same line) — but only
    // when they are not set in the body's size: text of the columns beside
    // a crossing line means the order cannot be known (below).
    loop {
        let joining: Vec<usize> = inside
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                ((s.size - body).abs() > 0.1 * body || s.text.chars().count() <= 2)
                    && crossing.iter().any(|c| {
                        (c.y - s.y).abs() <= 0.5 * c.size.max(s.size)
                            && (c.size - body).abs() > 0.1 * body
                    })
            })
            .map(|(i, _)| i)
            .collect();
        if joining.is_empty() {
            break;
        }
        for i in joining.into_iter().rev() {
            crossing.push(inside.remove(i));
        }
    }
    // Lines over both columns must stand clear of the columns: no column
    // text beside them. Otherwise the gutter is not a gutter there, and the
    // order cannot be known.
    let mut separators: Vec<(f64, f64)> = Vec::new(); // top, bottom of each crossing line
    for s in &crossing {
        let (top, bottom) = (s.top(), s.bottom());
        match separators.iter_mut().find(|(t, b)| top < *b && bottom > *t) {
            Some(sep) => *sep = (sep.0.min(top), sep.1.max(bottom)),
            None => separators.push((top, bottom)),
        }
    }
    separators.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    for (top, bottom) in &separators {
        let beside = inside.iter().any(|s| {
            let overlap = s.bottom().min(*bottom) - s.top().max(*top);
            overlap > 0.3 * s.size
        });
        if beside {
            return None;
        }
    }
    // Which column a span belongs to. A span reaching into the gutter (but
    // not across it) belongs to the column it starts or ends in; none is
    // ever left out.
    let on_left = |s: &Span| {
        if s.x1 <= middle {
            true
        } else if s.x0 >= middle {
            false
        } else if s.x0 < gutter.from {
            true
        } else if s.x1 > gutter.to {
            false
        } else {
            s.x0 + s.x1 <= 2.0 * middle
        }
    };
    // Bands between the crossing lines; in each, the left column, then the right.
    let mut out = Vec::new();
    let mut columns = 2u8;
    let mut band_top = f64::NEG_INFINITY;
    let mut bounds: Vec<(f64, Option<(f64, f64)>)> =
        separators.iter().map(|&(t, b)| (t, Some((t, b)))).collect();
    bounds.push((f64::INFINITY, None));
    for (limit, separator) in bounds {
        let in_band = |s: &&Span| {
            let centre = (s.top() + s.bottom()) / 2.0;
            centre > band_top && centre <= limit
        };
        let left: Vec<&Span> = inside
            .iter()
            .copied()
            .filter(|s| on_left(s) && in_band(s))
            .collect();
        let right: Vec<&Span> = inside
            .iter()
            .copied()
            .filter(|s| !on_left(s) && in_band(s))
            .collect();
        for (index, side) in [left, right].into_iter().enumerate() {
            if side.is_empty() {
                continue;
            }
            *block += 1;
            let (mut lines, n) = arrange(&side, body, depth + 1, block)?;
            columns = columns.max(n + 1);
            for line in &mut lines {
                line.column = line.column.max(index as u8);
            }
            out.extend(lines);
        }
        if let Some((top, bottom)) = separator {
            let over: Vec<&Span> = crossing
                .iter()
                .copied()
                .filter(|s| {
                    let centre = (s.top() + s.bottom()) / 2.0;
                    centre >= top && centre <= bottom
                })
                .collect();
            *block += 1;
            out.extend(lines_of(&over, body, 0, block));
            band_top = bottom;
        }
    }
    // Every character drawn must be read exactly once; if not, the order is
    // not known.
    let letters = |t: &str| t.chars().filter(|c| !c.is_whitespace()).count();
    let drawn: usize = spans.iter().map(|s| letters(&s.text)).sum();
    let read: usize = out.iter().map(|l| letters(&l.text)).sum();
    (drawn == read).then_some((out, columns))
}

/// Lines standing apart at the very top or bottom of the page (a running
/// head, a page number): the spans of the first or last line, when a wide
/// space separates it from the rest and it lies within the outer eighth of
/// the page.
fn peel(spans: &mut Vec<&Span>, body: f64, height: f64, top: bool) -> Vec<Span> {
    if spans.len() < 4 {
        return Vec::new();
    }
    let mut order: Vec<usize> = (0..spans.len()).collect();
    order.sort_by(|&a, &b| {
        let (ya, yb) = (spans[a].y, spans[b].y);
        let o = ya.partial_cmp(&yb).unwrap_or(Ordering::Equal);
        if top {
            o
        } else {
            o.reverse()
        }
    });
    let first = spans[order[0]].y;
    let line: Vec<usize> = order
        .iter()
        .copied()
        .take_while(|&i| (spans[i].y - first).abs() <= 0.5 * spans[i].size.max(body))
        .collect();
    let Some(&next) = order.get(line.len()) else {
        return Vec::new();
    };
    let edge = |i: usize| {
        if top {
            spans[i].bottom()
        } else {
            spans[i].top()
        }
    };
    let line_edge = line.iter().map(|&i| edge(i)).fold(
        if top {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        },
        |a, b| if top { a.max(b) } else { a.min(b) },
    );
    let next_edge = if top {
        spans[next].top()
    } else {
        spans[next].bottom()
    };
    let gap = if top {
        next_edge - line_edge
    } else {
        line_edge - next_edge
    };
    // Set apart by a clear space, or — a smaller head or footer on a compact
    // page — by a narrower one.
    let smaller = line.iter().all(|&i| spans[i].size <= 0.9 * body);
    // In the outer eighth of the page — or, set smaller than the text, in
    // its outer quarter (a book-size page with wide margins).
    let reach = if smaller { 4.0 } else { 8.0 };
    let outer = if top {
        first <= height / reach
    } else {
        first >= height * (reach - 1.0) / reach
    };
    if !outer || (gap < 0.6 * body && !(smaller && gap >= 0.2 * body)) {
        return Vec::new();
    }
    // A heading at the top of one column ("Psalm 24" over the right column,
    // set a little higher than the left column's first line) is not a
    // running head: body-size, on one side of the page's middle, with words
    // and no passage reference ("1:5") or page number in it.
    if top && !smaller {
        let left = spans.iter().map(|s| s.x0).fold(f64::INFINITY, f64::min);
        let right = spans.iter().map(|s| s.x1).fold(f64::NEG_INFINITY, f64::max);
        let centre = (left + right) / 2.0;
        let x0 = line
            .iter()
            .map(|&i| spans[i].x0)
            .fold(f64::INFINITY, f64::min);
        let x1 = line
            .iter()
            .map(|&i| spans[i].x1)
            .fold(f64::NEG_INFINITY, f64::max);
        let text: String = line
            .iter()
            .map(|&i| spans[i].text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let numbers = text
            .split_whitespace()
            .filter(|w| w.chars().all(|c| c.is_ascii_digit()))
            .count();
        let one_side = x1 < centre || x0 > centre;
        let words = text.chars().any(char::is_alphabetic);
        if one_side && words && numbers < 2 && !text.contains(':') {
            return Vec::new();
        }
    }
    let mut taken: Vec<Span> = line.iter().map(|&i| spans[i].clone()).collect();
    taken.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap_or(Ordering::Equal));
    let mut keep = vec![true; spans.len()];
    for i in line {
        keep[i] = false;
    }
    let mut k = keep.into_iter();
    spans.retain(|_| k.next().unwrap_or(true));
    taken
}

/// Notes at the foot of the page: text smaller than the body's that stands
/// entirely below all of the page's body-size text, a little apart from it.
/// Taken out of `spans` and returned, so they are never read between the
/// columns (a note at the foot of the left column before the right column).
fn notes<'a>(spans: &mut Vec<&'a Span>, body: f64) -> Vec<&'a Span> {
    let is_body = |s: &Span| (s.size - body).abs() <= 0.1 * body && s.text.chars().count() > 2;
    let Some(body_bottom) = spans
        .iter()
        .filter(|s| is_body(s))
        .map(|s| s.bottom())
        .reduce(f64::max)
    else {
        return Vec::new();
    };
    let below: Vec<usize> = (0..spans.len())
        .filter(|&i| spans[i].top() > body_bottom)
        .collect();
    if below.is_empty()
        || below
            .iter()
            .any(|&i| spans[i].size > 0.9 * body && spans[i].text.chars().count() > 2)
    {
        return Vec::new();
    }
    let top = below
        .iter()
        .map(|&i| spans[i].top())
        .fold(f64::INFINITY, f64::min);
    if top - body_bottom < 0.2 * body {
        return Vec::new();
    }
    let mut out = Vec::new();
    for i in below.into_iter().rev() {
        out.push(spans.remove(i));
    }
    out.reverse();
    out
}

/// The text in drawing order, one line per baseline change (the order the
/// text layer gives, used when the reading order cannot be established).
fn drawing_order(glyphs: &[Glyph], width: f64) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::new();
    let mut text = String::new();
    let mut last: Option<&Glyph> = None;
    let mut first_char = 0;
    let mut flush = |text: &mut String, first_char: usize, y: f64, size: f64| {
        let t = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !t.is_empty() {
            lines.push(Line {
                text: t,
                x0: 0.0,
                x1: width,
                y,
                size,
                column: 0,
                block: 0,
                first_char,
                role: LineRole::Drawn,
                marked: None,
                styled: false,
            });
        }
        text.clear();
    };
    for (index, g) in glyphs.iter().enumerate() {
        if let Some(p) = last {
            let new_line = !g.upright
                || !p.upright
                || (f64::from(g.y) - f64::from(p.y)).abs()
                    > 0.5 * f64::from(p.size).max(f64::from(g.size)).max(1.0)
                || f64::from(g.x) < f64::from(p.x) - f64::from(p.size).max(1.0);
            if new_line {
                flush(&mut text, first_char, f64::from(p.y), f64::from(p.size));
                first_char = index;
            } else if f64::from(g.x) - f64::from(p.end) > 0.18 * f64::from(g.size)
                && !text.ends_with(' ')
            {
                text.push(' ');
            }
        }
        text.push_str(&g.text);
        last = Some(g);
    }
    if let Some(p) = last {
        flush(&mut text, first_char, f64::from(p.y), f64::from(p.size));
    }
    lines
}

/// The gutter of a page's running text, found from the page alone (the same
/// steps as [`layout_page`] up to the columns).
fn body_gutter(page: &PageGlyphs) -> Option<Gutter> {
    let all = spans(&page.glyphs);
    let mut refs: Vec<&Span> = all.iter().collect();
    if refs.is_empty() {
        return None;
    }
    let body = page_body_size(&refs);
    let _ = peel(&mut refs, body, page.height, true);
    let _ = notes(&mut refs, body);
    let _ = peel(&mut refs, body, page.height, false);
    find_gutter(&refs, body)
}

fn layout_page(page: PageGlyphs, hint: Option<Gutter>) -> PageLayout {
    let PageGlyphs {
        number,
        width,
        height,
        glyphs,
    } = page;
    let chars = glyphs.iter().filter(|g| !g.text.trim().is_empty()).count();
    let rotated: Vec<Glyph> = glyphs.iter().filter(|g| !g.upright).cloned().collect();
    let rotated_chars = rotated.iter().filter(|g| !g.text.trim().is_empty()).count();
    let all = spans(&glyphs);
    if all.is_empty() && rotated_chars == 0 {
        return PageLayout {
            number,
            width,
            height,
            order: PageOrder::Empty,
            lines: Vec::new(),
            rotated_chars: 0,
            chars,
        };
    }
    // Text set at an angle beyond a few characters (a sideways table, a
    // margin note): the page is not read by position.
    // Right-to-left text (Hebrew, Arabic) may be drawn in logical order,
    // right to left; spans assume left to right. Such a page keeps its
    // drawing order. So does a page with an unreasonable number of pieces
    // (the steps below are quadratic in places).
    let right_to_left = glyphs
        .iter()
        .flat_map(|g| g.text.chars())
        .filter(|&c| {
            matches!(c, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}')
        })
        .count();
    if rotated_chars > 20.max(chars / 50) || right_to_left * 5 > chars || all.len() > MAX_SPANS {
        return PageLayout {
            number,
            width,
            height,
            order: PageOrder::Uncertain,
            lines: drawing_order(&glyphs, width),
            rotated_chars,
            chars,
        };
    }
    let mut refs: Vec<&Span> = all.iter().collect();
    let body = page_body_size(&refs);
    let mut block = 0u32;
    let mut lines = Vec::new();
    let head = peel(&mut refs, body, height, true);
    // Notes before the page number: a single note line at the foot of the
    // page is a note, not a footer.
    let notes = notes(&mut refs, body);
    let foot = peel(&mut refs, body, height, false);
    if !head.is_empty() {
        let h: Vec<&Span> = head.iter().collect();
        let mut head_lines = lines_of(&h, body, 0, &mut block);
        for line in &mut head_lines {
            line.role = LineRole::Head;
        }
        lines.extend(head_lines);
    }
    let order = match arrange_page(&refs, body, &mut block, hint) {
        Some((body_lines, columns)) => {
            lines.extend(body_lines);
            if columns > 1 {
                PageOrder::Columns(columns)
            } else {
                PageOrder::Single
            }
        }
        None => {
            return PageLayout {
                number,
                width,
                height,
                order: PageOrder::Uncertain,
                lines: drawing_order(&glyphs, width),
                rotated_chars,
                chars,
            };
        }
    };
    if !notes.is_empty() {
        block += 1;
        let note_size = body_size(&notes);
        // Notes line by line across the page: a note's sign and the verse it
        // names stay on one line even when a gap between notes happens to
        // fall where the columns' gutter is. Notes are never Scripture text,
        // so their order never makes the page uncertain.
        let mut note_lines = lines_of(&notes, note_size, 0, &mut block);
        for line in &mut note_lines {
            line.role = LineRole::Note;
        }
        lines.extend(note_lines);
    }
    if !foot.is_empty() {
        block += 1;
        let f: Vec<&Span> = foot.iter().collect();
        let mut foot_lines = lines_of(&f, body, 0, &mut block);
        for line in &mut foot_lines {
            line.role = LineRole::Foot;
        }
        lines.extend(foot_lines);
    }
    if !rotated.is_empty() {
        // A few characters at an angle: kept, after the page's text.
        let mut extra = drawing_order(&rotated, width);
        for line in &mut extra {
            line.block = block + 1;
            line.role = LineRole::Rotated;
        }
        lines.extend(extra);
    }
    // The last safeguard: every character drawn is read exactly once, or the
    // page keeps its drawing order and is uncertain.
    let letters = |t: &str| t.chars().filter(|c| !c.is_whitespace()).count();
    let drawn: usize = glyphs.iter().map(|g| letters(&g.text)).sum();
    let read: usize = lines.iter().map(|l| letters(&l.text)).sum();
    if drawn != read {
        return PageLayout {
            number,
            width,
            height,
            order: PageOrder::Uncertain,
            lines: drawing_order(&glyphs, width),
            rotated_chars,
            chars,
        };
    }
    PageLayout {
        number,
        width,
        height,
        order,
        lines,
        rotated_chars,
        chars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(text: &str, x: f64, y: f64, size: f64) -> Vec<Glyph> {
        // One glyph per character, 0.5 em wide; spaces drawn as gaps.
        let mut out = Vec::new();
        let mut at = x;
        for c in text.chars() {
            if c != ' ' {
                out.push(Glyph {
                    text: GlyphText::from(c.to_string().as_str()),
                    x: at as f32,
                    y: y as f32,
                    end: (at + 0.5 * size) as f32,
                    size: size as f32,
                    width: 500,
                    upright: true,
                });
            }
            at += 0.5 * size;
        }
        out
    }

    fn page(glyphs: Vec<Glyph>) -> PageLayout {
        layout_page(
            PageGlyphs {
                number: 1,
                width: 600.0,
                height: 800.0,
                glyphs,
            },
            None,
        )
    }

    fn texts(p: &PageLayout) -> Vec<String> {
        p.lines.iter().map(|l| l.text.clone()).collect()
    }

    /// Two columns of `n` lines each, drawn line by line across the page.
    fn two_columns(n: usize, gutter: f64) -> Vec<Glyph> {
        let mut out = Vec::new();
        for i in 0..n {
            let y = 100.0 + 14.0 * i as f64;
            out.extend(g(&format!("left line {i} words here"), 40.0, y, 10.0));
            out.extend(g(
                &format!("right line {i} more words"),
                40.0 + 130.0 + gutter,
                y,
                10.0,
            ));
        }
        out
    }

    #[test]
    fn two_columns_drawn_across_are_read_column_by_column() {
        for gutter in [8.0, 20.0, 60.0] {
            let p = page(two_columns(10, gutter));
            assert_eq!(p.order, PageOrder::Columns(2), "gutter {gutter}");
            let t = texts(&p);
            assert_eq!(t.len(), 20);
            for i in 0..10 {
                assert_eq!(t[i], format!("left line {i} words here"));
                assert_eq!(t[10 + i], format!("right line {i} more words"));
            }
        }
    }

    #[test]
    fn one_column_stays_one_column() {
        let mut glyphs = Vec::new();
        for i in 0..20 {
            glyphs.extend(g(
                &format!("this is line {i} of a page set in one column only"),
                40.0,
                100.0 + 14.0 * i as f64,
                10.0,
            ));
        }
        let p = page(glyphs);
        assert_eq!(p.order, PageOrder::Single);
        assert_eq!(p.lines.len(), 20);
        assert_eq!(
            p.lines[3].text,
            "this is line 3 of a page set in one column only"
        );
    }

    #[test]
    fn a_title_over_both_columns_divides_them_into_bands() {
        let mut glyphs = Vec::new();
        for i in 0..8 {
            let y = 100.0 + 14.0 * i as f64;
            glyphs.extend(g(&format!("top left {i} text"), 40.0, y, 10.0));
            glyphs.extend(g(&format!("top right {i} text"), 200.0, y, 10.0));
        }
        glyphs.extend(g(
            "THE GOSPEL ACCORDING TO MATTHEW AND MORE WORDS",
            60.0,
            240.0,
            14.0,
        ));
        for i in 0..8 {
            let y = 270.0 + 14.0 * i as f64;
            glyphs.extend(g(&format!("low left {i} text"), 40.0, y, 10.0));
            glyphs.extend(g(&format!("low right {i} text"), 200.0, y, 10.0));
        }
        let p = page(glyphs);
        assert_eq!(p.order, PageOrder::Columns(2));
        let t = texts(&p);
        assert_eq!(t[0], "top left 0 text");
        assert_eq!(t[7], "top left 7 text");
        assert_eq!(t[8], "top right 0 text");
        assert_eq!(t[16], "THE GOSPEL ACCORDING TO MATTHEW AND MORE WORDS");
        assert_eq!(t[17], "low left 0 text");
        assert_eq!(t[25], "low right 0 text");
    }

    #[test]
    fn a_running_head_and_a_page_number_stand_apart() {
        let mut glyphs = g("Genesis 1:1 1 Genesis 2:3", 150.0, 40.0, 9.0);
        glyphs.extend(two_columns(10, 20.0));
        glyphs.extend(g("17", 290.0, 780.0, 9.0));
        let p = page(glyphs);
        let t = texts(&p);
        assert_eq!(t[0], "Genesis 1:1 1 Genesis 2:3");
        assert_eq!(t[1], "left line 0 words here");
        assert_eq!(t[11], "right line 0 more words");
        assert_eq!(t.last().unwrap(), "17");
    }

    #[test]
    fn a_raised_verse_number_stays_in_its_line_and_a_large_initial_opens_the_first() {
        let mut glyphs = g("4", 40.0, 124.0, 28.0);
        glyphs.extend(g("Then he went", 62.0, 110.0, 10.0));
        glyphs.extend(g("2", 132.0, 106.5, 6.0));
        glyphs.extend(g("and came", 138.0, 110.0, 10.0));
        glyphs.extend(g("back again to", 62.0, 124.0, 10.0));
        glyphs.extend(g("the city.", 40.0, 138.0, 10.0));
        let p = page(glyphs);
        assert_eq!(
            texts(&p),
            ["4 Then he went 2 and came", "back again to", "the city."]
        );
    }

    #[test]
    fn a_line_crossing_the_gutter_beside_column_text_makes_the_page_uncertain() {
        let mut glyphs = two_columns(10, 20.0);
        // A long line in the middle, over the gutter, with column text on the same height.
        glyphs.extend(g(
            "x a line that runs across the gutter here",
            100.0,
            157.0,
            10.0,
        ));
        let p = page(glyphs);
        assert_eq!(p.order, PageOrder::Uncertain);
        // Nothing is lost: every character is still there.
        let chars: usize = p
            .lines
            .iter()
            .map(|l| l.text.chars().filter(|c| !c.is_whitespace()).count())
            .sum();
        assert_eq!(chars, p.chars);
    }

    #[test]
    fn no_character_is_dropped_or_repeated() {
        let mut glyphs = g("Head 1", 150.0, 40.0, 9.0);
        glyphs.extend(two_columns(12, 15.0));
        glyphs.extend(g("† 1:5 a note at the foot of the page", 40.0, 700.0, 7.0));
        let p = page(glyphs);
        let mut got: Vec<char> = p.text().chars().filter(|c| !c.is_whitespace()).collect();
        let mut want: Vec<char> = p
            .lines
            .iter()
            .flat_map(|l| l.text.chars())
            .filter(|c| !c.is_whitespace())
            .collect();
        got.sort_unstable();
        want.sort_unstable();
        assert_eq!(got, want);
        assert_eq!(got.len(), p.chars);
    }

    #[test]
    fn a_heading_at_the_top_of_one_column_stays_in_that_column() {
        let mut glyphs = g("Psalm 24", 250.0, 60.0, 10.0);
        for i in 0..12 {
            let y = 80.0 + 14.0 * i as f64;
            glyphs.extend(g(&format!("{} end of psalm 23 left", i + 1), 40.0, y, 10.0));
            glyphs.extend(g(&format!("{} psalm 24 right text", i + 1), 250.0, y, 10.0));
        }
        let p = page(glyphs);
        let t = texts(&p);
        let heading = t.iter().position(|l| l == "Psalm 24").unwrap();
        let left_end = t
            .iter()
            .position(|l| l == "12 end of psalm 23 left")
            .unwrap();
        assert!(left_end < heading, "{t:?}");
        assert!(p.lines.iter().all(|l| l.role != LineRole::Head));
    }

    #[test]
    fn right_to_left_text_keeps_its_drawing_order() {
        let mut glyphs = Vec::new();
        for i in 0..20 {
            glyphs.extend(g("בראשית ברא אלהים", 300.0, 100.0 + 14.0 * i as f64, 10.0));
        }
        assert_eq!(page(glyphs).order, PageOrder::Uncertain);
    }

    #[test]
    fn a_line_reaching_into_the_gutter_stays_in_its_column() {
        // One left-column line runs on into the space between the columns;
        // it was once neither left nor right and was lost.
        let mut glyphs = Vec::new();
        for i in 0..10 {
            let y = 100.0 + 14.0 * i as f64;
            let left = if i == 4 {
                "a long left line reaching into gap"
            } else {
                "left words here ok"
            };
            glyphs.extend(g(left, 40.0, y, 10.0));
            if i < 8 {
                glyphs.extend(g("right words here", 240.0, y, 10.0));
            }
        }
        let drawn: usize = glyphs.iter().map(|g| g.text.chars().count()).sum();
        let p = page(glyphs);
        let read: usize = p
            .lines
            .iter()
            .map(|l| l.text.chars().filter(|c| !c.is_whitespace()).count())
            .sum();
        assert_eq!(read, drawn);
        assert_eq!(p.order, PageOrder::Columns(2));
        let t = texts(&p);
        let long = t.iter().position(|l| l.starts_with("a long")).unwrap();
        let right = t.iter().position(|l| l == "right words here").unwrap();
        assert!(long < right, "{t:?}");
    }

    #[test]
    fn a_page_with_one_line_in_its_right_column_is_read_with_the_documents_gutter() {
        // The last page of a book: its right column holds one line, too little
        // to show a gutter; the document's other pages share theirs.
        let mut pages: Vec<PageGlyphs> = (1..=12)
            .map(|n| PageGlyphs {
                number: n,
                width: 600.0,
                height: 800.0,
                glyphs: two_columns(10, 20.0),
            })
            .collect();
        let mut last = g("1 Samuel 31:1 327 1 Samuel 31:13", 150.0, 50.0, 8.0);
        last.extend(g("31", 120.0, 100.0, 18.0));
        last.extend(g("and fasted seven days.", 190.0, 100.0, 10.0));
        for i in 0..10 {
            last.extend(g(
                &format!("left line {i} words here"),
                40.0,
                120.0 + 14.0 * i as f64,
                10.0,
            ));
        }
        pages.push(PageGlyphs {
            number: 14,
            width: 600.0,
            height: 800.0,
            glyphs: last,
        });
        let layouts = Collector {
            pages,
            ..Default::default()
        }
        .layouts();
        let p = layouts.last().unwrap();
        assert_eq!(p.order, PageOrder::Columns(2), "{:?}", texts(p));
        let t = texts(p);
        assert_eq!(t.last().unwrap(), "and fasted seven days.");
        assert!(!t.iter().any(|l| l.contains("31 and")));
    }

    #[test]
    fn a_line_with_a_wide_space_inside_it_makes_a_one_column_page_uncertain() {
        // Two columns run together on one line, with no gutter found: the
        // order is not guessed.
        let mut glyphs = Vec::new();
        for i in 0..8 {
            glyphs.extend(g(
                &format!("this is line {i} of a page set in one column"),
                40.0,
                100.0 + 14.0 * i as f64,
                10.0,
            ));
        }
        glyphs.extend(g("31", 120.0, 230.0, 10.0));
        glyphs.extend(g("and fasted seven days.", 300.0, 230.0, 10.0));
        let p = page(glyphs);
        assert_eq!(p.order, PageOrder::Uncertain);
    }

    #[test]
    fn a_single_note_line_at_the_foot_is_a_note_and_its_larger_sign_joins_it() {
        let mut glyphs = two_columns(10, 20.0);
        glyphs.extend(g("†", 40.0, 726.0, 8.0));
        glyphs.extend(g(
            "15:2 The word translated “Lord” is “Adonai”.",
            46.0,
            728.0,
            6.0,
        ));
        let p = page(glyphs);
        let last = p.lines.last().unwrap();
        assert_eq!(last.role, LineRole::Note);
        assert!(last.text.starts_with("† 15:2"), "{}", last.text);
    }

    #[test]
    fn raised_small_letters_are_marked_for_the_recogniser() {
        let mut glyphs = g("Lot, qui voyageait", 40.0, 100.0, 12.0);
        glyphs.extend(g("c", 150.0, 96.0, 7.0));
        glyphs.extend(g("Et la contrée", 154.0, 100.0, 12.0));
        let p = page(glyphs);
        let line = &p.lines[0];
        assert_eq!(line.text, "Lot, qui voyageait cEt la contrée");
        assert_eq!(
            line.marked.as_deref(),
            Some(format!("Lot, qui voyageait {RAISED_OPEN}c{RAISED_CLOSE}Et la contrée").as_str())
        );
    }

    #[test]
    fn a_narrow_column_of_numbers_is_not_a_column() {
        // Verse numbers in a margin: never read apart from their text.
        let mut glyphs = Vec::new();
        for i in 0..12 {
            let y = 100.0 + 14.0 * i as f64;
            glyphs.extend(g(&format!("{}", i + 1), 40.0, y, 10.0));
            glyphs.extend(g(
                &format!("the words of verse {} in a psalm", i + 1),
                70.0,
                y,
                10.0,
            ));
        }
        let p = page(glyphs);
        assert_eq!(p.order, PageOrder::Single);
        assert_eq!(p.lines[0].text, "1 the words of verse 1 in a psalm");
    }
}

#[cfg(test)]
mod glyph_probe {
    // Temporary: AW_GLYPH_PDF=file AW_GLYPH_PAGE=n AW_GLYPH_WORD=text
    use super::*;
    #[test]
    #[ignore]
    fn glyphs() {
        let path = std::env::var("AW_GLYPH_PDF").unwrap();
        let page: usize = std::env::var("AW_GLYPH_PAGE").unwrap().parse().unwrap();
        let word = std::env::var("AW_GLYPH_WORD").unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let doc = lopdf::Document::load_mem(&bytes).unwrap();
        let mut c = Collector::default();
        pdf_extract::output_doc_page(&doc, &mut c, page as u32).unwrap();
        let p = &c.pages[0];
        if std::env::var("AW_GLYPH_LAYOUT").is_ok() {
            let letters = |t: &str| t.chars().filter(|c| !c.is_whitespace()).count();
            let drawn: usize = p.glyphs.iter().map(|g| letters(&g.text)).sum();
            let rotated = p.glyphs.iter().filter(|g| !g.upright).count();
            let page = PageGlyphs {
                number: 1,
                width: p.width,
                height: p.height,
                glyphs: p.glyphs.clone(),
            };
            let l = layout_page(page, None);
            let read: usize = l.lines.iter().map(|x| letters(&x.text)).sum();
            eprintln!(
                "order {:?} drawn {drawn} read {read} rotated {rotated} spans {}",
                l.order,
                spans(&p.glyphs).len()
            );
            let all = spans(&p.glyphs);
            let in_spans: usize = all.iter().map(|s| letters(&s.text)).sum();
            eprintln!("letters in spans {in_spans}");
            let mut refs: Vec<&Span> = all.iter().collect();
            let body = page_body_size(&refs);
            let _ = peel(&mut refs, body, p.height, true);
            let _ = notes(&mut refs, body);
            let _ = peel(&mut refs, body, p.height, false);
            eprintln!("body {body} gutter {:?}", find_gutter(&refs, body));
            let mut block = 0;
            eprintln!(
                "column_lines ok {}",
                column_lines(&refs, body, 0, &mut block).is_some()
            );
            for line in lines_of(&refs, body, 0, &mut block) {
                let pieces: Vec<&Span> = refs
                    .iter()
                    .copied()
                    .filter(|s| {
                        (s.y - line.y).abs() <= 0.5 * line.size
                            && s.x0 >= line.x0 - 1e-6
                            && s.x1 <= line.x1 + 1e-6
                    })
                    .filter(|s| s.size < 1.6 * body)
                    .collect();
                let g = widest_gap(&pieces);
                if g > 2.5 {
                    eprintln!("wide {g:.2} {}", line.text);
                }
            }
            for g in p
                .glyphs
                .iter()
                .filter(|g| g.text.chars().count() != 1)
                .take(10)
            {
                eprintln!("multi {:?}", g.text.to_string());
            }
            return;
        }
        if std::env::var("AW_GLYPH_SPANS").is_ok() {
            for sp in spans(&p.glyphs) {
                eprintln!(
                    "SPAN {}/{} y{:.1} s{:.1} {}",
                    sp.styled, sp.compared, sp.y, sp.size, sp.text
                );
            }
            return;
        }
        let text: String = p.glyphs.iter().map(|g| g.text.to_string()).collect();
        let at = text.find(&word).unwrap();
        let start = text[..at].chars().count();
        for g in p
            .glyphs
            .iter()
            .skip(start.saturating_sub(4))
            .take(word.chars().count() + 8)
        {
            eprintln!(
                "{:?} x{:.1} y{:.1} end{:.1} size{:.2} w{}",
                g.text.to_string(),
                g.x,
                g.y,
                g.end,
                g.size,
                g.width
            );
        }
    }
}
