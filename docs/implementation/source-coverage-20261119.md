# Whole-source coverage and general PDF recovery

Owner follow-up, 19 November 2026, on draft PR #54. The owner approved the next
single Windows installer build while requesting general PDF behaviour and the
case of a whole work devoted to one Bible book, chapter or passage. These
additions are completed and checked before activating that approved build.

## Two different kinds of evidence

A source can concern Romans 8 without repeating every verse reference and
without providing a trustworthy page for each verse. Its relevance is useful
even when precise navigation is unavailable.

Whole-source coverage therefore belongs to the source, independently of its
page/segment destinations. The result can say **Covers Romans 8** and offer the
whole source. Opening it without a destination uses the reader's normal saved
position and mode. It does not turn the cover, first paragraph or most frequent
mention into a claimed passage location.

| Available evidence | Source recommendation and opening behaviour |
| --- | --- |
| A real book/chapter/verse destination containing the request | Keep structural descent and open the deepest reliable ancestor when deliberately requested. |
| Whole-source coverage, with no usable precise destination | Recommend the work with its coverage label; opening the source resumes reading. |
| Whole-source coverage plus incidental mentions | Offer the whole source and keep every mention as a separate explicit choice. |
| Mentions only | Retain the existing mention fallback. |
| Neither structure, coverage nor mentions | Do not manufacture a recommendation or page. |

Coverage can overlap a larger selected passage without claiming to cover all of
it. A work covering Romans 8 is relevant when Romans 7–9 is selected, but its
label remains Romans 8 and it supplies no invented destination. Structural page
navigation still requires containment of the requested passage.

Changing the workspace Scripture keeps the current source in place. **Go to
current passage** remains explicit. When only whole-source relevance is known,
it explains that no precise place is known and keeps the reader's position and
history. When mentions are available, it offers those choices.

## Automatic evidence and owner correction

Automatic coverage requires an unambiguous scoped title/subtitle, appropriate
bibliographic context and corroborating real structural evidence. Another
Bible book or a competing chapter/passage in the source structure can reject
the inference. A broader containing ancestor is considered separately from a
concrete competing section.

Literal-reference frequency is never evidence that the whole work covers a
passage. Series names, ambiguous titles, person names and ordinary uses of words
such as Job or Acts must not become whole-source claims merely through keyword
matching. The extraction and Scripture recognition pipelines are unchanged.

The shared **Edit details** form and Settings expose the same coverage choices:

- Detect automatically.
- Set coverage, with one natural-language or canonical reference per line.
- No specific passage, which persists suppression of an incorrect automatic
  whole-source inference.

Confirmed coverage supports a work without usable Contents or a precise page.
The last option affects whole-source inference; real structural destinations
and literal mentions retain their own evidence. Invalid or ambiguous manual
input rejects the metadata save atomically. An ordinary title/author edit that
does not change coverage omits the coverage update entirely.

## Storage and compatibility

Coverage uses namespaced JSON in the existing `sources.data` column and retains
unrelated JSON keys. It does not create synthetic `source_passages` rows, segment
IDs, page numbers or annotation anchors. Existing source identity, files, text,
citations and manual page links remain intact. A requested coverage write cannot
overwrite malformed legacy data; an unrelated metadata edit can preserve that
raw data unchanged.

The source result adds coverage alongside its existing locations. An empty
locations list is valid for a coverage-only result. The existing locations API
continues returning actual locations only. Effective coverage is recalculated
from current metadata and structural evidence, so no new database migration or
passage-index version is introduced. Whole-source choices survive a structure
rebuild and normal database backup/restore.

## PDF generality

The earlier correction has no production branch keyed to the supplied PDF's
filename, fingerprint, size, page numbers or font family. It uses typography
relative to body text, real native destinations/page labels and corroborated
printed-page mappings.

The [generality audit](pdf-extraction-generality-20261119.md) adds independent
invented fixtures for mixed Roman/decimal labels, duplicate labels, two-column
contents, different font families/body sizes with large covers, and a verified
destination with no text. All five variants pass without further extraction
changes. The actual private PDF remains additional evidence, not a production
special case or a public fixture.

These rules improve tested classes of PDFs without claiming universal success.
Image-only or broken text still needs OCR; genuinely ambiguous pages and
unstructured prose cannot safely acquire invented headings. A whole-source
recommendation provides a useful separate fallback when its scope is known.

## Verification before the approved build

- Final complete locked native suite: **1,137 passed, zero failed, nine ignored**.
  Formatting and all-target Clippy with warnings denied pass on the same final
  input files. Coverage inference/storage/navigation has 92 distinct focused
  checks; the five additional PDF variants are included in the full run.
- Final complete frontend suite: **1,362/1,364**. The known presentation
  clicker/Space assertion failed again, and a teaching autosave assertion failed
  in this run. Both passed in one unchanged focused invocation (two passed,
  96 filtered). The teaching failure is newly observed, not one of the two
  failures reported for the preceding candidate. Its output does not identify
  which saved field differed or establish the timing cause. Relevant teaching,
  presenter, shared control and harness files are unchanged from both prior
  candidates; neither fixture opens a source reader or imports a source.
  The focused pass does not make the complete run wholly green.
- All 24 new coverage UI cases pass in the full run. Typecheck, ESLint,
  translation consistency/corpus and the production frontend build pass.
- Final frontend and native inputs are unchanged during their recorded checks.
  No schema, dependency lock or separate Scripture recognition change is part
  of this follow-up.

The [verification record](source-coverage-20261119-verification.json) retains
counts, commands, exact changed-code hashes, the frontend limitation and the
intermediate corrections. The original actual-PDF findings remain in the
earlier source-navigation report; the extraction code is unchanged here.

## Approved delivery

The approved delivery uses the existing
`build/abidingword-i35-i42-20261109` branch and
`windows-combined-owner-20261109.yml` workflow, pinned to the final verified
application commit. There is one application compilation and one NSIS package,
with no build retry, merge, release, updater publication or deployment. Build
provenance and payload verification are recorded after that run. Windows desktop
acceptance follows normal installation and use; Linux tests do not establish it.
