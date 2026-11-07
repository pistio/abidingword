# Windows owner correction ledger — Issues #35, #34, #33

This ledger supplements, and does not replace, the original A.4 acceptance ledger.
No row is Windows-accepted without owner evidence. No new Teaching integration is
in scope. Starting branch `codex/issue-31-atomic-study-transfer` was verified by
HTTPS fetch at `3badf22504939a91020168728913e89fc04b935c`; the new branch is
`codex/windows-owner-corrections-35`. Its ancestors include the exact A.4 head
`b85b9fc7b6a4dfee02bd1ae07a5a05fe7597f669` and atomic fix
`35526df382495f3000e7a2aea7b26de842211b9f`. No checkout/reset of main.

## Evidence received

Issue #35's redacted owner diagnostic: 2,719 segments, 398 text pages, about
2,191,290 bytes, 4,112 **explicit literal** links; no saved structure, zero
headings; query null. Original mode/page 24 is an observation, not a verified
Acts 6 destination. The private original PDF/JSON is not in this checkout.
The issue text and full owner feedback on #34/#33, PR #32 and linked
#22/#26/#25/#27/#19/#31 were read. Do not infer the PDF's fonts, actual heading
locations or exact failure cause from the counts alone.

| Requirement | Technical state | Evidence / remaining gate |
| --- | --- | --- |
| #35 timed stages: file/hash, stored segments, PDF page/layout, native outline/tags, heading alignment, SQL/index/commit, UI refresh | Implemented | Job timings, numeric progress/elapsed time, exported metadata and UI refresh timing; see processing review |
| #35 background work; reading/writing available | Implemented | Explicit blocking worker; parsing outside DB mutex; concurrent read and actual write regression |
| #35 duplicate rebuild prevention | Implemented | One analysis at a time per application, retained job state, immediate UI disabling; native duplicate/retry tests |
| #35 cancellation, deadline, failed SQL, safe retry | Implemented | Cooperative page/stage/64-block checks; five-minute deadline at safe boundaries; commit gate; transactional rollback tests; no hard interruption inside a third-party page/font parser |
| #35 persist metadata and links together; preserve literal provenance | Implemented | Structure-only reindex avoids re-reading all body references; source bytes and all 4,112 literal IDs/timestamps retained |
| #35 zero headings must not imply usable Contents | Implemented | Explicit warning and manual-line correction path; stored analysis distinguished from useful headings |
| #35 consistent report display/save/query | Implemented | Same native diagnostic generator; save returns exact written report; busy/error feedback; stale query results cleared; no-query report explicitly general status |
| #35 owner-shaped performance and memory | Partial | Invented 400-page/2.19MB/2719-segment/4112-link fixture: ~6.1s vs ~27.2s for previous full-text reindex alone. Fixed page/text/heading limits; actual Windows memory/performance still open |
| #35 actual MacArthur rebuild completes/persists | **OWNER GATE OPEN** | Rerun on original Windows profile copy; export stage timings and optional Apg 6 query |
| #34 actual TOC → headings → body fallback hierarchy | Pending stage | Existing hierarchy preserved; no ranking-only patch in #35 |
| #34 printed TOC, native PDF headings, canon/order/ambiguous book safety | Pending stage | Require grounded destinations and separate uncertainty; no forced alternate-canon mapping |
| #34 generated Contents and actual Acts introduction | **OPEN** | Existing synthetic suites retained; no actual-file success claim |
| #34 Greek NT as Source | **OPEN / separate case** | Source metadata only; no recognition-lineage merge or verified-Scripture claim |
| #34 ordinary-language heading control | Pending stage | Clarify Document structure in all six languages |
| #34 one combined ReaderToolbar/PdfViewer row | Pending stage | Secondary viewer toolbar must be integrated, not merely CSS-hidden |
| #34 recurring three-pane layout, real WebView2 geometry/DPI/focus | **WINDOWS GATE OPEN** | This executor is Linux; browser evidence does not satisfy WebView2 |
| #33 exact original seven-part POZETEK and substantive guidance | Pending stage | Preserve existing study writing and stable keys; investigate historical introduction |
| #33 explicit reversible repair, duplicates, restore-missing | Pending stage | No automatic rename/delete/merge of existing content |
| #33 compact manually resizable fields; initial dismissible Hinweise | Pending stage | Preserve autosave, full text and manual preferences |
| #33 Ten-Step guidance / three concise thinking questions | Pending stage | Keep order/keys and only the three existing questions; no added lecture |
| #33 always-modal named section deletion, including empty | Pending stage | Focus, cancel, Escape/Enter and recovery required |
| #33 simpler additions and remove ordinary phase selector | Pending stage | Preserve saved phases, Q&A and all existing data |
| #34 simple Teaching Textfeld hinzufügen | Pending stage | Preserve distinct Q&A/Scripture and saved block roles/privacy/export |
| Schema 27, migrations 0001–0027, #31 atomicity, A.4 features | Preserved | Historical SQL and original ledger untouched; next migration reserved for provisional Bible safety |

## Windows owner steps for the first stage (#35)

No paid Actions dispatch or installer publication is authorized by these steps.
Once the owner approves a build of the reviewed source, use a disposable/copied
schema-27 profile with `ABIDING_WORD_DATA_DIR` set explicitly and retain the A.3
backup/matching executable. Never open primary data during development.

1. Open the original MacArthur Source, then its heading/structure panel. Before
   rebuilding, optionally save a report with **Apg 6** entered. General status
   (`query: null`) is useful but is not an Acts 6 matching diagnosis.
2. Choose Build/Rebuild. Record visible stage, page count and elapsed time.
   Close the panel and verify Scripture/Study reading and writing still respond;
   reopen it to see the same job. Do not start a second build.
3. On a copied profile, test Cancel during page processing. Wait for the safe
   boundary; cancellation is not instantaneous during one page/font decode.
   Before commit it must leave prior structure and links unchanged. Retry.
4. A successful nonempty build reports heading and indexed-link counts. Zero
   headings is explicitly not usable Contents; use Original/line correction or
   send the metadata report. A failure must name its phase and permit retry.
5. Reopen/restart and confirm the structure persists. Save a report with
   **Apg 6**, then select Acts 6 → Source result → record the actual heading,
   physical/printed page and observed reader target in Reading and Original.
   Do not label a broader Acts introduction as chapter-specific commentary.
6. Return stage timings/error code and screenshots (review heading labels before
   sharing). There is no need to upload the copyrighted PDF. Keep #22/#35 open
   until the actual-file result is verified; manual correction is separate from
   automatic recognition.
