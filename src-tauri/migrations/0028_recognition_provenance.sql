-- Abiding Word 0028: documentary Scripture recognition provenance.
-- First used in the application completion branch; no old migrations edited.
-- Rows are inserted only after a complete, freshly gated recognition pass.
-- Existing Bibles have NO row in bible_recognition_editions and therefore
-- MUST remain explicitly 'legacy / edition not confirmed' at the UI boundary.

CREATE TABLE bible_recognition_runs (
    id                       TEXT PRIMARY KEY NOT NULL,
    translation_id           TEXT NOT NULL REFERENCES bible_translations(id) ON DELETE CASCADE,
    recogniser_commit        TEXT NOT NULL CHECK (length(recogniser_commit) = 40),
    document_manifest_sha256 TEXT NOT NULL CHECK (length(document_manifest_sha256) = 64),
    gate_generation          INTEGER NOT NULL CHECK (gate_generation >= 1),
    completed_at             TEXT NOT NULL,
    UNIQUE (id, translation_id)
);
CREATE INDEX bible_recognition_runs_translation
    ON bible_recognition_runs(translation_id, completed_at);

CREATE TABLE bible_recognition_editions (
    translation_id       TEXT PRIMARY KEY NOT NULL
                         REFERENCES bible_translations(id) ON DELETE CASCADE,
    current_run_id       TEXT NOT NULL,
    confirmed_by_person  INTEGER NOT NULL DEFAULT 0
                         CHECK (confirmed_by_person IN (0, 1)),
    confirmed_at         TEXT,
    updated_at           TEXT NOT NULL,
    FOREIGN KEY (current_run_id, translation_id)
        REFERENCES bible_recognition_runs(id, translation_id),
    CHECK ((confirmed_by_person = 0 AND confirmed_at IS NULL)
        OR (confirmed_by_person = 1 AND confirmed_at IS NOT NULL))
);

-- Metadata/evidence, never an alternate Bible-text store. In particular,
-- the raw text of an unverified verse must not be persisted as Scripture.
-- Full source files and document hashes already live in bible_documents.
CREATE TABLE bible_recognition_verse_evidence (
    translation_id     TEXT NOT NULL REFERENCES bible_translations(id) ON DELETE CASCADE,
    verse_id           INTEGER NOT NULL REFERENCES bible_verses(id),
    document_id        TEXT NOT NULL REFERENCES bible_documents(id) ON DELETE CASCADE,
    run_id             TEXT NOT NULL,
    source_sha256      TEXT NOT NULL CHECK (length(source_sha256) = 64),
    -- For EPUB only; NULL for other formats.
    entry_path         TEXT,
    entry_sha256       TEXT CHECK (entry_sha256 IS NULL OR length(entry_sha256) = 64),
    recognition_state  TEXT NOT NULL
                       CHECK (recognition_state IN ('verifiedFromDocument', 'unverified')),
    reason             TEXT,
    provenance_json    TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(provenance_json)),
    excluded_json      TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(excluded_json)),
    PRIMARY KEY (translation_id, verse_id),
    FOREIGN KEY (run_id, translation_id)
        REFERENCES bible_recognition_runs(id, translation_id),
    CHECK ((entry_path IS NULL AND entry_sha256 IS NULL)
         OR (entry_path IS NOT NULL AND entry_sha256 IS NOT NULL)),
    CHECK ((recognition_state = 'verifiedFromDocument' AND reason IS NULL)
        OR (recognition_state = 'unverified' AND length(reason) > 0))
) WITHOUT ROWID;
CREATE INDEX bible_recognition_evidence_document
    ON bible_recognition_verse_evidence(document_id, recognition_state);

-- Edition-specific printed numbering; equal-looking numbers do not
-- automatically establish equivalence across editions.
CREATE TABLE bible_recognition_book_numbering (
    translation_id  TEXT NOT NULL REFERENCES bible_translations(id) ON DELETE CASCADE,
    book_id         INTEGER NOT NULL REFERENCES bible_books(id),
    run_id          TEXT NOT NULL,
    agreement       TEXT NOT NULL
                    CHECK (agreement IN ('withinReference', 'departs', 'undetermined')),
    details_json    TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(details_json)),
    PRIMARY KEY (translation_id, book_id),
    FOREIGN KEY (run_id, translation_id)
        REFERENCES bible_recognition_runs(id, translation_id)
) WITHOUT ROWID;

-- Fail closed: neither write ordering may leave Scripture text with an
-- explicitly unverified recognition record.
CREATE TRIGGER bible_recognition_unverified_insert_guard
BEFORE INSERT ON bible_recognition_verse_evidence
WHEN NEW.recognition_state = 'unverified'
 AND EXISTS (SELECT 1 FROM bible_verse_texts
             WHERE translation_id = NEW.translation_id AND verse_id = NEW.verse_id)
BEGIN
    SELECT RAISE(ABORT, 'unverified recognition cannot coexist with Scripture verse text');
END;

CREATE TRIGGER bible_recognition_unverified_update_guard
BEFORE UPDATE OF recognition_state, translation_id, verse_id ON bible_recognition_verse_evidence
WHEN NEW.recognition_state = 'unverified'
 AND EXISTS (SELECT 1 FROM bible_verse_texts
             WHERE translation_id = NEW.translation_id AND verse_id = NEW.verse_id)
BEGIN
    SELECT RAISE(ABORT, 'unverified recognition cannot coexist with Scripture verse text');
END;

CREATE TRIGGER bible_verse_texts_recognition_insert_guard
BEFORE INSERT ON bible_verse_texts
WHEN EXISTS (SELECT 1 FROM bible_recognition_verse_evidence
    WHERE translation_id = NEW.translation_id AND verse_id = NEW.verse_id
      AND recognition_state = 'unverified')
BEGIN
    SELECT RAISE(ABORT, 'unverified recognition cannot be stored as Scripture verse text');
END;

CREATE TRIGGER bible_verse_texts_recognition_update_guard
BEFORE UPDATE OF translation_id, verse_id, text ON bible_verse_texts
WHEN EXISTS (SELECT 1 FROM bible_recognition_verse_evidence
    WHERE translation_id = NEW.translation_id AND verse_id = NEW.verse_id
      AND recognition_state = 'unverified')
BEGIN
    SELECT RAISE(ABORT, 'unverified recognition cannot be stored as Scripture verse text');
END;

-- Anchor evidence to the correct document Bible and exact reviewed bytes.
CREATE TRIGGER bible_recognition_evidence_source_insert_guard
BEFORE INSERT ON bible_recognition_verse_evidence
WHEN NOT EXISTS (SELECT 1 FROM bible_documents
    WHERE id = NEW.document_id AND translation_id = NEW.translation_id
      AND sha256 = NEW.source_sha256)
BEGIN
    SELECT RAISE(ABORT, 'recognition evidence does not match saved source');
END;

CREATE TRIGGER bible_recognition_evidence_source_update_guard
BEFORE UPDATE OF document_id, translation_id, source_sha256 ON bible_recognition_verse_evidence
WHEN NOT EXISTS (SELECT 1 FROM bible_documents
    WHERE id = NEW.document_id AND translation_id = NEW.translation_id
      AND sha256 = NEW.source_sha256)
BEGIN
    SELECT RAISE(ABORT, 'recognition evidence does not match saved source');
END;