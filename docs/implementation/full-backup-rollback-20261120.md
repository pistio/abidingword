# Full backup restore — file-phase safety and rollback

**20 November 2026 · draft implementation stacked on PR #56 · issue #57.**

## Release blocker identified

A complete `.awbackup` restore previously committed its SQLite database before completing the managed-file phase. A disk I/O error, permission error or collision while restoring a later imported PDF, Bible file or image could return an error **after** the database had already been replaced. A directory occupying a filename was merely logged and skipped, allowing a successful result with a missing imported file.

This draft changes the order and adds a rollback log **in memory**:

1. Verify the complete backup package and stage every file under the application's cache. Reject bad hashes, unsafe paths, extra files, newer schema and malformed manifests.
2. Make and verify the pre-restore SQLite safety copy.
3. Install staged files. For every conflicting original, move it into a unique directory in `backups/pre-restore/` and record the path before installing the replacement. Refuse destinations which are directories, symlinks or have a symlink ancestor. Hash any existing source in constant memory rather than loading enormous PDFs into RAM.
4. On any file-phase error, remove newly installed files in reverse order and put saved originals back. The active SQLite database has **not been changed**.
5. Once *all* managed files have succeeded, restore/reinitialise SQLite. If that stage fails, use the pre-restore safety database and also undo the file changes.
6. On complete success, keep the original conflicting source files beside the pre-restore safety database, preserve the standard restored-file counters and require the UI to reload.

The app does not rewrite imported source text and does not access private owner files. All new automated tests use invented disposable SQLite and source files.

## Tests

- Failed later file (directory where `01-second.txt` should be): verify the previously replaced `00-first.txt` returns to its exact old bytes and the active database retains its newer note.
- Successful restore: database and both source files match the archive; the superseded original remains in the aside directory with exact bytes.
- Local source path symlink to unrelated file (Unix): refused, no writes outside managed sources.
- Existing prior tests: schema-migration rollback, integrity, missing files, citation/tombstone preservation, original file/quote provenance.

## Still outstanding — NOT a completed crash-atomicity guarantee

**Power loss or process termination in the short file-installation window is not yet recoverable automatically.** An in-memory journal disappears in a crash even though the pre-restore database safety copy and moved originals survive. Before publication, implement and test a persistent recovery journal plus startup reconciliation or a proven equivalent crash-safe two-phase scheme. Include Windows fault injection, locked files, disk-full and long-library cases.

Also verify the new code in full Rust CI and a disposable native Windows profile. **Do not merge, release or call issue #57 closed on the basis of this draft.** No historical schema migration, recognizer code, publisher source or live user data was changed.
