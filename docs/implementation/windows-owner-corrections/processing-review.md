# Stage 1 — observable, atomic PDF structure processing (#35)

## Finding and correction

The real diagnostic proves **absence of saved structure**, not absence of possible
headings in the PDF. A successful no-query saved report also does not prove that
a queried diagnostic or a build completed. No real-file root cause is invented.

The previous rebuild command performed managed file read/hash, all layout/native
extraction, heading alignment and full passage reindex with no progress. Its SQL
phase reparsed every paragraph for explicit references, even though source text
was unchanged, holding the shared database mutex throughout. The new path:

- runs CPU/file work explicitly on a blocking worker, with the DB mutex released;
- retains bounded per-source job status/timings outside the panel (at most eight
  terminal records, one active analysis), available after closing/reopening;
- exposes numeric page/segment progress, elapsed time, terminal success/failure,
  cancellation and retry, including an honest zero-heading warning;
- checks cancellation/deadline between pages, structure stages and batches of 64
  indexing blocks; cancellation accepted before the commit gate rolls back;
- saves structure plus refreshed structure links in one transaction, materialises
  the response before commit, and retains existing literal link identities;
- records safe stage names, durations/counts/error codes only in processing logs
  and diagnostic metadata, never source paragraphs, names, paths or keys.

The five-minute deadline is **cooperative**, not a hard kill of an in-progress
third-party font/page decode. File input is capped at 200 MiB, rebuilds at 3,000
pages and 8,000,000 text bytes, structure at 120,000 lines/5,000 headings. These
limits and page-at-a-time collectors reduce retained work; they do not establish
an OS-enforced memory limit on decompression inside the PDF parser. Real Windows
performance/memory and actual MacArthur acceptance remain open.

## Measured fixture

Invented German text only: 400 physical pages, 398 text pages, 2,719 stored
segments, 4,112 pre-existing literal links; file size 2,191,359 bytes (unused legal
padding stream approximates file/hash work). It is a processing-shape fixture,
not a reconstruction of the owner's book or a real-file acceptance substitute.

Linux debug build, Rust 1.99.1, no debug info:

| Stage | Representative duration |
| --- | ---: |
| file read/hash | 76 ms |
| stored segments | 4 ms |
| PDF page/layout read | 2,758 ms |
| initial alignment | 6 ms |
| native outline/tags | 26 ms |
| headings | 42 ms |
| block/text alignment | 359 ms |
| metadata preparation/store | 223 ms |
| structure-link indexing | 2,604 ms |
| commit | 2 ms |
| complete new rebuild | **6,106 ms** |
| previous whole-text reindex alone, same fixture | **27,246 ms** |

Measured with `AW_PDF_BASELINE_PROFILE=1 cargo test --locked --test
pdf_build_processing -- --nocapture`. Ordinary tests omit the extra baseline
reindex and enforce a generous 60-second regression budget for the new rebuild.
On Linux they also measure process peak RSS with a coarse 768-MiB regression
ceiling, including concurrent fixture tests; Windows memory is still an owner gate.
A repeat measured 6,704 ms and 80,924 KiB peak RSS (about 79 MiB) for the
isolated Linux test process. Timing varies by machine; do not claim these
numbers for the real MacArthur PDF.
All literal IDs/timestamps and source bytes are asserted preserved. Injected
cancellation at store/index/commit and a real SQL trigger failure compare a
fingerprint of complete source/file/segment/structure/link rows after rollback.
A separate thread verifies database reads and a real write remain available
while pages are being parsed; the cancelled job leaves no saved structure.

## Diagnostics and screenshots

Create and Save both call the same native generator on a worker. Save returns the
actual report written atomically, so the panel displays that report rather than a
stale earlier result. Query edits clear old results/copy availability. Empty query
is labelled general status and explicitly suggests `Apg 6` for candidate review.
Processing timings are added to the existing metadata-only report when a job is
available in this app session. App restart retains saved structure, not live jobs.

Browser captures use the original UI from `3badf22504939a91020168728913e89fc04b935c`
and the changed UI against the existing invented handbook and actual PDF.js. The
progress state adapter is synthetic; native processing is independently tested.

- [Before, 900](images/35-before-900.png)
- [Progress, 640](images/35-progress-640.png), [900](images/35-progress-900.png), [1440](images/35-progress-1440.png)
- [Cancelled/retry, 900](images/35-cancelled-900.png)
- [Browser check results](images/35-browser-checks.json)

`tests/screenshots/owner-processing.mjs` exercises page counts, elapsed-time label,
duplicate prevention, close, cancel, retry and dialog overflow, with no page errors.
This is Linux Chromium, **not WebView2**; no Windows acceptance is claimed.

See the [requirement ledger and owner procedure](acceptance-ledger.md). All
original A.4 and atomic transfer gates remain intact. No new Teaching integration,
recogniser changes, historical SQL changes, merge, release, updater or paid Actions.

## Stage verification

Complete Rust suite: **1,084 passed, 0 failed, 8 preexisting ignored**. Complete
frontend suite: **1,220 passed / 113 files**. Format, Clippy with warnings denied,
TypeScript, ESLint, six-language localisation consistency/corpus and frontend /
native builds passed. Existing A.4 structure, privacy/export, backup/restore and
#31 rollback regressions remain enabled. The original acceptance ledger and
all migration SQL files are unchanged. Browser checks above passed in all four
scenes; real MacArthur / Greek NT / WebView2 owner gates remain open.

## Follow-up: automatic Contents also uses persisted analysis

The final route review found another old path: `library_generated_contents`
parsed a whole PDF synchronously and retained its answer only in a process-local
cache keyed by Source ID. That could leave Contents work invisible and the
metadata diagnostic showing no saved structure. This is a technical mechanism,
not proof of the precise failure on the owner's file.

For an old PDF with no saved structure, automatic Contents now uses the same
blocking worker, retained progress/cancellation/deadline and transactional
structure/link persistence as explicit Build. The reader shows a nonblocking
analysis status with **Progress / retry** leading to the measurable panel.
A saved analysis, including zero headings, is reused instead of triggering a
second whole-file parser. Failure/cancellation is retained and requires explicit
retry during the session. The unbounded session-only PDF Contents cache is removed.
DOCX fallback extraction also runs on the blocking worker, outside the IPC thread.

Three native regressions cover automatic persistence and fresh-session reuse,
empty saved analysis without re-parsing, and retained failure/explicit retry.
The nonempty case uses the same invented 400-page fixture; it is not actual-file
acceptance. Two reader regressions cover pending progress with reading still
usable, and safe failure/retry without exposing error text.
