CREATE TABLE IF NOT EXISTS extracts (
    id                          TEXT        NOT NULL        PRIMARY KEY,
    created_date                TEXT        NOT NULL        DEFAULT(datetime('now')),
    modified_date               TEXT        NOT NULL        DEFAULT(datetime('now')),
    cell_id                     TEXT        NOT NULL        REFERENCES cells(id) ON DELETE CASCADE,
    inner_html                  TEXT        NOT NULL,
    status                      TEXT        NOT NULL        DEFAULT '"Pending"'
);

CREATE INDEX IF NOT EXISTS extracts_cell_id_index ON extracts(cell_id);

CREATE TRIGGER extracts_update_modified_date_after_update
    AFTER UPDATE ON extracts
    WHEN OLD.modified_date == NEW.modified_date
BEGIN
    UPDATE extracts
    SET modified_date = datetime('now')
    WHERE id = NEW.id;
END;

CREATE TRIGGER extracts_add_to_deleted_entities_after_delete
    AFTER DELETE ON extracts
BEGIN
    INSERT INTO deleted_entities (entity_name, entity_id, entity_created_date, deleted_date)
    VALUES ('extracts', OLD.id, OLD.created_date, datetime('now'));
END;
