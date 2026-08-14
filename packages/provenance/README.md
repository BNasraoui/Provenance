# `@quality-sh/provenance` TypeScript SDK

This package is an optional typed façade over the Provenance Rust engine. It
does not implement graph semantics or persistence in JavaScript.

```sh
npm install @quality-sh/provenance
npx provenance init --path . --scope default --path-prefix .
```

Define a spec without touching the engine:

```ts
import { defineSpec, requirement, rule } from "@quality-sh/provenance";
import { createShareLink } from "./share-links.js";

const expiry = rule("expiry")
  .statement("Share links must expire within 30 days")
  .implementedBy(createShareLink);
const sharing = requirement("sharing")
  .statement("Users can securely share documentation")
  .rules(expiry);
const spec = defineSpec("share-links")
  .requirements(sharing)
  .build();

export default spec;
export const { sharing: sharingRequirement } = spec.handles.requirements;
export const { expiry: expiryRule } = sharingRequirement.rules;
```

Each fluent call returns a new immutable declaration. `build()` validates and
finalizes the declarations into frozen handles. Construction is synchronous,
deterministic, and in-memory: importing this module does not write state or
start a process. Reusing one Rule declaration under several Requirements emits
one Rule with several relationships. Equal local Rule keys remain distinct when
their declaration objects and parent Requirements differ.

`implementedBy()` accepts a direct named import or namespace member. The SDK
reads that expression from the spec source and records the imported module and
exported symbol; it never inspects the function object, its name, or its body.
Calls, conditionals, computed members, and local values fail clearly because
they do not provide one durable source identity. Rust checks that the resolved
file belongs to the repository and owns the canonical implementation binding.
Production code does not import Provenance.

Moving a local Rule to a shared declaration, or back, preserves its canonical
ID when Rust finds exactly one owned candidate. If several local Rules could
become the shared Rule, apply fails instead of guessing. An immutable
`.id(existingId)` call can choose the canonical record, but this lifecycle slice
is additive: it does not retire the other Rule or remove old graph edges.

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
import { expiryRule } from "./provenance.spec.js";

await expiryRule.verify("share-link-expiry", async () => {
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

The original object-options declarations and callback form of `defineSpec()`
remain compatible. The older top-level API uses a process-local registry and
`verify()` applies pending declarations automatically. New code should prefer
fluent declarations plus explicit `apply(spec)` so imports stay free of hidden
persistence.

See `examples/typescript-sdk/` for package-name consumption through a local npm
dependency.
