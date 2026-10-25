# Looking after the library: archive, Trash, Settings → Sources (v0.24)

v0.24 gives sources and Bibles a recoverable lifecycle, and the library a
place to be looked after: **Settings → Sources**. User guide pages:
[Your library](guide/05-library.md) and
[Settings](guide/08-settings-updates-backups.md). Windows acceptance steps:
[windows-acceptance-v0.24.md](windows-acceptance-v0.24.md).

## 1. States and what each one means

| State | In the lists, Sources pane, search, passage sources | Marks and citations | How to leave it |
|---|---|---|---|
| **In use** (`active`) | yes | shown | Archive, Move to Trash |
| **Archived** | not in the lists, the Sources pane or passage sources; **still found by search**, marked *Archived* | kept and still shown in Marks | Restore, Move to Trash |
| **In Trash** | no | kept, hidden | Restore, Delete permanently, or after 30 days |
| **Deleted permanently** | — | a source becomes the citation tombstone it always became (studies keep a record of what they cited); a Bible's text rows go | — |

The same states and the same commands serve **Bibles and sources**
(`db::library_manage`). A Bible leaving *in use* stops being the Bible read by
passage (the setting is cleared; another Bible is chosen as before). Notes,
Scripture marks and references belong to canonical verses, not to a Bible:
they are untouched by any of this.

`library_remove_source` and `bible_remove` (the older *Remove* commands) are
now the same as **Delete permanently** for one item, and like it they work
only **from Trash**: `remove_sources` / `remove_bibles` refuse an item that is
in use or archived, and a refused batch removes nothing. The interface no
longer offers a direct *Remove*; it offers Archive and Move to Trash.

## 2. Deleting safely

- **One transaction** per action: all items change, or none does.
- **Files after the database.** Only after the commit are managed copies
  deleted, and only inside Abiding Word's own folder for that item
  (`sources/<category>/<id>`, `sources/bibles/<id>`), never a path outside it. **A file outside Abiding Word's storage — the
  person's original — is never touched.**
- **Interrupted or failed deletion.** If a file cannot be deleted (locked,
  permissions) or the app stops between commit and deletion, the folder is
  left behind. At start-up `sweep_leftovers` looks at the folders inside the
  category folders: the folder of an item **deleted permanently** (its
  record kept as a citation tombstone) is deleted; a folder the database does
  not know at all is **not deleted but moved** to
  `leftovers/<category>/<id>` in the data folder, so nothing that might be
  the person's is lost to a database restored from an older backup. Folders
  of live items, archived or in Trash, stay. Only plain folders are touched
  (never a link), and only inside Abiding Word's own storage.
- **Retention.** At start-up `empty_expired_trash` deletes for good what has
  been in Trash for more than `TRASH_RETENTION_DAYS` (30), **one item at a
  time** (its own transaction, then its files), so one item that fails does
  not hold back the others. A failure is logged and retried next start.
- **Stale references.** A study citing a deleted source keeps its tombstone
  (title, author, excerpt) as before; marks of a trashed source are hidden,
  not deleted, and come back on restore.

## 3. The confirmation dialog

`design-system/ConfirmDialog.tsx`, used for every destructive action:

- centred, over a dimmed backdrop; `role="alertdialog"` with
  `aria-labelledby` (the question) and `aria-describedby` (what happens);
- **focus starts on Cancel**; Enter on Cancel cancels; Escape and a click on
  the backdrop cancel; focus returns to the control that opened it;
- the destructive button is always last and in the danger style;
- deleting **several** items permanently asks for their **number to be typed**
  (`typeToConfirm`); the button stays disabled until it matches, and Enter in
  that field never confirms.

After Archive or Move to Trash, a notice offers **Undo**.

## 4. Settings → Sources

- **List**: every source in a state (*In use / Archived / Trash*, with counts),
  paged (`MAX_PAGE` = 200), searchable by title, author or file name, sortable
  by title, author, category, date added, language and state; filters for
  category, language, format, collection, tag, *no readable text* (scanned
  PDFs) and *duplicates*. Searches escape `%` and `_`. With 3 000 sources a page loads in
  well under 2 s in the test (`sources_content_hash` and `sources_state`
  indexes; duplicates are sources sharing a content hash).
- **Each source**: its import warnings, extraction status and duplicates are
  shown; *Details* edits title, subtitle, author, publisher, edition,
  year, ISBN, DOI, language, category (now including *Commentary*), licence
  status, licence and notes — the text and its passages are untouched.
- **Collections**: user folders, nested (moving a collection into its own
  descendant is refused), one source in any number of them, the file never
  copied; renaming, moving and removing (removing a collection never removes
  its sources).
- **Tags**: many per source, shared case-insensitively, renamed and removed
  in one place.
- **Many at once**: select (or *Select all on this page*), then Archive, Move
  to Trash, Restore, Delete permanently, Add to a collection, Tag, Untag.
  `MAX_BATCH` = 5 000 per action.

