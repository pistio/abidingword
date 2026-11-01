# Draft PR preparation

Base: `fix/windows-milestone-a3-home`
Head: `feature/windows-milestone-a3-pdf-structure`
Title: **A.3 Stage 2: structure-aware Library PDF sections and navigation**

Selecting Acts 6 can currently lead to a literal mention in an unrelated part of a PDF. This change persists aligned outline/tag/layout evidence, indexes real section starts separately from mentions, and keeps exact heading identity through the Source result, Contents and Reading/Original navigation. Existing PDF imports offer an explicit structure build; new imports collect it while staging.

Reversible app-owned corrections, original page/line provenance and bounded metadata overlays preserve original files, immutable segment text/IDs, citations and annotations. Original PDF page bookmarks remain separately accessible when exact text alignment is unavailable. Reading adds headings, text roles and printed-page evidence. All six UI languages are included.

Schema 27 is additive and required for backed-up structure/correction history; historical SQL 1–26 and the recognition/shared layout implementation remain unchanged. Restore a matching pre-upgrade backup to revert to a schema-26 executable. Do not open an upgraded profile with an older executable.

The handover records exact implementation/delivery SHAs and final validation. Synthetic tests and Linux browser/native evidence do **not** accept the owner-reported MacArthur defect: the actual PDF and native Windows evidence remain required. Issue #22 stays open for that acceptance. Stage 1 is a separate base PR; no editor redesign, Q&A, recognition merger or online catalogue is included.

Do not merge or dispatch paid build/release workflows without the owner's separate instruction. No installer or updater has been published.

Tested application source: `15cac960a93f260385c868080ec580d4075e2c5f`.

Validation: full frontend 1,189/107 files plus the added continuity regression (focused PDF file 9/9); full locked Rust 1,054 passed, 8 existing ignored; typecheck/build, lint, localisation, fmt and all-target Clippy passed. Real native Linux structure/Original smoke and synthetic full-workspace browser checks passed. Historical SQL 1–26 unchanged.

Draft creation was attempted after push; the session GitHub API returned Forbidden. No PR was created. Use the comparison link in the handover and this body for manual draft creation.
