# Independent review: canon and versification (A1.5)

**Reviewer:** a separate agent, read-only, code at `7c7565a` and the
disclosed 113-edition reports. Notes kept at review time; findings
below as reported, triage added.

## Findings

| # | Severity | Finding | Evidence | Triage |
|---|---|---|---|---|
| 1 | **Critical for any store keyed by book-chapter-verse** | Vulgate and Synodal Psalm numbering is stored under `PSA N` with the document's N, one off from English numbering, and verified. Only psalms whose count differs by more than 6 are refused. | `latVUC-epub`: 91 of 150 Psalms usable, 1 211 verified; same in `engdra-rtxtbrk`; `F12` (Russian Synodal) 1 140 verified | **Addressed**: `scripture::numbering::assess` marks the book `departs` (tests `a_vulgate_psalm_nine_departs`, `a_psalter_numbering_its_titles_departs`); the harness reports it per edition; handover §5 forbids joining editions by printed number |
| 2 | **Critical, same** | Hebrew numbering (Psalm titles as verse 1; Malachi 3:19-24) is kept and verified with no flag. The "Hebrew numbering" note exists only in the USFM import path. | `hboWLC-epub`: 150 of 150 Psalms usable, Malachi in 3 chapters | **Addressed** as finding 1 |
| 3 | Critical (unverified only) | The long-chapter tolerance (±6) lets another chapter's text stay *usable but unverified* under wrong references. | `G01`: REV 13-14 hold Rev 15-16 text, 35 usable wrong references, 0 verified | **Handover §4**: only verified verses are Scripture text |
| 4 | High | "Epistle Of Jeremiah" is read as Jeremiah; refused only because the chapter is too long. | `engwyc2017-epubapoc` | **Fixed**: "epistle of jeremiah" and "letter of jeremy" added; test `every_name_of_the_letter_of_jeremiah_is_the_book_outside_the_66` |
| 5 | Coverage | Hebrew-style chapter divisions refused as errors (JOL 3-4, LEV 6, NUM 16-17, 1KI 4-5, 1CH 5-6, JOB 41). | `hboWLC-epub`, `deutkw-docx`, `frajnd-epub`, `deuelbbk` | Documented |
| 6 | Coverage (refusal correct) | Catholic Esther 10-16 and Daniel 3, 13, 14 refused; Septuagint numbering refused; Greek additions left out. | `latVUC-epub`, `engdra`, `engbrenton`, `grcbrent` | Documented |
| 7 | Coverage | Book names known in nine languages only; others find nothing (reported, never attached to another book). | `F12` Russian, `G02` Swedish, `F13` Italian | Documented |
| 8 | Medium | "Standard numbering" (66 books, KJV-tradition chapters, maximum verse count over five English systems, ±6 tolerance) is undocumented to users. | `canon.rs:71`, `verse_counts.rs`, `structure.rs:57-65` | Handover §5 |
| 9 | Medium | No canon or versification is stored with the result. | `Recognition` | **Addressed** as finding 1 (the evidence is computed from the result; storing it is migration 0028's, handover §8) |
| 10 | Low | An unnumbered verse 1 is inferred, marked `unnumbered`, never verified. | `structure.rs:7366-7376` | Sound |

## Sound (reviewer)

* Deuterocanonical books are named and left out, never merged into a
  canonical book; unknown titles are never attached to the previous book.
* Missing verses are never fabricated: a gap keeps the printed numbers,
  warns, and un-verifies the verse before it.
* No report has a verified wrong reference or verified extra verse
  against the edition's own numbering.
* Septuagint detection is conservative (costs coverage, never misfiles).

## What the handover must state

References are the document's **printed** numbers under the 66-book
ordinals. "Verified" means the text matches the document at that printed
number, not that the number means the same verse in another edition.
The harness scores each edition against its own numbering, so it cannot
see findings 1-2.
