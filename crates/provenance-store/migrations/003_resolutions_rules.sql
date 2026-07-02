DROP TABLE IF EXISTS resolutions;
DROP TABLE IF EXISTS rules;

CREATE TABLE resolutions (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    title TEXT NOT NULL,
    position TEXT NOT NULL,
    rationale TEXT NOT NULL,
    status TEXT NOT NULL,
    review_on TEXT,
    review_triggers TEXT NOT NULL,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX idx_resolutions_status_review ON resolutions(scope_id, status, review_on);

CREATE TABLE rules (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    rule_code TEXT NOT NULL,
    statement TEXT NOT NULL,
    status TEXT NOT NULL,
    severity TEXT NOT NULL,
    expression TEXT NOT NULL,
    inputs TEXT NOT NULL,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX idx_rules_code ON rules(scope_id, rule_code);
CREATE INDEX idx_rules_status_severity ON rules(scope_id, status, severity);
