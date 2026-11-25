-- Abiding Word — migration 0022: import jobs (v0.24)
--
-- Adding many sources at once (several files, or a folder) is a job that
-- outlives one screen: the files found are listed, each is read and checked
-- (one at a time, so memory stays bounded), the person reviews and corrects
-- the list, then the chosen ones are added one by one. The job and its items
-- are kept here so it can be cancelled at any point, and a job stopped by a
-- crash or a closed app is shown again as interrupted, to continue or
-- discard. Nothing here holds a source's text: an item records only the
-- file, what was found about it and, once added, the source it became.
--
-- Additive: two new tables. Details: docs/library-management-v0.24.md.

CREATE TABLE import_jobs (
    id                  TEXT PRIMARY KEY NOT NULL,
    kind                TEXT NOT NULL CHECK (kind IN ('sources')),
    -- review: read and checked, waiting for the person; importing: adding;
    -- interrupted: was importing when the app stopped; finished; cancelled.
    state               TEXT NOT NULL CHECK (state IN ('review', 'importing', 'interrupted', 'finished', 'cancelled')),
    -- Shared choices for every item added: a collection and tags (JSON list).
    collection_id       TEXT REFERENCES source_collections (id) ON DELETE SET NULL,
    tags                TEXT NOT NULL DEFAULT '[]',
    -- What the scan left out: files of kinds the library cannot read, and
    -- files beyond the limit.
    unsupported_count   INTEGER NOT NULL DEFAULT 0,
    over_limit_count    INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE import_job_items (
    id                  TEXT PRIMARY KEY NOT NULL,
    job_id              TEXT NOT NULL REFERENCES import_jobs (id) ON DELETE CASCADE,
    position            INTEGER NOT NULL,
    path                TEXT NOT NULL,
    file_name           TEXT NOT NULL,
    size                INTEGER NOT NULL,
    -- waiting: not read yet; ready; duplicate (already in the library, or
    -- earlier in this job); problem (cannot be added); imported; failed.
    status              TEXT NOT NULL CHECK (status IN ('waiting', 'ready', 'duplicate', 'problem', 'imported', 'failed')),
    -- Whether it is to be added (the person's choice; duplicates start unchosen).
    chosen              INTEGER NOT NULL DEFAULT 1 CHECK (chosen IN (0, 1)),
    sha256              TEXT,
    format              TEXT,
    title               TEXT,
    author              TEXT,
    language            TEXT,
    source_type         TEXT,
    -- Why the category was proposed: 'fileType', 'title' or 'folder'.
    type_evidence       TEXT,
    page_count          INTEGER,
    duplicate_of        TEXT,
    duplicate_title     TEXT,
    problem_code        TEXT,
    problem             TEXT,
    problem_params      TEXT,
    source_id           TEXT REFERENCES sources (id) ON DELETE SET NULL,
    updated_at          TEXT NOT NULL,
    UNIQUE (job_id, position)
);

CREATE INDEX import_job_items_status ON import_job_items (job_id, status);
