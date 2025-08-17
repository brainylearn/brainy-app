-- TODO: initiate the database at start
CREATE TABLE folders(
    id                          TEXT        PRIMARY KEY,
    name                        TEXT,
    parent_id                   TEXT,
    FOREIGN KEY(parent_id) REFERENCES folders(id) ON DELETE CASCADE,
    UNIQUE (name, parent_id)
);

CREATE TABLE files(
    id                          TEXT        PRIMARY KEY,
    name                        TEXT,
    parent_id                   TEXT,
    FOREIGN KEY(parent_id) REFERENCES folders(id) ON DELETE CASCADE,
    UNIQUE (name, parent_id)
);
