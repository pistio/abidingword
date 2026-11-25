-- Abiding Word — migration 0023: recognition the person owns, and verses
-- whose place is known but whose text is not verified (v0.24.3)
--
-- * bible_recognition_corrections: the person's corrections to how a Bible's
--   documents were read (which book a heading stands for, or none). Each is
--   anchored to the document's content (its SHA-256), the heading as written
--   and its place, so that reading the same document again applies it, a
--   changed document asks for it to be checked, and a correction that no
--   longer fits anything is shown, never dropped silently. What the
--   recogniser found itself is kept beside the person's choice.
-- * bible_verse_origins.basis: whether a verse's place was found by the
--   recogniser alone ('automatic') or under the person's correction
--   ('corrected'); first_offset: where in its first paragraph the verse
--   begins (characters).
-- * bible_verse_places: verses whose book, chapter and verse were found in a
--   document but whose text could not be verified (its exact boundaries or
--   what is mixed into it are in doubt). Their place can be opened in the
--   document; their text never enters the Bible's verses.
-- * bible_document_apparatus: what the recogniser set apart from the verses
--   (footnotes, cross-references, headings, running heads, introductions,
--   subscriptions) with where it stands, so nothing excluded is lost.
--
-- Additive: new tables and new columns with defaults; existing rows keep
-- their meaning ('automatic'). Details: docs/scripture-integrity-v0.24.3.md.

CREATE TABLE bible_recognition_corrections (
    id                 TEXT PRIMARY KEY NOT NULL,
    translation_id     TEXT NOT NULL REFERENCES bible_translations (id) ON DELETE CASCADE,
    -- The document's content when the correction was made.
    document_sha256    TEXT NOT NULL,
    -- Its place among the Bible's documents (0 = the first).
    document_position  INTEGER NOT NULL,
    kind               TEXT NOT NULL CHECK (kind IN ('book')),
    -- The heading as written, and where it stands.
    heading            TEXT NOT NULL,
    page               INTEGER,
    segment            INTEGER,
    -- What the recogniser read it as (NULL: no book) and what the person
    -- chose (NULL: leave it out).
    detected_book      TEXT,
    chosen_book        TEXT,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL
);

CREATE UNIQUE INDEX bible_recognition_corrections_anchor
    ON bible_recognition_corrections (translation_id, document_position, heading, IFNULL(page, -1), IFNULL(segment, -1));

ALTER TABLE bible_verse_origins
    ADD COLUMN basis TEXT NOT NULL DEFAULT 'automatic' CHECK (basis IN ('automatic', 'corrected'));
ALTER TABLE bible_verse_origins ADD COLUMN first_offset INTEGER;

CREATE TABLE bible_verse_places (
    translation_id  TEXT NOT NULL REFERENCES bible_translations (id) ON DELETE CASCADE,
    verse_id        INTEGER NOT NULL REFERENCES bible_verses (id),
    document_id     TEXT NOT NULL REFERENCES bible_documents (id) ON DELETE CASCADE,
    first_segment   INTEGER,
    last_segment    INTEGER,
    first_page      INTEGER,
    last_page       INTEGER,
    first_offset    INTEGER,
    basis           TEXT NOT NULL DEFAULT 'automatic' CHECK (basis IN ('automatic', 'corrected')),
    -- Why the text is not verified (a code the interface explains).
    reason          TEXT NOT NULL CHECK (reason IN ('possibleHeading', 'reference', 'noteSign',
                        'unnumbered', 'length', 'psalmTitle', 'boundary')),
    PRIMARY KEY (translation_id, verse_id)
) WITHOUT ROWID;

CREATE TABLE bible_document_apparatus (
    id              INTEGER PRIMARY KEY,
    translation_id  TEXT NOT NULL REFERENCES bible_translations (id) ON DELETE CASCADE,
    document_id     TEXT NOT NULL REFERENCES bible_documents (id) ON DELETE CASCADE,
    kind            TEXT NOT NULL CHECK (kind IN ('footnote', 'crossReference', 'heading',
                        'subscription', 'runningHead', 'introduction', 'frontMatter', 'notes',
                        'otherBook')),
    page            INTEGER,
    segment         INTEGER,
    -- The verse it stands with, when known.
    verse_id        INTEGER,
    content         TEXT NOT NULL
);

CREATE INDEX bible_document_apparatus_place
    ON bible_document_apparatus (translation_id, document_id, segment);
CREATE INDEX bible_document_apparatus_verse
    ON bible_document_apparatus (translation_id, verse_id);
