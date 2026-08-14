# `@quality-sh/provenance` TypeScript SDK

This package is an optional typed façade over the Provenance Rust engine. It
does not implement graph semantics or persistence in JavaScript.

```sh
npm install @quality-sh/provenance
npx provenance init --path . --scope default --path-prefix .
```

Define a spec without touching the engine:

```ts
import { defineSpec, requirement, rule, source } from "@quality-sh/provenance";
import { createShareLink } from "./share-links.js";

export const shareLinks = defineSpec("share-links")
  .requirements(
    requirement("sharing")
      .statement("Users can securely share documentation")
      .description("Controls for links shared outside the organization")
      .from(
        source("sharing-policy")
          .name("Sharing policy")
          .document("docs/sharing-policy.md"),
      )
      .rules(
        rule("expiry")
          .statement("Share links must expire within 30 days")
          .implementedBy(createShareLink),
      ),
  )
  .build();
```

Each fluent call returns a new immutable declaration. `build()` validates and
finalizes the desired-state document and collects Sources linked with
`Requirement.from(...)`; they do not need to be repeated in `.sources(...)`.
Construction is synchronous,
deterministic, and in-memory: importing this module does not write state or
start a process. `build()` returns the frozen semantic handles tests import.
The built spec exposes typed Requirement and Rule paths directly while keeping
the same objects under `.handles` for compatibility. Source `.name(...)`
overrides the key-derived canonical display name, and
Requirement `.description(...)` can replace the canonical description on a
later apply. Rust remains responsible for reconciling those desired values.

Helpers can name construction values directly, including across files:

```ts
import type {
  RequirementDeclaration,
  RuleDeclaration,
  SpecAuthoring,
} from "@quality-sh/provenance";

export function expiryRule<
  const Spec extends string,
  const RequirementKey extends string,
>(
  requirement: RequirementDeclaration<Spec, RequirementKey>,
): RuleDeclaration<Spec, "expiry", RequirementKey> {
  return requirement.rule("expiry").statement("Share links expire");
}

export function sharing<const Spec extends string>(author: SpecAuthoring<Spec>) {
  return author.requirement("sharing").statement("Shares expire");
}
```

`SourceDeclaration`, `RequirementDeclaration`, and `RuleDeclaration` describe
immutable construction snapshots, not materialized records or finalized spec
handles. Their literal spec and Requirement parameters keep helpers from
mixing declarations from different contexts.

Rules created through a Requirement have a requirement-local declaration
address, so equal local keys under different Requirements remain distinct. A
Rule created through the spec context has an explicitly shared address:

```ts
export const authenticatedExpiry = provenance
  .rule("authenticated-expiry")
  .statement("Authenticated access expires");

const shares = provenance
  .requirement("shares")
  .statement("Shares expire")
  .rules(authenticatedExpiry);
const sessions = provenance
  .requirement("sessions")
  .statement("Sessions expire")
  .rules(authenticatedExpiry);

export default provenance.build(shares, sessions);
```

This emits one Rule with two relationships. Shared versus local identity is
chosen by where the Rule is declared, not inferred from JavaScript object
reuse. Sources linked with `.from(...)` are collected transitively by `build()`.

`implementedBy()` accepts an exported function or class through a direct named
import or a non-computed namespace member. The SDK reads that expression from
the spec source and records the imported module and exported symbol; the runtime
value exists only for TypeScript assignability. It never inspects a function or
class name, body, prototype, or object identity, and it never constructs a class.
Calls, conditionals, computed members, instance methods, anonymous closures,
constructed values, and local functions fail clearly because they do not provide
one durable source identity. Rust checks that the resolved file belongs to the
repository and owns the canonical implementation binding. Production code does
not import Provenance.

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
import { shareLinks } from "./provenance.spec.js";

await shareLinks.requirements.sharing.rules.expiry.verify("share-link-expiry", async () => {
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

Spec-scoped declaration factories, object-options declarations, and the
callback form of `defineSpec()` remain available as compatibility surfaces. The
older object-options API uses a process-local registry and `verify()` applies
pending declarations automatically. New code should prefer the nested fluent
form above plus explicit `apply(spec)` so imports stay free of hidden
persistence.

See `examples/typescript-sdk/` for package-name consumption through a local npm
dependency.
