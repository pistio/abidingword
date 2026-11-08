# Issue 42 — editorial coverage and owner review

This branch follows the targeted #35/#33 branch at
`38c785be723f4ff4ca7ba8c885aeabff3a89df7a`. Changes are separated into native
dialogs/errors/Source diagnostics, Teaching/Settings wording, display formatting,
and generated review evidence. No paid Actions, installer, updater, merge,
release or deployment was performed.

**Status: first AI editorial tranche, not complete whole-application editorial
acceptance.** The inventory is comprehensive; individual linguistic adjudication
of every unchanged string and every source candidate remains open. Neither
generated coverage nor a passing key check is presented as native-speaker
approval. No human native-speaker proofreading has occurred in this task.

## What is covered

- The complete current six-language corpus is regenerated in
  [native-polish-review-v0.19.md](../native-polish-review-v0.19.md), including
  original English and German, all nine namespaces and plural branches. The
  existing v0.18.10 terminology decisions remain the starting glossary.
- [inventory.json](inventory.json) fingerprints every TS/TSX and Rust source
  file inspected by the inventory (375 files), records every statically named
  native UserMessage code (216), and lists possible hardcoded source strings.
  All 216 codes have translations in all six languages, including plural-only
  keys. Candidates include CSS, logs, provider prompts, technical metadata,
  product names and UI text; a candidate is **not** a confirmed visible leak.
- All locale keys are inventoried, not just the strings changed since v0.18.10.
  Per-language files below list every individually edited key, its original and
  revised wording, context and reason. They explicitly distinguish that work
  from unchanged inventory entries. Do not close #42 on inventory coverage alone.
- A source audit found English native file-dialog titles/filter labels, an
  unsupported-link error, raw English PDF diagnostic verdict/evidence, twelve
  missing native Library/material/word/field messages, and a plural-key existence
  bug. These paths now use the active interface language. Unknown coded errors
  use a localised generic failure rather than exposing an English technical
  fallback; the stable error code remains available on the API error.
- Timeout copy now explains the outcome, lack of saved changes and next action,
  rather than “a safe processing boundary”. Duplicate identical timeout notices
  are suppressed. Structure-based evidence, manual links, inherited context,
  Contents entries and literal mentions remain distinct. Technical JSON exports
  retain their original schema/diagnostic fields; UI wording is localised.
- Brazilian Portuguese Source processing/diagnostic text uses arquivo, salvar,
  detectado, compartilhar and natural progressive forms; action references match
  the actual buttons. Teaching and AI-key labels receive focused register and
  terminology corrections. German Bibelstudium/Bibellehre distinctions and Korean
  성경공부 labels are restored where inconsistent or duplicated. Optional Spanish
  POZETEK application guidance uses the established tú register without changing
  the method. Bible quotations retain their own original register.
- Dates explicitly use en-GB; Portuguese formatting uses pt-BR. Stored language
  IDs, timestamps and data are unchanged. Presenter time, number/size formatting
  and displayed conjunction lists use the same regional policy. This does not
  change Bible recognition, book identities, imported Scripture or parsing.

## Feature-area matrix

| Area | Review/evidence in this tranche | Still open |
| --- | --- | --- |
| Common/navigation/accessibility | Complete key/source inventory; registers, British formatting, native picker captions, narrow-pane screenshots | Individually adjudicate all unchanged labels and ARIA/keyboard paths; Windows high-DPI checks |
| Library/Source/PDF/EPUB/DOCX | Native-message coverage, headings/diagnostics/timeout copy, raw English leaks, pt-BR review; six-language timeout screenshots | All non-PDF reader states and original-file Windows acceptance |
| Scripture/books/canon/references | Existing registry names preserved; plural-native-error fix; no imported text or recognition changes | Independent linguistic pass over every remaining grammar/import/edition warning; do not infer canon approval from key coverage |
| Bible Study/POZETEK | Source-backed compact hints/Kern triplet in parent; terminology and Spanish register corrections; writing/repair/Undo regressions | Confirm specialist Kern terminology with owner/native reviewers; original unpublished guide unavailable except owner's issue excerpts |
| Ten-Step | Accepted guidance and resources byte/wording protected; no substantive or editorial change | No new method acceptance requested |
| Teaching/Q&A/templates | Three source-backed hints in parent; German/Korean/Portuguese workspace terminology; template/privacy terminology preserved | Full individual pass of all remaining templates/Q&A and exports; EVA/Bible Lesson primary guidance not supplied |
| Materials/presentation/exports | Materials error localised; privacy distinction checked in all six languages; presenter clock variant; existing glossary retained | All generated-document/slide/error contexts in six languages, long text and high DPI |
| Optional AI/settings | All-six-language review of assistant intro, key storage, model/search and image-consent wording; pt-BR action references corrected | Every provider-specific context; live DeepSeek Korean correction remains owner acceptance |
| Backup/restore/lifecycle | All-six-language review of backup privacy, key exclusion, replacement confirmation, recovery copy and file-preservation warnings; retained | Remaining destructive-state/restore UI contexts on Windows |
| Native/Tauri/error handling | 216 statically named codes × six locales; plural and unknown-code regressions; translated picker options tested | Dynamically composed/uncoded paths and platform-owned dialog chrome need contextual adjudication |

