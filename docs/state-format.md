# State Format

`.provenance/state/` is the canonical store. Records are newline-delimited JSON with stable string `id` fields, `schema_version`, and deterministic ordering by primary key inside each shard.

Scopes live in `manifest.json`; shard paths derive from scope IDs. Cache files and volatile fields are forbidden in state shards.

Schema version `1` includes the local graph fields plus imported/cloud review metadata. Optional fields are omitted when absent, but preserved when present: domain grouping for root requirements, requirement descriptions and source references, source references/clauses/effective/review/supersession dates/commit pins, draft/review statuses, resolution context/enforcement/confidence/input references/actor approval/supersession metadata, resolved thread status, rule name/type/modality/confidence/extraction/source-location metadata, deployed services, rule-to-service bindings, proposal confidence, and material-claim confidence.

Proposal schema v1 also recognizes the `asserted` promotion state and optional `builds_on`
proposal IDs. Older records without `builds_on` deserialize as an empty lineage. The parser
accepts legacy spelling `swarm_asserted`, but canonical serialization uses `asserted`.

Concurrent writers serialize JSONL shard mutations with advisory lock files under `.provenance/cache/locks/`. A writer holds the corresponding shard lock across the full read-modify-write cycle, then atomically replaces the shard file. Lock files are derived cache artifacts, not state, and must not be committed. Readers do not take locks; the atomic replace contract means they see either the old complete shard or the new complete shard.
