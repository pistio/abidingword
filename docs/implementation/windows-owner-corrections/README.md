# Windows owner correction handover

Application source: **`571b8fc0633b6909b6532e1c6b254e2dbb357f45`**.
Development branch: **`codex/windows-owner-corrections-35`**.
No main merge, release, updater publication, paid Actions dispatch or deployment.

## Reviewable commits

| Application commit | Correction |
| --- | --- |
| `c4052f560aea645f3ac9c5ab594157fdaa54f79d` | #35 worker stages, progress, cancellation, deadlines, rollback, safe diagnostics and performance |
| `443b9dd2319a9361ed09604d8245436c7db5e734` | #34 grounded evidence hierarchy, printed/Greek Source metadata, combined reader controls and contained overlays |
| `8d20852369794dbfe9a580110b51c4a668a822a2` | #33 canonical method, persistent reversible repair, compact editor, hints/modal deletion and simple Teaching additions |
| `3ff5d8a74ba5098c69429acfec5280d18e4d91ed` | #35 automatic Contents uses the same observable, persisted pipeline; no hidden session-only parser |
| `86ce623ed3666d131a58edce22d9e613368ee71a` | Regression assertion distinguishes seven method parts from the separate optional application; application bytes unchanged |
| `571b8fc0633b6909b6532e1c6b254e2dbb357f45` | Repair Undo also preserves open note drafts and unsubmitted part names, which SQLite cannot yet see |

Verified parent branch: `codex/issue-31-atomic-study-transfer` at
`3badf22504939a91020168728913e89fc04b935c`, still unchanged remotely. Both exact
A.4 `b85b9fc7b6a4dfee02bd1ae07a5a05fe7597f669` and atomic fix
`35526df382495f3000e7a2aea7b26de842211b9f` are ancestors. Schema stays **27**;
0001–0027 match the approved catalogue digest
`acb4707b448735285346834b3cab04caa61fe3696079b84a7e5d9c35b4cd4e13`.
0028 remains reserved for the separate provisional Bible-import lineage.
Claude's Scripture-recognition Set F lineage remains isolated and NOT READY.

The [requirement ledger](acceptance-ledger.md) supplements the unchanged original
[A.4 acceptance ledger](../milestone-a4/acceptance-ledger.md). Technical completion
never changes an owner gate to accepted.

## Validation

The final frozen source passed **1,094 Rust tests / 0 failed / 8 existing ignored**
and **1,231 frontend tests in 114 files / 0 failed**. Typecheck, ESLint, Rust fmt,
Clippy all-targets with warnings denied, six-language consistency/corpus,
frontend production build and native build passed. The production build retains
its existing vendor-chunk-size advisory. All three browser review scripts passed.
See [machine-readable verification](verification.json) and the clean
[source/migration proof](build-provenance.json).

Application bytes were frozen at the SHA above before this documentation/workflow-only
handover. The handover branch tip adds only review artifacts and the manual workflow.

See [processing evidence and limitations](processing-review.md),
[Source/reader review](source-reader-review.md) and
[Study/editor review](study-editor-review.md). The complete regression suites
include the #31 all-or-nothing transfer and SQL/read/commit failure injections,
Q&A privacy-safe exports, citations, provenance, templates, offline use, optional
AI consent and scripted DeepSeek/Korean assistance. No real provider was called.

All five A.4 workstreams remain present: Source structure/diagnostics; configurable
compact reader; writing-first Study/Teaching; classified DeepSeek block assistance;
Q&A/exports/atomic transfer. Existing behavior was corrected in place, with changed
control routes reflected in tests. No new Teaching integration was developed.

## Before/after screenshots

All examples are invented; browser evidence is **Linux Chromium**, not WebView2.

- Reader: [before: two toolbars](images/34-before-original-900.png),
  [after: one toolbar beside existing writing](images/34-original-900-1.png),
  [geometry/zoom/focus results](images/34-browser-checks.json).
- Editor: [before: ten lines](images/33-before-1440.png),
  [after: five resizable lines and empty-part hints](images/33-after-1440.png),
  [empty-part deletion modal](images/33-delete-modal-900.png),
  [explicit repair and duplicate preview](images/33-repair-preview-1440.png),
  [drag-resize/repair browser results](images/33-browser-checks.json).
- Processing at the final source: [visible progress](images/final-processing/35-progress-900.png),
  [cancelled/retry](images/final-processing/35-cancelled-900.png),
  [presentation checks](images/final-processing/35-browser-checks.json).
  This browser job state is scripted; actual processing, persistence, performance
  and rollback are tested independently in Rust.

## Prepared Windows installer

[windows-owner-corrections-acceptance-build.yml](../../../.github/workflows/windows-owner-corrections-acceptance-build.yml)
is manual-only, pinned to the exact application SHA above, with pinned actions,
Node/Rust versions and historical-migration verification. It builds an unsigned
NSIS review artifact and uploads installer checksum, source and environment
provenance. Read-only repository permissions; no signing secrets, releases,
updater configuration or production publication. **It has not been dispatched.**
A paid runner requires separate explicit owner approval. The prior A.4 installer
workflow is preserved for comparison.

After an approved build, confirm its provenance says the application SHA above
(the workflow/handover SHA is separate), schema 27 and the installer checksum.
Close Abiding Word and retain the matching A.3 backup/executable. Restore/copy a
profile into a disposable review directory with the original managed Sources.
Do not auto-launch from the installer. Launch the installed executable from
PowerShell with the copied directory selected, for example:

```powershell
$env:ABIDING_WORD_DATA_DIR = 'C:\AW-review\owner-corrections-profile-copy'
& 'C:\path\to\installed\abiding-word.exe'
```

Use the actual installed executable path and a review profile you created; do
not point this at the primary data directory. Follow the detailed owner checks
in the ledger and the two review documents, especially:

1. Rebuild/open the actual MacArthur file; record stages, counts, elapsed time,
   cancellation/retry, restart persistence and an **Apg 6** diagnostic query.
   Test automatic and manually corrected targeting separately. Repeat Contents
   review on *Basisinformationen zur Bibel* and Greek NT as a separate Source.
2. Test actual WebView2 geometry at narrow panes and 100/125/150/200% scaling:
   Contents, Original, 400% zoom, Reading/Original restoration, page jump,
   splitter/window resizing, focus and adjacent writing.
3. Validate original POZETEK and Ten-Step guidance against the owner's guide;
   preview old/duplicate parts, cancel, repair, inspect writing/notes/citations/
   Teaching links, Undo, then verify safe refusal after writing in a new part.
4. Validate resizable fields, dismiss/reopen hints and named empty/populated
   deletion using keyboard and recovery. Test simple Teaching text, distinct
   Q&A/Scripture, all existing templates and private/public exports.
5. Perform the outstanding real DeepSeek Korean correction and original A.4
   Q&A/export/transfer owner checks. AI remains optional; offline work remains
   available throughout.

## Genuine remaining gates

This executor is Linux; it cannot establish Windows WebView2 behavior, Windows
memory/performance or native accessibility. The owner's actual PDFs and full
query-specific diagnostic JSON are not in this checkout. Published metadata
counts establish the missing-analysis condition, not the fonts/headings or the
correct Acts destination. **MacArthur is not declared fixed.**

Cancellation/deadlines are cooperative at page/stage boundaries: a third-party
single-page/font decode cannot be hard-interrupted. The retained stage/timings
make a pathological decode observable. No claim of a hard parser memory cap.

Stop here for Windows owner acceptance before further Teaching integration.
