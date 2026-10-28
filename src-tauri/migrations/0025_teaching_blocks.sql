-- Abiding Word — migration 0025: Teaching blocks
--
-- A teaching can be written as ordered blocks (a section, a writing area, a
-- question, a private leader note, …) copied from a template when it is made.
-- Teachings made before this migration have no blocks and no template key:
-- they keep the columns and points they already have, and open in the editor
-- they were written in.
--
-- Additive only: two columns, one table, three nullable columns. Nothing
-- existing is rewritten. A teaching with no study is not introduced here;
-- study_id stays required so that can be added later without reshaping blocks.
-- Details: the Teaching block design on feature/teaching-three-templates.

ALTER TABLE teaching_preparations ADD COLUMN template_key TEXT;
ALTER TABLE teaching_preparations ADD COLUMN template_version INTEGER;

-- One piece of a block-based teaching. parent_id groups it under a section
-- (one level). point_id, when set, is the outline point presentations already
-- follow. config is JSON: participantFacing, role, guidanceKey.
CREATE TABLE teaching_blocks (
    id              TEXT PRIMARY KEY NOT NULL,
    preparation_id  TEXT NOT NULL REFERENCES teaching_preparations (id) ON DELETE CASCADE,
    parent_id       TEXT REFERENCES teaching_blocks (id) ON DELETE CASCADE,
    position        INTEGER NOT NULL,
    block_type      TEXT NOT NULL,
    block_key       TEXT,
    title           TEXT NOT NULL DEFAULT '',
    body            TEXT NOT NULL DEFAULT '',
    config          TEXT,
    point_id        TEXT REFERENCES teaching_points (id) ON DELETE SET NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT,
    revision        INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX teaching_blocks_preparation
    ON teaching_blocks (preparation_id, position) WHERE deleted_at IS NULL;

-- Where a passage, a study link or a kept image hangs on a block. NULL for
-- everything already stored, which stays attached to the whole teaching or a point.
ALTER TABLE teaching_passages ADD COLUMN block_id TEXT;
ALTER TABLE teaching_finding_links ADD COLUMN block_id TEXT;
ALTER TABLE teaching_materials ADD COLUMN block_id TEXT;
