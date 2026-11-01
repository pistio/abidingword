-- App-owned PDF structure. Original files, segment identities/text and citation
-- anchors are deliberately untouched. Full backups include corrections/history.
CREATE TABLE source_pdf_structures (
    source_id TEXT PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    file_hash TEXT NOT NULL,
    etag TEXT NOT NULL,
    version INTEGER NOT NULL,
    document TEXT NOT NULL CHECK (json_valid(document)),
    overrides TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(overrides)),
    history TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(history)),
    updated_at TEXT NOT NULL
);
ALTER TABLE source_passages ADD COLUMN structure_node_id TEXT;
ALTER TABLE source_passages ADD COLUMN structure_scope TEXT;
ALTER TABLE source_passages ADD COLUMN structure_basis TEXT;
ALTER TABLE source_passages ADD COLUMN structure_role TEXT;
CREATE INDEX source_passages_structure ON source_passages(source_id, structure_node_id)
    WHERE structure_node_id IS NOT NULL;
