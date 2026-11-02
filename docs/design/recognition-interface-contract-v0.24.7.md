# Recognition ↔ provisional-import contract: v0.24.7 addendum

**Status:** a review for coordination with Codex's provisional-import work.
The v0.24.5 contract and the v0.24.6 addendum still stand, including the
correction about the migration number. This addendum lists only what v0.24.7
changes on the recognition side. Nothing here implements or changes a
migration or a stored schema.

## Migration numbering

The A.3 PDF branch already uses migration `0027_pdf_structure.sql`. The
provisional Bible import safeguards (standing column, review tables) take
the next free number on the A.3 lineage, provisionally **`0028`**. Do not
create a second `0027` or rewrite an applied migration. v0.24.7 adds nothing
that 0028 must store. Recognised document-derived Bible text stays
provisional whatever any evaluation shows.

## What does not change

* **No new unverified-reason values.** `boundary`, `possibleHeading`,
  `noteSign`, `reference` and `unnumbered` are reused. The values
  `bible_verse_places.reason` can hold are the same, so no CHECK constraint
  or consumer mapping changes.
* **Book identities:** no identifier, ordinal or versification table is
  added. A Kings-type title whose numbering the document leaves open now
  names **no book** (the book is left unnamed and not imported). The other
  path would risk 1 Samuel being filed as 1 Kings.
* **Existing data:** nothing is re-read, renamed or migrated.

## What changes in what the recogniser hands over

| Change | Interface effect |
|---|---|
| A chapter whose number would only be inferred from verse numbers starting again at 1 is refused | New import issue code `bibleImport.documentChapterUnmarked` (Warning, six languages). It is shown in the import review and is not stored as verse status. The refused chapter's verses are not handed over. |
| Text in Markdown raw HTML blocks is read | More verses may be handed over from Markdown files that use raw HTML. An unparsed note caller inside it gives `noteSign`, an existing value. |
| Contents pages and letter-spaced running heads are page furniture | Fewer misnamed books. No new values. |

Re-reading a document with v0.24.7 can therefore hand over fewer chapters
than v0.24.6 did, in exchange for none filed by inference. It can also hand
over more verses for documents v0.24.6 misread. The provisional-import revision
flow must treat both as an ordinary re-recognition.
