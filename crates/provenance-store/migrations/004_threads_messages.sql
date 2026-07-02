DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS threads;

CREATE TABLE threads (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    parent_type TEXT NOT NULL,
    parent_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX idx_threads_parent_status ON threads(scope_id, parent_type, parent_id, status);

CREATE TABLE messages (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    thread_id TEXT NOT NULL,
    role TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    ai_metadata TEXT,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX idx_messages_thread_order ON messages(scope_id, thread_id, created_at, id);
