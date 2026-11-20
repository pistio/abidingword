# Database-only recovery from daily snapshots — Windows acceptance

**20 November 2026 · draft, not accepted on Windows · stacked on PR #55.**

## Why this is a separate recovery path

Abiding Word automatically stores at most seven verified daily SQLite copies under `backups/daily/`. They preserve **Studies, Teaching, notes, citations, source metadata and reading places** from the database. They do **not** include imported PDFs, Bibles, images or any files in `sources/`. A daily snapshot is **not** a portable `.awbackup` and must never be presented as a full backup.

The new **Settings → Data and backup → Automatic recovery points** workflow deliberately requires the user to request a list, select an eligible record and confirm a full database replacement after seeing the warning. On success the app tells them to reload.

## Native safety contract

1. The native API accepts a strictly parsed filename identifier, **never** a user-supplied absolute path. The file must lie in the app's own daily-backup folder. Symlinks, directory traversal and unrelated filenames are refused.
2. Show damaged or schema-incompatible candidate files as **unavailable**, not as selectable. Only same-schema, integrity-checked, migration-history-valid snapshots may be used.
3. Recheck the candidate immediately before restoring; copy it into a private staging file and validate the copy again, so the bytes restored are those that were checked.
4. Before any mutation, write and verify a full **database-only safety copy of current work** to `backups/pre-restore/`. The previous SQLite database must remain recoverable.
5. Restore through SQLite's backup API under the existing database mutex. Run the ordinary post-restore integrity/startup checks. If anything fails, automatically put the safety copy back, or report its exact location for manual recovery.
6. **Never move, replace, copy or delete managed source, Bible, image or other files.** Existing source-file mismatches or missing files remain the user's responsibility. No automatic upload or external call.
7. The original daily point and any damaged copies remain intact. This workflow uses no schema migration, frozen recognizer code, Set H documents, signing, updater or release service.

## Required Windows owner verification

Use only a disposable Windows profile with invented Study/Teaching data and an unrelated PDF/EPUB. Never restore a daily point against the owner's live personal profile in acceptance testing.

- Write a fictional Study, Teaching, private leader block, participant question and citation, open a PDF in Original mode, create a complete `.awbackup` and record its checksum.
- Create a daily copy, edit both Study and Teaching and confirm the newer words are saved. Select **Show recovery points**; a date and database version are visible, with corrupted/older-schema candidates disabled.
- Review the restore and press **Cancel** once: all current writing and files remain exactly unchanged.
- Confirm on the disposable profile. Verify that the older Study/Teaching text returns, the previous newer text is preserved in `backups/pre-restore/`, and that all managed imported PDF/EPUB bytes are unchanged.
- Check old citations with files that are still present and deliberately missing in the test folder; no fabricated page destination or silent file restoration.
- Repeat with a corrupt snapshot and a tampered filename; the live database must not change, no safety copy should be created, and errors should be understandable in EN/DE/KO/FR/ES/PT.
- Reopen the app and verify writing, private/participant export separation and recent-reader state.
- Separately test a **full** `.awbackup` restore into a second disposable installation, showing that original documents also come back.

**Acceptance status:** implementation plus automated tests pending full CI; no Windows owner acceptance. This branch must remain draft until independently reviewed and tested. It does not certify the overall release.
