# Recognition ↔ provisional-import contract — v0.24.6 addendum

**Status:** review for coordination with Codex's provisional-import work
(migration 0027 and the consumer gate, `provisional-bible-imports-v0.24.4.md`).
Nothing here implements or changes migration 0027 or any stored schema. At
this addendum Codex's integration lineage
(`integration/windows-library-stage3-teaching-study`) still ends at migration
0026 (`e2564b5`). The v0.24.5 contract (`recognition-interface-contract-v0.24.5.md`)
stands; this lists what v0.24.6 changes on the recognition side.


> **Correction — migration number (owner, at the set F authorisation).** The
> A.3 PDF branch already uses migration `0027_pdf_structure.sql`. The
> provisional Bible import safeguards (standing column, review tables) must
> therefore take the next free number, provisionally **`0028`**, when they are
> integrated onto the A.3 lineage. No conflicting `0027` is to be created and
> no applied migration rewritten. Wherever this addendum, the v0.24.5
> contract or the v0.24.4 design say "migration 0027" for the provisional
> import, read "the provisional-import migration (0028 on the A.3 lineage)";
> those earlier records are kept as written. Recognised document-derived
> Bible text stays provisional whatever any evaluation shows.

## What does not change

* **No new unverified-reason values.** Every v0.24.6 rule uses an existing
  reason (`boundary`, `possibleHeading`, `noteSign`, `reference`). The
  values `bible_verse_places.reason` can hold are the same as before, so no
  CHECK constraint or consumer mapping has to change. Any future new value
  will be proposed to Codex before it is used.
* **The four kinds of text** stay distinct: structured-source verses;
  recognised verses (provisional for the whole Bible, whatever the
  recogniser's per-verse result); individually unverified verses (places
  only); the original document text.
* **Book identities and versification:** unchanged. Books outside the 66
  stay named and left out (`ESG`, `DAG`, `TOB` …); no identifier, ordinal or
  versification table is added.
* **Existing data:** nothing is re-read, renamed or migrated by this work.

## What changes in what the recogniser hands over

| Change | Effect on the import / consumers |
|---|---|
| Markdown footnotes (`[^id]` callers, `[^id]: …` definitions) are read as notes | Definitions are set apart as notes, as EPUB and Word notes already are: never verse text, counted in the import's note line count, and still part of the original document text kept with the Bible. Callers are taken out of the verse and kept as apparatus (`noteCaller`, kind `footnote`) with their verse. |
| A chapter label whose number cannot be read refuses that chapter | New import issue code `bibleImport.documentChapterUnread` (Warning; six languages). Issues are shown in the import review, not stored as verse status. |
| Greek alphabetic chapter numerals are read | More chapters found in Greek documents; nothing else changes. |
| More verses held back as unverified (running-head, Selah, look-only heading and note-tail rules) | More `bible_verse_places` rows and fewer `bible_verse_texts` rows for some documents. |

## Dependencies on the provisional-import architecture (unchanged, restated)

1. **Provisional standing and the gate** (0027, Codex): until they exist,
   recognised verse text cannot be presented as anything but what the
   current branch shows, and the NOT READY status holds for any user-facing
   release.
2. **Re-recognition and review staleness:** reading an existing Bible again
   with v0.24.6 moves verses between texts and places; the 0027 revision
   counter must make earlier sample reviews show as stale.
3. **Known open finding protected only by provisional standing:** a
   recogniser-verified verse can still be wrong on unseen layouts (v0.24.5
   set E). A sample review never certifies an edition.
