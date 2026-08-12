# `provenance` TypeScript SDK (POC)

This package is an optional typed façade over the Provenance Rust engine. It
does not implement graph semantics or persistence in JavaScript.

Define a spec without touching the engine:

```ts
import { defineSpec } from "provenance";

const spec = defineSpec("share-links", ({ requirement }) => {
  const sharing = requirement("sharing", {
    statement: "Users can securely share documentation",
  });
  const expiry = sharing.rule("expiry", {
    statement: "Share links must expire within 30 days",
  });

  return { sharing, expiry };
});

export default spec;
export const { expiry, sharing } = spec.handles;
```

Construction is synchronous, deterministic, and in-memory. `defineSpec`
finalizes mutable builders into frozen handles. Importing this module does not
write state or start a process.

Materialize only this spec at a deliberate entry point:

```ts
import { apply } from "provenance";
import spec from "./provenance.spec.js";

await apply(spec);
```

A test imports the actual rule handle and runs its callback in Node:

```ts
import { expiry } from "./provenance.spec.js";

await expiry.verify(async () => {
  // Exercise ordinary production code with the test runner of your choice.
}, { key: "share-link-expiry" });
```

The handle keeps an owner-local declaration address, not a mutable database
ID. Rust resolves that address to the canonical Rule when verification begins.
Calling `verify` before applying the spec fails before the callback runs. A
failed callback is recorded and the original error is rethrown.

The POC still invokes a `provenance` binary on `PATH`. These environment
variables override the defaults:

- `PROVENANCE_BIN`: engine binary; default `provenance`
- `PROVENANCE_REPO`: repository; default current directory
- `PROVENANCE_SCOPE`: scope; default `default`
- `PROVENANCE_SPEC_OWNER`: declaration owner; default `spec://typescript`
- `PROVENANCE_VERIFICATION_OWNER`: evidence producer; default `ci://typescript`

`configure()` provides the same settings in code. A managed, package-supplied
engine is planned but is not part of this POC yet.

The original top-level `source()` / `requirement()` functions remain for POC
compatibility. They use a process-local registry and `verify()` applies pending
declarations automatically. New code should prefer `defineSpec()` plus explicit
`apply(spec)` so imports stay free of hidden persistence.

This is an in-repository POC package, not an npm release. See
`examples/typescript-sdk/` for package-name consumption through a local npm
dependency.
