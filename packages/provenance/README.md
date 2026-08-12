# `@quality-sh/provenance` TypeScript SDK

This package is an optional typed façade over the Provenance Rust engine. It
does not implement graph semantics or persistence in JavaScript.

```sh
npm install @quality-sh/provenance
npx provenance init --path . --scope default --path-prefix .
```

Define a spec without touching the engine:

```ts
import { defineSpec } from "@quality-sh/provenance";

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
import { apply, plan } from "@quality-sh/provenance";
import spec from "./provenance.spec.js";

await apply(spec);
```

Preview the same reconciliation without writing canonical state:

```ts
const proposed = await plan(spec);
```

Updated resources include field-level `before` and `after` values. Affected
Rules also list the implementation and verification sites that may need
review. Provenance computes both `plan` and `apply` through the same Rust
reconciliation path.

A test imports the actual rule handle and runs its callback in Node:

```ts
import { expiry } from "./provenance.spec.js";

await expiry.verify("share-link-expiry", async () => {
  // Exercise ordinary production code with the test runner of your choice.
});
```

The handle keeps an owner-local declaration address, not a mutable database
ID. Rust resolves that address to the canonical Rule when verification begins.
Calling `verify` before applying the spec fails before the callback runs. A
failed callback is recorded and the original error is rethrown.

The package installs a matching Rust engine through a platform-specific
optional dependency. It does not download a binary from an install script,
compile Rust, or require a global CLI. Before its first operation, the SDK
checks that the engine speaks the supported protocol. Rust then finds the
nearest enclosing Provenance or Git project for each command.

Published targets are macOS arm64/x64, Windows x64, and glibc Linux x64. An
unsupported host fails with the supported target list. These environment
variables override the defaults:

- `PROVENANCE_BIN`: explicit development engine; default packaged engine
- `PROVENANCE_REPO`: explicit repository; default nearest enclosing project
- `PROVENANCE_SCOPE`: scope; default `default`
- `PROVENANCE_SPEC_OWNER`: declaration owner; default `spec://typescript`
- `PROVENANCE_VERIFICATION_OWNER`: evidence producer; default `ci://typescript`

`configure()` provides the same settings in code. The SDK still uses one short
process per command; it does not start a daemon.

The original top-level `source()` / `requirement()` functions remain for POC
compatibility. They use a process-local registry and `verify()` applies pending
declarations automatically. New code should prefer `defineSpec()` plus explicit
`apply(spec)` so imports stay free of hidden persistence.

See `examples/typescript-sdk/` for package-name consumption through a local npm
dependency.
