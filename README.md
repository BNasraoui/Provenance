# Local Provenance

This directory includes a standalone Rust CLI for local-first Provenance.

```bash
cargo run -p provenance-cli -- init --path /tmp/provenance-smoke --scope default --path-prefix .
cargo run -p provenance-cli -- check --repo /tmp/provenance-smoke --format json
cargo run -p provenance-cli -- materialize --repo /tmp/provenance-smoke --format json
```

The canonical store is `.provenance/state/` and should be committed. The SQLite cache under
`.provenance/cache/` is generated and ignored.
