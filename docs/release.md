# Release

Build the local binary with:

```sh
cargo build --release -p provenance-cli --all-features
```

Distribute `target/release/provenance`. Users should commit `.provenance/state/` and ignore `.provenance/cache/`.
