//! The harness's own USFM verse-text reader, rules `aw-usfm-text-1`
//! (evaluation candidate A1.5-E3).
//!
//! One of two readers of the reference's verse text, written apart: this one,
//! in Rust, reads the source in one streaming pass with a stack of open
//! spans; the check's, in Python (`scripts/evaluation/usfm_text.py`), lexes,
//! builds a tree and walks it. They share no code, table or regular
//! expression with each other or with the app's USFM reader
//! (`abiding_word_lib::scripture::usfm`), which the independent E2 review
//! found inserting a space where a note was removed (528 verses of the
//! bundled references). Both follow `docs/evaluation/usfm-text-rules.md`,
//! taken from the USFM 3 specification; the scorer compares their per-verse
//! text digests and refuses any difference.
//!
//! A verse's text is what the edition prints: notes are removed with their
//! content and nothing is put in their place; character spans keep their
//! text except `\va`, `\vp`, `\ca`, `\fig`, `\rq` and `\cat`; attribute
//! lists and milestones are not text; USFM white space (U+0020, tab, CR, LF)
//! runs become one U+0020 and the ends are trimmed, every other character
//! (a no-break space included) is kept; `~` is U+00A0 and `//` nothing; a
//! paragraph or table-cell boundary inside a verse is one space.
//!
//! What cannot be read unambiguously is a [`Problem`], never a guess, and no
//! problem drops a verse: every `\v` marker becomes a verse (or a verse
//! before any chapter), and an independent count of the `\v` markers in the
//! source must agree.

use std::collections::BTreeMap;

pub const RULES: &str = "aw-usfm-text-1";

/// USFM white space once CR and CRLF are read as LF.
fn is_usfm_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n')
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Problem {
    pub code: &'static str,
    pub line: Option<usize>,
    pub detail: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Verse {
    pub chapter: String,
    pub label: String,
    pub text: String,
    /// The line of the `\v` marker in the file (CR and CRLF read as LF).
    pub line: usize,
    /// `va` (alternate verse number), `vp` (published verse character), as printed.
    pub alternate: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Chapter {
    pub label: String,
    pub line: usize,
    pub alternate: Option<String>,
    pub published: Option<String>,
}

/// One USFM book file as read.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Book {
    pub rules: &'static str,
    /// From the first `\id` with a code, ASCII upper case.
    pub code: Option<String>,
    pub id_lines: usize,
    pub bom: bool,
    pub verses: Vec<Verse>,
    pub chapters: Vec<Chapter>,
    pub verses_before_chapter: usize,
    /// `\v` markers counted apart from the reading.
    pub verse_markers: usize,
    pub problems: Vec<Problem>,
}

/// What a marker is, by its name without level digits or `-` suffix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    /// A paragraph whose text is verse text while a verse is open.
    Body,
    /// A paragraph that is never verse text (titles, headings, introductions).
    Other,
    /// A table cell: a boundary inside a row.
    Cell,
    Note,
    /// A part of a note (`\fr`, `\ft`, `\xt`...).
    NotePart,
    /// A character span whose text is the edition's.
    Keep,
    /// A character span whose content is not verse text.
    Drop,
    Sidebar,
    Unknown,
}

fn role(stem: &str) -> Role {
    match stem {
        "p" | "m" | "po" | "pr" | "cls" | "pmo" | "pm" | "pmc" | "pmr" | "pi" | "mi" | "nb"
        | "pc" | "ph" | "b" | "q" | "qr" | "qc" | "qm" | "qd" | "lh" | "li" | "lf" | "lim"
        | "tr" => Role::Body,
        "usfm" | "ide" | "sts" | "rem" | "h" | "toc" | "toca" | "mt" | "mte" | "ms" | "mr"
        | "s" | "sr" | "r" | "d" | "sp" | "sd" | "cl" | "cp" | "cd" | "qa" | "lit" | "periph"
        | "restore" | "imt" | "is" | "ip" | "ipi" | "im" | "imi" | "ipq" | "imq" | "ipr" | "iq"
        | "ib" | "ili" | "iot" | "io" | "iex" | "imte" | "ie" => Role::Other,
        "th" | "thc" | "thr" | "tc" | "tcc" | "tcr" => Role::Cell,
        "f" | "fe" | "x" | "ef" | "ex" => Role::Note,
        "fr" | "fq" | "fqa" | "fk" | "fl" | "fw" | "fp" | "fv" | "ft" | "fdc" | "fm" | "efm"
        | "xo" | "xk" | "xq" | "xt" | "xta" | "xop" | "xot" | "xnt" | "xdc" => Role::NotePart,
        "add" | "bk" | "dc" | "k" | "nd" | "ord" | "pn" | "png" | "addpn" | "qt" | "sig"
        | "sls" | "tl" | "wj" | "em" | "bd" | "it" | "bdit" | "no" | "sc" | "sup" | "ndx"
        | "rb" | "pro" | "w" | "wg" | "wh" | "wa" | "jmp" | "qs" | "qac" | "litl" | "lik"
        | "liv" | "ior" | "iqt" => Role::Keep,
        "va" | "vp" | "ca" | "fig" | "rq" | "cat" => Role::Drop,
        _ => Role::Unknown,
    }
}

