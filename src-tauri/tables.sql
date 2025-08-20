-- TODO: initiate the database at start
CREATE TABLE folders(
    id                          TEXT        PRIMARY KEY,
    name                        TEXT,
    parent_id                   TEXT,
    FOREIGN KEY(parent_id) REFERENCES folders(id) ON DELETE CASCADE,
    UNIQUE (name, parent_id)
);

-- The id of root is 00000000-0000-0000-0000-000000000001
INSERT INTO folders(id, name, parent_id) VALUES (X'00000000000000000000000000000001', 'root', NULL);

CREATE TABLE files(
    id                          TEXT        PRIMARY KEY,
    name                        TEXT,
    parent_id                   TEXT        NOT NULL,
    FOREIGN KEY(parent_id) REFERENCES folders(id) ON DELETE CASCADE,
    UNIQUE (name, parent_id)
);

