# Abiding Word — Milestone A.3 Stage 2 owner handover

**Status: implemented for review; the MacArthur / native Windows acceptance gate remains OPEN.**

- Repository: `pistio/abidingword`.
- Branch: `feature/windows-milestone-a3-pdf-structure`.
- Base: completed Stage 1, `fix/windows-milestone-a3-home` at `5fab020012d0db42899dd7ff0b3d1b47f249fc04` (implementation `0fad2ab94717d6d0df6609b4bac809e350843a21`). This remains stacked above A.2 → A.1 → A → integration; `main` was not used as the development baseline.
- Tested Stage 2 application source: `15cac960a93f260385c868080ec580d4075e2c5f`.
- Draft PR: not created: the GitHub API returned `Forbidden` in this cloud session. [Prepared body](milestone-a3-pdf/draft-pr.md); [comparison](https://github.com/pistio/abidingword/compare/fix%2Fwindows-milestone-a3-home...feature%2Fwindows-milestone-a3-pdf-structure?expand=1).

## What changed

Library PDF imports now retain text-aligned outline/tag/layout evidence, original page/line coordinates and hierarchy. Grounded passage/chapter starts precede biblical-book fallback and body mentions. A result carries its exact heading identity even when several headings share a stored paragraph. This is documentary address evidence, not a judgement of theological or commentary relevance.

Reading uses heading/text-role overlays and known printed page labels alongside physical PDF pages. Corrected hierarchical Contents and Original navigation retain exact section identity; a visible centred target wins over preceding-page context during mode switching. Original PDF page bookmarks remain separately available without overriding corrections. Unaligned or incomplete structure stays uncertain; Original and ordinary text remain accessible.

Existing PDF Sources offer **Document structure → Build structure**. New imports build it while staging. The panel supports title/level/mapping corrections, exclusions, adding an unambiguous visible line as a section, Undo and reversible Reset. Rebuild does not replace original files or stored segment text/IDs. The copied file is hash-checked against the preview before commit. Citations/highlights and correction history survive full backup/restore. Historical citation locator values are preserved; printed-page metadata does not silently rewrite them.

## Database and rollback

Stage 2 adds **schema 27**, solely for app-owned PDF metadata/history and nullable derived-link evidence. All historical SQL 1–26 still matches the earlier independent manifest. Recognition and shared layout code remain unchanged; Claude's v0.24.3 verdict remains **NOT READY**.

Use a disposable/copied Windows profile for review and retain a complete pre-upgrade backup with its matching executable. An older schema-26 executable must not open the upgraded database. Revert by restoring the matching backup into a separate profile. The normal pre-migration backup and upgrade rollback paths are tested. See [architecture and bounds](milestone-a3-pdf/architecture.md).

## Actual validation

- Frontend: **1,189 passed / 107 files** in the full run; the additional continuity regression was added afterwards without frontend application changes, and the focused PDF reader file passed **9/9**. Later typecheck/build and lint include that test.
- Rust: **1,054 passed, zero failed, eight existing ignored / 89 result groups** in the final full locked serial run.
- Typecheck/build, lint, localisation, formatting and all-target locked Clippy: passed. All 26 historical SQL hashes match the earlier manifest; recognition/shared layout unchanged.
- Synthetic browser layouts/scaling, German/Korean UI, before comparisons and separate original-bookmark navigation: passed. Actual native Linux main-window metadata and PDF.js Original smoke: passed with schema 27, one safety backup, integrity `ok`, no FK violations.

[Detailed validation](milestone-a3-pdf/validation.md), [command logs and historical hashes](milestone-a3-pdf/checks), [browser results](milestone-a3-pdf/images/checks.json).

Synthetic full-workspace comparisons: [Stage 1 plain Reading](milestone-a3-pdf/images/before-reading-900.png), [structured Reading](milestone-a3-pdf/images/reading-900-1.png), [Original](milestone-a3-pdf/images/original-900-1.png), [German structure panel](milestone-a3-pdf/images/structure-de-900.png), [Korean](milestone-a3-pdf/images/structure-ko-900.png). These use an invented PDF and actual Rust import/index metadata, with real PDF.js. They do not reproduce the unavailable MacArthur file.

## Owner review boundary

Run the [reproducible Windows check](milestone-a3-pdf/windows-owner-check.md) on the original legally owned MacArthur edition. Record file SHA-256, expected and actual first section/page for Acts 6, Reading/Original handoff, printed/physical pagination, corrections, citations and backup/restore. Synthetic passes cannot close Issue #22. Distinguish automatic detection from a result achieved by manual correction.

The separate schema-27 manual acceptance workflow is prepared and pinned to the tested application SHA. It has **not** been dispatched; paid runner use requires the owner's separate instruction. The Stage 1 schema-26 workflow remains unchanged. No installer, updater, release or deployment was published; no real Windows user data or NiPogi was accessed.

## Issue and follow-up matrix

| Item | State |
|---|---|
| #21 / A.3 Stage 1 file-and-folder home | Separate pushed branch and [handover](milestone-a3-home-handover.md); owner review |
| #22 / A.3 Stage 2 PDF structure | Implemented and tested synthetically; original-file/native-Windows acceptance OPEN |
| Writing-first Study/Teaching editor redesign | Next dedicated UI milestone; not folded into either A.3 stage |
| #19 DeepSeek correction regression | Backlog; cause and fix not claimed |
| Paired Q&A, safe exports, Study-to-Teaching transfer | Planned next stage, subject to its own implementation review |
| Broader structured Source reading | Retained; this branch targets the approved PDF stage only |
| Provisional Bible imports / recognition | Isolated; no unfinished Claude work merged or modified |
| Six-language theological library / website / folder integrations | Research/architecture backlog; no online service, payments or cloud commerce implemented |

Stop here for owner review. No merge into `main` or integration has been performed.
