# Abiding Word — continuity, current state and complete known roadmap

**Snapshot: 19 November 2026, continued 20 November 2026. Development environment: Codex Cloud / GitHub; Windows is the primary native acceptance environment. This is not NiPogi deployment work.**

This handover reconciles the surviving implementation records, current source, open issues and later owner decisions. It replaces neither the original concept nor historical evidence. It prevents an older handover, an unclosed issue or an outdated milestone table from restarting completed work or losing future obligations.

**Current position, recovery continuation on 20 November:** active work is `feature/daily-snapshot-recovery-20261120`, handed over at `b8d9af869c95395a44cc6d9a3867626ab83d9804` and stacked on draft PR #55 (`da4afbf58a0a94452cfeac5725524637e9af71d5`), itself stacked on draft PR #54. The milestone completes daily recovery, save settlement, preservation/privacy integration and release-readiness evidence. Final implementation `e8f52ff668fbc02880925809bedf0f0b8e26bf83` passes all permitted local checks: 1,427 frontend tests, 1,174 Rust tests (nine existing ignored) and 36 browser flows. Exact evidence is recorded in [daily recovery verification](daily-recovery-20261120-verification.json). The exact pushed-head CI result is recorded in [draft PR #56](https://github.com/pistio/abidingword/pull/56). The [revised publication matrix](publish-readiness-20261120.md) and [disposable Windows procedure](windows-owner-acceptance-20261120.md) are the current acceptance ledger.

