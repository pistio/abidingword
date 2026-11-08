# Handover: Scripture recognition Phase A1 + A2

For the owner's review. **Stopped here.** There is no Set H, no merge to
`main`, no pull request, release, installer, updater or deployment, and no
paid Actions. No AI service was called, and no database migration was made.
The detail is in `docs/scripture-recognition-a1-a2.md`.

## Exact SHAs

| What | Where |
|---|---|
| Frozen recogniser (baseline, unchanged) | `c96756d` |
| v0.24.7 branch with Set G result and assessment (unchanged) | `claude/scripture-recognition-v0.24.7` at `ea5f944` |
| **A1**: structure-aware EPUB reading and exact provenance | `claude/recognition-a1-structure` |
| A1 failing tests | `37e8822` |
| A1 implementation | `54dfc92` |
| A1 harness report | `ba09acb` |
| Optional-AI interface proposal (design only) | `57568b6` |
| A1 fixes from the regression | `8d02b97`, `205a453` |
| A1 measured state | `205a453` |
| A1 evidence docs | `3fe1475` and the commit adding this handover |
| **A2**: PDF engine measurement | `claude/recognition-a2-pdf-engines` at `c5d81e3` |

Set G's blind result, the v0.24.7 report and every earlier evaluation record
are untouched. Sets B–G are disclosed, and nothing here is independent
evidence.

## What was delivered

1. **A working structure-aware EPUB/HTML prototype.** It is isolated and off
   unless `structural::stage_documents` is called.
   * Link roles come from where a link leads, never from its label.
   * Note callers are marked, not cut.
   * Note blocks are found from the links; floated and absolutely
     positioned blocks are apparatus.
2. **Exact span provenance**, with tests (18). Each verse records:
   * file and entry identity with SHA-256;
   * ordered character and byte spans;
   * explicitly excluded ranges;
   * a deterministic quote;
   * the trace status, the id check and the reasons.

   Nothing is invented between spans.
3. **A measured PDF engine comparison** (A2): 57 checksummed PDFs and five
   engines, with extraction separated from recognition.
4. **A regression against the frozen recogniser** on every disclosed EPUB.
   * Frozen reading: identical to `c96756d`.
   * Structure-aware reading: verified wrong text falls from 4 152 to 8 (all
     8 pre-existing), verified verses not in the edition from 5 to 0, and
     verified verses rise from 500 071 to 517 281.
   * G01: 4 144 wrong to 0, coverage 28.2 % to 78.1 %.
5. **The integration recommendation for Codex** (§5 and §6 of the report):
   * a call site behind an off-by-default setting, reusing the existing
     unverified reasons;
   * a proposed provenance table for after 0028, with no number reserved;
   * the `PdfGlyphSource` shared interface.
6. **This handover.**

## Recommendation for Codex

* **A1.** Wire `structural::stage_documents` as an opt-in import path once
  0028's provisional standing exists. Its refusals arrive as ordinary
  `boundary` or `noteSign` unverified verses, so 0028's gates need nothing
  new. Persisting provenance is a later, separate migration, proposed in
  report §5.
* **A2.** Do not change the production PDF backend yet. Codex's Sources
  library and Bible import should agree the `PdfGlyphSource` interface
  first. Then measure PDFium glyphs through `pdf_layout.rs` against the
  current path on the A2 corpus, keeping pdf-extract as the fallback.
* **Shared files.** No A1/A2 code touches Codex's A.4 branches, migrations
  0027/0028, Windows acceptance work, Source navigation (#22, #34, #35) or
  user databases.

## Unresolved risks

1. **Generalisation is unproven.** Every number is on disclosed documents,
   and three faults were found only by running the whole corpus. A1 needs a
   new blind set, built after a freeze, before any claim of reliability.
2. **The gate is only as good as its evidence.** Ids and verse-number
   classes are measured per document. A generator whose ids follow an
   unseen scheme gives "no evidence", which is safe but adds no protection.
3. **Headings in the text flow** (7 `francl` verses, 1 `porbr2018`) are
   still verified with extra words. A1 does not address non-link structure.
4. **Coverage cost.** 16 correct `porbr2018` verses are refused because the
   recogniser removes inline quotation references by text pattern without
   reporting the range. The recogniser should report its removals.
5. **Untraced text across entries** (145 verses in G01). The tracer works
   within one entry; verses spanning two entries are refused.
6. **Performance.** The structure-aware reading is about 22 % slower in the
   test build, and its peak memory is higher (about 55 MB of source model per
   Bible). An optimised build was not measured.
7. **PDF.** PDFium through the app's layout code is unmeasured, especially
   for Hebrew. Release-build timings of the current extractor were not
   taken. CVE history and MuPDF commercial terms could not be checked
   online.
8. **Interface strings.** Provenance reasons are English, for audits. Any
   review-UI exposure needs the six-language i18n work.

## Next steps (each needs the owner's decision)

1. Review A1 and A2 separately (two branches; pull requests only if asked).
2. If A1 is accepted:
   1. freeze it;
   2. have a fresh, isolated builder make a new blind EPUB-heavy set;
   3. seal it and run it once.
3. If A2 is accepted, prototype the PDFium glyph adapter behind
   `PdfGlyphSource`, coordinated with Codex. Then measure it on the A2
   corpus against the current path, with no backend switch.
4. Recogniser follow-ups found by A1: report removed ranges, and handle
   headings run into verses. These are rule changes to the frozen
   recogniser, so they need the owner's go-ahead.
5. AI assistance stays design only (`design/recognition-ai-assistance-interface.md`)
   until the measurement gate it describes can be run.
