# TypeScript SDK fixture

This small app keeps production code unaware of Provenance. The spec declares
traceability with the package named `provenance`; the test imports the typed
`expiry` handle and verifies ordinary production code.

From this directory, with the Rust CLI built:

```sh
npm install
provenance init --path . --scope default --path-prefix .
PROVENANCE_BIN=../../target/debug/provenance npm test
provenance rules show --id expiry --format json
provenance sdk verification-runs --rule expiry --format json
provenance wiki build --format json
```

The `file:../../packages/provenance` dependency exercises normal npm package
resolution without claiming that this POC has been published to npm.
