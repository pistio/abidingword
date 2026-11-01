# Windows Milestone A.2 owner review

This patch starts from A.1 `1b61656d4407a7511b3615b2c91c8411d8c1faff` on `fix/windows-milestone-a2`. The intended PR base is `fix/windows-milestone-a1`, not main. The integration baseline remains `e2564b570abab360722d379c18955272165ec4ac`. PRs #17 and #18 were verified as drafts through their public GitHub pages. Delivery SHAs and final checks are recorded in [validation](milestone-a2/validation.md).

## Changes and remaining acceptance

| Request | Implemented and checked | Owner acceptance still needed |
|---|---|---|
| Acts 6 source navigation | Every explicit open receives a new reader instance, even for the same segment. The reader rejects a wrong source, missing target, or substituted segment. Passage matches are distinct from recent-position links; duplicate quick links to matched works are removed. Literal-only results explicitly say no direct discussion has been verified. | Reproduce with the actual MacArthur PDF on Windows. That file was not supplied. This patch does **not** establish that its indexing/ranking or theological relevance is correct. |
| Workspace stability | Target scrolling is contained within the reader. Contents and source return preserve pane geometry, reader position and result focus. Splitter values reflect displayed widths; one keyboard step moves 24 px. At 640 px, three open panes stack into independently scrollable regions rather than shrinking writing below its minimum. | Windows WebView2, OS scaling, owner documents and actual window dragging. |
| Listing deletion | Study and Project rows have an accessible action menu, named confirmation, immediate Undo and workspace Trash. Existing lifecycle APIs retain durable restore and schema-26 Teaching snapshot/detachment semantics. | Review discoverability, cancel, Undo, reopen Trash and restore with a disposable profile. |
| Missing original sections | One/all restore for historical/current POZETEK and Ten-Step methods, including all original parts removed. Stable keys determine identity. Native definitions supply original fields, guidance and phase. Surviving writing and custom sections remain. Pending autosaves settle before restoration. | Review wording and sequence with a copied Study, including six-language UI. |

## Source audit and limits

The existing PDF adapter provides physical page and segment provenance but no reliable document heading hierarchy. A literal reference cannot establish commentary or contextual explanation. This patch does not add a semantic classifier, alter recognition, or claim that a higher-ranked plain match is direct discussion. Existing heading context remains contextual evidence only.

The audit follows `src-tauri/src/db/library.rs` from segment insertion and passage extraction through candidate collection, per-segment ranking and work grouping, then `src-tauri/src/library/relevance.rs` and the PDF/EPUB adapters. Existing ordering prefers explicit manual links and matching heading context over neutral mentions, places cross-references below them, and places very broad ranges last. Candidates must overlap the selected verse range. Source type is not relevance evidence. `PassageSourceList` passes the ranked segment ID, and `SourceReader` opens that ID through the bounded segment-window API. No ranking rule can manufacture missing PDF section boundaries. The unchanged ranking is retained with its regressions; genuinely distinguishing dedicated PDF discussion remains dependent on document structure and the original-file investigation.

The defects demonstrated independently of the owner's file were reuse of a reader instance for an explicit reopen, insufficient returned-target validation, ambiguous cached-position links, target scrolling that could affect ancestors, and splitter preference values differing from visible widths. Their regressions exercise actual UI paths. Native fixtures check that an invented Acts 7 cross-reference to Acts 6 is a mention with its exact physical page, without promoting it to verified commentary. The browser fixture is a real, generated three-page PDF with an outline and invented text, rendered by PDF.js. It is not a MacArthur edition or a substitute for its acceptance test.

Owner procedure, using a disposable profile or copied backup only:

1. Record the exact source filename/edition/language, application source SHA, current passage and physical page before testing. Start with Acts 6 selected, with no stale Acts 7 selection.
2. Expand the relevant work. Record its match type and location. Click that passage match, not the separately labelled recent-position link.
3. Compare Reading text, Original physical PDF page and citation. Distinguish physical page from a printed page number. Confirm whether the match is only a cross-reference or actual Acts 6 discussion.
4. Navigate elsewhere, return, reopen the same match, switch modes, open Contents, use history, and repeat after restarting. A match must reopen its exact indexed target; a recent link intentionally resumes reading position.
5. If it still fails, capture selected passage, expanded match/location, displayed physical page and citation, the relevant document heading, and whether the result was a mention. Preserve the copied database and source for a targeted index/ranking investigation. Do not describe an unrelated page as verified commentary.

The actual MacArthur case remains an open acceptance item. Claude's multilingual recognition workstream is untouched and its NOT READY verdict is unchanged.

## Restoration and deletion semantics

Restoration creates new empty sections rather than undeleting removed writing. The disclosure explicitly states this distinction. Restore deleted writing through the existing recovery/backup paths where available; this action cannot reconstruct it. Empty original-section deletion offers Undo only after the backend has confirmed it was empty. Custom sections and unsupported/custom template identities never receive speculative method restoration.

Original method sections return to canonical method order in the existing method slots, including reordered survivors. Custom sections keep their slots and relative order. Surviving section IDs and written content do not change. Restoring an already-present key is a no-op; unknown keys reject atomically. No migrations or historical SQL bytes changed.

Study deletion continues to detach Teaching while preserving saved snapshots. Undo restores the Study identity without silently relinking Teaching. Project deletion uses the existing descendant behavior and named explanation; durable Trash remains the same implementation exposed from Settings.

## Review evidence

All screenshots use invented data on Linux Chromium. DPR emulation is not Windows OS scaling.

| View | Evidence |
|---|---|
| Combined panes | [900 px](milestone-a2/images/workspace-900-1.png), [1100 px](milestone-a2/images/workspace-1100-1.png), [1440 px](milestone-a2/images/workspace-1440-1.png) |
| PDF and Contents in full workspace | [900 px / 125% DPR](milestone-a2/images/pdf-contents-900-1.25.png), [1100 px / 150% DPR](milestone-a2/images/pdf-contents-1100-1.5.png), [1440 px / 200% DPR](milestone-a2/images/pdf-contents-1440-2.png) |
| Narrow three-pane layout | [640 px](milestone-a2/images/pdf-contents-640-1.png) |
| Listing confirmation | [Named Project confirmation](milestone-a2/images/listing-confirmation.png) |
| Empty method restoration | [Missing-part controls](milestone-a2/images/restore-missing.png), [restored Time](milestone-a2/images/restored-time.png) |
| Earlier A.1 baseline | [A.1 review and screenshots](milestone-a1-windows-ux-handover.md) |

`tests/screenshots/milestone-a2.mjs` runs against the existing screenshot harness with `a1Review=1&a2Review=1`. It asserts 640/900/1100/1440 widths, DPR 1/1.25/1.5/2, no pane horizontal overflow, real PDF canvas, indexed reopen after cached page navigation, Contents geometry/scroll, source return focus, long EPUB reading, passage change while reading, splitter values and preserved Study writing. [Measured results](milestone-a2/workspace-results.json) accompany the screenshots. A.1 browser workflows are also rerun separately; see validation.

## Windows build and review boundary

The existing manual-only Windows acceptance workflow is prepared for the exact implementation SHA recorded in validation. It produces an unsigned NSIS review installer with source/environment provenance and checksums, preserves the pinned historical migration catalogue, and does not configure the updater or publish a release. It has **not been dispatched**. No Windows installer was produced in this Linux session. The requested new Windows build and owner acceptance remain outstanding; a workflow configuration is not a completed build.

Git transport works, but the managed environment rejects GitHub API access with Forbidden. The saved onboarding network draft has not changed the active runtime policy. No credential or network-policy workaround was used. Draft PR creation status is recorded in validation.

Stop for owner review here. Milestone C, Milestone B, provisional architecture and Milestone D have not begun. No merge, installer publication, paid Actions dispatch, NiPogi deployment or real Windows user-data access was performed.
