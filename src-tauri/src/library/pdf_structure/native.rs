//! Bounded reading of PDF-owned outline/tags/page labels with lopdf. No remote
//! actions, scripts, guessed destinations or unaligned tagged text are trusted.
use super::{compact, Anchor, MAX_CHARS, MAX_HEADINGS};
use lopdf::{content::Content, decode_text_string, Dictionary, Document, Object, ObjectId};
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct Candidate {
    pub title: String,
    pub level: u8,
    pub parents: Vec<String>,
    pub basis: String,
    pub anchor: Anchor,
}
pub struct Role {
    pub page: u32,
    pub text: String,
    pub role: String,
}
#[derive(Default)]
pub struct Native {
    pub headings: Vec<Candidate>,
    pub roles: Vec<Role>,
    pub labels: BTreeMap<u32, String>,
    pub tagged: bool,
    pub unaligned_tags: usize,
    pub truncated: bool,
}

fn deref<'a>(doc: &'a Document, value: &'a Object) -> Option<&'a Object> {
    let mut value = value;
    for _ in 0..16 {
        match value {
            Object::Reference(id) => value = doc.get_object(*id).ok()?,
            _ => return Some(value),
        }
    }
    None
}
fn dict<'a>(doc: &'a Document, value: &'a Object) -> Option<&'a Dictionary> {
    deref(doc, value)?.as_dict().ok()
}
fn number(value: &Object) -> Option<f64> {
    match value {
        Object::Integer(v) => Some(*v as f64),
        Object::Real(v) if v.is_finite() => Some(f64::from(*v)),
        _ => None,
    }
}
fn text(value: &Object) -> Option<String> {
    decode_text_string(value)
        .ok()
        .map(|s| compact(&s))
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(300).collect())
}

fn name_tree(
    doc: &Document,
    value: &Object,
    out: &mut HashMap<Vec<u8>, Object>,
    seen: &mut HashSet<ObjectId>,
    depth: usize,
) {
    if depth > 12 || seen.len() >= MAX_HEADINGS * 2 || out.len() >= MAX_HEADINGS {
        return;
    }
    if let Object::Reference(id) = value {
        if !seen.insert(*id) {
            return;
        }
    }
    let Some(d) = dict(doc, value) else {
        return;
    };
    if let Ok(values) = d.get(b"Names").and_then(Object::as_array) {
        for pair in values
            .chunks_exact(2)
            .take(MAX_HEADINGS.saturating_sub(out.len()))
        {
            if let Ok(key) = pair[0].as_str() {
                out.insert(key.to_vec(), pair[1].clone());
            }
        }
    }
    if let Ok(kids) = d.get(b"Kids").and_then(Object::as_array) {
        for child in kids.iter().take(MAX_HEADINGS) {
            name_tree(doc, child, out, seen, depth + 1);
        }
    }
}
fn destination<'a>(
    doc: &'a Document,
    value: &'a Object,
    names: &'a HashMap<Vec<u8>, Object>,
) -> Option<&'a Vec<Object>> {
    let mut value = value;
    for _ in 0..16 {
        value = deref(doc, value)?;
        match value {
            Object::Array(array) => return Some(array),
            Object::Dictionary(d) => value = d.get(b"D").ok()?,
            Object::Name(key) | Object::String(key, _) => value = names.get(key)?,
            _ => return None,
        }
    }
    None
}
fn outline_anchor(
    title: &str,
    page: u32,
    top: Option<f64>,
    left: Option<f64>,
    anchors: &[Anchor],
) -> Option<Anchor> {
    let compact_title = compact(title);
    let matches: Vec<_> = anchors
        .iter()
        .filter(|a| a.page == page && compact(&a.original_text) == compact_title)
        .collect();
    if matches.len() == 1 {
        return Some(matches[0].clone());
    }
    let top = top?;
    // A precise supplied destination may have a different bookmark title.
    // Require proximity to its baseline; an unresolvable page-only outline
    // is still available in Original, but cannot invent a text segment target.
    let distance =
        |a: &Anchor| (a.baseline - top).abs() + left.map(|x| (a.x0 - x).abs() * 0.1).unwrap_or(0.0);
    let mut nearby: Vec<_> = anchors
        .iter()
        .filter(|a| a.page == page && (a.baseline - top).abs() <= a.size * 2.0)
        .collect();
    nearby.sort_by(|a, b| distance(a).total_cmp(&distance(b)));
    let first = *nearby.first()?;
    if nearby
        .get(1)
        .is_some_and(|next| distance(next) - distance(first) < first.size.max(1.0) * 0.3)
    {
        return None;
    }
    Some(first.clone())
}

