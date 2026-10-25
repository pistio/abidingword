# Windows acceptance: v0.24 on top of 0.5.0-dev.43

The Development/Test build on the owner's Windows computer is
**0.5.0-dev.43** (schema 19). The branch `claude/beautiful-curie-vm0eve` adds
v0.24: recoverable deletion, Settings → Sources, adding many sources at once,
document Bibles you can find passages in, German book names, Greek New
Testaments and verified downloads of original-language editions. This list is
for use **after** the owner has chosen to merge and a signed test update has
been published by the normal process. Nothing here builds, signs or publishes.

## 1. What changes for existing data

| Migration | What it does to an existing database | Risk |
|---|---|---|
| 0020 `library_organisation` | `state` and `state_changed_at` columns on sources and Bibles (every row stays *in use*); four new empty tables for collections and tags; two indexes | additive |
| 0021 `bible_passage_locations` | one new empty table (marked places in document Bibles) | additive |
| 0022 `import_jobs` | two new empty tables (batch imports) | additive |

The app first copies the database to
`backups\pre-migration\abiding-word-v19-<time>.db` and checks it. A dev.43
database upgraded to schema 22 is refused by dev.43 ("written by a newer
version"): going back means restoring that copy (or a folder copy), and work
done after the update is lost by going back. Tested automatically:
`tests/migration_v024.rs` (dev.43 → v0.24 keeps every source, Bible and
verse; everything in use; new tables empty) and `tests/windows_rollback.rs`.

Before updating, make a folder copy (**Settings → Data and backup → Back up
everything…**).

## 2. Checklist

Prepare: a folder with a few PDFs/EPUBs/Word/Markdown files (one duplicated
under another name, one scanned PDF, one in a subfolder named *Sermons*), a
Bible you own as a PDF with selectable text, ideally one with two columns.

**A — Deleting safely (Bibles and sources)**

- [ ] A1. Library: **Move to Trash…** on a book. The window is centred over a
      dimmed background, **Cancel** has the focus. Press **Enter**: nothing is
      deleted. Open it again, press **Escape**, then again click outside it:
      nothing is deleted. Focus returns to the button each time.
- [ ] A2. Move it to Trash; **Undo** brings it back with its marks.
- [ ] A3. Settings → Sources → **Show → Trash**: restore one; delete one
      permanently. Your original file in its own folder is still there.
- [ ] A4. Select three in Trash → **Delete permanently**: the button stays
      disabled until you type **3**; Enter in the field does nothing.
- [ ] A5. Settings → Bible texts → **⋯ → Archive** a Bible: it leaves the
      translation menu; **Restore** brings it back. Trash and restore one.
- [ ] A6. Close and reopen the app: everything is where it was.

**B — Settings → Sources**

- [ ] B1. Search by an author; filter by language and category; sort by
      title, then by date added (both directions).
- [ ] B2. Open a source's details, correct its title and category, add tags;
      its text and marks are unchanged.
- [ ] B3. Select several → **Organise → Add to a collection…** (new, inside
      another collection); filter by that collection. Rename the collection;
      remove it (the sources stay).
- [ ] B4. Archive several, find them under **Archived**, restore them.

**C — Adding many sources**

- [ ] C1. Settings → Sources → **Add many…** → **Choose a folder…** with
      *Include files in subfolders* ticked. Progress shows *Reading N of M*.
- [ ] C2. The review shows the duplicate as *Same file* (not ticked), the
      scanned PDF as *Cannot be added* with the reason, the file in *Sermons*
      as *Sermon — Suggested from its folder's name*, the others as *The usual
      category for this kind of file: check it*. Files of other kinds are
      counted as left out.
- [ ] C3. Correct a title; choose a new collection and two tags; **Add**.
      Progress shows *Adding N of M*; all added sources are in the collection
      with the tags.
- [ ] C4. Start another batch and press **Stop** while adding: the files
      added so far stay; the rest are not added.
- [ ] C5. Start a batch of several large PDFs and close the app while it adds.
      Reopen: Settings → Sources says the import was not finished;
      **Continue** finishes it without adding any file twice.
- [ ] C6. Add a Bible from USFM files of two different Bibles chosen
      together (two files of each): it is refused with both names.

**D — Document Bibles you can find passages in**

- [ ] D1. Add the PDF Bible **as a document**. In the Scripture pane the
      line under its title says no passages can be found yet.
- [ ] D2. **What can I do with this Bible? → Look for verses…**. If some
      chapters are doubtful, **Use the N verses found without doubt**; the
      Bible stays a document and the line lists the books found.
- [ ] D3. Type a passage in **Find** (one that was found): its verses appear
      in the edition's own words; **Show it on page …** opens the page.
      Compare a few verses with the page: same words, none missing or added.
- [ ] D4. Type a passage that was not found: *…has not been identified yet*;
      **Read it in …** opens your usual Bible instead. No other Bible's words
      are shown as this edition's.
- [ ] D5. Go to a page with Psalm 23 → **Mark a passage here** → type
      *Psalm 23* → **Save the mark**. Find Psalm 23: the mark is listed and
      opens that page. Remove the mark. Your notes on Psalm 23 are unchanged.
- [ ] D6. A two-column PDF: chapters whose columns were read across each
      other are reported (*verse … appears inside an earlier verse*), not
      used; other chapters can be.
- [ ] D7. A scanned PDF Bible still opens page by page; a page can be marked.

**E — German**

- [ ] E1. Interface Deutsch: the book list shows *1. Mose (Genesis)* …
      *5. Mose (Deuteronomium)*; a passage reads *1. Mose 1,1*.
- [ ] E2. Typing *Gen 1,1*, *Genesis 1,1*, *1. Mose 1,1*, *Dtn 6,4* all go
      to the right place. Exports (presentation, Materials) say *1. Mose 1,1*.
- [ ] E3. Spot-check German screens for *Sie/Ihr* addressing the reader:
      there should be none.

**F — Original-language Bibles**

- [ ] F1. Add a Bible → *Included*: the four Greek New Testaments are listed
      under Ελληνικά; the confirmation says *27 of the 66 books*. Add the
      SBLGNT; John 1:1 reads in accented Greek; the copyright line is in its
      details.
- [ ] F2. *Available to download*: OSHB shows 28.6 MB and MorphGNT 8.9 MB.
      **Download OSHB…**: the confirmation shows 40 files and the size;
      **Cancel** downloads nothing. Download it: progress is shown, then the
      usual review with the name and licence filled in. Add it; Genesis 1:1
      is in Hebrew with lexical numbers and grammar.
- [ ] F3. Disconnect from the internet: the added Bibles still work; a
      download attempt says it could not be completed and keeps nothing.

**G — Other canons**

- [ ] G1. Add a USFM Bible that contains Tobit or Greek Esther: the review
      names them as books Abiding Word cannot show yet; the 66 books are
      added as before.

**H — Six languages**

- [ ] H1. Switch through English, Deutsch, 한국어, Français, Español,
      Português on Settings → Sources, Add many…, a document Bible's passage
      panel and Add a Bible: no untranslated keys, no clipped text.

## 3. Rollback

If anything is wrong: quit, restore the pre-migration copy (or the folder
copy) as described in `docs/updates.md`, and reinstall 0.5.0-dev.43. Work
done after the update is lost by going back.
