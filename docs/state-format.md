# State Format

`.provenance/state/` is the canonical store. Records are newline-delimited JSON with stable string `id` fields, `schema_version`, and deterministic ordering by primary key inside each shard.

Scopes live in `manifest.json`; shard paths derive from scope IDs. Cache files and volatile fields are forbidden in state shards.

Schema version `1` includes the local graph fields plus imported/cloud review metadata. Optional fields are omitted when absent, but preserved when present: domain grouping for root requirements, requirement descriptions and source references, source references/clauses/effective/review/supersession dates/commit pins, draft/review statuses, resolution context/enforcement/confidence/input references/actor approval/supersession metadata, resolved thread status, rule name/type/modality/confidence/extraction/source-location metadata, deployed services, rule-to-service bindings, proposal confidence, and material-claim confidence.

Concurrent writers serialize each complete logical mutation with an exclusive generation lock under `.provenance/cache/locks/`. A `StateStore` transaction is the boundary: it stages and syncs every replacement in that mutation before publication, keeps rollback copies, and records a recovery journal before activating any shard. This is not a transaction across multiple CLI commands. Generation snapshots and exports take the same lock and recover an interrupted publication before reading, so those cooperating readers expose either the old complete generation or the new complete generation. Lock, journal, staging, and rollback files are derived cache artifacts, not state, and must not be committed.
