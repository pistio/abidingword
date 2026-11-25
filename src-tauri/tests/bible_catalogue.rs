//! v0.24 Available Bibles by download: every file must be exactly the
//! verified one (size and SHA-256) or nothing is kept; only listed hosts;
//! the files then go through the normal Bible import. No network: a scripted
//! transport answers.

use std::collections::HashMap;
use std::future::Future;
use std::pin::pin;
use std::sync::Mutex;
use std::task::{Context as TaskContext, Poll, RawWaker, RawWakerVTable, Waker};

use abiding_word_lib::assistant::http::{HttpCall, HttpReply, Transport};
use abiding_word_lib::error::{AppError, AppResult};
use abiding_word_lib::scripture::catalogue::{self, CatalogueFile, Download};
use abiding_word_lib::scripture::import::{self, ImportOrigin};
use sha2::{Digest, Sha256};

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    fn raw() -> RawWaker {
        fn clone(_: *const ()) -> RawWaker {
            raw()
        }
        fn noop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    // SAFETY: the waker does nothing and holds no data.
    let waker = unsafe { Waker::from_raw(raw()) };
    let mut cx = TaskContext::from_waker(&waker);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
    }
}

/// Serves fixed bytes per address, and records what was asked for.
struct Served {
    files: HashMap<String, (u16, Vec<u8>)>,
    asked: Mutex<Vec<String>>,
}

impl Transport for Served {
    async fn send(&self, call: HttpCall) -> AppResult<HttpReply> {
        self.asked.lock().unwrap().push(call.url.clone());
        let (status, body) = self
            .files
            .get(&call.url)
            .cloned()
            .unwrap_or((404, Vec::new()));
        Ok(HttpReply { status, body })
    }
}

const BASE: &str =
    "https://raw.githubusercontent.com/example/bible/0123456789abcdef0123456789abcdef01234567/";

fn listed(files: &[(&str, &[u8])]) -> Download {
    Download {
        id: "test".into(),
        name: "Invented Bible".into(),
        short_name: "INV".into(),
        language: "English".into(),
        language_code: "en".into(),
        publisher: "Nobody".into(),
        license_type: "Public Domain".into(),
        copyright_notice: "Invented for tests.".into(),
        books: 2,
        contains: vec![],
        version: "test".into(),
        files: files
            .iter()
            .map(|(name, body)| CatalogueFile {
                name: name.to_string(),
                url: format!("{BASE}{name}"),
                sha256: Sha256::digest(body)
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
                size: body.len() as u64,
            })
            .collect(),
    }
}

fn serve(files: &[(&str, &[u8])]) -> Served {
    Served {
        files: files
            .iter()
            .map(|(name, body)| (format!("{BASE}{name}"), (200, body.to_vec())))
            .collect(),
        asked: Mutex::default(),
    }
}

const JHN: &[u8] = b"\\id JHN Invented Bible\n\\c 1\n\\v 1 In the beginning invented words.\n\\v 2 More invented words.\n";
const ROM: &[u8] = b"\\id ROM Invented Bible\n\\c 1\n\\v 1 Invented words to Rome.\n";

#[test]
fn a_verified_download_is_kept_and_read_like_any_bible_file() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("downloads/test");
    let files: &[(&str, &[u8])] = &[("43-JHN.usfm", JHN), ("45-ROM.usfm", ROM)];
    let mut seen = Vec::new();
    let paths = block_on(catalogue::fetch(
        &listed(files),
        &dir,
        &serve(files),
        |done, total| seen.push((done, total)),
    ))
    .unwrap();
    assert_eq!(seen, [(1, 2), (2, 2)]);
    assert_eq!(paths.len(), 2);
    let staged = import::stage(&paths, ImportOrigin::UserFiles).unwrap();
    let preview = staged.preview(Default::default());
    assert!(preview.can_import, "{:?}", preview.issues);
    assert_eq!(preview.book_count, 2);
}

#[test]
fn one_altered_file_means_nothing_is_kept() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("downloads/test");
    let files: &[(&str, &[u8])] = &[("43-JHN.usfm", JHN), ("45-ROM.usfm", ROM)];
    let mut served = serve(files);
    // The second file is not the verified one (same size, other bytes).
    let mut altered = ROM.to_vec();
    altered[20] ^= 1;
    served
        .files
        .insert(format!("{BASE}45-ROM.usfm"), (200, altered));
    let result = block_on(catalogue::fetch(&listed(files), &dir, &served, |_, _| {}));
    match result {
        Err(AppError::Import(m)) => assert_eq!(m.code, Some("catalogue.notVerified")),
        other => panic!("{other:?}"),
    }
    assert!(!dir.exists(), "a partial download is never left behind");
}

#[test]
fn a_missing_file_or_an_unlisted_host_is_refused() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("downloads/test");
    let files: &[(&str, &[u8])] = &[("43-JHN.usfm", JHN)];
    let mut served = serve(files);
    served.files.clear();
    let missing = block_on(catalogue::fetch(&listed(files), &dir, &served, |_, _| {}));
    assert!(
        matches!(missing, Err(AppError::Import(ref m)) if m.code == Some("catalogue.unreachable"))
    );
    assert!(!dir.exists());

    let mut elsewhere = listed(files);
    elsewhere.files[0].url = "https://example.org/43-JHN.usfm".into();
    let served = serve(files);
    let refused = block_on(catalogue::fetch(&elsewhere, &dir, &served, |_, _| {}));
    assert!(
        matches!(refused, Err(AppError::Import(ref m)) if m.code == Some("catalogue.notVerified"))
    );
    assert!(
        served.asked.lock().unwrap().is_empty(),
        "nothing is asked of an unlisted host"
    );

    let mut climbing = listed(files);
    climbing.files[0].name = "../escape.usfm".into();
    let refused = block_on(catalogue::fetch(&climbing, &dir, &serve(files), |_, _| {}));
    assert!(refused.is_err());
    assert!(!temp.path().join("downloads/escape.usfm").exists());
}

#[test]
fn the_catalogue_lists_its_editions_with_sizes_and_licences() {
    let list = catalogue::downloads();
    let oshb = list.iter().find(|d| d.id == "oshb").unwrap();
    assert_eq!(oshb.books, 39);
    assert!(oshb.copyright_notice.contains("CC BY 4.0"));
    assert_eq!(oshb.size(), 28_634_502);
    let morphgnt = list.iter().find(|d| d.id == "morphgnt").unwrap();
    assert!(morphgnt.copyright_notice.contains("CC BY-SA 3.0"));
    assert_eq!(morphgnt.size(), 8_936_874);
}
