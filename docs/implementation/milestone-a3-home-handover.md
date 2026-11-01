# A.3 Stage 1 — unified Bible Study home

Tested implementation: `0fad2ab94717d6d0df6609b4bac809e350843a21`.

Review branch: `fix/windows-milestone-a3-home`, based on verified A.2 head `c4d045f4c63586a0a351509a050adc458259394c`. Its last tested implementation was `450bd95ef9bedabc248e1746fbbf8585fe19e72a`. PR #20 remains a draft into A.1; #18 and #17 remain drafts into A and the combined integration respectively. This branch does not start from main.

The A.3 brief and Issue #21 supersede prior acceptance claims. The owner's A.2 Windows review remains unacceptable for homepage hierarchy and the MacArthur case. This change is Stage 1 only; PDF structure work belongs on a separate Stage 2 branch after this reviewable snapshot.

## Issue-resolution matrix

| Issue #21 requirement | Implementation | Acceptance limit |
|---|---|---|
| One file-and-folder home | Root documents and named Project folders share one list. Folder children appear only inside that folder. Every library size has New study and New project actions. | Review on Windows with the owner's copied profile. |
| No separate organiser | The organiser component is removed. Historical saved organiser locations resolve to the same home. Folder breadcrumb and Study Back return to their actual parent. Root creation never chooses the last named Project. | Root/folder creation and keyboard navigation in all six languages. |
| Preserve historical standalone Teaching | Removed from ordinary home/folder entries. Settings → Data and backup → Legacy standalone Teaching lists historical manuscripts, opens the exact selected record, and retains explicit linking/editing and saved materials. No automatic attachment or new standalone creation is added. | Open legacy, block and unlinked retained sessions on a disposable Windows profile. |
| Quiet Trash | Trash is in the footer. Row menus retain named confirmation; immediate Undo, durable Trash, restore and typed-name permanent deletion reuse the established lifecycle services. The last-Study list remains mounted so Undo survives deletion. | Review cancellation, last Study, whole folder, restart and restore. |
| Rename and move | Per-row File actions expose Rename and Move. Moves preserve Study IDs and linked Teaching ownership atomically, including trashed sessions. Existing editor Project/Study title controls remain available. | Inspect saved citations, private materials and presentations after moving in/out of folders. |
| Data and localisation | No migration changes. Root ownership uses existing backed-up settings; all six UI languages are supplied. | Consistency checks do not replace native-speaker review. |

## Root ownership without rewriting data

The existing Project model remains authoritative. Each workspace remembers an implicit root Project ID under `studies.rootProject.<workspace-id>` in the existing settings table. Full backup/restore carries this marker. Project package import remains a named Project and does not rewrite the receiving workspace's root.

On first use, exactly one Project with an established six-language default title and no description can be adopted as the historical implicit container. Its ID, title, Studies and Teaching are untouched. A differently named Project is never collapsed just because it is the only Project. If multiple possible historical defaults exist, no candidate is guessed: they remain visible folders and a new root container is created. Historical records do not contain an explicit default flag, so an intentionally named default-title Project cannot be distinguished perfectly from the old implicit default. No data is deleted or merged. Users can move documents explicitly if a historical container was renamed or remains ambiguous.

New root Studies always use this stable ID. Named folders remain separate even if the device last worked in one. A removed/trashed root is not silently restored; its records remain in durable Trash. Previously saved direct navigation to the implicit Project returns to home.

## Move transaction and preservation

Schema 26 enforces matching Project ownership for Studies and linked Teaching. The move service validates live source/destination Projects in the same workspace, then uses one transaction to temporarily clear links, move the Study, and restore every linked active/trashed Teaching to the same Study ID under the new Project. It does not call lifecycle unlink logic or convert findings into detached snapshots. Other connections cannot observe an intermediate committed state. Any failure rolls back the entire transaction.

Standalone Teaching remains with its existing Project. Linked Teaching blocks, findings, citations, materials and presentations retain IDs and content. Existing soft-delete behavior remains distinct: deleting a Study detaches Teaching and preserves snapshots; Undo restores the Study without silently relinking Teaching. Moving does not detach it.

