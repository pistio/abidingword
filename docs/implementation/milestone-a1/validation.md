# A.1 validation and delivery

Date: 2026-10-31 (UTC). Baseline: `f6e7c209ea4e14775368beee30e54dad3754ab4d`. Branch: `fix/windows-milestone-a1`. The latest fetched integration head is `e2564b570abab360722d379c18955272165ec4ac`; neither remote baseline moved during this task.

## Automated results

| Check | Result |
|---|---|
| Full frontend `npm test` | **1,161 passed, 104 files**, no failures. Includes subprocess-based updater/provenance regressions; run in the normal cloud execution context. |
| TypeScript | Pass (`npm run typecheck`, also checked by the final production build). |
| ESLint | Pass, no warnings. |
| Localisation | Pass: 6 languages × 9 areas, 3,764 English keys, 129 components. |
| Corpus generation | Regenerated; corpus/terminology tests pass in the full suite. The generated audit retains 249 review findings, primarily shared spellings/proper names and existing editorial items; these are not a native-speaker approval. |
| Production frontend build | Pass. Existing Vite large-chunk advisory remains; no build failure. |
| Rustfmt | Pass. |
| Full Rust `cargo test --locked -- --test-threads=1` | **1,036 passed, 0 failed, 8 existing ignored**, 85 result groups including doctests. |
| Clippy `--all-targets --locked -- -D warnings` | Pass, no warnings. |
| Historical migrations | All 26 files match the reviewed SHA-256 manifest byte for byte; no migration added. |
| Complete workspace browser regression | Pass at 900/1100/1440 px with three panes, 640 px with the default two panes. Reading and Original EPUB Contents retain width/scroll and restore focus. Long Teaching selection remains contained; writing fields use their section width. |
| Native app/build smoke | Fresh disposable Linux profile: native dev build succeeds, app logs `startup complete`, schema ledger count/max 26/26, integrity `ok`, no foreign-key issues. No real Windows data used. |

The native on-disk lifecycle test covers saved and trashed Teaching, blocks/points, finding/citation snapshots, passages/materials and presentations, stable IDs, reopening Trash, restore and permanent Project removal. The native source tests use invented German Acts content and exact indexed segment IDs; the app test opens the corresponding chapter location rather than a plain mention.

## Development failures resolved

Initial runs exposed old section/mention order assertions, obsolete legacy-template expectations, locale address/terminology issues and an ambiguous language accessibility name. These were corrected with corresponding behavioral coverage. A new app regression initially selected a chapter instead of explicitly selecting its verse; its setup now follows the real passage/verse controls and verifies the reader's exact target. The final frontend suite above is a complete passing rerun.

The first full native attempt stopped on five old source-order assertions. Each was reviewed against the revised evidence rule; the 20-test source-quality suite then passed. Final full native results are recorded above after rerun. CLI tests originally hit subprocess `EPERM` inside the restricted command sandbox; the full frontend rerun in the normal cloud context passed without weakening those tests.

Screenshots identified the 18-pixel section selector and an overflowing Teaching session selector that isolated component checks missed. Both were corrected and the complete workspace measurements repeated.

## Review limits

- Windows WebView2, Windows picker dialogs, display scaling and the owner's copied schema-26 profile require target-platform acceptance.
- The actual MacArthur file and owner Windows screenshots were not available. Its exact section/page target remains unverified. A source without stored heading evidence is not promoted to commentary; whole-book PDF structure remains separate work.
- Structured PDF Reading is outstanding, with a separate implementation plan. Claude's recognition/provisional-import work and NOT READY verdict are unchanged.
- No Milestone B implementation, destructive migration, update publication, merge, paid Actions request or NiPogi access.

## Delivery

Implementation commit: `6b0d9a16fb6f7b9f239446cdb4c4c1f141b05f8d`. The following documentation commit records this report, screenshots and raw [check logs](checks/). Both delivery commits use `[skip ci]` to avoid remote workflow execution. Owner review is the stopping boundary.


Draft PR creation was attempted after pushing the implementation, using:

```sh
gh pr create --repo pistio/abidingword --base feature/windows-ux-milestone-a-review --head fix/windows-milestone-a1 --draft --title 'Windows Milestone A.1 corrections' --body-file docs/implementation/milestone-a1/draft-pr.md
```

Result: `Post "https://api.github.com/graphql": Forbidden`. No PR was created. The running managed environment has no custom allowed hosts and its effective host list omits `api.github.com`. The saved onboarding configuration draft already includes that host, but the running policy has not applied it. Git fetch/push to `github.com` succeeds. No credentials or TLS settings were changed.

The exact [prepared PR body](draft-pr.md) and [branch comparison](https://github.com/pistio/abidingword/compare/feature/windows-ux-milestone-a-review...fix/windows-milestone-a1?expand=1) make the correction reviewable while GitHub API access is unavailable. Create it as a **draft**, targeting the Milestone A feature branch; do not retarget it to `main`. The API restriction is a remaining delivery blocker, not a passing PR result.
