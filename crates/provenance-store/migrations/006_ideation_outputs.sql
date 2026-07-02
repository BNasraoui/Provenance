CREATE TABLE IF NOT EXISTS contributions (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    participant_slot TEXT NOT NULL,
    stance TEXT NOT NULL,
    strongest_finding TEXT NOT NULL,
    uncertainty TEXT NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_contributions_target ON contributions(scope_id, target_type, target_id);

CREATE TABLE IF NOT EXISTS synthesis_packets (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_synthesis_packets_target ON synthesis_packets(scope_id, target_type, target_id);

CREATE TABLE IF NOT EXISTS proposal_cards (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    proposal_key TEXT NOT NULL,
    proposal_type TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    traceability TEXT NOT NULL,
    promotion_state TEXT NOT NULL,
    duplicate_of TEXT,
    superseded_by TEXT,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_proposal_cards_target ON proposal_cards(scope_id, target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_proposal_cards_state ON proposal_cards(scope_id, promotion_state);

CREATE TABLE IF NOT EXISTS promotion_decisions (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    proposal_id TEXT NOT NULL,
    decision TEXT NOT NULL,
    rationale TEXT NOT NULL,
    actor TEXT NOT NULL,
    canonical_artifact TEXT,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_promotion_decisions_proposal ON promotion_decisions(scope_id, proposal_id);
