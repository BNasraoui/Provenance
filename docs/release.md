# Release

Build the local binary with:

```sh
cargo build --release -p provenance-cli --all-features
```

A release scans clean: `provenance coverage scan --path . --scope default --validate-rules --strict` exits zero, so every marker cites a real rule and every active rule has a verification site.

Distribute `target/release/provenance`. Users should commit `.provenance/state/` and ignore `.provenance/cache/`.