Native fixtures verify concurrent app connections create one root container, the actual root marker is restored from backup, an injected failure during reattachment rolls back, invalid/cross-workspace destinations are rejected, active/trashed linked sessions follow their Study, a standalone manuscript stays put, findings/citations/blocks/presentations survive, a former folder can be removed after moving out, and full backup/restore retains root and graph identity. All documents and databases are disposable and invented.

## Screenshots and regression scope

The screenshot script uses actual application components with an identical invented profile on archived A.2 application code and the current implementation. Only the common test fixture and archived harness asset allow-list differ from baseline; no baseline application code was edited. Before screenshots include the owner's reported shape: one implicit Project, a Study and historical standalone Teaching. A second shape has root documents and two named folders.

| View | Before | After |
|---|---|---|
| Owner-shaped profile, 900 px | [A.2](milestone-a3-home/images/before/owner-900.png) | [A.3](milestone-a3-home/images/owner-900.png) |
| Owner-shaped profile, 1440 px | [A.2](milestone-a3-home/images/before/owner-1440.png) | [A.3](milestone-a3-home/images/owner-1440.png) |
| Mixed documents/folders | [900 px](milestone-a3-home/images/before/home-900.png) | [900 px](milestone-a3-home/images/home-900-1.png), [1440 px](milestone-a3-home/images/home-1440-1.png) |
| Narrow combined workspace | — | [640 px](milestone-a3-home/images/home-640-1.png) |
| Scaling emulation | — | [125% DPR](milestone-a3-home/images/home-900-1.25.png), [150% DPR](milestone-a3-home/images/home-1100-1.5.png), [200% DPR](milestone-a3-home/images/home-1440-2.png) |
| Language and folders | — | [German](milestone-a3-home/images/home-de-900.png), [Korean](milestone-a3-home/images/home-ko-900.png), [inside folder](milestone-a3-home/images/folder-1100-1.png) |
| Secondary actions | — | [Move](milestone-a3-home/images/move.png), [named folder confirmation](milestone-a3-home/images/folder-trash.png), [legacy management](milestone-a3-home/images/legacy-management.png) |

These are Linux Chromium screenshots, not Windows evidence. DPR emulation does not establish Windows WebView2 or OS-scaling acceptance. Reproduce with `tests/screenshots/milestone-a3-home.mjs`; recorded geometry is in `milestone-a3-home/workspace-results.json`. Native startup/build validation is separate.

Existing editor regressions now explicitly enter their named fixture folder through `renderAppInProject` rather than relying on the removed single-Project auto-entry. New homepage tests call `renderApp` directly. Assertions for editor content, exports and private data are retained. A language-switch regression checks that the same editor and pending writing remain mounted; root ownership is independent of later interface-language changes.

Final counts, exact SHAs, captured checks and PR status are recorded in [validation](milestone-a3-home/validation.md). Failed development runs are excluded from passing results. No paid Actions, automatic update, release, merge, NiPogi deployment or real Windows profile access is part of this branch.

## Owner Windows procedure

Use a disposable profile or a copy of a backup. Check zero, one and several named folders, the old implicit default, a renamed historical container, root documents and historical standalone Teaching. Verify no document is duplicated in root and folder. Create a root Study after working in a folder, create inside the folder, move in/out, rename, open Teaching and inspect retained writing/citations/private flags/presentations. Cancel deletion, delete the last Study and Undo, trash a folder and restore it after restart, then inspect durable Trash's stronger permanent confirmation using disposable data only. Switch languages while a Study or presentation is open. Review 640/900/1100/1440 widths and 100/125/150/200% Windows scaling.

## Retained backlog and next boundary

Stage 2 / Issue #22 remains a structure-aware PDF task. Ranking-only changes are not a substitute for hierarchy. The original MacArthur PDF is unavailable here; no synthetic result accepts its Acts 6 navigation. Claude's Bible recognition remains isolated and its previous NOT READY verdict is unchanged.

The next dedicated UI milestone is the approved minimalist writing-first editor, with secondary controls behind disclosures/menus and clear private indicators. It is not implemented by this homepage change. Also retain Issue #19 (DeepSeek Korean correction: inspect selected block/stale IDs, provider connection and suggestion application without blaming the API), paired Q&A, safe question exports, selected Study-to-Teaching transfer, citation/private-content safeguards, provisional Scripture-integrity work on its proper lineage with Claude coordination, and the six-language theological catalogue/rights/website planning. No commerce infrastructure is started.
