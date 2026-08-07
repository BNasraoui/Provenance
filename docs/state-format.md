# State Format

`.provenance/state/` is the canonical store. Records are newline-delimited JSON with stable string `id` fields, `schema_version`, and deterministic ordering by primary key inside each shard.

Scopes live in `manifest.json`; shard paths derive from scope IDs. Cache files and volatile fields are forbidden in state shards.

Schema version `1` includes the local graph fields plus imported/cloud review metadata. Optional fields are omitted when absent, but preserved when present: domain grouping for root requirements, requirement descriptions and source references, source references/clauses/effective/review/supersession dates/commit pins, draft/review statuses, resolution context/enforcement/confidence/input references/actor approval/supersession metadata, resolved thread status, rule name/type/modality/confidence/extraction/source-location metadata, deployed services, rule-to-service bindings, proposal confidence, and material-claim confidence.

Modern proposal definitions are immutable `proposed` rows. Assertions live in
`ideation/assertions.jsonl`; dispositions use `ideation/dispositions.jsonl`. Readers accept
the previously shipped `ideation/promotion_decisions.jsonl` path only for the exact frozen
historical audit. Import also accepts the old `promotion_decisions` export field instead of,
but never alongside, `dispositions`; all other top-level fields are closed. Effective state is
derived in the order
`proposed`, `asserted`, then disposition. `ideation/landings.jsonl` stores one validated
swarm batch per line so a run is published atomically. Import validates a staged complete
state directory and renames it into place only after `check` succeeds.

Pre-lifecycle terminal proposal rows are not rewritten. The compiled, versioned shipped-v1
fingerprint policy freezes both the exact shipped terminal definitions and their historical
disposition audit; any added, omitted, or changed frozen row fails validation. Embedded terminal
state remains authoritative even if lifecycle rows are present. The audit fingerprint includes
only dispositions targeting that frozen terminal set, so allowlisted modern lifecycle records can
coexist in the same scope. New terminal definitions are never accepted as modern ingress.

Repository access first takes `.provenance/cache/locks/repository.publication.lock`. Writers then
take a scope lifecycle lock when applicable and finally a shard lock; this repository, lifecycle,
shard order is mandatory. Multi-shard writers hold the publication lock for their complete
operation, and aggregate readers hold it for their complete view or copy one locked snapshot.
Import holds the publication lock across recovery, snapshot, staged validation, and publication,
so cooperative readers and writers cannot observe or modify the brief directory-rename gap. Lock
files are derived cache artifacts, not state, and must not be committed.

Import publication uses a durable `.provenance/cache/import-publication.json` marker and unique
staging/backup directory. Repository access recovers an interrupted publication before reading:
if live state is absent, the backup is restored; if live state exists, pending backup cleanup is
finished. Files and containing directories are synced where the platform supports directory
`fsync`. Portable filesystems do not provide an atomic directory exchange, so the guarantee is
not overstated as crash-atomic: cooperating access never sees missing live state, interruption is
recoverable on next access, and any import command that returns failure leaves the old live state.

Graph reference v1 canonicalizes a selected scope into a JSON object with fixed graph
families and records sorted by stable ID. JSON object keys are lexicographically ordered
before SHA-256 hashing. The projection contains the selected manifest scope and its
sources, domains, requirements, boundaries, topics, questions, resolutions, rules,
services, service bindings, and edges. Threads, messages, contributions, synthesis
packets, proposals, assertions, dispositions, cache data, and wiki output are excluded.