## 5. Adding many sources at once (Part C)

**Settings → Sources → Add many…** adds several files, or a folder:

1. **Scan** (`library::batch::scan`): lists PDF, EPUB, Word, Markdown and
   text files, with or without subfolders. It reads no file's contents,
   follows no symbolic links, skips hidden and system folders, goes at most
   8 folder levels down and lists at most 2 000 files; files of other kinds
   and files beyond the limit are **counted and shown**, not silently dropped.
2. **Read and check, one file per step** (`library_batch_step`): the same
   extraction as a single import, then only what the review needs is kept
   (title, author, language, page count, content hash); the text is let go.
   So memory stays bounded for thousands of files, the interface shows
   progress, and **Stop** works between any two files.
3. **One review screen**: every file with its status — *Ready*, *Same file*
   (already in the library, or the same content earlier in this list; found
   by **content hash**; not chosen unless ticked), *Cannot be added* (with the
   reason, e.g. scanned PDF) — and per-file corrections of title, author and
   category. A **shared collection** (existing or new) and **tags** apply to
   every source added.
4. **Categories only from evidence**: a word in the title, the file name or
   the folder's name that says what it is (*commentary, sermon, lecture,
   article, notes* in the six languages, whole words only). Otherwise the
   file type's usual category is proposed and labelled *The usual category
   for this kind of file: check it*. A correction is labelled *Chosen by you*.
5. **Add, one file per step** (`library_batch_import_step`): exactly the
   single-file path (`commands::library::add_staged`: copy, then one database
   transaction, nothing kept on failure). The file is read again and must
   still have the reviewed hash; a file **changed since its check** is not
   added (*changed after it was checked*) and can be read again with **Try
   again**. **Stop** ends the job: files already added stay.

**Jobs persist** (migration 0022, `import_jobs`, `import_job_items`): a job
in progress when the app stops is marked *interrupted* at the next start and
shown in Settings → Sources (*Continue* / *Discard*). A file whose source was
saved just before the crash, before its item was marked, is recognised (a
source with the same content hash created since the job began and not linked
to any item) and linked rather than added twice. Discarding a job forgets the
list, never the sources added.

**Bibles of several files.** A USFM Bible's files are one edition when
chosen together. What a file's `\id` line says is used as evidence: after
removing the book's own names (`\h`, `\toc`, `\mt` lines and the English
name), its code, file names, numbers and date or time stamps ("Wed Mar 3
2021 10:15 GMT": weekday, month and time-zone words), a label that recurs (in at least two
files and a quarter of the files that have one) is taken as a Bible's name.
Two different recurring names refuse the import (*These files say they come
from different Bibles … add each one on its own*): different translations are
never combined. Beside one recurring name, another name found in two files or
more that has **no word in common** with it refuses the import too, however
few its files (two books of the King James Version among ten of the World
English Bible). Some editions put a different title per book on the `\id`
line; such labels do not recur and are not evidence. Duplicate books (two
files for John) and conflicting chapters were already refused; missing
chapters are reported per book. All bundled Bibles import unchanged with this
check (`tests/scripture.rs::every_bundled_bible_is_recorded_and_imports_cleanly`).

## 6. Migrations 0020 and 0022

Additive: `sources.state` (`CHECK` active/archived/trashed, default active)
and `state_changed_at`; the same two columns on `bible_translations`;
`source_collections` (parent with `ON DELETE CASCADE`), `source_collection_items`,
`source_tags` (unique name, `NOCASE`), `source_tag_items`; indexes
`sources_state`, `sources_content_hash`. Every existing row stays *in use*.
An older version of the app opening this database ignores the new columns and
tables (rollback is covered by `tests/windows_rollback.rs`).
Migration 0022 adds the two import-job tables (additive).

## 7. Tests

- Rust `tests/library_manage.rs`: archive/Trash/restore keeps everything;
  only trashed sources are removed and a refused batch removes nothing;
  retention; a Bible in Trash leaves the list and returns whole; details
  without touching text; nested collections refusing cycles; shared tags;
  3 000-source list speed; restart: expired Trash and leftover folders go,
  live files stay.
- Interface `tests/ui/sources-settings.test.tsx` (search, filters, bulk
  archive and Undo, Trash and typed confirmation, keyboard safety, details,
  collections), plus Bibles, library and passage-sources tests updated for
  Trash.
- Batch import: Rust `tests/library_batch.rs` (folder read one file per
  step, evidence-based categories, duplicates by content, problems, shared
  collection and tags, cancel, changed file and retry, crash recovery without
  adding twice, nothing readable), `library::batch` unit tests (scan scope,
  evidence words); `tests/scripture.rs` (different editions refused, one
  edition with and without labels accepted); interface
  `tests/ui/batch-import.test.tsx`.
