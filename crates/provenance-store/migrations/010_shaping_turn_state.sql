ALTER TABLE requirements ADD COLUMN fog TEXT;
ALTER TABLE topics ADD COLUMN claimed_by TEXT;
ALTER TABLE topics ADD COLUMN claimed_at INTEGER;
ALTER TABLE questions ADD COLUMN resolution_method TEXT NOT NULL DEFAULT 'grill';
ALTER TABLE questions ADD COLUMN claimed_by TEXT;
ALTER TABLE questions ADD COLUMN claimed_at INTEGER;
CREATE INDEX IF NOT EXISTS idx_topics_claimed_by ON topics(scope_id, claimed_by);
CREATE INDEX IF NOT EXISTS idx_questions_claimed_by ON questions(scope_id, claimed_by);
