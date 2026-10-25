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
| **Archived** | no (still found in Settings → Sources, marked *Archived* when found by search there) | kept, shown when restored | Restore, Move to Trash |
| **In Trash** | no | kept, hidden | Restore, Delete permanently, or after 30 days |
| **Deleted permanently** | — | a source becomes the citation tombstone it always became (studies keep a record of what they cited); a Bible's text rows go | — |

The same states and the same commands serve **Bibles and sources**
(`db::library_manage`). A Bible leaving *in use* stops being the Bible read by
passage (the setting is cleared; another Bible is chosen as before). Notes,
Scripture marks and references belong to canonical verses, not to a Bible:
they are untouched by any of this.

`library_remove_source` and `bible_remove` (the older *Remove* actions) now
move to Trash; permanent deletion is only possible **from Trash**
(`remove_sources` / `remove_bibles` refuse anything else, and a refused batch
removes nothing).

## 2. Deleting safely

- **One transaction** per action: all items change, or none does.
- **Files after the database.** Only after the commit are managed copies
  deleted, and only inside Abiding Word's own folder for that item
  (`sources/<category>/<id>`, `sources/bibles/<id>`), never a path outside it. **A file outside Abiding Word's storage — the
  person's original — is never touched.**
- **Interrupted or failed deletion.** If a file cannot be deleted (locked,
  permissions) or the app stops between commit and deletion, the folder is
  left behind. At start-up `sweep_leftovers` removes folders inside the
  category folders whose id has **no live item**; folders of live items stay.
- **Retention.** At start-up `empty_expired_trash` deletes for good what has
  been in Trash for more than `TRASH_RETENTION_DAYS` (30). A failure is logged
  and retried next start; nothing is half-removed (each item is its own
  transaction plus file step).
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

## 5. Migration 0020

Additive: `sources.state` (`CHECK` active/archived/trashed, default active)
and `state_changed_at`; the same two columns on `bible_translations`;
`source_collections` (parent with `ON DELETE CASCADE`), `source_collection_items`,
`source_tags` (unique name, `NOCASE`), `source_tag_items`; indexes
`sources_state`, `sources_content_hash`. Every existing row stays *in use*.
An older version of the app opening this database ignores the new columns and
tables (rollback is covered by `tests/windows_rollback.rs`).

## 6. Tests

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
