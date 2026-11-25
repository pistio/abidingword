Milestone A refines the existing Library, Bible Study and Teaching workflows from `integration/windows-library-stage3-teaching-study` at `e2564b570abab360722d379c18955272165ec4ac`.

- Expose multi-file/folder import, add physical PDF page navigation in both reader modes, clarify heading versus literal-reference evidence, and show actual session opening history with compact additional matches.
- Label Study inputs by type and make Step-by-step and Continuous writing distinct while retaining the same stored content and mounted editors. Attribute the Ten-Step method and distinguish the independent guidance.
- Provide primary New study and secondary New Teaching actions, responsive rectangular choices, German-only EVA terminology, and discoverable optional Study linking and participant visibility.

No backend code, historical migration, dependency/lockfile or release workflow changes. Exactly three built-in Teaching templates remain; standalone/legacy Teaching and the existing privacy/provenance paths are retained. Claude’s recognition work is untouched and its v0.24.3 verdict remains NOT READY.

Validation: all 1,157 frontend tests and 1,034 Rust tests passed; eight pre-existing Rust report/generator tests remain ignored. Typecheck, lint, six-language/corpus checks, formatting, Clippy, and frontend/native builds passed. Fresh native Linux startup reached schema 26 with database/FK integrity verified. Browser rendering checked at 900/1100/1440px. The existing bundle-size advisory remains.

Owner handover and synthetic browser screenshots: `docs/implementation/milestone-a-windows-ux-handover.md`. Draft for Milestone A review only. Real Windows/WebView2 acceptance remains pending. Do not merge to main, dispatch Actions, publish installers or begin Milestone B through this PR.
