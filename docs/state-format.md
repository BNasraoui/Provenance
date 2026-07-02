# State Format

`.provenance/state/` is the canonical store. Records are newline-delimited JSON with stable string `id` fields, `schema_version`, and deterministic ordering by primary key inside each shard.

Scopes live in `manifest.json`; shard paths derive from scope IDs. Cache files and volatile fields are forbidden in state shards.

Schema version `1` includes the local graph fields plus imported/cloud review metadata. Optional fields are omitted when absent, but preserved when present: domain grouping for root requirements, requirement descriptions and source references, source references/clauses/effective/review/supersession dates, draft/review statuses, resolution context/enforcement/confidence/input references/actor approval/supersession metadata, resolved thread status, rule name/type/modality/confidence/extraction/source-location metadata, deployed services, and rule-to-service bindings.
