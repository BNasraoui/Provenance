# `provenance` TypeScript SDK (POC)

This package is an optional typed façade over the Provenance Rust CLI. It does
not implement graph semantics or persistence in JavaScript.

```ts
import { apply, requirement } from "provenance";

export const sharing = requirement("sharing", {
  statement: "Users can securely share documentation",
});

export const expiry = sharing.rule("expiry", {
  statement: "Share links must expire within 30 days",
});

await apply();
```

A test imports the actual rule handle and runs its callback in Node:

```ts
import { expiry } from "./provenance.spec.js";

await expiry.verify(async () => {
  // Exercise ordinary production code with the test runner of your choice.
});
```

`verify` applies pending declarations automatically, starts a verification
run through the Rust engine, executes the callback in Node, and completes the
run as passed or failed. A failed callback is recorded and the original error
is rethrown.

The Rust `provenance` binary must be on `PATH`. These environment variables
override the defaults:

- `PROVENANCE_BIN`: engine binary; default `provenance`
- `PROVENANCE_REPO`: repository; default current directory
- `PROVENANCE_SCOPE`: scope; default `default`
- `PROVENANCE_SPEC_OWNER`: declaration owner; default `spec://typescript`
- `PROVENANCE_VERIFICATION_OWNER`: evidence producer; default `ci://typescript`

`configure()` provides the same settings in code and clears declarations held
by the current process. `apply()` must succeed before reading a handle's
canonical `id`; Rust assigns that identity and returns it to the façade.

This is an in-repository POC package, not an npm release. See
`examples/typescript-sdk/` for package-name consumption through a local npm
dependency.
