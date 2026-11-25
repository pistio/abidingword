-- Migration 0026 requires the runner's FK-off, exclusive-transaction rebuild protocol.
CREATE TABLE teaching_preparations_new (
    id              TEXT PRIMARY KEY NOT NULL,
    project_id      TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    study_id        TEXT REFERENCES studies (id) ON DELETE SET NULL,
    title           TEXT NOT NULL,
    -- sermon, bible_study, youth, children, lesson, talk, other (validated by
    -- the application); NULL when not said.
    kind            TEXT,
    audience        TEXT,
    purpose         TEXT,
    central_message TEXT,
    introduction    TEXT NOT NULL DEFAULT '',
    conclusion      TEXT NOT NULL DEFAULT '',
    -- The teacher's own notes (not shown to the audience).
    notes           TEXT NOT NULL DEFAULT '',
    position        INTEGER NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT,
    revision        INTEGER NOT NULL DEFAULT 1
, template_key TEXT, template_version INTEGER);
INSERT INTO teaching_preparations_new (id,study_id,title,kind,audience,purpose,central_message,introduction,conclusion,notes,position,created_at,updated_at,deleted_at,revision,template_key,template_version,project_id) SELECT t.id,t.study_id,t.title,t.kind,t.audience,t.purpose,t.central_message,t.introduction,t.conclusion,t.notes,t.position,t.created_at,t.updated_at,t.deleted_at,t.revision,t.template_key,t.template_version ,s.project_id FROM teaching_preparations t LEFT JOIN studies s ON s.id=t.study_id;
DROP TABLE teaching_preparations;
ALTER TABLE teaching_preparations_new RENAME TO teaching_preparations;
CREATE TABLE teaching_finding_links_new (
    id              TEXT PRIMARY KEY NOT NULL,
    preparation_id  TEXT NOT NULL REFERENCES teaching_preparations (id) ON DELETE CASCADE,
    point_id        TEXT REFERENCES teaching_points (id) ON DELETE CASCADE,
    finding_id      TEXT REFERENCES findings (id) ON DELETE SET NULL,
    position        INTEGER NOT NULL,
    created_at      TEXT NOT NULL,
    deleted_at      TEXT
, block_id TEXT, snapshot TEXT);
INSERT INTO teaching_finding_links_new (id,preparation_id,point_id,finding_id,position,created_at,deleted_at,block_id) SELECT id,preparation_id,point_id,finding_id,position,created_at,deleted_at,block_id FROM teaching_finding_links;
DROP TABLE teaching_finding_links;
ALTER TABLE teaching_finding_links_new RENAME TO teaching_finding_links;
CREATE INDEX teaching_preparations_study
    ON teaching_preparations (study_id, position) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX teaching_finding_links_once
    ON teaching_finding_links (preparation_id, COALESCE(point_id, ''), finding_id)
    WHERE deleted_at IS NULL;
CREATE INDEX teaching_finding_links_finding ON teaching_finding_links (finding_id);
CREATE INDEX teaching_preparations_project ON teaching_preparations(project_id,position) WHERE deleted_at IS NULL;
CREATE TRIGGER teaching_project_insert BEFORE INSERT ON teaching_preparations
WHEN NEW.study_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM studies WHERE id=NEW.study_id AND project_id=NEW.project_id)
BEGIN SELECT RAISE(ABORT,'teaching study belongs to another project'); END;
CREATE TRIGGER teaching_project_update BEFORE UPDATE OF project_id,study_id ON teaching_preparations
WHEN NEW.study_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM studies WHERE id=NEW.study_id AND project_id=NEW.project_id)
BEGIN SELECT RAISE(ABORT,'teaching study belongs to another project'); END;
CREATE TRIGGER teaching_study_project_update BEFORE UPDATE OF project_id ON studies
WHEN EXISTS (SELECT 1 FROM teaching_preparations WHERE study_id=OLD.id AND project_id<>NEW.project_id)
BEGIN SELECT RAISE(ABORT,'linked teaching belongs to another project'); END;
