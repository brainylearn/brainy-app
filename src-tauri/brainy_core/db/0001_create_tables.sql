CREATE TABLE folders(
    id                          TEXT        NOT NULL        PRIMARY KEY,
    name                        TEXT        NOT NULL,
    parent_id                   TEXT,
    FOREIGN KEY(parent_id) REFERENCES folders(id) ON DELETE CASCADE,
    UNIQUE (name, parent_id)
);

-- The id of root is 00000000-0000-0000-0000-000000000001
INSERT INTO folders(id, name, parent_id) VALUES (X'00000000000000000000000000000001', 'root', NULL);

CREATE TABLE files(
    id                          TEXT        NOT NULL        PRIMARY KEY,
    name                        TEXT        NOT NULL,
    parent_id                   TEXT        NOT NULL,
    FOREIGN KEY(parent_id) REFERENCES folders(id) ON DELETE CASCADE,
    UNIQUE (name, parent_id)
);

CREATE TABLE cells(
    id                          TEXT        NOT NULL        PRIMARY KEY,
    content                     TEXT        NOT NULL        DEFAULT "",
    cell_type                   TEXT        NOT NULL,
    cell_index                  INTEGER     NOT NULL,
    file_id                     TEXT        NOT NULL,
    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE,
    UNIQUE (file_id, cell_index)
);
