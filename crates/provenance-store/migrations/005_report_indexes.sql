CREATE INDEX IF NOT EXISTS idx_edges_scope_type_from ON edges(scope_id, edge_type, from_type, from_id);
CREATE INDEX IF NOT EXISTS idx_edges_scope_type_to ON edges(scope_id, edge_type, to_type, to_id);
CREATE INDEX IF NOT EXISTS idx_resolutions_scope_status_review ON resolutions(scope_id, status, review_on);
CREATE INDEX IF NOT EXISTS idx_rules_scope_status_severity ON rules(scope_id, status, severity);
