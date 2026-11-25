The A.2 owner review rejected the Study homepage's separate organiser, prominent Trash and standalone Teaching entry. This Stage 1 patch replaces them with one file-and-folder home: root Studies and named Project folders, New study/New project actions, breadcrumbs, per-row rename/move/Trash and quiet footer recovery.

Root ownership uses the existing backed-up settings table. Moves retain Study IDs and atomically carry linked active/trashed Teaching without converting findings into snapshots; standalone records stay with their existing Project and are managed from Settings → Data and backup. No historical migration changes or user-data rewrites.

References #21. Stage 2 / #22 remains separate; this patch does not claim to fix or accept the unavailable MacArthur PDF. The writing-first editor redesign, Issue #19 and the other approved backlog remain deferred.

See `docs/implementation/milestone-a3-home-handover.md` and its validation report for exact SHAs, complete test results, ownership/backup behavior and before/after combined-workspace screenshots. Keep this a draft against `fix/windows-milestone-a2`; no merge, release or paid workflow dispatch.
