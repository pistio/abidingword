-- Abiding Word — migration 0020: archive, Trash and organisation (v0.24)
--
-- Sources and Bibles gain a state: 'active' (the default, as before),
-- 'archived' (kept and searchable, out of the everyday lists) or 'trashed'
-- (recoverable for a while, then removed for good). Removing for good is
-- what removal always did (a source becomes a citation tombstone, a Bible's
-- rows go); it now happens only from Trash or after the retention period.
--
-- Sources can be organised in collections (user folders, optionally nested;
-- one source in any number of them, its file never copied) and tagged
-- (many tags per source). The category (source_type) is unchanged.
--
-- Additive: new columns with defaults that keep every row as it was, new
-- tables and indexes. Details: docs/library-management-v0.24.md.

ALTER TABLE sources ADD COLUMN state TEXT NOT NULL DEFAULT 'active'
    CHECK (state IN ('active', 'archived', 'trashed'));
-- When the state last changed: a trashed source is removed for good once
-- this is older than the retention period.
ALTER TABLE sources ADD COLUMN state_changed_at TEXT;

CREATE INDEX sources_state ON sources (workspace_id, state, created_at)
    WHERE deleted_at IS NULL;
-- Duplicates (the same file added twice) are found by content hash.
CREATE INDEX sources_content_hash ON sources (content_hash) WHERE deleted_at IS NULL;

ALTER TABLE bible_translations ADD COLUMN state TEXT NOT NULL DEFAULT 'active'
    CHECK (state IN ('active', 'archived', 'trashed'));
ALTER TABLE bible_translations ADD COLUMN state_changed_at TEXT;

-- Collections: a tree of named groups per workspace. The tree is logical:
-- nothing on disk follows it.
CREATE TABLE source_collections (
    id            TEXT PRIMARY KEY NOT NULL,
    workspace_id  TEXT NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    -- NULL at the top. Removing a collection removes those inside it (the
    -- sources themselves stay).
    parent_id     TEXT REFERENCES source_collections (id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE INDEX source_collections_parent ON source_collections (workspace_id, parent_id);

CREATE TABLE source_collection_items (
    collection_id  TEXT NOT NULL REFERENCES source_collections (id) ON DELETE CASCADE,
    source_id      TEXT NOT NULL REFERENCES sources (id) ON DELETE CASCADE,
    added_at       TEXT NOT NULL,
    PRIMARY KEY (collection_id, source_id)
);

CREATE INDEX source_collection_items_source ON source_collection_items (source_id);

-- Tags: names unique per workspace, whatever their case.
CREATE TABLE source_tags (
    id            TEXT PRIMARY KEY NOT NULL,
    workspace_id  TEXT NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    created_at    TEXT NOT NULL
);

CREATE UNIQUE INDEX source_tags_name ON source_tags (workspace_id, name COLLATE NOCASE);

CREATE TABLE source_tag_items (
    tag_id     TEXT NOT NULL REFERENCES source_tags (id) ON DELETE CASCADE,
    source_id  TEXT NOT NULL REFERENCES sources (id) ON DELETE CASCADE,
    PRIMARY KEY (tag_id, source_id)
);

CREATE INDEX source_tag_items_source ON source_tag_items (source_id);
