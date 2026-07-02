ALTER TABLE sources ADD COLUMN commit_pin TEXT;
ALTER TABLE proposal_cards ADD COLUMN confidence REAL;
CREATE INDEX IF NOT EXISTS idx_sources_commit_pin ON sources(scope_id, commit_pin);
