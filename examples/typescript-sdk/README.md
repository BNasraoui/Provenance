# TypeScript SDK fixture

This small app keeps production code unaware of Provenance. The spec declares
traceability with the package named `provenance`; the test imports the typed
`expiry` handle and verifies ordinary production code.

From this directory, with the Rust CLI built:

```sh
npm install
provenance init --path . --scope default --path-prefix .
PROVENANCE_BIN=../../target/debug/provenance npm test
provenance rules list --format json
provenance sdk verification-runs --format json
provenance wiki build --format json
```

`npm test` compiles the app, applies the spec through a separate entry point,
then runs the typed verification. Importing `provenance.spec.ts` alone has no
engine or persistence side effect.

The `file:../../packages/provenance` dependency exercises normal npm package
resolution without claiming that this POC has been published to npm.
