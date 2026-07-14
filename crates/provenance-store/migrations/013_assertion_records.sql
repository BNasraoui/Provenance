CREATE TABLE assertion_records (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    proposal_id TEXT NOT NULL,
    synthesis_packet_id TEXT NOT NULL,
    supporting_claim_ids TEXT NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (scope_id, id),
    UNIQUE (scope_id, proposal_id)
);