The privacy wording checked in all six languages preserves: only slides reach
the audience; private Teaching notes stay out of participant Materials; AI is
optional and works on request; API keys stay in the local password store and
out of backups; full backups are private and restore replaces current work with
a preceding safety copy. These checks are linguistic, not a new security audit.

## Independent language notes and evidence

The complete corpus contains 4,015 logical English strings (4,156 English keys
including plural forms). Raw key counts are EN 4,156; DE 4,156; KO 4,016;
FR/ES/pt-BR 4,296 each. These differences reflect the locales' plural forms.
This tranche individually edits 269 keys across the six languages; that count
does not imply individual adjudication of every other key.

| Language | Register/glossary decisions | Change log | UI evidence |
| --- | --- | --- | --- |
| en-GB | British spelling/date order; Bible Study, Bible Teaching, Materials and manuscript distinctions retained | [English](en.md) | [Before](images/en-before-timeout-900.png) / [after](images/en-after-timeout-900.png) |
| de | Familiar du; Bibelstudium versus Bibellehre; no new teaching methodology | [German](de.md) | [Before](images/de-before-timeout-900.png) / [after](images/de-after-timeout-900.png) |
| ko | Respectful endings; 성경공부 / 성경 가르침 / 틀; remove duplicated 성경; distinguish foot/endnotes from commentary | [Korean](ko.md) | [Before](images/ko-before-timeout-900.png) / [after](images/ko-after-timeout-900.png) |
| fr | Consistent vous; curly apostrophes; nonbreaking spaces in new quoted-name messages | [French](fr.md) | [Before](images/fr-before-timeout-900.png) / [after](images/fr-after-timeout-900.png) |
| es | Existing tú; Spain-leaning established vocabulary (ordenador, añadir), broadly understandable; no switch to vosotros or regional Bible-name changes | [Spanish](es.md) | [Before](images/es-before-timeout-900.png) / [after](images/es-after-timeout-900.png) |
| pt-BR | Você, arquivo and salvar; Brazilian progress forms; preserve meaningful keep/save distinctions | [Brazilian Portuguese](pt.md) | [Before](images/pt-before-timeout-900.png) / [after](images/pt-after-timeout-900.png) |

The parent also includes [24 Kern screenshots/checks](../../implementation/owner-followup/images/browser-checks.json)
for all six languages at 900/1440px. [Twelve timeout cases](images/browser-checks.json)
exercise the real React UI with an invented profile and synthetic native failure.
No page errors or dialog horizontal overflow were observed. These are Linux
Chromium tests, **not Windows WebView2 or actual MacArthur acceptance**. Native
dialog options are unit-tested; OS-owned dialog buttons require Windows review.

## Reproduction and preservation

```sh
node scripts/i18n-editorial-inventory.mjs
npm run i18n:corpus
npm run i18n:check
python3 scripts/test-editorial-contract.py
python3 scripts/test-pdf-index-query.py
```

The contract test preserves every existing key, placeholder name and ICU plural
branch, all Ten-Step wording, native source/resources and Bible names relative
to this tranche's parent. All migrations, schema 27, reserved 0028 and original
sources remain untouched. UI-language changes never rewrite user writing.
The #35 offline query-work regression checks the production SQL uses the source
index and stays below 100,000 VM steps for 4,112 links (observed 45,200).

Verification: typecheck, ESLint, localisation and corpus checks, the editorial
preservation contract, the query-work regression, and 36 recorded browser cases
(24 parent Kern cases plus 12 timeout cases). All 216 statically named native
codes are covered in six locales; plural-only and unsupported-code cases have
dedicated runtime regressions. Rust checks on the unchanged native source pass:
1,095 tests, eight existing ignored; formatting and locked all-target Clippy with
warnings denied. Local locked Rust release compilation passed in 7m46s.
Frontend production build passed with its existing large-chunk warning. The
final clean frontend run passed all 1,246 tests in 117 files, with zero failures.
The local native
release build is Linux compilation evidence, not a Windows installer. Do not
dispatch an old acceptance workflow and assume it builds these branches: the
existing installer workflow is still pinned to its separately reviewed source.

## Owner gates

Keep #35/#33/#42 and PR #37 owner gates open. The real MacArthur PDF must rebuild
and persist on Windows, then an explicitly entered Apg 6 diagnostic must show
the actual target. Source ranking changes are not claimed here. Windows layout,
original PDF, real DeepSeek Korean correction, privacy-safe exports and all A.4
acceptance gates remain intact. Do not publish the private workshop/guide files.

Whole-application individual editorial adjudication, remaining source candidates,
unsampled UI contexts and qualified native-speaker proofreading are explicitly
outstanding. This draft supplies a reproducible complete inventory and concrete
reviewed corrections; it does not substitute those for the exhaustive approval
requested by #42.