struct OutlineWalk<'a> {
    doc: &'a Document,
    pages: HashMap<ObjectId, u32>,
    names: HashMap<Vec<u8>, Object>,
    anchors: &'a HashMap<u32, Vec<Anchor>>,
    seen: HashSet<ObjectId>,
    count: usize,
    found: Vec<Candidate>,
}
impl OutlineWalk<'_> {
    fn siblings(&mut self, first: &Object, level: u8, parents: &[String]) {
        if level > 12 || self.count >= MAX_HEADINGS {
            return;
        }
        let mut current = Some(first.clone());
        while let Some(value) = current {
            if self.count >= MAX_HEADINGS {
                break;
            }
            if let Object::Reference(id) = value {
                if !self.seen.insert(id) {
                    break;
                }
            }
            self.count += 1;
            let Some(node) = dict(self.doc, &value) else {
                break;
            };
            current = node.get(b"Next").ok().cloned();
            let title = node.get(b"Title").ok().and_then(text);
            let action = node.get(b"A").ok().and_then(|v| dict(self.doc, v));
            let target = node.get(b"Dest").ok().or_else(|| {
                action
                    .filter(|a| a.get(b"S").and_then(Object::as_name).ok() == Some(b"GoTo"))
                    .and_then(|a| a.get(b"D").ok())
            });
            if let (Some(title), Some(target)) = (&title, target) {
                if let Some(dest) = destination(self.doc, target, &self.names) {
                    let page = dest.first().and_then(|p| match p {
                        Object::Reference(id) => self.pages.get(id).copied(),
                        Object::Integer(index) if *index >= 0 => u32::try_from(index + 1)
                            .ok()
                            .filter(|p| *p <= self.pages.len() as u32),
                        _ => None,
                    });
                    if let Some(page) = page {
                        let kind = dest.get(1).and_then(|v| v.as_name().ok());
                        let y = match kind {
                            Some(b"XYZ") => dest.get(3).and_then(number),
                            Some(b"FitH") | Some(b"FitBH") => dest.get(2).and_then(number),
                            _ => None,
                        };
                        let left = if kind == Some(b"XYZ") {
                            dest.get(2).and_then(number)
                        } else {
                            None
                        };
                        let page_anchors =
                            self.anchors.get(&page).map(Vec::as_slice).unwrap_or(&[]);
                        let height = page_anchors.first().map(|a| a.page_height);
                        if let Some(anchor) = outline_anchor(
                            title,
                            page,
                            y.zip(height).map(|(y, h)| h - y),
                            left,
                            page_anchors,
                        ) {
                            self.found.push(Candidate {
                                title: title.clone(),
                                level,
                                parents: parents.to_vec(),
                                basis: "outline".into(),
                                anchor,
                            });
                        }
                    }
                }
            }
            if let Ok(child) = node.get(b"First") {
                let mut enclosing = parents.to_vec();
                if let Some(title) = title {
                    enclosing.push(title);
                }
                self.siblings(child, level + 1, &enclosing);
            }
        }
    }
}

