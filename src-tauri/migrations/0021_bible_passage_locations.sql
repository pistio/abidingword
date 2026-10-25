-- Abiding Word — migration 0021: where passages are in a Bible's documents (v0.24)
--
-- A Bible kept as a document becomes navigable by passage where its places
-- are established:
--
-- * verses recognised in its text and confirmed are stored as for any Bible
--   (bible_verse_texts, with bible_verse_origins pointing back into the
--   document) — now also for only the chapters that were recognised without
--   any doubt, the rest staying unverified;
-- * a passage the person marked by hand on a page, a part of an EPUB or Word
--   document, or a paragraph is kept here, as a canonical range and an exact
--   place in the document. Nothing is copied from the document: the place is
--   shown, never text taken from it.
--
-- The canonical range (start_key/end_key: book × 1,000,000 + chapter × 1,000
-- + verse) is what notes and references use; the place is this edition's own.
-- Removing a mark changes nothing else. Additive: one new table, one index.
-- Details: docs/scripture-navigation-v0.24.md.

CREATE TABLE bible_passage_locations (
    id              TEXT PRIMARY KEY NOT NULL,
    translation_id  TEXT NOT NULL REFERENCES bible_translations (id) ON DELETE CASCADE,
    document_id     TEXT NOT NULL REFERENCES bible_documents (id) ON DELETE CASCADE,
    start_key       INTEGER NOT NULL,
    end_key         INTEGER NOT NULL,
    -- 'manual': marked by the person (the only kind written in v0.24).
    origin          TEXT NOT NULL CHECK (origin IN ('manual')),
    -- The place: a PDF page, a stored paragraph (segment position), or an
    -- EPUB/DOCX part; at least one.
    page            INTEGER,
    segment         INTEGER,
    item            TEXT,
    -- Words the person selected there, kept as a check that the place is right.
    quote           TEXT,
    created_at      TEXT NOT NULL,
    CHECK (end_key >= start_key),
    CHECK (page IS NOT NULL OR segment IS NOT NULL OR item IS NOT NULL)
);

CREATE INDEX bible_passage_locations_range
    ON bible_passage_locations (translation_id, start_key, end_key);
