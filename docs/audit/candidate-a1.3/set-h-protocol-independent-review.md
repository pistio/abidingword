# Set H independent-evaluation protocol — review amendments (12 November 2026)

**Status: DRAFT protocol amendments, not authorisation.** Companion to `docs/audit/candidate-a1.3/set-h-protocol.md` in PR #51. This document proposes corrections to be agreed **before** any Set H inputs are selected, created, downloaded, sealed or run. No recognition or application code change is requested. Candidate code remains `7c7565a7a3df8f3dcc9711c8701af8c94dbad6af`; all B–G and v1 records remain immutable historical evidence.

## 1. Roles and information isolation

- **Owner:** approves the fixed selection constraints, provenance criteria, scoring rules, resource ceilings and one-run decision.
- **Builder:** independently selects inputs and an edition-matched reference oracle before seal, without access to recognition code, recognition audit results, or candidate harness. Builder records files accessed, environment and licence constraints. Selection cannot be adapted to recogniser outcomes.
- **Ground-truth reviewer:** separately checks edition/version alignment, sample source paragraphs, chapter/verse boundaries, omissions, versification, and uncertainty. Reviewer signs a pre-seal exception ledger; cannot adjust scoring after observing results.
- **Runner:** verifies sealed digests, invokes the exact frozen harness once, archives every artifact and crash/timeout without editing it; does not make fixture or oracle changes.
- **Assessor:** examines results after the run is complete, publishes case-level errors and uncertainty, and never retroactively changes the initial raw score.

The isolated builder may use **a separately supplied versioned and checksummed audit executable/script** without receiving the whole repository, development corpora or earlier results. The mere label “independent” is insufficient: the seal must record the builder's input-access manifest, method and any overlaps. Model or agent used for each role, including if Claude Fable 5.2 was used, must be recorded as provenance; model identity does **not** substitute for information separation.

## 2. Set composition and controls

Proposed ≥24 documents: ≥10 EPUB, ≥8 PDF, ≥4 DOCX; include English, German, French, Spanish, Portuguese, Korean, Greek, Hebrew and Latin with predeclared representation, both pointed/unpointed Hebrew where authentic sources exist, varying canon/tradition, footnote styles and book structures. Aim for ≥12 authentic published documents with independent edition-aligned reference text, not merely generated derivatives. Report authentic and generated results separately.

Two **distinct control classes** must not be conflated:
- **Non-Scripture negative controls**: files that contain no Biblical verse text and no legitimate reference-specific Scripture data. Here *zero verified verses* is an appropriate expectation, established by a pre-seal inspection.
- **Scripture-containing non-Bible documents**: commentaries, lectionaries and hymnals may quote genuine Biblical words. They are *mixed-content challenges*, not automatic zero-output controls. The oracle must predeclare which passages are present, their edition/versification where knowable, and which assignments are unsupported. Ambiguous quotations are scored as unresolved, never silently “correct”.

Selection by the builder must not require using the exact 113 disclosed editions. Avoid same-source and same-converter leakage where feasible; report overlap instead of disguising it.

## 3. Reference oracle and licence

For each authentic document, pin publisher/edition, revision or printing, language, canon/versification, provenance URL or acquisition route, file digest and permitted processing/storage rights. A superficially similar translation or another revision is not a sufficient ground truth. Before sealing, reconcile the reference to the actual document at selected independent samples and record known disagreements across omitted/added verses, punctuation, orthography and deuterocanonical numbering. Where oracle authenticity cannot be established, predeclare the document **exploratory / unscorable for wrong-text rate**; do not convert it to a clean pass.

For generated fixtures: run a source→generated-document audit specific to the **actual output format** (DOCX/EPUB/PDF), plus a source/oracle comparison. The existing `audit-derived-completeness.py` is specifically a USFM→DOCX audit on the examined corpus; it is not a universal PDF/EPUB auditor. Use independent manual/alternative-engine spot checks where complete machine comparison is unavailable, and disclose remaining unverified portions. Version and SHA-256 the builder's audit scripts. Capture transformation and input hashes at seal time.

Before relying on an audit script, exercise synthetic **mutation controls** on separate disposable fixtures: removal of `\\mi` and previously unseen text-bearing paragraph markers, truncated verse, extra words, reordered text, wrong book/chapter/verse, a mid-verse note caller, and a file with legitimate absent verses. An audit that fails to detect the intended alteration is not qualified for that class.

## 4. Seal and execution boundaries

A seal includes document and oracle SHA-256, builder code/tool checksums, manifest, pre-seal exception ledger, role declarations and access records, negative/mixed-content classifications, time/resource ceilings, and checksum of the exact candidate harness binary/commit/tree. Keep copyrighted documents outside public repository artifacts unless licensing permits. Don't put private source text into public error comments. Store the sealed manifest before any recogniser run; its timestamp and digest are evidence of ordering, not proof of independence by themselves.

**Exactly one** complete frozen-candidate run. No retries after seeing failures, no new baselines, no choosing a preferred result, and no changes to candidate or oracle. A crash, timeout, zero coverage or refused document remains a result. Set H becomes disclosed immediately after the single run.

## 5. Scoring and interpretation

Publish the raw unreclassified outcomes plus separately annotated adjudication:
1. Wrong verified **text** against a matched-edition oracle; verified references absent in that edition; unsupported assignments to quotations in mixed-content files.
2. Verified correct and conservative refusals, **denominators** by document/format/language/script/canon and by authentic vs generated source.
3. Source/provenance-span accuracy, exclusions, boundaries and whether a verse spans discontinuous original ranges; include negative cases and failure-to-open rates.
4. Resource ceilings and observed peak memory/time, including timeout/crash; declare exact ceilings *before* seal.
5. Oracle uncertainty and source/edition divergence remain visible, with initial wrong classification preserved in a parallel ledger rather than erased.

**Proposed strict research gate:** zero unambiguous false verifications and nonexistent references, zero on genuine no-Scripture controls, no crashes/resource-limit breaches. A case with unresolved ground truth is not a zero-error success. Report coverage without allowing high refusal rates to masquerade as broad reliability. Define minimum coverage for a separately named **utility** gate only after inspecting disclosed development outcomes and before sealing H; do not invent that threshold after H. Passing this research gate **does not** authorise production, migration 0028, or declaring any imported verse verified at edition level.

## 6. Outstanding pre-authorisation checklist

- [ ] Owner approves the amended protocol, selection composition, independent roles, resource ceilings and scoring.
- [ ] An independent builder and ground-truth reviewer are available, with actual information separation documented.
- [ ] Edition-aligned authentic files and usable reference oracles are established, with legal rights addressed.
- [ ] Format-specific completeness assurance and mutation controls are documented; unknown gaps remain explicit.
- [ ] The frozen candidate SHA and harness checksums are verified; previous disclosed seals unmodified.
- [ ] Explicit *separate* owner authorisation to construct and execute Set H is obtained.

**No Set H inputs have been created or evaluated in this review.** No tag, merge, Actions, installer, deployment, production recogniser integration, PDFium integration or database migration is authorised.