/// Read MCID text with the producer's font encoding. Unmatched/unsupported
/// forms are omitted; exact alignment to the independent layout is required.
fn marked_text(doc: &Document, page_id: ObjectId) -> BTreeMap<i64, String> {
    let Ok(fonts) = doc.get_page_fonts(page_id) else {
        return BTreeMap::new();
    };
    let encodings: HashMap<_, _> = fonts
        .into_iter()
        .filter_map(|(name, font)| font.get_font_encoding(doc).ok().map(|e| (name, e)))
        .collect();
    let Ok(bytes) = doc.get_page_content(page_id) else {
        return BTreeMap::new();
    };
    let Ok(content) = Content::decode(&bytes) else {
        return BTreeMap::new();
    };
    let mut active: Vec<Option<i64>> = Vec::new();
    let mut font = None;
    let mut out: BTreeMap<i64, String> = BTreeMap::new();
    fn append(value: &Object, encoding: &lopdf::Encoding<'_>, text: &mut String) {
        match value {
            Object::String(bytes, _) => {
                if let Ok(decoded) = Document::decode_text(encoding, bytes) {
                    text.push_str(&decoded);
                }
            }
            Object::Array(values) => {
                for value in values {
                    append(value, encoding, text);
                }
            }
            Object::Integer(n) if *n < -100 => text.push(' '),
            Object::Real(n) if *n < -100.0 => text.push(' '),
            _ => {}
        }
    }
    let mut total = 0usize;
    for operation in content.operations.iter().take(1_000_000) {
        if total > 500_000 || out.len() > 20_000 {
            break;
        }
        match operation.operator.as_str() {
            "BDC" => {
                if active.len() >= 64 {
                    return BTreeMap::new();
                }
                active.push(
                    operation
                        .operands
                        .get(1)
                        .and_then(|v| dict(doc, v))
                        .and_then(|d| d.get(b"MCID").ok())
                        .and_then(|v| v.as_i64().ok()),
                );
            }
            "BMC" => {
                if active.len() >= 64 {
                    return BTreeMap::new();
                }
                active.push(None);
            }
            "EMC" => {
                active.pop();
            }
            "Tf" => {
                font = operation
                    .operands
                    .first()
                    .and_then(|v| v.as_name().ok())
                    .and_then(|name| encodings.get(name))
            }
            "Tj" | "TJ" | "'" | "\"" => {
                if let (Some(id), Some(encoding)) = (active.iter().rev().find_map(|x| *x), font) {
                    let text = out.entry(id).or_default();
                    if text.len() > 200_000 {
                        continue;
                    }
                    if matches!(operation.operator.as_str(), "'" | "\"") {
                        text.push(' ');
                    }
                    let before = text.len();
                    for value in operation
                        .operands
                        .iter()
                        .skip(if operation.operator == "\"" { 2 } else { 0 })
                    {
                        append(value, encoding, text);
                    }
                    total += text.len().saturating_sub(before);
                }
            }
            "T*" | "ET" => {
                if let Some(id) = active.iter().rev().find_map(|x| *x) {
                    if let Some(text) = out.get_mut(&id) {
                        text.push(' ');
                    }
                }
            }
            _ => {}
        }
    }
    out
}

struct TagWalk<'a> {
    doc: &'a Document,
    page_ids: HashMap<ObjectId, u32>,
    text: HashMap<(u32, i64), String>,
    anchors: &'a HashMap<u32, Vec<Anchor>>,
    seen: HashSet<ObjectId>,
    count: usize,
    found: Vec<Candidate>,
    roles: Vec<Role>,
    unaligned: usize,
    role_map: Option<&'a Dictionary>,
}
impl TagWalk<'_> {
    fn walk(
        &mut self,
        value: &Object,
        parent_page: Option<u32>,
        depth: usize,
        excluded: Option<&'static str>,
    ) {
        if depth > 32 || self.count >= MAX_HEADINGS * 8 {
            return;
        }
        if let Object::Reference(id) = value {
            if !self.seen.insert(*id) {
                return;
            }
        }
        let Some(value) = deref(self.doc, value) else {
            return;
        };
        if let Object::Array(items) = value {
            for item in items.iter().take(MAX_HEADINGS * 8) {
                self.walk(item, parent_page, depth + 1, excluded);
            }
            return;
        }
        let Ok(node) = value.as_dict() else {
            return;
        };
        self.count += 1;
        let page = node
            .get(b"Pg")
            .and_then(Object::as_reference)
            .ok()
            .and_then(|id| self.page_ids.get(&id).copied())
            .or(parent_page);
        let role = node.get(b"S").and_then(Object::as_name).ok().map(|s| {
            self.role_map
                .and_then(|m| m.get(s).ok())
                .and_then(|v| v.as_name().ok())
                .unwrap_or(s)
        });
        let excluded = match role {
            Some(b"Note") => Some("note"),
            Some(b"Artifact") => Some("artifact"),
            _ => excluded,
        };
        let level = role.filter(|_| excluded.is_none()).and_then(|s| match s {
            b"H1" => Some(1),
            b"H2" => Some(2),
            b"H3" => Some(3),
            b"H4" => Some(4),
            b"H5" => Some(5),
            b"H6" => Some(6),
            _ => None,
        });
        let kind = excluded.or(match role {
            Some(b"Note") => Some("note"),
            Some(b"L") | Some(b"LI") | Some(b"LBody") => Some("list"),
            Some(b"BlockQuote") | Some(b"Quote") => Some("quote"),
            Some(b"P") => Some("paragraph"),
            _ => None,
        });
        if level.is_some() || kind.is_some() {
            let mut ids = Vec::new();
            fn ids_of(
                doc: &Document,
                value: &Object,
                page: Option<u32>,
                out: &mut Vec<(u32, i64)>,
                depth: usize,
            ) {
                if depth > 12 || out.len() > 1000 {
                    return;
                }
                let Some(v) = deref(doc, value) else {
                    return;
                };
                match v {
                    Object::Integer(id) => {
                        if let Some(p) = page {
                            out.push((p, *id));
                        }
                    }
                    Object::Array(a) => {
                        for v in a.iter().take(1000) {
                            ids_of(doc, v, page, out, depth + 1);
                        }
                    }
                    Object::Dictionary(d) => {
                        let page = d
                            .get(b"Pg")
                            .and_then(Object::as_reference)
                            .ok()
                            .and_then(|id| {
                                doc.get_pages()
                                    .into_iter()
                                    .find(|(_, p)| *p == id)
                                    .map(|(n, _)| n)
                            })
                            .or(page);
                        if let (Some(p), Ok(id)) = (page, d.get(b"MCID").and_then(Object::as_i64)) {
                            out.push((p, id));
                        } else if let Ok(k) = d.get(b"K") {
                            ids_of(doc, k, page, out, depth + 1);
                        }
                    }
                    _ => {}
                }
            }
            if let Ok(k) = node.get(b"K") {
                ids_of(self.doc, k, page, &mut ids, 0);
            }
            let written = ids
                .iter()
                .filter_map(|id| self.text.get(id))
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            if let Some(page) = page {
                let wanted = compact(&written);
                let exact: Vec<_> = self
                    .anchors
                    .get(&page)
                    .into_iter()
                    .flatten()
                    .filter(|a| compact(&a.original_text) == wanted)
                    .collect();
                if let (Some(level), [anchor]) = (level, exact.as_slice()) {
                    self.found.push(Candidate {
                        title: compact(&written),
                        level,
                        parents: Vec::new(),
                        basis: "tagged".into(),
                        anchor: (*anchor).clone(),
                    });
                } else if level.is_some() {
                    self.unaligned += 1;
                }
                if let Some(role) = kind {
                    if !written.trim().is_empty() {
                        self.roles.push(Role {
                            page,
                            text: written,
                            role: role.into(),
                        });
                    }
                }
            }
        }
        if let Ok(k) = node.get(b"K") {
            self.walk(k, page, depth + 1, excluded);
        }
    }
}

