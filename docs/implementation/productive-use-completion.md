# Abiding Word — productive-use completion record

**27 November 2026. Status: implementation combined and local checks passed; source CI and native Windows owner acceptance are separate gates.** This record describes the candidate for personal use. It does not claim an accepted Windows release.

## Goal and completion boundary

The goal remains **Word → Understanding → Teaching**: study Scripture carefully, retain trustworthy notes and source citations, and prepare faithful teaching and usable materials. Abiding Word remains a local-first, source-owned Tauri/React/Rust/SQLite desktop workspace, useful offline without a hosted service. Windows is the primary acceptance platform. Ordinary work starts with a Study and proceeds to optional Teaching; existing standalone Teaching and earlier work remain preserved. The person supplies interpretation and writing; optional assistance does not replace that authorship. The [continuity record](continuity-20261119.md) retains the settled methods, languages and product decisions.

The immediate finish line is one dependable Windows session from importing a Bible through Study, Teaching and participant output, with restart and recovery demonstrated. This record adds no further feature programme.

## Candidate identity

| Item | Recorded identity |
|---|---|
| Working branch | `completion/windows-productive-use-20261126` |
| Application base | Main `14c64267e5beb9e399a7c8684934548681df6c5e` |
| Bible metadata work | [PR #67](https://github.com/pistio/abidingword/pull/67), head `1a498b8c394ad83d170be829ff645357b2b111e0` |
| Late Claude work | `b12bc9f13aa7fbebb41694de847cba495957fd4f` |
| Combined implementation | `fd2b1bec285b5122362c3cf95fc42c1ae6d3a486` |
| Installer source pin | Set by the prepared workflow after the product identity is committed; includes all application changes and native test/launcher scripts |
| Source review and CI | Candidate pull request for `completion/windows-productive-use-20261126`; its Checks tab records the exact tested head |

The product recognition run identity advances with this implementation. Frozen recogniser `08c6faac8c92c6c791ec1e7bbbe997362fa1059f`, E3 and Collection H remain untouched. Historical runs must retain their original identities and results. **Collection H cannot attest a byte-identical current product after the new guard.** Product verification and the independent evaluation remain separate, as required by the [recognition completion milestone](scripture-recognition-completion-20261125.md).

The following metadata-only commit updates the workflow pin and this record.
Its application sources are identical to the installer pin. The workflow
remains inactive until an explicitly approved push to
`build/aw9-windows-acceptance-20261127` with the approval commit message;
neither this candidate branch nor a main merge starts Windows packaging.

## What this candidate completes

### Source-faithful Scripture and existing editions

The independent USFM reference comparison passes for **all 13 bundled editions and 311,761 verse texts**. Relative to previous parsing, **553 verse texts are corrected: 528 historical note-spacing cases, 21 BSB acrostic-heading cases and four non-breaking-space cases**. This establishes fidelity to those source files, not universal document recognition or edition authenticity. The [independent reader](../../src-tauri/tests/support/usfm_fidelity_reference.rs) and [fidelity tests](../../src-tauri/tests/usfm_fidelity.rs) are distinct from the production parser.

Sixteen Korean Psalm titles contain 36 control characters: 18 U+0001 and 18 U+0003. Only those titles are quarantined, with a visible warning; verse text and original files are preserved. Malformed USFM markup blocks import. The document-recognition guard conservatively withholds the merged Acts 28:28/30/31 candidate and never guesses verse 29.

Existing installed Bibles are not silently rewritten. **Settings → Bible texts → Check saved text…** compares kept USFM files, reports changes and offers an explicit separate copy with a new translation ID. Matching text offers no creation action; invalid previews cannot commit. Earlier editions, source identity, notes and citations remain available. See [copy preservation tests](../../src-tauri/tests/bible_text_copy.rs) and [interface tests](../../tests/ui/bible-text-copy.test.tsx).

Import requirements now identify missing metadata, Language has required-field semantics, and Bible details are editable. Edition confirmation remains optional and cannot verify uncertain text. Earlier document Bibles visibly offer rechecking from their kept sources. Footnote-caller labels and recovery explanations are present in all six interface languages. Claude's late work retains the end-to-end recognition journey and Presenter warning for incomplete Scripture; withheld wording stays outside audience and participant output. See the [journey test](../../src-tauri/tests/recognition_journey.rs).

### Recoverable full-backup restore

Issue [#57](https://github.com/pistio/abidingword/issues/57) now has a sealed, durable restore journal, recovered **before SQLite opens**. Interrupted operations preserve recovery material; incomplete or damaged recovery gates database access and gives an actionable explanation. User originals are not rewritten. The [journal implementation](../../src-tauri/src/portability/restore_journal.rs) and [failure-boundary tests](../../src-tauri/src/portability/restore_journal_tests.rs) record this behavior.

## Verification receipts

| Check | Current result |
|---|---|
| Typecheck, frontend lint, localisation, release resources | **PASS** |
| Complete frontend suite | **PASS: 1,467 tests in 138 files** |
| Independent USFM comparison | **PASS: 13 editions / 311,761 verse texts** |
| Focused native suites | **PASS:** import 3; corrected copy 4; daily recovery 10; portability 19; recognition journey 3; USFM fidelity 7 |
| Durable restore journal | **PASS: 13 tests**, including interruption during a partial undo copy; one subprocess fixture is ignored outside its parent tests |
| Production frontend build | **PASS** |
| Complete native suite / Clippy | **PASS: 1,229 passed / 0 failed / 10 ignored**; Clippy clean with warnings denied |
| Actual-crate bundled word check, 13 editions | **PASS**, including source text and word offsets (15 tests; one existing measurement ignored) |
| Four Windows-native PDF cases | Actual Windows module and test bodies cross-type-checked; Windows execution **NOT RUN** |
| Source CI | See the candidate pull request Checks tab for the exact head and current CI result |
| Installed Windows acceptance | **NOT RUN** on this new candidate |

These are separate receipts. Linux tests and browser interface tests do not establish Windows process limits, installed WebView behavior or Microsoft Word output.

## Remaining path to personal-use acceptance

The previously installed `056333b` candidate is historical. Its build approval was consumed by that run; **no new Windows build has started**. The new launcher and future-build native checks are prepared. [ScriptureAcceptance.md](../../scripts/windows/ScriptureAcceptance.md) specifies executable hashes, an isolated disposable profile and the observations to retain.

1. Finish the source review and CI, obtain approval for the concrete combined candidate and one new Windows build, then retain installer/executable hashes. The earlier consolidation approval was completed by PR #63; it does not authorize every future merge.
2. Run the four native PDF cases and the installed import exercise: EPUB, DOCX and text-layer PDF; progress/cancellation; image-only refusal; heading corrections; provisional edition status; and continued withholding of uncertain verses. Check the separate-copy workflow preserves the earlier Bible.
3. Reopen through the hash-checking launcher. Verify saved editions, original locations and status, then complete Study → Teaching → Materials using the invented project. Inspect Presenter warnings and actual Word/Markdown participant outputs for verified Scripture and absence of private leader material.
4. Complete the disposable recovery/reopen observations and retain source hashes, smoke receipts and screenshots. Record failures or skipped observations explicitly. **Windows acceptance PASS is entered only after the owner observes this coherent workflow on the pinned candidate.** Then promote that passed source to the normal Windows application with a durable working profile; the temporary acceptance profile is not the destination for everyday Study work.

Signing, updater deployment and public publication are outside this record. Their separate decisions remain in [publish readiness](publish-readiness-20261120.md); they do not create another open-ended development roadmap before useful personal work.
