# Teaching Stage 2 correctness handover

28 October 2026. Development only; no pull-request merge, installer, release, production data,
paid service or Stage 3 implementation.

## Exact ancestry and integration contract

Correction branch: `fix/teaching-stage2-output-integrity`.

| Source | Full SHA | Treatment |
| --- | --- | --- |
| Restored Windows baseline | `0551f22061b3b3219315930c5061be501e94f350` | Ancestor; main was not used |
| Original Teaching block implementation | `4e41542067b5635bfab611467bcdbc9cb5873599` | Preserved unchanged in ancestry |
| Original Teaching parent with handover | `66ae2df6dd9a86958e323ffbc704514df51d1bcd` | Merged into correction branch; original handover retained |
| PR #6 completion | `756d0097a07cf7105ce7c12313c0a380503b707e` | Correction branch starts here; completion is not cherry-picked |
| Correctness implementation | `314db4d16b35e079f2c894ab93cef949a544fd8e` | Repairs and regressions; documentation follows in a separate commit |
| Teaching-only ancestry merge | `e38bff1f4c06543dc923fb01647063aedc9632d7` | Parents are the PR #6 head and original Teaching handover tip |
| Current Windows branch / Library squash | `b3120f54b0dcece6c078727c97240828680dd63c` | Not an ancestor; Library work is deliberately absent |

The correction PR targets **`cursor/teaching-stage-2-completion-9953`**, not main.
This makes its implementation diff the repairs plus the original handover, while
the correction branch itself contains the complete Teaching Stage 2 history.
After owner review, merging the correction into that branch updates PR #6; then
PR #6 can be reviewed into `feature/teaching-three-templates`. Do not apply PR #6
again by cherry-pick, independently merge a second copy of its completion, or
merge the Windows/Library or simplification branches into this correction branch.
No such follow-up integration has been performed here.