The original Source/focus delivery records below are **historical evidence**, not the active branch's final verification. Later owner instructions and PR #55 confirm the `13b65b7` Windows candidate's **Settings → Sources → Edit details** acceptance. Preserve that, the accepted handbook correction, Source→Original correction (#49), POZETEK structure/resizing and Ten-Step methodology. The later [completed PR #54 build record](https://github.com/pistio/abidingword/pull/54#issuecomment-7043179084) confirms `13b65b7` was delivered by run `44758102948` on attempt 1; its setup EXE SHA-256 is `3fc34919f4544c4346d8280f18b2c96d1abb5b04e0e2c77e164919c5f9003109`. This supersedes earlier “not yet in the Windows ZIP” statements in the historical sections. The recovery branch has no new Windows build or native acceptance yet. Existing Teaching, Q&A, Materials, transfer and portability implementations are retained rather than rebuilt from stale issue descriptions.

**Independent recognition boundary:** the owner now reports Claude is running withheld Collection H. The 19 November freeze and protocol notes in section 6 remain a dated record, not a claim that H has not subsequently started. This recovery milestone does not inspect withheld documents, change the frozen candidate, integrate unapproved recognition results or use reserved migration 0028.

## 1. Ultimate objective

Abiding Word is a **local-first, source-owned, multilingual Bible Study Workspace** for careful Scripture study, faithful teaching and a lifelong personal body of biblical knowledge.

- **Scripture at the centre. Your study around it.**
- **Word → Understanding → Teaching.**
- **Study with diligence. Teach with love.**
- **Simple enough to begin with a Bible verse. Powerful enough to write a commentary.**

The fuller intellectual journey is **Text → Understanding → Theology → Application → Audience → Teaching**. The person supplies their theological and hermeneutical convictions, preferred translations and methods, trusted sources, observations, interpretation and writing. Software supplies dependable retrieval, organisation, provenance, reusable work and optional bounded assistance.

The study is a persistent structured object, not a chat transcript. A user should return years later to earlier findings, unresolved questions, sources and conclusions, see how they were used in subsequent teaching, and continue from that accumulated work. One substantial study can support several audience-specific expressions: sermon, small-group discussion, lesson, leader guide, worksheet, slides, devotional, course, article or commentary material.

The governing pedagogical commitments are love of God and neighbour, diligent handling of Scripture, contextual interpretation and clear, appropriate teaching. The concept cites 2 Timothy 2:15. Scripture remains the norm for evaluating secondary sources. The general product accommodates a user's stated confessional profile; it must not silently flatten theological disagreement or turn the owner's personal library into everybody's mandatory library. Theological preferences never alter deterministic factual retrieval.

The accepted implementation architecture is **Tauri, React/TypeScript, Rust and SQLite**, useful offline on one device without a hosted Abiding Word service. The earlier concept's browser/PWA-first suggestion does not override this later frozen desktop direction. Windows is the primary acceptance platform; Linux/Chromebook compatibility remains, with macOS, mobile and optional cross-device capabilities in the wider roadmap.

Authoritative references: [concept](../concept.md), [architecture](../architecture.md), [product backlog](../product-backlog.md). Historical status sentences in these documents require the supersession rules below.

## 2. Historical Source/focus delivery and evidence

| Item | Snapshot |
|---|---|
| Application review | [Draft PR #54](https://github.com/pistio/abidingword/pull/54), stacked on the reviewed Windows lineage |
| Latest application follow-up | `13b65b74903b7a02e1c8c0f6ef4a5a92c1814e54`; tree `50c15740b989c478900d86a31d606956fc3b9f93`; preserves Settings/language work and corrects writing/Presenter focus; not yet in the Windows ZIP |
| Delivered Windows application source | `bcf4858591f6d306838773ebc884550ca27d19b6` |
| Delivered Windows application tree | `f2a06a00276d4913d909dc859dadc429044ab402` |
| Global Source work | General PDF extraction/navigation fixes and whole-source coverage implemented; later Settings/menu and multilingual-hierarchy follow-up described in section 2.3 |
| Delivered candidate's complete native suite | 1,137 passed; zero failures; nine ignored; follow-up evidence is separate |
| Delivered candidate's complete frontend suite | Historical run: 1,362 / 1,364 passed, with Presenter and Teaching failures. The later follow-up passes all 1,364; no timing-cause repair is claimed. |
| Windows build | [Run 44621293307](https://github.com/pistio/abidingword/actions/runs/44621293307), started `2026-11-19T21:41:26Z` |
| Isolated build commit | `ca169b377233ca678421699c4aa36d906c090617` |
| Build outcome | **Success on attempt 1**, completed `2026-11-19T21:57:32Z`; one application compilation and one NSIS package. Downloaded installer and bundled payload verified. Native desktop acceptance remains pending. |
| Approval scope | The one Windows build was already approved. It does not imply main merge, automatic update, signing, public release or NiPogi deployment. |

Current application and Source records:

- [Writing/Presenter focus continuation](focus-restoration-20261120.md) and [exact verification](focus-restoration-20261120-verification.json).

- [Source navigation and real-file investigation](source-navigation-20261119.md).
- [Global PDF extraction and generality](pdf-extraction-generality-20261119.md).
- [Whole-source Scripture coverage](source-coverage-20261119.md).
- [Source navigation verification](source-navigation-20261119-verification.json) and [whole-source verification](source-coverage-20261119-verification.json).
- [Settings editing and multilingual hierarchy](source-settings-aliases-20261119.md) and its [separate verification](source-settings-aliases-20261119-verification.json).

The owner supplied the original reference PDF; earlier statements that it was unavailable are superseded. Rust import and rebuild checks on Linux reproduced and corrected its extraction/navigation failure. This is evidence about the supplied file and tested classes of documents, not a guarantee for every possible PDF. Original source files, extracted copyrighted text, private databases and owner screenshots are not publication artifacts.

### 2.1 Whole-source recommendations and global PDF behaviour

The whole-source feature is already in the exact candidate above. It is not a new implementation task for the next agent.

| Evidence | Meaning and reader behaviour |
|---|---|
| Real passage destination or explicit confirmed location | Navigate to that actual place when the person requests it. |
| Real containing chapter/book section without a finer target | Navigate to the genuine broader section and label its scope honestly. |
| Known whole-work scope without a precise location | Show, for example, **Covers Romans 8**; opening the work resumes its normal saved position and mode. |
| Whole-work scope plus incidental mentions | Offer the work and retain each actual mention as an explicit alternative. |
| Mentions alone | Retain the existing literal-mention fallback. |
| No adequate evidence | Do not manufacture a relevant source or page. |

Recommendation ranking and physical navigation are related but distinct. Precise structural/manual recommendations lead; whole-source coverage and broader real ancestors follow, ordered by fit; incidental-only sources come last. Physical navigation still requires a grounded target. Whole-source relevance may use overlap: a Romans 8 work can be useful for Romans 7–9 while its label remains **Romans 8**. It does not claim to treat the whole larger selection, or every verse exhaustively.

Automatic scope requires an unambiguous scoped title/subtitle, appropriate bibliographic context and corroborating actual structure. A competing biblical section can invalidate the narrow inference. Citation frequency, filenames, publisher/series wording alone and ambiguous names such as Job/Acts are insufficient. **Edit details → More details** and Settings expose **Detect automatically**, **Set coverage**, and **No specific passage**. Manual coverage supports books without usable Contents; suppression persists across rebuild. Changing Scripture keeps the reader in place; **Go to current passage** is deliberate and explains when only whole-work relevance is known.

The PDF correction removes the general failure where oversized cover/front-matter headings displaced real book headings from the three largest font-size classes. Heading prominence is measured relative to body text, printed Contents is recovered independently, and page labels/destinations must be grounded. There is no production exception for the owner's filename, fingerprint, size, opening-page map or font family.

On the one supplied 400-page handbook, both fresh import and Rebuild recovered **75/75 printed Contents targets**: 69 validated text anchors and six honest page-only appendix/index targets. **63 real book openings cover all 66 canonical books**, because Samuel, Kings and Chronicles have shared openings. There are zero invented chapter/verse destinations, and all 1,849 existing segment IDs/text survive Rebuild. For this exact handbook, Acts 6/6:1 use the actual Acts book opening at physical page 232; Romans 8/8:1 use the Romans opening at physical page 239. These are observed destinations, not hard-coded rules for other PDFs.

Five separate invented generality regressions cover Roman/restarted decimal labels, ambiguous duplicate labels, two-column Contents, three font/body-size combinations with large covers, and a verified page with no extractable text. They supplement existing outline, Unicode/columns, cancellation/rollback and source-identity checks. This evidence does not estimate a failure probability across arbitrary PDFs. Image-only scans, broken text and structurally ambiguous prose remain explicit limitations. Older analysis needs one **Chapters and headings → Rebuild structure** after installation; reimport is unnecessary.

The older Issue #49 bottom-border symptom was accepted by the owner on 19 November and is closed. Preserve the accepted containment correction. That observation does not establish every document format, scaling setting, toolbar path or later Source change. See [owner acceptance](issue-49-original/owner-acceptance-20261119.md).

### 2.2 Completed Windows delivery and owner acceptance

[Download the normal Windows installer ZIP](https://github.com/pistio/abidingword/actions/runs/44621293307/artifacts/14577518643) · [Successful build](https://github.com/pistio/abidingword/actions/runs/44621293307) · [Complete static verification record](source-coverage-windows-build-20261119-verification.json)

| Verified item | Result |
|---|---|
| Installer ZIP | 33,681,409 bytes; SHA-256 `bbe6279764e7f3aae37947d985f8bae8d42fe67eb2238d1f4b0e29dd95656c20` |
| Setup executable | 33,690,155 bytes; SHA-256 `f31c247d8bf4904e564ba3d26c4e8033b537650cbda97da3dcbae5c704b7c250` |
| All three artifact archives | GitHub size/digest and ZIP CRC checks pass |
| Diagnostic inventory | All 15 files match their manifest |
| Configured Scripture resources | All 13 match both the manifest and exact source |
| Application | PE32+/x64, 30,038,016 bytes; source identity and all 27 registered migrations match |
| Expected packaging change | Exactly three bytes change `__TAURI_BUNDLE_TYPE_VAR_UNK` to `__TAURI_BUNDLE_TYPE_VAR_NSS`; every other application byte is identical |
| NSIS utility | Matches its pinned packaging-source SHA-256 |
| Windows execution | The Windows runner compiled and packaged; the installer and application were **not launched for native desktop acceptance** during this verification |

The ZIP contains one normal unsigned setup executable, named `Abiding-Word-Sources-Follow-up-schema27-bcf4858591f6d306838773ebc884550ca27d19b6-x64-setup.exe`. The GitHub artifact is retained until **3 December 2026**. The separate diagnostic archive is evidence, not a portable application. Static NSIS extraction reported the same `BadCmd=13` subtype metadata as the earlier verified installer; extraction succeeded and complete application/resource comparisons passed. This does not claim that the NSIS installation script was executed or that Windows trust checks passed.

**Later owner report:** the supplied handbook correction is fixed. Retain that acceptance; do not ask the next agent to rediscover the same failure. It does not establish success for every other source or close the remaining native/Teaching/provider checks. The later Settings/menu and alias changes below are not in this installer.

Compact normal-application acceptance:

1. Close Abiding Word, extract the ZIP and run its setup executable. This is the new Sources candidate, not the earlier Original-view installer.
2. Open the existing supplied Source. In **Chapters and headings**, run **Rebuild structure** once. Check that it completes and the Contents persist when reopening the source.
3. Change Scripture while reading: the source should remain at the current place. Use **Go to current passage** to navigate deliberately. On the supplied handbook, Acts 6 resolves to the broader Acts opening (physical page 232), and Romans 8 to the broader Romans opening (physical page 239), rather than pretending chapter-specific sections exist.
4. Check Reading/Original, Back/Forward and source-detail editing preserve the intended place, identity and view.
5. On a source actually devoted to a passage, inspect or set its coverage in **Edit details → More details**. Confirm a whole-work result such as **Covers Romans 8** appears for Romans 8:28 without inventing a page, while real mentions remain explicit alternatives. Test **No specific passage** if correcting an erroneous automatic scope.
6. Keep the remaining A.4 checks in section 5 open until separately observed; this short Source acceptance does not certify every Teaching, provider, Office or high-DPI path.

### 2.3 Later Settings and multilingual-hierarchy follow-up

The owner requested an explicit edit action in Settings and broader language/name variants in the book → chapter → verse hierarchy. These are implemented in [the follow-up](source-settings-aliases-20261119.md):

- **Settings → Sources → ⋯ → Edit details** opens the existing shared editor for active/archived sources, focuses Title, preserves source identity/text and refreshes language filters after saving.
- Source headings use all 101 installed language dictionaries, including German **Sprüche/Sprichwörter/Sprueche/Sprichwoerter**, **Prediger/Kohelet**, **1. Mose/Genesis**, formal Mose forms and same-book parenthetical synonyms. Ambiguities and meaningful accent distinctions remain protected.
- Source index version 6 refreshes structural links from versions 4/5 without replacing literal/manual links. Existing supplied EPUB Contents refreshes on opening after file/text identity checks.
- PDF analysis version 5 recommends an explicit **Rebuild structure** where older extraction may have omitted destinations whose printed Contents and body headings use different name variants. Stored JSON version 1 and schema 27 remain unchanged.
- The Settings/language frontend run is 1,364/1,364; the complete locked Rust suite passes 1,146 tests with zero failures and nine ignored. All 34 focused source/PDF tests and the required static/build checks pass. Exact identity and verification are in the companion record; preserve the earlier installer's separate evidence rather than rewriting it.

This follow-up requires a subsequent Windows build and native acceptance. The earlier single-build permission has already been used; no additional paid Actions, merge, tag, release or updater publication are part of this follow-up.

### 2.4 Writing and Presenter focus follow-up, 20 November

The recorded delayed-focus investigation is now reproduced and corrected. Closing information can no longer take keyboard focus back from a newly selected writing field. Closing presenter panels, closing the audience window, and leaving/reopening Presenter preserve the latest focus choice. Escape/Cancel still return to the opener when there is no newer target. Saving or cancelling a Study note also preserves a newly selected passage field or reopened draft. The fix covers the shared information control, Study note capture and both presentation components; it changes no save API, debounce, keyboard routing, database schema or recognizer code.

Controlled-frame regressions check actual Study/Teaching writing and save payloads, as well as presenter keyboard behavior and panel lifecycle. The before-fix writing examples show that intended keystrokes never reached the writing field; the information Save itself succeeded. This proves a focus defect, not a failure of the storage API, and does not retrospectively establish the cause of every historical failing assertion.

[Implementation and acceptance](focus-restoration-20261120.md) and [verification](focus-restoration-20261120-verification.json) record the exact candidate. The next Windows build is prepared as an inert recipe; the earlier single-build allowance was already used. Existing `bcf4858` installer evidence remains separate. No additional build, main merge, signing, release or updater publication is inferred.

## 3. Implemented foundation — preserve rather than rebuild

| Area | Existing capabilities and boundaries |
|---|---|
| Scripture | Installed editions, canonical passage navigation, multilingual natural references, verse/word marks, search/concordance, supported original-language annotations and exact edition provenance. Document Bible recognition has its separate unfinished acceptance programme. |
| Library | PDF, EPUB, DOCX, Markdown and text; metadata/licensing fields; tags and nested collections; content-hash duplicates; bulk import jobs; archive/Trash/restore; managed-file safety; exact citations. |
| Reader | Reading/Original modes, internal links and footnotes, Back/Forward, search, Contents, annotations, PDF structure/corrections, full-reader/pane transitions and session return positions. Saved analysis is reused until an explicit rebuild. |
| Study | Current writing methods, legacy structured definitions/answers, POZETEK, Ten-Step and word/topic/book/character research; personal reusable blocks; retained evidence with exact edition, returned verse, query and source text; independent snapshots. |
| Teaching | EVA, Sermon/Talk and Bible Lesson; blank/personal and legacy/new block templates; multiple sessions per Study; project ownership; optional stored Study link; saved provenance after unlink/deletion; participant/leader visibility. |
| Outputs | Manuscripts/guides, question sheets, worksheets/listening notes, Markdown, DOCX, print/PDF paths, presentations, PPTX/PDF slides, speaker sheet, delivery package, Presenter View and audience window. |
| Optional assistance | Off by default; provider-neutral and local-provider support; explicit context preview; wording/review help; source-excerpt assistance and selected-source comparison; checked quotations; sourced stories/images. Human authorship remains central. |
| Portability | `.awproject`, `.awbackup`, restore, pre-migration backups and integrity checks, stable/remapped IDs, exact-file reconnection and honest missing-source/edition behaviour. |
| Language | en-GB; German `du`; respectful Korean; French `vous`; Spanish `tú`; pt-BR `você`. Full recorded AI editorial review exists; native/human assessment is distinct. |

**Teaching Stage 3 is implemented**, not an outstanding coding task: [standalone/project ownership handover](teaching-stage3-standalone-handover.md), followed by [schema-26 validation](schema26-validation-handover.md). Migration 0026, optional Study relationships, preserved findings/citation snapshots, lifecycle handling and package support exist.

**However, its ordinary standalone creation UI was later superseded by owner Issue #21.** Normal use starts with a Bible Study, then optional Teaching. The home is one file/folder view of Studies and Projects. Existing standalone Teaching records and backend support remain available through a quiet legacy recovery/management route; do not delete, attach or transform them automatically.

Original-language development also advanced beyond the old backlog. [v0.22](../original-language-v0.22.md) implements OSHB-specific OSIS and MorphGNT import, selected explicit verse mapping, annotation-index refresh, Strong's-only opt-in search and provenance-based descriptions of supported Greek grammar. [v0.24 catalogue](../original-language-catalogue-v0.24.md) includes four Greek editions and user-triggered, hash-pinned OSHB/MorphGNT downloads. Do not re-list these completed foundations as wholly unbuilt.

## 4. Owner decisions that supersede earlier directions

| Topic | Controlling decision |
|---|---|
| Ordinary entry | One file/folder Bible Study home; normal Teaching creation from a Study. Legacy standalone records retained discreetly. |
| POZETEK identity | Exactly **Personen, Ort, Zeit, Ereignisse, Textzusammenhang, Erklärungen, Kern**. The erroneous Thema/Erkenntnisse/Konsequenzen replacement must not return. |
| Application | Separate optional reflection after the seven parts; not an eighth POZETEK letter. |
| Method names | POZETEK name/acronym only in German. Established descriptive names elsewhere; English **People**, stable internal `persons` IDs. |
| Hints | P/O/Z/E/T/E concise and source-faithful. Kern retains three compact sequential explanations: Textgegenstand, Textaussage, Textthema, above one writing field. |
| Ten-Step | Owner explicitly accepted its current method/content. Do not restart substantive editing. Flag genuine protected linguistic questions separately. |
| Writing interface | Writing first; moderate manually resizable fields; meaningful dismissible hints for empty parts; simple Text/Q&A additions; quiet discoverable controls; named modal deletion even for an empty section. |
| Thinking checks | Keep the three concise questions about following from the text, overlooked matters and assumptions; no explanatory lecture. |
| Teaching transfer | Current snapshot-transfer implementation preserved. Further new integration paused until the simpler Teaching workflow is accepted. |
| Source matching | Honour owner-confirmed links and structural corrections. Automatic navigation follows a reliable Contents/heading branch to the deepest real ancestor containing the whole requested passage; a narrower subsection must not substitute for an entire chapter. Without a usable structural destination, literal mentions remain explicit choices. Across recommended sources, precise structural/manual matches lead; whole-source coverage and broader usable ancestors follow by fit; literal-only sources follow those. |
| Broad source relevance | A whole work devoted to a passage/chapter/book can be offered as a whole-source result, honestly distinguished from an exact passage location. Whole-source relevance does not invent page precision. |
| Data protection | Preserve source bytes, saved writing, annotations, citations, historical migration bytes, independent Study/Teaching models and unavailable-edition honesty. |

The source for compact hints is the owner's supplied method/workshop material and verified issue excerpts. Do not claim access to private original files unless actually inspected. Relevant records: [Issue #33](https://github.com/pistio/abidingword/issues/33), its later comments, [8 November follow-up](owner-followup-35-33.md) and [19 November wording follow-up](owner-followup-20261119.md).

## 5. Current issue and acceptance ledger

The read-only GitHub check found **15 open issues**. An open issue can contain a completed implementation or a later accepted subset; its oldest text is not today's status.

| Issue | Current state and remaining work |
|---|---|
| [#19](https://github.com/pistio/abidingword/issues/19) DeepSeek/Korean | Field-target root cause fixed in A.4 and mocked regressions exist. Actual owner-configured DeepSeek correction on Windows remains to check, including preview, explicit apply, stale targets and unchanged original writing. |
| [#21](https://github.com/pistio/abidingword/issues/21) Study home | Implemented and owner-accepted in principle, recorded by #28. Preserve the file/folder home and legacy compatibility; reconcile stale issue status rather than rebuilding it. |
| [#22](https://github.com/pistio/abidingword/issues/22) PDF hierarchy | Implemented hierarchy/corrections plus today's actual-file/global fixes. Owner reports the supplied handbook is fixed; other formats/sources and the later multilingual follow-up still need their native acceptance. |
| [#25](https://github.com/pistio/abidingword/issues/25) Writing first | Both editors implemented and subsequently corrected. Complete owner acceptance across empty/populated methods, menus, autosave, focus, saved drafts, privacy and scaling remains. |
| [#26](https://github.com/pistio/abidingword/issues/26) Compact toolbar | Shared toolbar and duplicate-PDF-row correction exist. Confirm one visible row, useful overflow, pins, page controls and accessibility at actual native widths/scales. #49 closure is not blanket #26 acceptance. |
| [#27](https://github.com/pistio/abidingword/issues/27) Q&A/exports/transfer | Implemented; genuine atomic transfer correction also exists. Owner acceptance of question-only/answer/full outputs, visibility and editable provenance-bearing snapshots remains. Respect pause on further new integration. |
| [#28](https://github.com/pistio/abidingword/issues/28) A.4 programme | Master owner-acceptance obligation: actual PDF, toolbar, both editors, live DeepSeek, Q&A/outputs/transfer and data preservation. A.3 was only partially accepted. |
| [#29](https://github.com/pistio/abidingword/issues/29) Recognizer failures/safety | Separate recognition evidence and provisional-standing programme. Never treat source-retrieval success as Scripture-text certification. |
| [#31](https://github.com/pistio/abidingword/issues/31) Transfer atomicity | Implemented in `35526df…` / PR #32 and preserved in later lineage. Keep transactional failure regressions; feature acceptance remains under #27. |
| [#33](https://github.com/pistio/abidingword/issues/33) Methods/controls | Canonical seven parts, repair/Undo, duplicate preservation, resizing, simple additions and modal deletion implemented. Owner confirmed correct structure/resizing, then requested shorter hints, now implemented. Compact guidance/Kern/source fidelity still needs final owner review. Ten-Step already accepted. |
| [#34](https://github.com/pistio/abidingword/issues/34) Consolidated corrections | Partly superseded by #49 acceptance and PR #54. Keep actual Greek NT-as-Source, remaining high-DPI/toolbar and simpler Teaching acceptance explicit. |
| [#35](https://github.com/pistio/abidingword/issues/35) Real Source processing | Original file available and correction measured; owner now reports the supplied handbook works. Preserve that accepted subset while keeping other sources and remaining detailed native checks open. Earlier null-query diagnostics are historical. |
| [#36](https://github.com/pistio/abidingword/issues/36) Recognition architecture | Claude-owned structural-first research, exact evidence spans and independent acceptance. Coordinate interfaces; do not merge its code into the Source fix casually. |
| [#38](https://github.com/pistio/abidingword/issues/38) NSIS packaging | Verified preflight correction exists and later installer delivery overcame the earlier acquisition failure. Use the pinned current build path, not the obsolete failed run/source. |
| [#42](https://github.com/pistio/abidingword/issues/42) Languages | Complete individual AI editorial pass exists. Remaining owner/native/Office/accessibility checks and four explicitly recorded questions are below. |

Supporting implementation records: [A.4 handover](milestone-a4-handover.md), [A.4 ledger](milestone-a4/acceptance-ledger.md), [later correction ledger](windows-owner-corrections/acceptance-ledger.md), [Library Stage 3](library-stage3-handover.md), [combined integration](windows-library-teaching-study-integration.md). Historical owner gates are superseded only by specific later evidence.

## 6. Scripture recognition — separate coordination lane

The research line reviewed in the **19 November** takeover was **A1.5 / draft PR #53**, not the old October v0.24.3 state. The subsequent owner report of an active withheld Collection H evaluation supersedes the historical not-yet-run status below; no new H outcome is certified here. GitHub establishes the last committed results; it does not establish whether a separate Claude process is currently executing. The user's report of active Claude work remains a coordination constraint. This takeover did not run, stop, restart or alter that process.

The owner repeated that Claude is working while requesting the Settings/name follow-up. The freeze snapshot below is the last reviewed evidence, not a claim that no later work exists. Review Claude's next report before marking its outstanding corrections or gates complete.

| Reference | Verified value |
|---|---|
| Research branch | `claude/recognition-a1.5-evaluation-qualification` |
| Frozen recognizer code | `08c6faac8c92c6c791ec1e7bbbe997362fa1059f` |
| Code tree | `be296f2570dcbcc251789b2b1e1bbe2c9e7c2798` |
| Latest reviewed documentation/tooling head | `4dd9f8e12be1c531f55ba5121423a184a6ae97cb`, reviewed 19 November; later Claude work requires its own report |
| Frozen harness SHA-256 | `920f7e28e6c03f0c2fd7832bb78ce7c2077d6767541f0b86bb02f2fec1ed3a8e` |
| Pipeline | `AW_PIPELINE=structural`, Rust/Cargo 1.97.0, Linux x86-64 |
| Application integration | Not completed; this is research evidence and an engineering handover |
| Independent Set H | Historical 19 November review: not created, selected, downloaded, inspected, sealed or run by that review. Current owner report: Claude is running the withheld evaluation; this recovery milestone has no access to its documents or results. |

Primary references: [PR #53](https://github.com/pistio/abidingword/pull/53), [A1.5 freeze](https://github.com/pistio/abidingword/blob/4dd9f8e12be1c531f55ba5121423a184a6ae97cb/docs/audit/candidate-a1.5/FREEZE.md), [Codex integration handover](https://github.com/pistio/abidingword/blob/4dd9f8e12be1c531f55ba5121423a184a6ae97cb/docs/handover-scripture-recognition-codex.md), [Set H decision package](https://github.com/pistio/abidingword/blob/4dd9f8e12be1c531f55ba5121423a184a6ae97cb/docs/audit/candidate-a1.5/set-h-decision-package.md).

### 6.1 Completed reported work

The offline, non-AI structural pipeline reads text-layer PDF, EPUB, DOCX, Markdown and text. It extracts the supplied text; it does not compose, translate, complete or repair Scripture. Improvements include structural note-link roles, ordered original spans and excluded apparatus ranges, source/entry hashes and offsets, conservative refusals, memory work, hardening, per-book printed-numbering evidence and the general raised-PDF-run rule that separates a note letter from an adjacent verse number.

All **113 disclosed editions** in the development corpus and Sets B–G have been rerun. Old statements that Set B is pending or Set C untested are superseded.

| Corpus outcome | A1.5 reported count |
|---|---:|
| Verified, correct text | 1,980,257 |
| Verified, wrong text against the reference | 9 |
| Verified reference absent from the edition | 0 |
| Correct text conservatively refused | 45,376 |
| Wrong text caught instead of verified | 17,518 |

These are corpus observations across editions, not distinct Bible verses. The nine retained mismatches belong to the original generated Spanish DOCX v1, whose generator omitted continuation paragraphs. They remain in the original score. The separately versioned corrected v2 has 31,072 verified, zero wrong and zero nonexistent references. That is a **fixture correction**, not a claimed recognizer improvement from nine to zero. Source fidelity cannot reveal words already absent from the input document or authenticate an edition.

The committed freeze reports 1,101 Rust tests and 26 corpus-tool tests passing, plus 1,053 frontend tests on that research lineage. These are different counts and a different branch from the Windows Sources candidate. All 24 EPUB and 56 PDF memory comparisons are complete: median/highest reading peaks are 119/249 MB for EPUB and 185/566 MB for PDF. Measured PDF memory does not mean the parser is safely bounded in-process.

### 6.2 Remaining recognition completion gates

1. **Independent evidence.** At the 19 November review, Set H was a protocol rather than a completed test. The current owner reports Claude is running that withheld evaluation; require its separately approved final report before claiming completion. The latest reviewed allocation is 24 documents: 10 EPUB, nine PDF and five DOCX; 19 scored, three controls and two mixed. Its declared authentic minimum is eight completed authentic scored documents across two formats and four languages, with 20,000 reference/printed verses. These are not 20,000 correctly verified verses. Below the declared minimum, the result must carry the protocol's qualification limit; coverage has no pass mark and must remain visible. Unmatched references and unavailable language slots are declared exploratory or omitted before sealing and recorded as deviations. Builder, reference reviewer, runner and assessor have distinct roles/access records. A frozen single run uses 30 minutes and 4 GB address space per document. Scored accuracy, control results, mixed allowed-reference compliance and execution outcomes are separate; mixed/exploratory documents do not enter the accuracy claim. No independent-set approval or success is invented. References must match the exact edition, revision, canon and versification. A document/reference divergence found after the run stays in the raw score with its explanation. If H fails, H becomes disclosed and a corrected candidate needs a new withheld set. The derived-document completeness auditor covers USFM→DOCX only. The remaining scoring evidence, path and isolation corrections below must be resolved before execution is approved.
2. **Provisional import storage and UI.** Migration **0028 is reserved for Codex and remains a proposal**, not something Claude implemented. Store document/entry hashes, separate recognition status and person-confirmed edition standing, original spans/exclusions or existing origin places, per-book numbering evidence and engine version. Extend existing `bible_verse_places`; preserve schema-27 Source structure and historical migrations. Say **Recognised from your document**. Do not populate or display unverified candidate text as a Scripture verse, even with a provisional label; show its original-document location and reason instead. Recognition remains provisional until edition confirmation, which itself does not certify missing text.
3. **Downstream consumers.** Reading, comparison, search/concordance, Study citations, Teaching, AI context and exports must all obey the standing and numbering rules. A cosmetic import warning alone is insufficient.
4. **Resource control and responsiveness.** Run PDF reading in a separate capped process: Linux address-space limits and a Windows Job Object, clean process termination and refused-document handling. Recognition's current entry points are synchronous; progress/cancellation hooks remain to build. The existing Source-structure rebuild progress/cancel feature is a different path and does not complete this requirement.
5. **Numbering/canon safety.** Preserve printed numbers and use validated agreement/mappings before cross-edition alignment. `withinReference` is not proof of English-numbering agreement. Never join traditions merely by identical `(book, chapter, verse)` values or misfile additional books.
6. **Unresolved extraction limits.** Scanned PDFs require future OCR; Hebrew RTL layouts and Hebrew-letter numerals remain limited/refused. Hidden/off-page PDF text and CSS-hidden EPUB content may still be read. Missing source text, unsupported language families and unfamiliar publishers/layouts remain limitations. Keep refusals honest rather than inventing text.
7. **Integration and Windows acceptance.** Review an exact combined candidate with these guarantees and real Windows evidence. Passing disclosed corpus scores does not authorise a wholesale research-branch merge, signing, release or automatic update.

### Follow-up from the 19 November review of `4dd9f8e`

Claude's update resolved most of the earlier class/execution/composition concerns. The independent review reran all 29 evaluation tests and 26 corpus-tool tests successfully, confirmed all 279 frozen source hashes and ten pinned tool/document hashes, and left the frozen recognizer unchanged. These counts belong to that research branch, not the application branch.

Three bounded corrections remain with Claude:

- **Bind scoring to committed evidence.** Synthetic fake-harness probes changed fail to pass by changing a mixed allowed-reference list, or by changing an external wrong-verse report / process-exit record and regenerating only the mutable external checksum list. For the latter cases, the committed run record and tracked worktree remained unchanged. Bind the complete seal (including allowed lists) to the recorded seal commit, and RUN.txt / OUTPUTS.sha256 / CHECK.json to an explicit committed run-record identity. Refuse missing or inconsistent anchors and verify outputs against the committed hashes. This does not allege alteration of Claude's rehearsal results.
- **Contain declared input paths.** Preflight accepted parent-relative escapes, absolute external paths and escaping symlinks in synthetic cases. Validate relative and resolved containment before opening/hashing documents, references and seal entries. Hash checking still applied in the reproduced cases; do not describe this as a demonstrated hash-inventory bypass.
- **Clarify independence.** The application repository is public; withholding credentials does not make its source inaccessible. Use fresh contexts without inherited project/chat material or a project checkout, with enforced approved-source network restrictions where available. Otherwise call the separation procedural and retain explicit rules/access logs. A private Set H repository protects that set, not the public recognizer source.

No actual Set H files were selected, downloaded, inspected, built, sealed or run in that review. The proposed next step is targeted tooling/regression corrections and updated pins, not a repeat of the entire disclosed corpus or timing comparison. Owner decisions on the concrete independent arrangement, source/licence scope and actual set still precede the separately authorised sealing/single run.

The application-integration concern is accurately documented in handover requirement 4 but remains unfinished: `apply_saved` / correction can re-enter recognition and lose the structural gate. Reapply structural verification/refusal rules and regenerate provenance/numbering after every recognition rerun. Migration 0028, consumer safety, process isolation, progress/cancellation and native acceptance remain separate Codex integration obligations. Do not modify the frozen recognizer in this Source/focus branch or mark the integration finished because the handover records it.

### 6.3 PDFium is a separate experiment

The older A2.1/PDFium branch `claude/recognition-a2.1-pdfium-glyphs` is at `afa4e8a699cf3a5137a333434b618aab4c8bdb09` (9 November), [PR #46](https://github.com/pistio/abidingword/pull/46). It improves some disclosed extraction cases but includes regressions, unresolved word-spacing behaviour and an unexecuted Windows helper. Memory caps, native Windows execution, packaging and security/provenance work remain. A separate process, watchdog and Job Object memory cap do not provide a privilege sandbox; restricted-token/AppContainer or equivalent isolation is a distinct gate. Security-fix coverage must be traced to the exact pinned binary and source, not inferred from a newer version number. It is not part of A1.5 or this Sources installer and is not an approved default replacement.

The later document-role clustering/global reconciliation design and optional AI labels/span pointers remain evidence-led future work. No model-generated verse text is permitted. Ordinary Source scope recommendations and extraction of Bible-edition verse text have distinct evidence contracts; success in either does not certify the other.

## 7. Language review: completed work and explicit remaining questions

[Full review record](../i18n/issue-42/README.md): at that review's recorded snapshot, all **25,396 locale leaves**, **4,045 logical English strings**, 564 existing-leaf edits and 180 additions, individual retain/edit/protected/question decisions, separate consistency reviews and 96 selected-context screenshots. This supersedes the earlier 269-key first tranche. It is an AI editorial review, not qualified human-native proofreading or Windows acceptance.

Four named questions remain:

1. Protected French Ten-Step syntax: “Que précède et suit ce passage ?” has a proposed grammatical correction, “Qu’est-ce qui précède et suit ce passage ?”; it was retained pending a narrow decision because accepted method text is protected.
2. The sample “Abiding is not a technique but a friendship.” attributed to “A pastor” needs its invented-versus-sourced provenance clarified.
3. Local-provider installation guidance should be understood against the default loopback setup; an advanced remote Address changes where content goes. Actual provider consent checks the address; the explanatory wording was flagged for a contextual decision.
4. Complete private EVA/YCC/Campus Connect/method sources were unavailable to reviewers. Existing owner-approved text and verified excerpts were retained. No invented methodological expansion or claimed primary-source access.

OS-owned dialogs, screen readers, real Office output, live provider response and native scaling remain distinct assessment contexts. Later feature wording must preserve the established register, method identity, uncertainty, privacy and interpolated/plural semantics.

## 8. Complete known future roadmap

These are retained product directions from the concept, architecture and backlog. They are not all immediate release blockers, scheduled commitments or newly authorised implementation tasks. Several older “open” bullets were completed later; the current state above takes precedence.

### 8.1 Scripture, references and advanced language research

- Finish independent document recognition and provisional safety, then measure accepted text, false references, wrong text, abstention and coverage separately on the frozen candidate.
- Broaden natural reference phrasing and Biblical Greek/Hebrew names beyond current packs; validate running-text references against appropriate edition/versification data.
- Support LXX/Vulgate/deuterocanonical and tradition-dependent numbering/identity without mislabelling text. Current catalogue excludes unsupported numbering, even when text is legally distributable.
- Add justified importers/converters: wider publisher OSIS, USX, Zefania/MySword/e-Sword and TEI/Swete, with format-specific evidence and tests.
- Extend licensed lexicons/glosses, interlinear/cross-edition alignment, deeper morphology/syntax, translation patterns, semantic range/cognates and Septuagint relationships.
- Add actual cross-reference/relationship datasets where justified. Current manually supplied cross-references and literal concept/person searches must remain honestly labelled.
- Retain exact distinctions: OSHB identifiers are not invented lemmas; Strong's-only is not morphology; unsupported codes remain raw. Word-index refresh and supported Greek grammar descriptions are already built.

### 8.2 Personal sources, readers and annotations

- OCR of scanned books, handwriting and photos, with quality/review evidence; transcription of audio/video/podcasts/lectures with timestamped provenance.
- Broader adapters for webpages, PowerPoint, exported notes and other owned material, preserving original files and document locations.
- Better references in RTL/vertical PDFs and across formatting boundaries; actual-format parity across PDF/EPUB/DOCX.
- Direct annotation creation in formatted EPUB/DOCX Original, note-here/bookmark parity and selections across PDF pages.
- Pen/stylus ink, separately stored from canonical and quote-linked marks; pressure data only if useful.
- Reconnect marks after reimport of the same file; open original marks from Study findings; explicitly offer cited Scripture as related passages.
- Citation styles/bibliographies and justified publication metadata; never silently reconnect to a different edition/file.
- Continue global structural and whole-work coverage testing without fabricated headings, exact-page claims or semantic commentary assertions. A diverse independently chosen real-source benchmark should report correct destinations, wrong destinations and unresolved targets separately; the five synthetic variants and the one actual handbook do not establish an arbitrary-PDF failure rate.
- Cross-restart reading-position persistence remains beyond Library Stage 3's session-only work; large-library organisation/performance requires actual usage evidence.

### 8.3 Studies, methods and lifelong knowledge

- Derive reusable methods from the person's own material through human-reviewed suggestions; keep originals distinct and unchanged.
- Share/import/export Study and Teaching structures through proposed **`.awstructure`**, not presentation-template `.awtemplate`.
- Improve long-study insertion/navigation and moving notes between parts where real usage warrants it.
- Richer persistent study history, earlier-version recovery/comparison, unresolved-question follow-up and reuse across subsequent teaching.
- Structured hermeneutical/confessional/source-trust profile beyond today's prose perspective setting; visible/editable and separate from deterministic retrieval.
- Explicit source-disagreement comparison and durable research memory, preserving who argues what and the user's own conclusion.
- Maintain independent template snapshots, legacy answers and separate Study/Teaching storage; template edits never rewrite authored work.

### 8.4 Optional assistance and research

- Whole-source/chapter help beyond selected words; genuine library-only research across sources; clearly differentiated library-plus-model and wider-scholarship modes.
- Wording help for notes, durable accepted-assistance history and persistent comparison context.
- Trust/priority controls, source-grounded comparison and transparent differences in interpretation.
- Provider-native search integration where warranted; persistent Linux credential storage if still outstanding.
- Additional image providers, including Google's direct image API, with independent setup and provenance.
- Preserve bounded preview/consent, local options, genuine source quotes, explicit acceptance and human authorship. No prominent whole-sermon or whole-study generation workflow.

### 8.5 Teaching, materials and presentation

- Complete Windows/Office/projector/second-display accessibility, privacy and output-fidelity acceptance of what already exists.
- Audio lesson steps and step-level media attachments; remember material choices; preserve paragraphs and Psalm titles where supplied by the edition.
- Core-generated printable PDF independent of the system print dialog, relevant to macOS; bounded material assistance and additional justified prayer/announcement sheets.
- Delivery-package import on another desktop; mobile delivery reader and paired phone presenter-remote transport.
- Remaining format fidelity, including WebP PowerPoint export and OpenType/CFF PDF fonts; presentation slides in Study search.
- Presentation-template extensions recorded for later: ODP, direct Drive import, gradients/shadows/nonrectangular shapes and per-slide layouts.
- Detachable Scripture/Study/Sources windows sharing one workspace, with monitor recovery and state synchronisation. The audience window is a foundation, not completion of this feature.

### 8.6 Portability, sync and integration

- Choose licence-appropriate library files carried in project packages; handle offline images/templates; partial restore of one Project and richer recovery history.
- Optional own-device sync through WebDAV/Nextcloud, local/Syncthing folder or S3-compatible storage using change journals, revisions and explicit conflicts. Never synchronise the live SQLite file directly.
- Local REST/MCP, domain events/webhooks, stable deep links and scoped integration permissions; remote LAN/Tailscale exposure only by explicit opt-in.
- **Abiding Word owns biblical domain data; Zekor indexes selected material; Bevro orchestrates.** Neither external system silently becomes canonical storage.
- Wider native platform packaging/acceptance and eventual tablet/mobile experience, without making a hosted service or self-hosted server compulsory.

### 8.7 Content, publication and product upkeep

- Multilingual theological/library catalogue, edition-specific rights/attribution and optional downloads; future content packs/shop planning without bundling it into current Windows work.
- General-user methods/confessional configuration and optional packs, keeping the owner's personal preferences configurable.
- Publication-grade document/web/handout/course/commentary workflows distinguishing authored text, quotations, Bible text, source summaries and AI assistance.
- Refresh user guide/screenshots and reconcile stale milestone/backlog statements against implementation and acceptance evidence.
- Later read-only sharing, co-teaching/comments/collaboration only when requested and with explicit permissions/privacy.

## 9. Deliberately deferred or conditional scope

The architecture defers beyond early MVP: complex syntax trees/large lexical datasets, handwritten OCR/full transcription, collaborative/CRDT editing, marketplaces, public publishing platform, mobile-first editor, complex citation engine and a large plugin ecosystem. Design should leave room, but these are not automatic present release blockers.

Presentation deliberately excludes unless real use creates a requirement: audience Q&A, live captions, recording/narration, AI rehearsal coach, complex animations/transitions, kiosk automation, free-form slide/master editor, collaborative live editing, arbitrary shortcut editor and extra drawing tools. Do not convert these exclusions into an unrequested feature programme.

Optional local encryption and broad commercial/content-market features remain future design possibilities, not additions authorised by this handover. Existing decisions on privacy and source ownership still apply.

## 10. Actionable continuation order

1. **Complete the recovery-and-preservation milestone at the actual latest recovery branch head.** Finish the common save barrier, daily/full restore UI and native safety checks, integrated Study/Teaching/privacy/portability evidence, regenerated language corpus and complete permitted checks. Record the exact final commit and every failure, not just passing assertions.
2. **Stop at owner review of the draft PR stacked on #55.** Keep previous Source/focus and method acceptances. No main merge, Windows build, signing, installer, release, update publication, NiPogi access, real-user-data operation or paid API call follows from this milestone.
3. **After separate delivery approval, accept the exact combined candidate on disposable Windows data.** Use the [current worksheet](windows-owner-acceptance-20261120.md) for Study→Teaching→outputs, both recovery types, safety-copy retention, source-file integrity, stale-writer prevention, participant privacy, native scaling and accessibility. Do not reuse an old Source binary as evidence for new recovery code.
4. **Coordinate the independent recognition lane without entering it.** The owner reports Claude is running withheld Collection H. Await its authorised report; do not inspect the documents, change the frozen candidate, treat disclosed scores as independent acceptance or use reserved migration 0028 here. Subsequent provisional-standing/consumer/process-isolation integration requires its own reviewed milestone.
5. **Resolve rights, distribution and remaining language/source decisions.** Use the [publication matrix](publish-readiness-20261120.md). Preserve resource hashes and notices; do not choose a software licence or signing identity on the owner's behalf. Keep live DeepSeek, Office and human/native language acceptance distinct from mocked/AI review.
6. **Select the next product milestone from demonstrated remaining gaps.** Retain wider research, source formats, knowledge continuity, output/mobile delivery and optional integrations without rebuilding already implemented Teaching or making deferred work an indefinite obstacle to a useful initial release.

No blanket product completion, recognizer readiness, main merge, updater publication or native acceptance is established by this document. The known roadmap is consolidated from inspected records; unseen private source guidance and future owner decisions remain explicit gaps rather than invented requirements.
