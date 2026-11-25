#!/usr/bin/env python3
"""Disposable SQLite contract tests for additive recognition migration 0028.

These intentionally use minimal pre-0028 parent-table schemas. They test the
new SQL's invariants, not the full native migration or application workflow.
"""
from pathlib import Path
import sqlite3
import unittest

ROOT = Path(__file__).resolve().parents[2]
MIGRATION = ROOT / "src-tauri/migrations/0028_recognition_provenance.sql"
SHA = "a" * 64
ENGINE = "b" * 40
MANIFEST = "c" * 64


def database():
    db = sqlite3.connect(":memory:")
    db.execute("PRAGMA foreign_keys = ON")
    db.executescript("""
        CREATE TABLE bible_translations (id TEXT PRIMARY KEY);
        CREATE TABLE bible_documents (id TEXT PRIMARY KEY, translation_id TEXT,
                                      sha256 TEXT);
        CREATE TABLE bible_verses (id INTEGER PRIMARY KEY);
        CREATE TABLE bible_books (id INTEGER PRIMARY KEY);
        CREATE TABLE bible_verse_texts (translation_id TEXT, verse_id INTEGER, text TEXT);
        INSERT INTO bible_translations VALUES ('T1'), ('T2');
        INSERT INTO bible_verses VALUES (43003016);
        INSERT INTO bible_books VALUES (43);
    """)
    db.execute("INSERT INTO bible_documents VALUES ('D1','T1',?)", (SHA,))
    db.executescript(MIGRATION.read_text(encoding="utf-8"))
    db.execute("INSERT INTO bible_recognition_runs VALUES ('R1','T1',?,?,1,'2026-11-25')",
               (ENGINE, MANIFEST))
    return db


def evidence(db, state, reason=None, document="D1", translation="T1", run="R1",
             digest=SHA):
    db.execute("""INSERT INTO bible_recognition_verse_evidence
       (translation_id,verse_id,document_id,run_id,source_sha256,
        recognition_state,reason)
       VALUES (?,43003016,?,?,?,?,?)""",
       (translation, document, run, digest, state, reason))


class RecognitionMigration0028(unittest.TestCase):
    def setUp(self):
        self.db = database()

    def tearDown(self):
        self.db.close()

    def test_existing_translation_does_not_gain_invented_confirmation(self):
        self.assertEqual(
            self.db.execute("SELECT COUNT(*) FROM bible_recognition_editions").fetchone()[0], 0)

    def test_unverified_evidence_cannot_turn_into_stored_bible_text(self):
        evidence(self.db, "unverified", "boundary")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO bible_verse_texts VALUES ('T1',43003016,'invented')")
        self.assertEqual(
            self.db.execute("SELECT COUNT(*) FROM bible_verse_texts").fetchone()[0], 0)

    def test_existing_text_blocks_unverified_evidence(self):
        self.db.execute("INSERT INTO bible_verse_texts VALUES ('T1',43003016,'invented')")
        with self.assertRaises(sqlite3.IntegrityError):
            evidence(self.db, "unverified", "boundary")
        self.assertEqual(
            self.db.execute("SELECT COUNT(*) FROM bible_recognition_verse_evidence").fetchone()[0], 0)

    def test_verified_to_unverified_requires_removing_text(self):
        evidence(self.db, "verifiedFromDocument")
        self.db.execute("INSERT INTO bible_verse_texts VALUES ('T1',43003016,'invented')")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("""UPDATE bible_recognition_verse_evidence
                SET recognition_state='unverified',reason='noteSign'
                WHERE translation_id='T1' AND verse_id=43003016""")
        self.db.execute("DELETE FROM bible_verse_texts WHERE translation_id='T1'")
        self.db.execute("""UPDATE bible_recognition_verse_evidence
            SET recognition_state='unverified',reason='noteSign'
            WHERE translation_id='T1' AND verse_id=43003016""")
        self.assertEqual(self.db.execute(
            "SELECT recognition_state FROM bible_recognition_verse_evidence"
        ).fetchone()[0], "unverified")

    def test_source_sha_and_translation_must_match(self):
        for kw in (dict(digest="f" * 64), dict(translation="T2")):
            with self.subTest(kw=kw), self.assertRaises(sqlite3.IntegrityError):
                evidence(self.db, "verifiedFromDocument", **kw)

    def test_reused_run_from_other_translation_is_refused(self):
        self.db.execute("INSERT INTO bible_documents VALUES ('D2','T2',?)", (SHA,))
        with self.assertRaises(sqlite3.IntegrityError):
            evidence(self.db, "verifiedFromDocument", document="D2", translation="T2")

    def test_confirmation_requires_timestamp(self):
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("""INSERT INTO bible_recognition_editions
                VALUES ('T1','R1',1,NULL,'now')""")
        self.db.execute("""INSERT INTO bible_recognition_editions
            VALUES ('T1','R1',0,NULL,'now')""")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("""UPDATE bible_recognition_editions
                SET confirmed_by_person=1 WHERE translation_id='T1'""")

    def test_provenance_json_must_be_valid(self):
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("""INSERT INTO bible_recognition_verse_evidence
                (translation_id,verse_id,document_id,run_id,source_sha256,
                 recognition_state,provenance_json)
                VALUES ('T1',43003016,'D1','R1',?,'verifiedFromDocument','broken')""", (SHA,))


if __name__ == "__main__":
    unittest.main()
