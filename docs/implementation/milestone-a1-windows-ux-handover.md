# Windows Milestone A.1 — owner review handover

## Scope and baseline

Implementation commit: `6b0d9a16fb6f7b9f239446cdb4c4c1f141b05f8d`.

Correction branch: `fix/windows-milestone-a1`, based exactly on `feature/windows-ux-milestone-a-review` at `f6e7c209ea4e14775368beee30e54dad3754ab4d` (PR #17's Milestone A work). The draft correction PR must target that feature branch. Do not merge to `main` or publish a Windows update without separate approval.

This implements the immediate A.1 corrections and the supplemental owner brief. It does not implement Milestone B, the catalogue service, provisional imports or Claude's Bible recognition work. Structured PDF Reading remains an outstanding functional feature with a [separate implementation plan](milestone-a1-structured-reading-plan.md).

## Resolution matrix

| Owner issue | Change and evidence | Remaining acceptance |
|---|---|---|
| 1. Acts 6 opens an incidental mention | Matching passage/chapter headings now precede unqualified literal mentions. Native `passage_sources` verifies an invented German Acts source opens its exact indexed chapter segment rather than a mention elsewhere in the same work; the full-app safety test exercises the corresponding reader target. Source IDs, locators and citation paths are unchanged. | The actual MacArthur edition/file is unavailable here. Re-test that edition on Windows, including its recorded source language. A missing PDF/book heading cannot be invented by ranking. |
| 2. Study/Project deletion | Consistent Danger zone in both information panels, named confirmation, soft delete by default, durable Settings → Data and backup → Trash, restore and typed-title permanent confirmation. A read-only `trash_list` uses existing tombstones; existing deletion/restoration services perform all mutations. | Use a copied/disposable Windows profile for owner acceptance. Never use the owner's live data for development. |
| 3. Three built-in Teaching templates | Settings offers only the three current block templates: Group Bible Study — Observation, Understanding, Application; Sermon / Talk; Bible Lesson — Children / Youth. Legacy definitions remain for existing sessions; personal templates remain editable and duplicable. | Confirm personal and historical sessions from the owner's copied profile remain editable. |
| 4. Study-first Teaching | General Study/Project New Teaching actions are removed. A Study's Teaching tab creates linked sessions and still supports multiple sessions. Existing standalone records have a compatibility list in their Project, only shown when records exist. The relationship indicator names the Study, exposes Open linked Study and an Edit affordance; standalone records expose Link to Bible Study. Link changes require Save. | Check one linked session, several sessions and a pre-existing standalone session; explicitly attach/unlink it and inspect saved citations. |
| 5. Duplicate preparation terminology | Redundant German Lehrvorbereitungen/Lehrvorbereitung labels are removed. Current Teaching navigation and the clearly named standalone compatibility route are retained. Technical preparation APIs and historical records remain intact. | Native-language wording review in all six locales remains welcome; no invented non-German EVA acronym was added. |
| 6. Unified Source import | Add Sources opens one batch/review flow for one file, multiple files or a folder. Files and Folder pickers are choices inside it. Recursive scanning, duplicate selection, correction, progress and interrupted jobs reuse the existing importer. Language correction is available alongside title, author and category. | Test actual Windows Files and Folder pickers, cancellation, a duplicate and one interrupted folder import. |
| 7. Language discoverability | A workspace globe opens Language directly. The same icon marks the Settings language jump and selector. Hover text lists native language names; all six native names remain in the selector. | Check from an unfamiliar UI language and after reopening Settings. |
| 8. EPUB Contents | Contents remains an overlay. Initial focus never scrolls reader ancestors; only its own list scrolls. The committed browser regression measures unchanged reader widths and scroll positions in Reading and Original, and verifies opener focus restoration. Existing history/Contents regressions remain in the suite. | Re-test the owner's EPUB in Windows, including a long outline and Back/Forward. |
| 9. Step selector | One full-width selector above aligned Previous/Next controls, with wrapping labels and equal row height. Measured in the complete workspace at 900/1100/1440 pixels and the default layout at 640. | Windows scaling and translated long section names. |
| 10. Teaching writing width | Block actions have a separate toolbar. Writing fields use their section width; single-line field controls stack. Session selection is constrained to its pane. The relationship editor is compact so the writing remains accessible. | Type, autosave, reorder and toggle privacy at narrow widths on Windows. |
| Complete workspace review | Real application components with synthetic data, including all three panes, not isolated component cards. Screenshots and measured browser checks are linked below. | Linux Chromium evidence does not replace Windows WebView2 acceptance; owner Windows screenshots were not attached to the briefs. |
| PDF structure | Recorded as an outstanding functional feature, independently of Bible recognition. The separate plan preserves original text/bytes, coordinates, citations, uncertainty and corrections. | Implement and review separately after the immediate correction review; do not silently declare it complete. |

## Data behavior to review explicitly

Deleting a Study uses the established schema-26 behavior: linked Teaching remains owned by its Project, retains saved finding/quote/citation snapshots and becomes unlinked. Restoring the Study restores its original identity and research but does not silently relink Teaching. The existing standalone compatibility route makes that retained work reachable. Reattachment is explicit.

Trashing a Project hides its entire graph together. Restoring it recovers the graph. Individually trashed Studies inside it appear in Trash after restoring the Project; they are not inadvertently restored with it. Permanent Project deletion removes its dependent graph and requires the exact Project title. Backups remain separate and available. No migrations were added or changed.

The native lifecycle regression uses an on-disk disposable database containing legacy and block Teaching, private notes, finding links, citation snapshots, materials, passages, presentations and presentation items, including trashed records. A second connection verifies that Trash survives reopening. Integrity/foreign-key checks and stable-ID assertions cover restore and permanent removal.

## Screenshots and reproducible layout checks

All texts and source materials are synthetic. Before images were captured from the exact baseline SHA above using the same screenshot harness.

| View | Before | After |
|---|---|---|
| Narrow Study | [900 px](milestone-a1/images/before/study-900.png) | [900 px](milestone-a1/images/study-900.png) |
| Narrow Teaching | [900 px](milestone-a1/images/before/teaching-900.png) | [900 px](milestone-a1/images/teaching-900.png) |
| Combined Study | — | [1100 px](milestone-a1/images/study-1100.png), [1440 px](milestone-a1/images/study-1440.png) |
| Combined Teaching | — | [1100 px](milestone-a1/images/teaching-1100.png), [1440 px](milestone-a1/images/teaching-1440.png) |
| EPUB Contents | — | [Reading, 900 px](milestone-a1/images/contents-reading-900.png), [Original, 1440 px](milestone-a1/images/contents-original-1440.png) |
| Language | — | [Globe destination](milestone-a1/images/language.png) |
| Lifecycle controls | — | [Named confirmation](milestone-a1/images/danger-zone.png), [durable Trash](milestone-a1/images/trash.png) |
| Simplified entry points | — | [Three templates](milestone-a1/images/templates.png), [Add Sources](milestone-a1/images/import.png) |
| Minimum desktop width | — | [640 px](milestone-a1/images/study-640.png) |

Run `tests/screenshots/milestone-a1.mjs` against the documented screenshot harness (`capture.mjs`). It accepts `PLAYWRIGHT_MODULE`, `CHROMIUM_PATH`, `SCREENSHOT_URL` and `SCREENSHOT_OUT`. It asserts navigation alignment, usable selector widths, full-width Teaching fields, contained session selection, Contents scroll/width stability and focus restoration. The 640-pixel check uses the default two-pane layout; the three-pane stress cases use 900–1440 pixels. Visibility remains user-controlled with the existing pane toggles.

## Validation and delivery

Final command results and the delivery commit are recorded in [the validation report](milestone-a1/validation.md). Earlier development failures are not counted as final passing evidence. No paid Actions, installer publication, merge, NiPogi deployment or real Windows data access is part of this patch.

Owner review is the boundary. Do not begin Milestone B until that review is complete. A successful test suite is not a Windows usability verdict, and it does not change Claude's separate NOT READY recognition verdict.