fn roman(mut n: usize, upper: bool) -> String {
    if n == 0 || n > 3999 {
        return n.to_string();
    }
    let mut out = String::new();
    for (value, text) in [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ] {
        while n >= value {
            out.push_str(text);
            n -= value;
        }
    }
    if upper {
        out.to_uppercase()
    } else {
        out
    }
}
fn labels(doc: &Document, pages: u32) -> BTreeMap<u32, String> {
    fn collect(
        doc: &Document,
        value: &Object,
        depth: usize,
        seen: &mut HashSet<ObjectId>,
        out: &mut BTreeMap<u32, Dictionary>,
    ) {
        if depth > 12 || seen.len() > 10_000 || out.len() > 5000 {
            return;
        }
        if let Object::Reference(id) = value {
            if !seen.insert(*id) {
                return;
            }
        }
        let Some(d) = dict(doc, value) else {
            return;
        };
        if let Ok(nums) = d.get(b"Nums").and_then(Object::as_array) {
            for pair in nums.chunks_exact(2).take(5000) {
                if let (Ok(n), Some(label)) = (pair[0].as_i64(), dict(doc, &pair[1])) {
                    if let Ok(n) = u32::try_from(n) {
                        out.insert(n, label.clone());
                    }
                }
            }
        }
        if let Ok(kids) = d.get(b"Kids").and_then(Object::as_array) {
            for kid in kids.iter().take(5000) {
                collect(doc, kid, depth + 1, seen, out);
            }
        }
    }
    let mut ranges = BTreeMap::new();
    let mut out = BTreeMap::new();
    if let Ok(value) = doc.catalog().and_then(|d| d.get(b"PageLabels")) {
        collect(doc, value, 0, &mut HashSet::new(), &mut ranges);
    }
    for index in 0..pages {
        if let Some((start, label)) = ranges.range(..=index).next_back() {
            let prefix = label
                .get(b"P")
                .ok()
                .and_then(|v| decode_text_string(v).ok())
                .map(|s| s.chars().take(100).collect::<String>())
                .unwrap_or_default();
            let n = label
                .get(b"St")
                .and_then(Object::as_i64)
                .ok()
                .filter(|n| *n > 0)
                .unwrap_or(1) as usize
                + (index - start) as usize;
            let suffix = match label.get(b"S").and_then(Object::as_name).ok() {
                Some(b"D") => n.to_string(),
                Some(b"r") => roman(n, false),
                Some(b"R") => roman(n, true),
                Some(b"a") | Some(b"A") if n <= 10_000 => {
                    let ch = char::from_u32(u32::from(b'a') + ((n - 1) % 26) as u32).unwrap_or('a');
                    let s = ch.to_string().repeat((n - 1) / 26 + 1);
                    if label.get(b"S").and_then(Object::as_name).ok() == Some(b"A") {
                        s.to_uppercase()
                    } else {
                        s
                    }
                }
                _ => String::new(),
            };
            let value = format!("{prefix}{suffix}");
            if !value.is_empty() && value.chars().count() <= 100 {
                out.insert(index + 1, value);
            }
        }
    }
    out
}

