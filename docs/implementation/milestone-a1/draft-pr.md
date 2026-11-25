The Windows owner review found misleading Source opening targets, missing Study/Project lifecycle controls, duplicated Teaching choices and compressed reader/writing layouts. This patch gives indexed chapter sections priority over bare mentions, adds recoverable Trash through existing lifecycle services, makes Teaching creation Study-first, unifies Source import and corrects the responsive controls.

- Exactly three current built-in Teaching templates; legacy sessions and personal templates stay intact.
- Existing standalone Teaching stays accessible and can be explicitly linked. Linked Study identity and Open/Edit actions are visible; saved research is never silently overwritten.
- A globe opens native-language choices directly. EPUB Contents preserves reader width, scroll position and focus.
- No migration changes, recognition changes, installer publication or Milestone B work.

Validation: **1,161 frontend tests and 1,036 Rust tests passed** (8 existing Rust ignores); typecheck, ESLint, i18n/corpus, Rustfmt, Clippy, production build and complete workspace checks pass. All 26 migrations are byte-identical.

See [the issue-by-issue owner handover](docs/implementation/milestone-a1-windows-ux-handover.md), [validation](docs/implementation/milestone-a1/validation.md) and its before/after screenshots. Layout checks exercise the complete application at 900, 1100 and 1440 pixels, plus the default 640-pixel layout. Lifecycle regression uses an on-disk disposable schema-26 graph.

Windows acceptance remains required. The actual MacArthur edition and owner Windows screenshots were unavailable; its real section/page target is not claimed verified. Missing PDF structure remains a separate outstanding feature, with its own implementation plan. Keep this draft at the owner-review boundary before Milestone B.

Base: `feature/windows-ux-milestone-a-review` (`f6e7c209ea4e14775368beee30e54dad3754ab4d`), not `main`.
