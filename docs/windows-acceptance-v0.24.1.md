# Windows acceptance: v0.24.1 (PDF Scripture recognition)

v0.24.1 sits on top of v0.24 on the branch `claude/beautiful-curie-vm0eve`
and changes how PDF text is read (`docs/pdf-reading-order-v0.24.1.md`). It
has **no migration**: the schema stays at 22, and nothing already stored is
read again. Use this list together with `docs/windows-acceptance-v0.24.md`,
**after** the owner has chosen to merge and a signed test update has been
published by the normal process. Nothing here builds, signs or publishes.

Prepare: `engwebp_all.pdf` and `deu1912_all.pdf` from eBible.org
(`scripts/fetch-real-editions.sh` lists the addresses and checksums), and a
two-column PDF book of your own that is not a Bible.

**A — Reading a two-column Bible PDF**

- [ ] A1. Add a Bible → *From a document…* → `engwebp_all.pdf`. The review
      appears within about a minute and reports most chapters as usable;
      the refused ones include Psalm 119 and Song of Songs (labels inside the
      text) with a reason each.
- [ ] A2. Add it with partial use. Open Genesis 1, John 3 and Romans 8 in the
      document Bible: the verses read in order, with no running heads, page
      numbers or footnote text inside them.
- [ ] A3. Mark 9:44 ends "…the fire is not quenched.’" with **no** `*` or `‡`.
- [ ] A4. 1 Samuel 31:13 reads to its end ("…and fasted seven days.").
- [ ] A5. Repeat A1–A2 with `deu1912_all.pdf`: every chapter usable; Psalm 1:1
      begins "Wohl dem, der nicht wandelt…"; Offenbarung 17 and 18 are where
      they should be.

**B — Precise places**

- [ ] B1. In a document Bible passage, **Show it on page N** opens the PDF on
      that page and marks the first words of the verse, scrolled into view
      (WebView2: the mark must be visible without scrolling by hand).
- [ ] B2. A verse whose first words occur twice on its page opens the page
      without a mark (no wrong mark).
- [ ] B3. A place marked by hand before importing the same PDF again is still
      there afterwards, and navigation goes to it.

**C — Sources (library)**

- [ ] C1. Import your two-column PDF book as a source. Reading view shows
      the text column by column, not line across line; search finds a phrase
      that runs over a line end inside a column.
- [ ] C2. Original view: zoom, page search and marks work as in v0.24; an
      existing mark on an older PDF source is where it was.
- [ ] C3. A scanned PDF is still reported as needing OCR.

**D — Six languages**

- [ ] D1. Switch through English, Deutsch, 한국어, Français, Español,
      Português and open a refused chapter's reason in the review: no
      untranslated keys (`bibleImport.documentLabel`, `…ReadingOrder`,
      `…RunningHead`, `…RaisedLetters`, `…ChapterJump`).

## Rollback

No schema change: reinstalling the previous Development/Test build keeps the
database usable. Document Bibles imported with v0.24.1 stay as imported.