/// The marker's name without a `-` suffix and without level digits:
/// `q2` is `q`, `qt1-s` is `qt`, `tc1-2` is `tc`.
fn stem(name: &str) -> &str {
    let head = name.split('-').next().unwrap_or(name);
    let bare = head.trim_end_matches(|c: char| c.is_ascii_digit());
    if bare.is_empty() {
        head
    } else {
        bare
    }
}

/// How the marker table reads a marker (for the rules document's own test).
pub fn role_name(marker: &str) -> &'static str {
    match role(stem(marker)) {
        Role::Body => "body",
        Role::Other => "other",
        Role::Cell => "cell",
        Role::Note => "note",
        Role::NotePart => "note part",
        Role::Keep => "kept",
        Role::Drop => "dropped",
        Role::Sidebar | Role::Unknown => "unknown",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Para {
    Start,
    Id,
    Chapter,
    Body,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expect {
    Nothing,
    Code,
    ChapterNumber(usize),
    VerseNumber(usize),
    PublishedChapter,
}

struct Open {
    role: Role,
    name: String,
    plus: bool,
    line: usize,
    /// Inside the span's attribute list (after `|`).
    attributes: bool,
    /// For `\va`, `\vp`, `\ca`: the number as printed.
    capture: Option<String>,
}

struct Reader {
    line: usize,
    book: Book,
    stack: Vec<Open>,
    milestones: Vec<(String, usize)>,
    para: Para,
    target: bool,
    expect: Expect,
    verse: Option<(Verse, String)>,
    chapter: Option<usize>,
    /// After `\c N`, before its first verse: whether anything but white
    /// space was there.
    chapter_rest: Option<(usize, bool)>,
}

/// The first word (up to USFM white space), and what follows it.
fn first_word(text: &str) -> (&str, &str) {
    let text = text.trim_start_matches(is_usfm_space);
    match text.find(is_usfm_space) {
        Some(at) => (&text[..at], &text[at..]),
        None => (text, ""),
    }
}

/// A verse's raw text as printed: `//` nothing, `~` U+00A0, USFM white
/// space runs one U+0020, the ends trimmed.
fn printed(raw: &str) -> String {
    let raw = raw.replace("//", "").replace('~', "\u{a0}");
    let mut out = String::with_capacity(raw.len());
    let mut gap = false;
    for c in raw.chars() {
        if is_usfm_space(c) {
            gap = true;
        } else {
            if gap && !out.is_empty() {
                out.push(' ');
            }
            gap = false;
            out.push(c);
        }
    }
    out
}

/// `\v` markers, counted the plainest way and apart from the reading: a
/// backslash and `v` followed by nothing that could continue a marker name.
fn count_verse_markers(source: &str) -> usize {
    source
        .match_indices("\\v")
        .filter(|(at, _)| {
            !source[at + 2..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '*' || c == '-')
        })
        .count()
}

impl Reader {
    fn problem(&mut self, code: &'static str, line: Option<usize>, detail: String) {
        self.book.problems.push(Problem { code, line, detail });
    }

    fn in_role(&self, roles: &[Role]) -> bool {
        self.stack.iter().any(|o| roles.contains(&o.role))
    }

    fn finish_verse(&mut self) {
        if let Some((mut verse, raw)) = self.verse.take() {
            verse.text = printed(&raw);
            self.book.verses.push(verse);
        }
    }

    fn end_chapter_rest(&mut self) {
        if let Some((line, blank)) = self.chapter_rest.take() {
            if !blank {
                let label = self
                    .chapter
                    .map(|c| self.book.chapters[c].label.clone())
                    .unwrap_or_default();
                self.problem(
                    "text_after_chapter_number",
                    Some(line),
                    format!("\\c {label} is followed by text"),
                );
            }
        }
    }

    /// Takes what a `\id`, `\c`, `\v` or `\cp` was waiting for.
    fn resolve(&mut self, word: &str) {
        let expect = std::mem::replace(&mut self.expect, Expect::Nothing);
        match expect {
            Expect::Nothing => {}
            Expect::Code => {
                if self.book.code.is_none() && !word.is_empty() {
                    self.book.code = Some(word.to_ascii_uppercase());
                }
            }
            Expect::PublishedChapter => {
                if let Some(c) = self.chapter {
                    self.book.chapters[c].published = Some(word.to_string());
                }
            }
            Expect::ChapterNumber(line) => {
                if word.is_empty() {
                    self.problem(
                        "chapter_number_missing",
                        Some(line),
                        "\\c without a number".into(),
                    );
                }
                self.book.chapters.push(Chapter {
                    label: word.to_string(),
                    line,
                    ..Default::default()
                });
                self.chapter = Some(self.book.chapters.len() - 1);
                self.chapter_rest = Some((line, true));
            }
            Expect::VerseNumber(line) => {
                if word.is_empty() {
                    self.problem(
                        "verse_number_missing",
                        Some(line),
                        "\\v without a number".into(),
                    );
                }
                match self.chapter {
                    None => {
                        self.book.verses_before_chapter += 1;
                        self.target = false;
                    }
                    Some(c) => {
                        self.verse = Some((
                            Verse {
                                chapter: self.book.chapters[c].label.clone(),
                                label: word.to_string(),
                                line,
                                ..Default::default()
                            },
                            String::new(),
                        ));
                        self.target = true;
                    }
                }
            }
        }
    }

    /// Ends a span: an alternate or published number is kept beside its
    /// verse or chapter.
    fn ended(&mut self, open: Open) {
        let Some(value) = open.capture else {
            return;
        };
        let value = printed(&value);
        match (open.name == "ca", self.chapter, self.target) {
            (true, Some(c), false) => self.book.chapters[c].alternate = Some(value),
            (_, _, true) => {
                if let Some((verse, _)) = self.verse.as_mut() {
                    verse.alternate.insert(open.name, value);
                }
            }
            _ => {}
        }
    }

    /// Closes every open span at a structural boundary: each one left open
    /// (apart from note parts, which close implicitly) is a problem.
    fn close_all(&mut self, line: usize, at: &str) {
        while let Some(open) = self.stack.pop() {
            let code = match open.role {
                Role::Note => Some("note_unclosed"),
                Role::Keep => Some("char_unclosed"),
                Role::Drop => Some("dropped_unclosed"),
                Role::Sidebar => Some("sidebar_unclosed"),
                _ => None,
            };
            if let Some(code) = code {
                self.problem(
                    code,
                    Some(open.line),
                    format!(
                        "\\{} opened on line {} is still open at {at} (line {line})",
                        open.name, open.line
                    ),
                );
            }
            self.ended(open);
        }
    }

    fn text(&mut self, mut text: &str) {
        if self.expect != Expect::Nothing {
            // The number is the first word of the text right after the
            // marker; white space alone there means it is missing.
            let (word, rest) = first_word(text);
            self.resolve(word);
            text = rest;
        }
        if text.is_empty() {
            return;
        }
        if self.stack.last().is_some_and(|o| o.attributes)
            || self.in_role(&[Role::Note, Role::NotePart, Role::Sidebar])
        {
            return;
        }
        if let Some(open) = self.stack.last_mut() {
            if open.role == Role::Drop {
                if let Some(capture) = open.capture.as_mut() {
                    capture.push_str(text);
                }
                return;
            }
        }
        if self.in_role(&[Role::Drop]) {
            return;
        }
        if self.target {
            if let Some((_, raw)) = self.verse.as_mut() {
                raw.push_str(text);
            }
            return;
        }
        let blank = text.chars().all(is_usfm_space);
        if let Some((_, rest_blank)) = self.chapter_rest.as_mut() {
            if self.para == Para::Chapter {
                *rest_blank &= blank;
                return;
            }
        }
        if self.para == Para::Body && self.chapter.is_some() && !blank {
            let chapter = self
                .chapter
                .map(|c| self.book.chapters[c].label.clone())
                .unwrap_or_default();
            let shown: String = text.trim().chars().take(40).collect();
            self.problem(
                "text_outside_verse",
                Some(self.line),
                format!("{chapter}: «{shown}»"),
            );
        }
    }

    fn bar(&mut self) {
        if self.expect != Expect::Nothing {
            self.resolve("");
        }
        match self.stack.last_mut() {
            Some(open) if open.role == Role::Keep && !open.attributes => open.attributes = true,
            Some(_) => {}
            None => {
                if self.target {
                    let line = self.line;
                    self.problem(
                        "bar_outside_span",
                        Some(line),
                        "| in verse text outside a character span".into(),
                    );
                }
            }
        }
    }

    fn milestone(&mut self, name: &str, line: usize) {
        if let Some(start) = name.strip_suffix("-s") {
            self.milestones.push((start.to_string(), line));
        } else if let Some(end) = name.strip_suffix("-e") {
            match self.milestones.iter().rposition(|(n, _)| n == end) {
                Some(at) => {
                    self.milestones.remove(at);
                }
                None => self.problem(
                    "milestone_unpaired",
                    Some(line),
                    format!("\\{name} ends no \\{end}-s"),
                ),
            }
        }
    }

    fn push(&mut self, role: Role, name: &str, plus: bool, line: usize) {
        let capture = (role == Role::Drop
            && matches!(name, "va" | "vp" | "ca")
            && !self.in_role(&[Role::Drop, Role::Note]))
        .then(String::new);
        if self.para == Para::Chapter
            && self.chapter_rest.is_some()
            && matches!(role, Role::Keep | Role::Drop)
            && name != "ca"
            && self.stack.is_empty()
        {
            if let Some((_, blank)) = self.chapter_rest.as_mut() {
                *blank = false;
            }
        }
        self.stack.push(Open {
            role,
            name: name.to_string(),
            plus,
            line,
            attributes: false,
            capture,
        });
    }

    fn marker(&mut self, name: &str, plus: bool, closing: bool, line: usize) {
        if self.expect != Expect::Nothing {
            self.resolve("");
        }
        if let Some(open) = self.stack.last() {
            if open.attributes && !(closing && open.name == name) {
                let detail = format!("\\{name} inside the attribute list of \\{}", open.name);
                self.problem("attributes_interrupted", Some(line), detail);
            }
        }
        if name == "esbe" && !closing {
            if self.stack.last().is_some_and(|o| o.role == Role::Sidebar) {
                self.stack.pop();
            } else {
                self.problem(
                    "sidebar_close_stray",
                    Some(line),
                    "\\esbe without \\esb".into(),
                );
            }
            return;
        }
        let structural = matches!(name, "v" | "c" | "id") && !closing && !plus;
        if self.in_role(&[Role::Sidebar]) {
            if structural {
                self.close_all(line, &format!("\\{name}"));
            } else {
                if name == "esb" && !closing {
                    self.problem(
                        "sidebar_nested",
                        Some(line),
                        "\\esb inside a sidebar".into(),
                    );
                }
                return;
            }
        }
        if name == "esb" && !closing {
            self.close_all(line, "\\esb");
            self.push(Role::Sidebar, name, false, line);
            return;
        }
        if structural {
            self.close_all(line, &format!("\\{name}"));
            if name == "v" {
                if self.para == Para::Chapter {
                    self.end_chapter_rest();
                }
                self.finish_verse();
                self.expect = Expect::VerseNumber(line);
            } else {
                self.end_chapter_rest();
                self.finish_verse();
                self.target = false;
                if name == "c" {
                    self.para = Para::Chapter;
                    self.expect = Expect::ChapterNumber(line);
                } else {
                    self.para = Para::Id;
                    self.book.id_lines += 1;
                    self.expect = Expect::Code;
                }
            }
            return;
        }
        let stem = stem(name);
        let role = role(stem);
        if !closing && !plus && matches!(role, Role::Body | Role::Other) {
            self.close_all(line, &format!("\\{name}"));
            self.end_chapter_rest();
            if role == Role::Body {
                self.para = Para::Body;
                self.target = self.verse.is_some();
                if let Some((_, raw)) = self.verse.as_mut() {
                    raw.push(' ');
                }
            } else {
                self.para = Para::Other;
                self.target = false;
                if stem == "cp" {
                    self.expect = Expect::PublishedChapter;
                }
            }
            return;
        }
        if !closing && (role == Role::Cell || name == "pb") {
            if !self.stack.is_empty() {
                self.close_all(line, &format!("\\{name}"));
            }
            if self.target {
                if let Some((_, raw)) = self.verse.as_mut() {
                    raw.push(' ');
                }
            }
            return;
        }
        let in_note = self.in_role(&[Role::Note]);
        match role {
            Role::Note => {
                if !closing {
                    if in_note {
                        self.problem(
                            "note_nested",
                            Some(line),
                            format!("\\{name} inside an open note"),
                        );
                    }
                    self.push(Role::Note, name, plus, line);
                    return;
                }
                while self.stack.last().is_some_and(|o| o.role == Role::NotePart) {
                    self.stack.pop();
                }
                if self
                    .stack
                    .last()
                    .is_some_and(|o| o.role == Role::Note && o.name == name)
                {
                    self.stack.pop();
                } else if in_note {
                    self.problem(
                        "note_close_mismatch",
                        Some(line),
                        format!("\\{name}* does not close the open note"),
                    );
                } else {
                    self.problem(
                        "note_close_stray",
                        Some(line),
                        format!("\\{name}* with no note open"),
                    );
                }
            }
            Role::NotePart => {
                if in_note {
                    let top_part = self.stack.last().is_some_and(|o| o.role == Role::NotePart);
                    if closing {
                        if top_part && self.stack.last().is_some_and(|o| o.name == name) {
                            self.stack.pop();
                        }
                        return;
                    }
                    if top_part && !plus {
                        self.stack.pop();
                    }
                    self.push(Role::NotePart, name, plus, line);
                } else if self.target && !self.in_role(&[Role::Drop]) {
                    self.problem(
                        "note_part_outside_note",
                        Some(line),
                        format!("\\{name} in verse text outside a note"),
                    );
                }
            }
            Role::Keep | Role::Drop => {
                if in_note {
                    if closing {
                        if self.stack.last().is_some_and(|o| {
                            o.name == name && matches!(o.role, Role::Keep | Role::Drop)
                        }) {
                            self.stack.pop();
                        }
                    } else if plus {
                        self.push(role, name, true, line);
                    }
                    return;
                }
                let spans_open = self.in_role(&[Role::Keep, Role::Drop]);
                if closing {
                    let matches_top = self.stack.last().is_some_and(|o| {
                        matches!(o.role, Role::Keep | Role::Drop)
                            && o.name == name
                            && o.plus == plus
                    });
                    if matches_top {
                        let open = self.stack.pop().expect("an open span");
                        self.ended(open);
                    } else if spans_open {
                        let top = self
                            .stack
                            .last()
                            .map(|o| o.name.clone())
                            .unwrap_or_default();
                        self.problem(
                            "span_close_mismatch",
                            Some(line),
                            format!("\\{}{name}* closes \\{top}", if plus { "+" } else { "" }),
                        );
                    } else {
                        self.problem(
                            "span_close_stray",
                            Some(line),
                            format!("\\{name}* with no span open"),
                        );
                    }
                    return;
                }
                if spans_open && !plus {
                    let top = self
                        .stack
                        .last()
                        .map(|o| o.name.clone())
                        .unwrap_or_default();
                    self.problem(
                        "span_nested_without_plus",
                        Some(line),
                        format!("\\{name} inside \\{top}"),
                    );
                }
                self.push(role, name, plus, line);
            }
            _ => {
                if stem.starts_with('z') {
                    self.problem(
                        "marker_private",
                        Some(line),
                        format!("\\{name}: a \\z marker whose text this reader cannot place"),
                    );
                } else {
                    self.problem(
                        "marker_unknown",
                        Some(line),
                        format!(
                            "\\{}{name}{}",
                            if plus { "+" } else { "" },
                            if closing { "*" } else { "" }
                        ),
                    );
                }
            }
        }
    }

    /// Reads the marker whose backslash is at `at`; returns where reading
    /// goes on.
    fn marker_at(&mut self, source: &str, at: usize) -> usize {
        let bytes = source.as_bytes();
        let line = self.line;
        let mut i = at + 1;
        if bytes.get(i) == Some(&b'*') {
            if self.expect != Expect::Nothing {
                self.resolve("");
            }
            self.problem(
                "milestone_close_stray",
                Some(line),
                "\\* that ends no milestone".into(),
            );
            return i + 1;
        }
        let plus = bytes.get(i) == Some(&b'+');
        if plus {
            i += 1;
        }
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_alphanumeric) {
            i += 1;
        }
        if i > start
            && bytes.get(i) == Some(&b'-')
            && bytes
                .get(i + 1)
                .is_some_and(|b| *b == b's' || *b == b'e' || b.is_ascii_digit())
        {
            i += 2;
            while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
        }
        let name = &source[start..i];
        if name.is_empty() {
            let shown: String = source[at..].chars().take(3).collect();
            self.problem("backslash_not_marker", Some(line), format!("{shown:?}"));
            return i;
        }
        let closing = bytes.get(i) == Some(&b'*');
        if closing {
            i += 1;
        } else {
            if let Some(&b) = bytes.get(i) {
                if matches!(b, b' ' | b'\t' | b'\n') {
                    if b == b'\n' {
                        self.line += 1;
                    }
                    i += 1;
                }
            }
            // A milestone: the marker, optionally an attribute list, then `\*`.
            if let Some(stop) = source[i..].find('\\').map(|k| i + k) {
                let between = &source[i..stop];
                if source[stop..].starts_with("\\*")
                    && (between.is_empty()
                        || between.trim_start_matches(is_usfm_space).starts_with('|'))
                {
                    if self.expect != Expect::Nothing {
                        self.resolve("");
                    }
                    self.line += between.matches('\n').count();
                    self.milestone(name, line);
                    return stop + 2;
                }
            }
            if name.ends_with("-s") || name.ends_with("-e") {
                if self.expect != Expect::Nothing {
                    self.resolve("");
                }
                self.problem(
                    "milestone_unterminated",
                    Some(line),
                    format!("\\{name} has no \\* ending"),
                );
                self.milestone(name, line);
                return i;
            }
        }
        self.marker(name, plus, closing, line);
        i
    }
}

/// Reads one USFM book file's text.
pub fn read(source: &str) -> Book {
    let bom = source.starts_with('\u{feff}');
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let source = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut r = Reader {
        line: 1,
        book: Book {
            rules: RULES,
            bom,
            ..Default::default()
        },
        stack: Vec::new(),
        milestones: Vec::new(),
        para: Para::Start,
        target: false,
        expect: Expect::Nothing,
        verse: None,
        chapter: None,
        chapter_rest: None,
    };
    let mut at = 0;
    while at < source.len() {
        let next = source[at..]
            .find(['\\', '|'])
            .map_or(source.len(), |k| at + k);
        if next > at {
            let piece = &source[at..next];
            r.text(piece);
            r.line += piece.matches('\n').count();
        }
        if next >= source.len() {
            break;
        }
        at = if source.as_bytes()[next] == b'|' {
            r.bar();
            next + 1
        } else {
            r.marker_at(&source, next)
        };
    }
    if r.expect != Expect::Nothing {
        r.resolve("");
    }
    let line = r.line;
    r.close_all(line, "the end of the file");
    r.end_chapter_rest();
    r.finish_verse();
    for (name, line) in std::mem::take(&mut r.milestones) {
        r.problem(
            "milestone_unpaired",
            Some(line),
            format!("\\{name}-s has no \\{name}-e"),
        );
    }
    let mut book = r.book;
    let controls: Vec<Problem> = book
        .verses
        .iter()
        .filter_map(|v| {
            v.text.chars().find(|c| c.is_control()).map(|c| Problem {
                code: "control_character",
                line: Some(v.line),
                detail: format!("{}:{} holds U+{:04X}", v.chapter, v.label, c as u32),
            })
        })
        .collect();
    book.problems.extend(controls);
    book.verse_markers = count_verse_markers(&source);
    if book.verse_markers != book.verses.len() + book.verses_before_chapter {
        let detail = format!(
            "{} \\v markers in the source, {} verses and {} before any chapter read",
            book.verse_markers,
            book.verses.len(),
            book.verses_before_chapter
        );
        book.problems.push(Problem {
            code: "verse_markers_unaccounted",
            line: None,
            detail,
        });
    }
    if book.id_lines > 1 {
        let detail = format!("{} \\id lines; one book per file", book.id_lines);
        book.problems.push(Problem {
            code: "id_repeated",
            line: None,
            detail,
        });
    }
    book
}

/// SHA-256 of a verse's text (UTF-8), lower-case hex.
pub fn text_sha256(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
