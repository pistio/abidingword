//! Finding files for a batch import (v0.24), and proposing a category only
//! from evidence.
//!
//! The scan only lists files: it reads no file's contents. It follows no
//! symbolic links, skips hidden and system folders, goes at most
//! [`MAX_DEPTH`] folders down, and stops listing at the item limit (what is
//! beyond it is counted, not listed).

use std::fs;
use std::path::{Path, PathBuf};

use crate::db::import_jobs::{FoundFile, MAX_ITEMS};
use crate::library::SourceType;

/// Folder levels a scan goes down.
pub const MAX_DEPTH: usize = 8;
const READABLE: &[&str] = &["pdf", "epub", "docx", "md", "markdown", "txt"];
const SKIPPED_FOLDERS: &[&str] = &[
    "node_modules",
    "$recycle.bin",
    "system volume information",
    "__macosx",
];

#[derive(Debug, Default)]
pub struct Scan {
    pub files: Vec<FoundFile>,
    /// Files of kinds the library cannot read.
    pub unsupported: usize,
    /// Readable files beyond the item limit.
    pub over_limit: usize,
}

fn readable(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| READABLE.contains(&e.to_ascii_lowercase().as_str()))
}

fn hidden(name: &str) -> bool {
    name.starts_with('.') || name.starts_with('~')
}

/// Lists the readable files among `paths`: files as chosen, folders (with
/// their subfolders when `recursive`) in name order.
pub fn scan(paths: &[PathBuf], recursive: bool) -> Scan {
    let mut out = Scan::default();
    for path in paths {
        visit(path, recursive, 0, &mut out);
    }
    out
}

fn visit(path: &Path, recursive: bool, depth: usize, out: &mut Scan) {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    if meta.is_file() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if hidden(&name) {
            return;
        }
        if !readable(path) {
            out.unsupported += 1;
            return;
        }
        let key = path.to_string_lossy().into_owned();
        if out.files.iter().any(|f| f.path == key) {
            return;
        }
        if out.files.len() >= MAX_ITEMS {
            out.over_limit += 1;
            return;
        }
        out.files.push(FoundFile {
            path: key,
            file_name: name,
            size: meta.len() as i64,
        });
        return;
    }
    if !meta.is_dir() || depth > MAX_DEPTH {
        return;
    }
    let folder = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if depth > 0 && (hidden(&folder) || SKIPPED_FOLDERS.contains(&folder.as_str())) {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    children.sort();
    for child in children {
        let is_dir = fs::symlink_metadata(&child).is_ok_and(|m| m.is_dir());
        if is_dir && !recursive {
            continue;
        }
        visit(&child, recursive, depth + 1, out);
    }
}

/// Words that say what a source is, in the interface languages.
const EVIDENCE: &[(SourceType, &[&str])] = &[
    (
        SourceType::Commentary,
        &[
            "commentary",
            "commentaries",
            "kommentar",
            "kommentare",
            "comentario",
            "comentarios",
            "comentário",
            "comentários",
            "commentaire",
            "commentaires",
            "주석",
        ],
    ),
    (
        SourceType::Sermon,
        &[
            "sermon",
            "sermons",
            "predigt",
            "predigten",
            "sermón",
            "sermones",
            "sermão",
            "sermões",
            "homily",
            "homilies",
            "설교",
        ],
    ),
    (
        SourceType::Lecture,
        &[
            "lecture",
            "lectures",
            "vorlesung",
            "vorlesungen",
            "conférence",
            "conférences",
            "conferencia",
            "conferencias",
            "palestra",
            "palestras",
            "강의",
        ],
    ),
    (
        SourceType::Article,
        &[
            "article",
            "articles",
            "artikel",
            "artículo",
            "artículos",
            "artigo",
            "artigos",
            "aufsatz",
            "aufsätze",
            "논문",
        ],
    ),
    (SourceType::Notes, &["notes", "notizen", "notas", "메모"]),
];

/// The category to propose and why: a word in the title (`title`), the file's
/// name (`fileName`) or its folder's name (`folder`) that says what it is; otherwise the file type's
/// usual category (`fileType`), which the review shows as such.
pub fn propose_type(format: &str, title: &str, path: &Path) -> (SourceType, &'static str) {
    let has = |text: &str| {
        let text = text.to_lowercase();
        EVIDENCE.iter().find_map(|(kind, words)| {
            words
                .iter()
                .any(|w| text.split(|c: char| !c.is_alphanumeric()).any(|t| t == *w))
                .then_some(*kind)
        })
    };
    if let Some(kind) = has(title) {
        return (kind, "title");
    }
    if let Some(kind) = path.file_stem().and_then(|n| has(&n.to_string_lossy())) {
        return (kind, "fileName");
    }
    if let Some(kind) = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| has(&n.to_string_lossy()))
    {
        return (kind, "folder");
    }
    (SourceType::default_for(format), "fileType")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_category_is_proposed_only_from_a_word_that_says_so() {
        let p = Path::new("/books/Romans.pdf");
        assert_eq!(
            propose_type("pdf", "A Commentary on Romans", p),
            (SourceType::Commentary, "title")
        );
        assert_eq!(
            propose_type("docx", "Römerbrief", Path::new("/x/Predigt/römer.docx")),
            (SourceType::Sermon, "folder")
        );
        // Whole words only: "Commentator" says nothing.
        assert_eq!(
            propose_type("pdf", "The Commentator", p),
            (SourceType::Book, "fileType")
        );
        assert_eq!(
            propose_type("markdown", "Romans", p),
            (SourceType::Notes, "fileType")
        );
    }

    #[test]
    fn the_scan_lists_readable_files_and_counts_the_rest() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("sub/deeper")).unwrap();
        fs::create_dir_all(root.join(".hidden")).unwrap();
        for (name, text) in [
            ("a.md", "A"),
            ("b.txt", "B"),
            ("c.exe", "C"),
            ("sub/d.md", "D"),
            ("sub/deeper/e.md", "E"),
            (".hidden/f.md", "F"),
            (".g.md", "G"),
        ] {
            fs::write(root.join(name), text).unwrap();
        }
        let flat = scan(&[root.to_path_buf()], false);
        let names: Vec<_> = flat.files.iter().map(|f| f.file_name.as_str()).collect();
        assert_eq!(names, ["a.md", "b.txt"]);
        assert_eq!(flat.unsupported, 1);
        let deep = scan(&[root.to_path_buf(), root.join("a.md")], true);
        let names: Vec<_> = deep.files.iter().map(|f| f.file_name.as_str()).collect();
        assert_eq!(names, ["a.md", "b.txt", "d.md", "e.md"]);
    }
}
