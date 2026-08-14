# TypeScript SDK fixture

This small app keeps production code unaware of Provenance. The spec declares
traceability with `@quality-sh/provenance`; the test follows the built spec's
typed `requirements.sharing.rules.expiry` path and verifies ordinary production
code. Sources linked by `Requirement.from(...)` are collected by `build()`.

From this directory, with the Rust CLI built:

```sh
npm install
../../target/debug/provenance init --path . --scope default --path-prefix .
PROVENANCE_BIN=../../target/debug/provenance npm test
../../target/debug/provenance rules list --format json
../../target/debug/provenance sdk verification-runs --format json
../../target/debug/provenance wiki build --format json
```

`npm test` compiles the app, applies the spec through a separate entry point,
then runs the typed verification. Importing `provenance.spec.ts` alone has no
engine or persistence side effect.

The `file:../../packages/provenance` dependency exercises normal npm package
resolution from the source checkout. The packed-install test covers the
published package shape and its bundled engine separately.
