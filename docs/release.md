# Release

Releases are published by GitHub Actions when a version tag is pushed.

## Targets

The release workflow builds and uploads:

- `provenance-<tag>-x86_64-pc-windows-msvc.zip`
- `provenance-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- `provenance-<tag>-x86_64-apple-darwin.tar.gz`
- `provenance-<tag>-aarch64-apple-darwin.tar.gz`
- `SHA256SUMS`

## Cut A Release

Update crate versions, then tag and push:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The `Release` workflow creates the GitHub Release, attaches archives, and generates release notes.

## Local Build

Build the local binary with:

```sh
cargo build --release -p provenance-cli --all-features
```

A release scans clean: `provenance coverage scan --path . --scope default --validate-rules --strict` exits zero, so every marker cites a real rule and every active rule has a verification site.

The binary lands at `target/release/provenance`. Users should commit `.provenance/state/` and ignore `.provenance/cache/`.
