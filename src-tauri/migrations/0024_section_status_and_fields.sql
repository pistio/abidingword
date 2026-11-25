-- Abiding Word — migration 0024: structured parts and part progress
--
-- A part of a study can say how far the person has got with it, and a part
-- made from a structured built-in template (POZETEK) keeps a snapshot of the
-- fields it was created with. Answers live in the existing study blocks: a
-- block says which field it answers, and a block of a repeatable field keeps
-- its entry values in its existing `data` column.
--
-- Additive only: three new columns. Every existing part is "not started"
-- with no fields; every existing block answers no field. Nothing existing is
-- rewritten.

-- not_started, in_progress, complete
ALTER TABLE study_sections ADD COLUMN status TEXT NOT NULL DEFAULT 'not_started'
    CHECK (status IN ('not_started', 'in_progress', 'complete'));

-- JSON snapshot {"template", "version", "fields": [...]}; NULL for plain parts.
ALTER TABLE study_sections ADD COLUMN fields TEXT;

-- The field (key in the part's snapshot) a block answers; NULL for free writing.
ALTER TABLE study_blocks ADD COLUMN field_key TEXT;
