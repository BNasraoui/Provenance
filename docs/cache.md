# Cache

`.provenance/cache/provenance.db` is generated from canonical JSONL, including graph records, domains, shaping records, services, and service bindings. It can be deleted and rebuilt with `provenance materialize`.

Migrations are applied transactionally and record applied versions in SQLite. The database is optimized for graph queries and is never the source of truth.