pub fn read(bytes: &[u8], anchors: &[Anchor]) -> Native {
    let Ok(doc) = Document::load_mem(bytes) else {
        return Native::default();
    };
    if doc.is_encrypted() {
        return Native::default();
    }
    let Ok(catalog) = doc.catalog() else {
        return Native::default();
    };
    let pages: HashMap<ObjectId, u32> =
        doc.get_pages().into_iter().map(|(n, id)| (id, n)).collect();
    let mut names = HashMap::new();
    if let Some(d) = catalog.get(b"Dests").ok().and_then(|v| dict(&doc, v)) {
        for (key, value) in d.iter().take(MAX_HEADINGS) {
            names.insert(key.clone(), value.clone());
        }
    }
    if let Some(tree) = catalog
        .get(b"Names")
        .ok()
        .and_then(|v| dict(&doc, v))
        .and_then(|d| d.get(b"Dests").ok())
    {
        name_tree(&doc, tree, &mut names, &mut HashSet::new(), 0);
    }
    let mut by_page: HashMap<u32, Vec<Anchor>> = HashMap::new();
    for a in anchors {
        by_page.entry(a.page).or_default().push(a.clone());
    }
    let mut walk = OutlineWalk {
        doc: &doc,
        pages: pages.clone(),
        names,
        anchors: &by_page,
        seen: HashSet::new(),
        count: 0,
        found: Vec::new(),
    };
    if let Some(first) = catalog
        .get(b"Outlines")
        .ok()
        .and_then(|v| dict(&doc, v))
        .and_then(|d| d.get(b"First").ok())
    {
        walk.siblings(first, 1, &[]);
    }
    let mut out = Native {
        truncated: walk.count >= MAX_HEADINGS,
        headings: walk.found,
        labels: labels(&doc, pages.len() as u32),
        ..Default::default()
    };
    if let Ok(tree) = catalog.get(b"StructTreeRoot") {
        out.tagged = true;
        out.truncated |= pages.len() > 3000;
        let mut marked = HashMap::new();
        let mut count = 0usize;
        let present: HashSet<_> = anchors.iter().map(|a| a.page).collect();
        for (page, id) in doc
            .get_pages()
            .into_iter()
            .take(3000)
            .filter(|(p, _)| present.contains(p))
        {
            if count >= MAX_CHARS {
                break;
            }
            for (mcid, text) in marked_text(&doc, id) {
                count += text.len();
                if count > MAX_CHARS {
                    break;
                }
                marked.insert((page, mcid), text);
            }
        }
        let role_map = dict(&doc, tree)
            .and_then(|d| d.get(b"RoleMap").ok())
            .and_then(|v| dict(&doc, v));
        let mut tags = TagWalk {
            doc: &doc,
            page_ids: pages,
            text: marked,
            anchors: &by_page,
            seen: HashSet::new(),
            count: 0,
            found: Vec::new(),
            roles: Vec::new(),
            unaligned: 0,
            role_map,
        };
        tags.walk(tree, None, 0, None);
        out.truncated |= tags.count >= MAX_HEADINGS * 8;
        out.headings.extend(tags.found);
        out.roles = tags.roles;
        out.unaligned_tags = tags.unaligned;
    }
    // At a shared anchor prefer the deepest producer heading. Its supplied
    // parent titles still carry grouping context without duplicating text.
    out.headings.sort_by(|a, b| {
        a.anchor
            .page
            .cmp(&b.anchor.page)
            .then(a.anchor.first_char.cmp(&b.anchor.first_char))
            .then_with(|| b.level.cmp(&a.level))
            .then(a.basis.cmp(&b.basis))
    });
    out
}