The independent review remains on
[`review/codex-integration-report-2026-10-28`](https://github.com/pistio/abidingword/blob/review/codex-integration-report-2026-10-28/docs/implementation/codex-integration-review-2026-10-28.md).

## Reproductions and fixes

### Scripture privacy and document output

Four frontend regressions failed on the uncorrected source: private/missing-block
references and quoted Scripture leaked into questions, worksheets and manuscripts,
and an outline heading disappeared whenever its body was nonempty. They now pass.

`materials/derive.ts` filters passage associations before deriving either reference
lists or quoted Scripture. The policy is:

| Association | Participant output | Explicit leader/private-notes output |
| --- | --- | --- |
| Teaching-wide, no block or point | Included according to the material's normal rules | Same |
| Live Scripture block, public throughout its parent chain | Eligible | Eligible |
| Private block or private ancestor | Excluded | Eligible only for a leader question sheet or manuscript with notes |
| Missing/deleted block, malformed parent chain, unknown point, conflicting block/point association | Excluded | Excluded |
| Legacy point association | Existing point-scoped rendering is retained when its point exists | Same |

Worksheets remain participant-facing even when unrelated leader options are
present. Parent chains fail closed on missing parents, non-section ancestors or
cycles. Ordinary writing blocks honor inherited privacy too.

Block manuscript output now emits an outline heading before its body. Other
writing-area titles remain decorative labels rather than redundant headings.
The preview, print page, Markdown/copy and DOCX input share that derived model.
New paired frontend/DOCX fixtures test the actual Word package's heading/body
order and private Scripture exclusion, including an explicitly requested leader
version. Existing legacy fixtures are preserved.

### Deterministic ownership and independent slides

Singleton roles are `introduction`, `conclusion`, `central_message` and `notes`.
The rule is one live owner per role in a Teaching, with every other block's
writing retained independently:

- Duplicating a block keeps its title, body, privacy and guidance but clears its
  role. Outline copies still receive their own real point.
- Explicit create/update attempts to claim an already owned singleton role are
  rejected before mutation.
- Starting templates and imported packages retain the first claimant and demote
  subsequent conflicting claims to ordinary writing. Saved template definitions
  also normalize repeated singleton claims.
- Old conflicting sessions are read with the first-created live block as owner
  (creation timestamp, then row order, independent of display reorder). Read
  projection does not delete or rewrite writing; subsequent block mutations
  reconcile persisted role metadata and compatibility columns transactionally.
  Export writes canonical role metadata so moving a project does not choose a
  different owner simply because its blocks were reordered.
- Editing the owner updates its compatibility field; changing/removing its role
  or deleting it clears that field when no owner remains. Preparation-field
  writes and their block mirror now share a transaction.
- Native block restoration keeps the former role if free; if another block owns
  it, the restored block returns without the role and with its original writing.
  No live owner is displaced. This database helper is tested; no new restore UI
  or Tauri command was introduced. Whole-backup restore preserves its complete
  snapshot and is tested separately.

New block-based slides link to the specific block ID, including introduction,
conclusion and message blocks. Introduction/conclusion wording remains speaker
manuscript rather than automatically becoming audience prose. Earlier
preparation-field slides retain their compatibility behavior and follow the
single role owner. Duplicates cannot overwrite each other's new slides.

The implementation also completes the pre-existing `block` presentation-source
support in the Rust allowed kinds, TypeScript mirror and editor labels/text.
Previously the proposer emitted this kind but the shared source lists omitted it.

### Block references and lifecycle

The Rust database boundary accepts passage attachments only to a live Scripture
block in the same Teaching, with valid live section ancestry. A request cannot
attach one passage to both a block and a point. Unknown, deleted,
cross-preparation and ineligible identifiers are rejected without saved changes.
Structural type changes that would strand active Scripture references, points or
children are rejected. New blocks cannot silently convert legacy sessions.

Deleting a block tombstones its directly associated passages, finding links and
materials; it never turns them into Teaching-wide associations. Native block
restoration does not silently resurrect those removed references: reattachment
is explicit. Deleting a section still lifts its children, but children that
inherited privacy retain it when lifted, preventing accidental publication.

Project import validates block/point/parent ownership and eligibility, remaps
block source IDs, and refuses cross-preparation links transactionally. Missing
block/parent associations are left out and counted rather than nulled into public
Teaching-wide/root associations. The original package is not modified. Conflicting
role import, complete project round trip, slide-ID remapping and full backup
restore have dedicated regression coverage.

Presentations check visibility when proposing, explicitly attaching a source,
selecting the primary passage/title subtitle, and rebuilding saved output.
Block-linked Scripture, point or block slides that become private/deleted are
excluded, including saved overrides; the shared document feeds live display,
delivery packages, PDF, PowerPoint and Markdown. Explicitly independent,
presentation-owned text is still the author's own content.

## Preservation boundaries

- All 25 migration SQL files compare byte-for-byte with PR #6's `756d009…`.
  No new migration or schema conversion was introduced.
- Legacy sessions retain their original editor/data; template versions 1 and 2,
  all three built-in block templates, custom blocks and optional guidance remain.
- No Library, recognition, Bible Study research-block or simplification code was
  modified. Portability changes are limited to Teaching integrity.
- No dependency declaration or lockfile changed. Four block-validation error keys are supplied in all six languages; no new
  navigation or workflow labels are introduced.

## Validation evidence

Repair commit: **`314db4d16b35e079f2c894ab93cef949a544fd8e`**.

| Executed check | Result |
| --- | --- |
| Frontend reproduction on uncorrected Teaching source | 4 tests failed for the intended privacy/heading assertions |
| Native reproductions against unchanged PR #6 archive | 2 new tests failed for duplicate ownership and invalid block acceptance; 2 existing tests passed |
| Full frontend `npm test -- --maxWorkers=2` | **95 files, 1,104 tests passed**, no failures |
| Final focused frontend output/block regressions | **13 passed** across two files (subset of full-suite coverage) |
| TypeScript typecheck / ESLint | Passed |
| Six-language parity | Passed: 6 languages × 9 areas; 3,418 English keys |
| Frontend build | Passed; existing large-bundle warning remains |
| `cargo fmt -- --check` | Passed |
| `cargo clippy --all-targets --locked -- -D warnings` | Passed |
| Complete `cargo test --locked` | **1,016 passed, 8 existing ignored, 0 failed**; doc tests contained 0 cases |
| Native Teaching block suite | **7 passed**, including restoration and inherited privacy after section deletion |
| Native actual DOCX suite | **9 passed**, including new block-heading/private-Scripture fixtures |
| Native project/backup portability suite | **13 passed**, including conflicting-role import and rejected cross-preparation links |
| Migration / Windows schema-history suites | Passed in complete native run; includes schema 25 and dev.44 compatibility |
| Frozen migration comparison | All 25 SQL files identical to `756d009…` |
| Committed-source provenance | `--require-clean` passed at the exact repair SHA, schema 25, 25 migrations |

The eight ignored native cases remain ignored; they were not reported as passes.
An intermediate native run had compiled its library before the last section-
privacy implementation was saved, then picked up the new test and failed that
assertion. The final complete run rebuilt the stable source, passed that test and
finished with exit status 0. Earlier compiler/Clippy findings were corrected; the
final checks above passed. This evidence was executed in this task and is separate
from the earlier agents' reported results.

Full-suite logs in the current cloud instance are `/tmp/aw-stage2-frontend.log`
and `/tmp/aw-stage2-native.log`; final Clippy, parity and build logs use the
`/tmp/aw-clippy-final.log`, `/tmp/aw-stage2-i18n.log` and
`/tmp/aw-stage2-build.log` paths. Logs are not committed or assumed to survive a
new environment. The fixtures and regression tests are committed and reproducible.

Native setup used Rust **1.99.1**, installed with rustup, and signed Debian Tauri
development packages extracted into a workspace-local sysroot. Toolchain and
sysroot are outside the checkout under `/workspace/.tools`; initialization is
`source /workspace/.tools/aw-native-env.sh` in this prepared cloud machine.
Native builds use three jobs and no development/test debug information. This is
current-instance validation, not evidence that a new environment snapshot was
published or that Windows desktop acceptance occurred.

## Remaining work and owner review

Do not merge, deploy, publish installers, release or implement Stage 3 as part of
this delivery. After reviewing the correction PR, reconcile its updated PR #6
into the original Teaching line once; later integration into the Windows line
and Study simplification remains a separate owner-approved task. The Library Undo
finding from the independent review remains outside this branch.

Windows GUI/manual acceptance is still required. Unit/native export tests do not
certify WebView behavior or Windows usability. No paid AI services were exercised.

GitHub API access returned `Forbidden` while ordinary Git worked. The environment
draft now includes the required `api.github.com` domain; saving that draft does
not apply or publish it. If API access remains unavailable, the branch and exact
PR description will be pushed/prepared, but PR creation must wait for the user to
review/save the environment settings and publish the environment as requested by
the configuration tool. No token value is needed in chat.
