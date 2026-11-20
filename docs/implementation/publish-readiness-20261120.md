# Abiding Word — publishable product completion programme

**20 November 2026 · owner-facing release contract · Codex/GitHub development.**

The deliverable is a **usable, local-first, multilingual Bible-study application**, not a perpetual sequence of PDF or cosmetic fixes. The scope is **Scripture → Understanding → Study → Teaching → reproducible outputs**, with persistent personal writing, original sources, exact citations, reliable backups and optional bounded AI help. Theology is the user's explicit, editable interpretive framework, not a hidden modification to retrieval. Biblical and source quotations must never be generated or misattributed.

This document is the consolidated **release go/no-go contract**. Existing detailed engineering handover: [continuity](continuity-20261119.md); actual Windows [18-check acceptance](windows-owner-acceptance-20261120.md); consolidated original [A.4 acceptance issue](https://github.com/pistio/abidingword/issues/28). These supersede outdated October and early-November claims that already-implemented functionality is absent. Conversely, code being implemented and automated tests being green are never themselves native user acceptance.

## Release decisions and boundaries

- **No unapproved public publication, auto-update push, `main` merge, signing, data migration or NiPogi deployment.** A public GitHub source repository is not a user-approved public licence or a tested binary release.
- Windows is the primary acceptance platform. Build once from a pinned SHA after review; use a separate disposable test profile for destructive tests. Existing Windows installer `13b65b7` is an **acceptance candidate**, not a signed production release.
- Human Study/Teaching writing is authoritative. Optional AI must not silently change it. Translation and source licensing must be respected. The first public release need not include future OCR, mobile editing, sync, cloud services or a marketplace.
- Do not turn Claude's unpublished A1.5 recognition experiment into a production Bible: independent withheld Set H remains unfinished and migration 0028 is reserved for safety integration. A recognizer which cannot establish reliable verse text must conservatively refuse and preserve original-document access.
- Preserve `schema 27` migrations, original imported source bytes, annotations, Study/Teaching relations, quotes/citations, and old projects/exports. Retention of private material is distinct from permission to distribute it.

## P0 — must pass before a normal public release

| Gate | Current verified state | What constitutes completion |
|---|---|---|
| **P0.1 Durable authorship** | Structured Study/Teaching writing, autosave, saved findings and citations already coded. New durable source-reading/position/history recovery is on separate `feature/durable-study-workspace-20261120`, **under CI, not accepted**. | End-to-end newly authored material survives save, restart, crash/recovery, profile upgrade, library changes and resuming sources. Failures must be visible; no silent overwriting of newer text. |
| **P0.2 Recoverability** | Verified whole `.awbackup` and `.awproject` mechanisms exist. Daily **database-only**, verified rolling snapshots now have an isolated restore path and Settings preview (draft PR #56, unaccepted on native Windows). Sources/images are excluded; damaged or different-schema copies are disabled, and a verified pre-restore database safety copy is mandatory. Full `.awbackup` still preserves original files. | Disposable profile: backup→modify→restore restores original source identity/files, exact study blocks, citations and private/participant material; corrupt backup refused without any change, and safe rollback tested. Full backups remain user accessible. |
| **P0.3 Complete Study→Teaching workflow** | Simplified file/folder home, writing-first Studies and Teaching, POZETEK seven real German parts, accepted Ten-Step, Q&A blocks, snapshot transfer and teaching methods are implemented. | Windows owner can start from Scripture, form a Study, keep/find sources, transfer selected evidence explicitly to Teaching, write lesson/sermon/group content and later edit it. Multiple teachings per Study and legacy records remain correct. |
| **P0.4 Provenance and confidentiality of outputs** | Markdown, Word, PDF, PPTX, presentation and question/leader/participant exports coded. Privacy tests reported; native Office and participant-output acceptance open. | Verify exact citation/translation, no invented pages, incomplete sources honestly labelled, *no private leader notes in participant materials*; original authorship and images/rights unchanged. Real installed Microsoft Office/PDF reader test. |
| **P0.5 Native reliability and accessibility** | Latest Windows schema-27 acceptance installer succeeded, owner confirmed **Edit details** and earlier handbook navigation. Browser tests green at prior pinned tree; other native matrix checks open. | Complete [Windows acceptance matrix](windows-owner-acceptance-20261120.md): 100/125/150/200% scaling, toolbar/screen reader keyboard, Reading/Original, non-handbook PDFs, typed Study/Teaching, Presenter/monitor, save and restore. Distinguish passes from not tested. |
| **P0.6 Bible edition fidelity** | Built-in Bible import, Greek/OSHB catalogues and natural reference languages exist. Claude A1.5 structural recognition is **research only**, independent Set H pending. | No unknown/provisional extracted text shown as verified Scripture. Implement/audit numbering, correction re-verification, provenance and all downstream consumers, process-isolated PDF import, cancel/progress and Windows acceptance on combined lineage. **Do not merge recognizer merely because disclosed figures look good.** |
| **P0.7 Language and live AI** | Six interface languages and extensive AI editorial review; actual native proofreading, live Korean DeepSeek issue #19 acceptance and four named wording/provenance questions remain. | Every release UI route understandable in EN/DE/KO/FR/ES/PT-BR, texts and errors correct, provider consent uses actual endpoint, live DeepSeek edits only the chosen field and never silently overwrites. Disable unsupported AI paths rather than invent success. |
| **P0.8 Rights and software licence** | Repository `README.md` explicitly says **no software licence selected**; bundled Scripture rights/attribution are recorded per edition, and some conditions need actual distribution review. | **Owner chooses software licensing/distribution model**; add appropriate `LICENSE` / terms / privacy docs; audit included editions, fonts, packages, images and method/template content before distributing publicly. No public claim of copyright-free material without evidence. |
| **P0.9 Signed, recoverable distribution** | Dev/test updater and signed-release architecture exists, but the last owner acceptance ZIP is **unsigned** and not published as a stable release. | Pinned versioned Windows installer, legitimate signing/verification, reproducible artifact hashes, no downgrade or unapproved auto-update, download/update/rollback test with a disposable profile, stable documentation and support/privacy contact. |

## P1 — important to complete the first usable candidate

1. **Longitudinal study memory and navigation:** real projects and folders, source/Scripture marks linked back to Study, expandable sections and committed writing, recent work; continue incomplete Studies without recreating them.
2. **Teaching acceptance:** all three built teaching templates (sermon/talk, Bible lesson, group/EVA), Q&A, participant/leader views, project move/detach, exact exported originals, correct Korean/German typography in Office.
3. **Broader source test matrix:** independently selected PDF/EPUB/DOCX with different headings, page labels, directionality and citations. Report correct/incorrect/unresolved physical targets separately; no special-case PDF code.
4. **Documentation/onboarding:** update the User Guide with screenshots from the actual combined Windows app; describe local file ownership, licencing, normal and daily DB-only backups, and how to recover from a bad source or failed import. Document genuine limitations clearly.
5. **Launch operations:** choose installer distribution, version sequence and test/update policy; produce changelog, issue triage, support steps and reproducible diagnosis that never includes private books or keys.

## Deferred, not hidden blockers

Full OCR for scanned books, handwriting or recordings; wider ancient canonical systems; complex lexical/interlinear/syntax work; cloud sync, WebDAV/Nextcloud and cross-device conflict management; MCP/Bevro/Zekor orchestration; mobile editor/presenter remote; Word/PowerPoint template marketplaces; CRDT collaboration; advanced publication platform and commercial shop. These remain in the concept/architecture/backlog, but **must not prevent a useful honest initial release** if excluded features are clearly labelled and gated.

## Acceptance discipline and next sequence

1. Finish and validate the durable-use/safety-copy branch, then merge it into the tested app lineage **only after owner review**.
2. Run real Windows all-day study→teaching→outputs and privacy/backup cases from the matrix. Fix demonstrated blockers, not synthetic-only cosmetic differences.
3. Integrate the separately accepted Scripture candidate with provisional recognition, re-verification and worker isolation. Independently evaluate it **before** authorising publication.
4. Choose licence and distributable content; complete native linguistic and Office checks; sign and version the final approved installer, and separately approve release/publication.
5. Mark each P0 gate with an observed evidence link and *the exact version tested*; a pass on one binary is not inherited automatically by another.

**Owner evidence already accepted:** the supplied *Basisinformationen zur Bibel* navigation, Original-view lower border issue #49 and Settings→Sources→Edit details; POZETEK structural corrections/field resizing and Ten-Step methodology within their stated scope. Don't reset or retest already accepted narrow corrections unless later code actually touches them.

**No release declared yet.** The user's publication objective is a fixed goal, not an instruction to skip the blockers above.
