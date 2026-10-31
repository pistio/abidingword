# Recognition ↔ provisional-import interface contract (review, v0.24.5)

**Status:** review only. Nothing here is implemented on the recognition
branch, and migration 0027 is **not** implemented here: it belongs to Codex
on the schema-26 integration lineage
(`integration/windows-library-stage3-teaching-study`, which at this review
ends at migration 0026, `e2564b5`). This note states what the recogniser
hands over and what the provisional design
(`provisional-bible-imports-v0.24.4.md`) must keep distinct, so the two lines
of work meet without either duplicating the other.

## 1. Four kinds of text, never merged

| Kind | Produced by | Stored today | Standing every consumer must respect |
|---|---|---|---|
| **Structured-source verse** | USFM, OSIS/verse maps, OSHB, MorphGNT, bundled texts: the source *declares* book, chapter and verse | `bible_verse_texts`; no row in `bible_verse_origins` | Verified by its source. The only text that may be called verified Scripture. |
| **Recognised provisional verse** | The document recogniser, verse found **and** its text verified by the recogniser's rules (`DocumentStage.unverified` has no entry for it) | `bible_verse_texts` + `bible_verse_origins` (`basis` = `automatic` or `corrected`) | **Provisional** (0027 `text_standing = 'provisional'` on the Bible). The recogniser's "verified" means "no rule found a doubt", never "checked by a person". Set D shows such text can still be wrong. |
| **Individually unverified verse** | The recogniser, verse found but its text in doubt (`unverified` reason: `boundary`, `possibleHeading`, `noteSign`, `reference`, `length`, `psalmTitle`, `unnumbered`) | `bible_verse_places` (reason, pages, segments); **no** text in `bible_verse_texts` | A place only: "not checked — open the page". Never quoted, searched as Scripture, sent to the assistant or put in materials. |
| **Original document text** | The document itself | `bible_documents`, `bible_document_segments`, `bible_document_search`, `bible_document_apparatus` | The document, shown as a document (with its notes, running heads and headings). Never a verse. |

A Bible read from documents holds the second, third and fourth kinds at
once; a USFM Bible holds only the first. The Bible-level standing (0027)
answers "may this Bible's verse text be used as Scripture"; the per-verse
facts (origins, places) answer "where did this verse come from".

## 2. What the recogniser guarantees to the import (unchanged in v0.24.5)

1. Every verse in `content` has either an origin (`origins`) or comes from a
   structured source; a verse the recogniser doubts is in `unverified` and
   the import writes it to places, not texts.
2. Books outside the 66 (`TOB`, `1MA`, `ESG`, `DAG`, …) are **named and left
   out**: no text of theirs reaches `content`. v0.24.5 adds Septuagint
   evidence for this (a Septuagint's Esther and Daniel are `ESG`/`DAG`, not
   `EST`/`DAN`); no new identifier, table or column is needed for it, and
   existing Bibles keep their rows (nothing is renamed or re-filed).
3. Refused chapters (structure errors) produce no verses at all.
4. Recognition is deterministic: the same document gives the same verses,
   places and refusals on every run.

## 3. What the consumer side must add (Codex, 0027 and the gate)

* A recognised provisional verse must never be presented as a
  structured-source verse: every consumer goes through the gate of the
  design's §4, so a reader cannot forget it.
* An individually unverified verse is never upgraded by the Bible-level
  standing: `text_standing` applies to `bible_verse_texts` rows only; places
  stay places whatever the standing.
* A sample review (design §5) records judgements; it never changes
  `text_standing`, never moves a place into texts and never certifies the
  edition. A review of twelve verses says nothing about the thirty thousand
  others; a recognised Bible stays provisional for its whole life.
* The original document text stays reachable from every recognised verse
  and place (origins/places carry segments and pages), so a person can
  always check a verse against the page it came from.

## 4. Points to settle with Codex (no change made here)

1. **Corrected verses.** `basis = 'corrected'` (the person chose the book)
   is still provisional text: the person corrected a book's identity, not
   the verse's words.
2. **Re-recognition.** `bible_document_recognise` after a recogniser upgrade
   may move verses between texts and places. The 0027 revision counter
   should bump on it, and earlier reviews show as "for an earlier reading".
3. **Outside books.** The import's note for a left-out book (`ESG`, `DAG`) is
   the only trace of a Septuagint's Esther and Daniel; if the product later
   stores such books, it needs their own ordinals (`canons.rs`, above 66) and
   their own versification, never the Hebrew canon's `EST`/`DAN`.
