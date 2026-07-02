DROP TABLE IF EXISTS sources;
DROP TABLE IF EXISTS requirements;
DROP TABLE IF EXISTS edges;

CREATE TABLE sources (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL,
    url TEXT,
    PRIMARY KEY (scope_id, id)
);

CREATE TABLE requirements (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    statement TEXT NOT NULL,
    status TEXT NOT NULL,
    PRIMARY KEY (scope_id, id)
);

CREATE TABLE edges (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    edge_type TEXT NOT NULL,
    from_type TEXT NOT NULL,
    from_id TEXT NOT NULL,
    to_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX idx_edges_from ON edges(scope_id, from_type, from_id);
CREATE INDEX idx_edges_to ON edges(scope_id, to_type, to_id);
