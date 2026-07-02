CREATE TABLE IF NOT EXISTS boundaries (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    requirement_id TEXT NOT NULL,
    statement TEXT NOT NULL,
    source_id TEXT,
    source_clause TEXT,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_boundaries_requirement ON boundaries(scope_id, requirement_id);

CREATE TABLE IF NOT EXISTS topics (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    requirement_id TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    links TEXT NOT NULL,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_topics_requirement ON topics(scope_id, requirement_id);
CREATE INDEX IF NOT EXISTS idx_topics_status ON topics(scope_id, status);

CREATE TABLE IF NOT EXISTS questions (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    topic_id TEXT NOT NULL,
    requirement_id TEXT NOT NULL,
    question TEXT NOT NULL,
    status TEXT NOT NULL,
    answer TEXT,
    links TEXT NOT NULL,
    resolution_id TEXT,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_questions_topic ON questions(scope_id, topic_id);
CREATE INDEX IF NOT EXISTS idx_questions_requirement ON questions(scope_id, requirement_id);
CREATE INDEX IF NOT EXISTS idx_questions_status ON questions(scope_id, status);
