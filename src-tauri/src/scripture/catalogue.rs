//! Available Bibles that are downloaded rather than shipped (v0.24).
//!
//! `resources/scripture/catalogue.json` lists each edition's files, every
//! one pinned to a fixed commit with its size and SHA-256. A download is
//! started only by the person (the interface shows the size first), goes
//! through [`crate::assistant::http::Transport`] (the one place requests
//! leave the app), and is kept only if **every** file is exactly the one
//! listed: one difference and nothing is kept. The files are then read by
//! the same importer and review screen as any Bible file; once added, the
//! Bible works offline. Licences and the editions left out are recorded in
//! `docs/original-language-catalogue-v0.24.md`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::assistant::http::{HttpCall, Method, Transport};
use crate::error::{AppError, AppResult, UserMessage};

const DATA: &str = include_str!("../../../resources/scripture/catalogue.json");
/// Hosts a catalogue file may come from.
const HOSTS: &[&str] = &["https://raw.githubusercontent.com/"];

#[derive(Debug, Clone, Deserialize)]
struct File {
    downloads: Vec<Download>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogueFile {
    pub name: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub language: String,
    pub language_code: String,
    pub publisher: String,
    pub license_type: String,
    pub copyright_notice: String,
    /// How many of the 66 books it holds.
    pub books: u8,
    /// Word information it carries: "lemma", "strong", "morph".
    pub contains: Vec<String>,
    pub version: String,
    #[serde(skip_serializing)]
    pub files: Vec<CatalogueFile>,
}

impl Download {
    pub fn size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
}

pub fn downloads() -> Vec<Download> {
    serde_json::from_str::<File>(DATA)
        .expect("catalogue.json is valid")
        .downloads
}

pub fn by_id(id: &str) -> AppResult<Download> {
    downloads()
        .into_iter()
        .find(|d| d.id == id)
        .ok_or_else(|| AppError::not_found("Bible", id))
}

fn refused(name: &str) -> AppError {
    AppError::Import(
        UserMessage::new(
            "catalogue.notVerified",
            format!(
                "“{name}” did not match the verified copy, so nothing was kept. Try again later."
            ),
        )
        .with("fileName", name),
    )
}

fn unreachable() -> AppError {
    AppError::Import(UserMessage::new(
        "catalogue.unreachable",
        "The download could not be completed. Check the internet connection and try again.",
    ))
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Downloads and verifies every file into `dir` (emptied first). On any
/// failure the folder is removed: a partial or altered download is never
/// left to be read. `progress(done, total)` is called after each file.
pub async fn fetch(
    download: &Download,
    dir: &Path,
    transport: &impl Transport,
    mut progress: impl FnMut(usize, usize),
) -> AppResult<Vec<PathBuf>> {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).map_err(|e| AppError::filesystem(dir, e))?;
    let result = async {
        let mut paths = Vec::new();
        for (index, file) in download.files.iter().enumerate() {
            if !safe_name(&file.name) || !HOSTS.iter().any(|h| file.url.starts_with(h)) {
                return Err(refused(&file.name));
            }
            let reply = transport
                .send(HttpCall {
                    method: Method::Get,
                    url: file.url.clone(),
                    headers: Vec::new(),
                    body: None,
                })
                .await
                .map_err(|_| unreachable())?;
            if reply.status != 200 {
                log::warn!("catalogue download {}: status {}", file.name, reply.status);
                return Err(unreachable());
            }
            let hash: String = Sha256::digest(&reply.body)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            if reply.body.len() as u64 != file.size || hash != file.sha256 {
                log::warn!("catalogue download {}: not the verified copy", file.name);
                return Err(refused(&file.name));
            }
            let path = dir.join(&file.name);
            fs::write(&path, &reply.body).map_err(|e| AppError::filesystem(&path, e))?;
            paths.push(path);
            progress(index + 1, download.files.len());
        }
        Ok(paths)
    }
    .await;
    if result.is_err() {
        let _ = fs::remove_dir_all(dir);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_file_is_pinned_to_a_commit_and_a_checksum() {
        let list = downloads();
        assert_eq!(
            list.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(),
            ["oshb", "morphgnt"]
        );
        for d in &list {
            assert!(!d.files.is_empty());
            for f in &d.files {
                assert!(safe_name(&f.name), "{}", f.name);
                assert!(HOSTS.iter().any(|h| f.url.starts_with(h)), "{}", f.url);
                // A 40-hex commit in the address: the file can never change.
                assert!(
                    f.url
                        .split('/')
                        .any(|p| p.len() == 40 && p.chars().all(|c| c.is_ascii_hexdigit())),
                    "{}",
                    f.url
                );
                assert_eq!(f.sha256.len(), 64);
                assert!(f.size > 0);
            }
        }
        // The Hebrew Bible carries its map to the English verse numbers.
        assert!(list[0].files.iter().any(|f| f.name == "VerseMap.xml"));
        assert_eq!(list[0].files.len(), 40);
        assert_eq!(list[1].files.len(), 27);
        assert_eq!(list[0].size(), 28_634_502);
    }
}
