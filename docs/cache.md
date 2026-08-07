# Cache

`.provenance/cache/provenance.db` is generated from canonical JSONL, including graph records, domains, shaping records, services, service bindings, source commit pins, proposal confidence, assertions, and derived proposal state. It can be deleted and rebuilt with `provenance materialize`.

Migrations are applied transactionally and record applied versions in SQLite. The database is optimized for graph queries and is never the source of truth.
Materialization runs the same lifecycle aggregate validator used by direct writes, swarm
landing, import, and `check` before clearing or loading cache tables. It copies canonical state
under the repository publication lock, then loads that coherent snapshot without holding a
synchronous filesystem lock across asynchronous SQLite work.
