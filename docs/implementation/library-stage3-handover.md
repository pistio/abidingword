# Library Stage 3 — navigation handover

## Baseline and scope

- Execution environment: Codex Cloud only.
- Development branch: `feature/library-stage3-navigation`.
- PR base: `fix/windows-dev44-schema-compatibility`.
- Verified base / Library Stage 2 merge: `b3120f54b0dcece6c078727c97240828680dd63c`.
- Git connectivity through the configured HTTPS proxy and remote-branch fetch succeeded before implementation. The older `main` checkout at `a7bc798` was not used as the development baseline.
- All migration files `0001–0024` matched the verified baseline byte for byte and were registered in order. This change adds no migrations and does not change the migration catalogue.
- Bible recognition, Teaching, Bible Study templates, dependency manifests, lockfiles, and release workflows are unchanged.

## Resulting behavior

### Collections and Library return

The everyday Library can browse a collection using a labelled select or the collection actions on a source row. Nested collections show their parent path. Membership follows the existing management behavior: selecting a collection shows its direct members, not every source in its descendants. A source can still belong to several collections without copying its file.

The same collection filter applies to the source list, name matches, and text matches. The optional `collectionId` text-search parameter uses the existing collection tables and an `EXISTS` filter, so membership in several collections cannot duplicate a hit. Existing callers may omit it. Archive and Trash behavior is preserved. Responses from an earlier collection/search request cannot append stale text results after the user changes filters.

Returning from a source restores the collection, type/language/status filters, list page, query draft, submitted query, loaded text-result pages, scroll position, and focus on the source/result that opened the reader. If that action is no longer available, focus falls back to the Library heading. Settings returns to the page from which it was opened.

### Reader, Scripture, and citations

Reader positions now carry the exact text paragraph and pixel offset, or Original PDF page and fraction, in addition to the existing formatted EPUB/DOCX place and reader history. The shell captures positions while the viewer is still mounted, before route/pane changes. Ordinary reopening of a source resumes its session position; an explicit search result or citation takes precedence over that remembered place.

Opening a Scripture reference from a full reader retains the source beside Scripture, using the existing indexed-reference action. A reader entered from the Library offers **Back to Library** after this transfer and returns to the previous collection/filter context. This navigates Scripture without editing the Study or its passage.

Cited words survive pane → full reader → pane transfers. Citation snapshots and provenance remain unchanged. Existing missing-file, archived-source, unavailable-citation, and formatted-reader behavior is reused. About this source remains a disclosure; its height is bounded so the reading controls remain reachable, and its availability note is outside the definition list.

Browsing state and reading positions are session-only. This change does not write source text, quotations, or navigation history to browser storage, and does not add cross-restart position persistence.

## Validation

Validation is performed sequentially where timing-sensitive native tests require an idle machine. The Codex Cloud container provides the native Linux libraries and writable workspace XDG config/data/cache directories for its non-root Tauri test process; `/workspace/.onboarding/native.sh` runs commands from `src-tauri/` with four Cargo build jobs and debug information disabled.

Commands:

```sh
npm run typecheck
npm run lint
npm run i18n:check
npm test -- --maxWorkers=2
npm run build

# From src-tauri/ (through the prepared Cloud native wrapper here):
cargo fmt -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked -- --test-threads=1
```

New regression coverage includes nested/direct collection membership; scoped name/text searches; late pagination responses; returning to filters, page, scroll and focus; loaded-result/query-draft restoration; exact text offset and history after Settings/reopening; explicit-target precedence; Scripture return without Study mutation; citation words/provenance through reader transfers; Original PDF page-fraction restoration; and native scoped-search pagination, duplicate prevention, archived sources, Trash, and backwards-compatible input deserialization.

Frontend validation: the full run executed **1,105 tests** (95 files): **1,101 passed**, three generated-corpus freshness checks failed because the new Library labels needed exporting, and one unmodified study-editor test exceeded the normal 15-second timeout during concurrent native compilation. After `npm run i18n:corpus` refreshed the two review documents, both affected files were rerun without a concurrent build: **36 tests passed**, including all four previously failing cases, with no timeout or assertion changes. The corpus `--check` also passed. Typecheck, ESLint, the six-language consistency check, and production build passed; the build emitted a chunk-size advisory.

Targeted native Library/citation/schema-upgrade validation: **40 tests passed**. Rust formatting and all-target Clippy with warnings denied passed. The first full native attempt stopped in Tauri mock-window initialization because the container’s default user directories were not writable. Pointing XDG config/data/cache directories to `/workspace/.onboarding/native-runtime` fixed the environment issue: all three capability enforcement checks passed without changing capability grants or tests. **The complete native rerun finished with 1,006 passed, 0 failed, and 8 pre-existing ignored tests.** No tests or assertions were disabled for Stage 3.

## Review and remaining acceptance

- Review and merge target must remain `fix/windows-dev44-schema-compatibility`, not `main`.
- A real Windows desktop visual/accessibility acceptance run is not available in Codex Cloud and has not been claimed. Check keyboard collection browsing, return focus, narrow/wide reader transfers, and PDF/EPUB/DOCX reading positions on the supported desktop.
- No merge, deployment, release, signing, paid service, or manual workflow dispatch is part of this task.
- GitHub API preflight for PR creation returned `Get "https://api.github.com/repos/pistio/abidingword": Forbidden`, although Git fetch works. The environment draft includes the required `api.github.com` allowlist addition; draft saving alone does not activate it. PR creation must be retried after API access is available. Do not substitute a compare URL for an actual created PR.
