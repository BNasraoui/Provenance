# TypeScript SDK proof of concept

## Outcome

The typed surface works with a small façade and a one-shot child-process
protocol. TypeScript owns pure declaration construction, immutable handles,
callback execution, and error serialization. The Rust engine assigns IDs,
validates and reconciles records, writes canonical state and graph edges, and
stores verification outcomes.

The package interface is:

```ts
import { defineSpec } from "@quality-sh/provenance";

const spec = defineSpec("share-links", ({ requirement, source }) => {
  const authority = source("linear:ABC-123", {
    kind: "linear",
    name: "Linear ABC-123",
    url: "https://linear.app/example/issue/ABC-123",
  });
  const sharing = requirement("sharing", {
    statement: "Users can securely share documentation",
    sources: [authority],
  });
  const expiry = sharing.rule("expiry", {
    statement: "Share links must expire within 30 days",
  });

  return { sharing, expiry };
});

export default spec;
export const { expiry, sharing } = spec.handles;
```

An explicit entry point calls `apply(spec)`. Importing the declaration module
only constructs and freezes values in memory. Tests import `expiry` and call
`expiry.verify("share-link-expiry", callback)`. The local key gives the durable
Verification binding a stable identity; the Rule itself remains a real typed
reference. The callback never crosses the process seam. Node
runs it between Rust-backed begin and complete commands. On failure, the SDK
sends a serialized error and rethrows the exact value caught from the callback.

## Process protocol

The SDK launches the CLI for each operation and exchanges one JSON document on
stdin/stdout:

- `provenance sdk info` reports the engine, protocol, state schema, and resolved
  project root. The SDK uses it to reject an incompatible engine before sending
  declarations or evidence.
- `provenance sdk apply` reconciles one complete declaration document.
- `provenance sdk plan` previews the same reconciliation without publishing it.
- `provenance sdk begin-verification` checks the rule and creates a running
  evidence record.
- `provenance sdk complete-verification` records passed or failed.
- `provenance sdk verification-runs` queries that evidence, optionally by rule.

No daemon, socket, native addon, FFI object graph, or callback bridge is used
in this POC. Each verification uses two short-lived Rust processes. Published
SDK packages resolve a platform-specific optional dependency containing the
Rust engine. Installation runs no binary download or Rust compilation.
`PROVENANCE_BIN` remains an explicit development override.

An explicit `--repo` / `PROVENANCE_REPO` setting wins. Otherwise the engine
walks upward from the working directory and selects the nearest initialized
Provenance project or Git root. This keeps project discovery in Rust so later
language SDKs share the same behaviour.

## Identity and ownership

Each declaration has two identities:

- a structured, owner-local declaration address used by typed handles;
- a canonical Provenance Stable ID assigned or accepted by Rust.

The address includes the spec and hierarchy. `sharing/expiry` and
`sessions/expiry` are distinct, as are equal top-level keys in different specs.
Declaration keys are not TypeScript variable names. Renaming:

```ts
export const expiry = sharing.rule("expiry", ...);
```

to:

```ts
export const shareLinkExpiry = sharing.rule("expiry", ...);
```

leaves both the declaration address and canonical ID unchanged. On reapply,
Rust first resolves the owner and address to the persisted Stable ID. A new
address receives a deterministic implicit ID; an explicit `id` can reuse an
existing Stable ID and is the current escape hatch when a declaration must move
without changing canonical identity. Immutable handles do not cache IDs;
`apply` returns them and Rust resolves later verification by owner and address.

Sources, requirements, and rules carry optional `declared_by` metadata.
Apply may create a missing record or update a record with the same owner. It
refuses to take over an unowned record or one owned by another integration,
and performs that check before writing. Omitted declarations and graph edges
are retained. Fields outside the small TypeScript interface are preserved;
source references declared by the spec are added rather than replacing
external references.

This is deliberately not a full lifecycle engine. The POC has no adoption,
pruning, automatic rename/move inference, deletion, or ownership-transfer
operation. Changing a rule's parent without an explicit ID creates a distinct
declaration. Reusing an explicit ID preserves the record, but relationships
remain additive and the old edge is not deleted.

## Verification evidence

Callback results are volatile run evidence, so they live under:

```text
.provenance/cache/scopes/<scope>/verification-runs.jsonl
```

Each `verify` call names a stable owner-local binding key. Rust materializes one
canonical Verification binding for that key and Rule, while method, repository
path, and symbol remain updateable facts. Each run cites that binding and
carries the canonical Rule ID, execution context, status, timestamps, and an
optional serialized error. A typed handle starts the run with its declaration address; Rust resolves
that address to the canonical ID and rejects an unapplied declaration before
Node executes the callback. Keeping runs in the existing derived cache prevents
every local test from dirtying Git-tracked canonical state. Declarations
themselves remain in `.provenance/state` and therefore appear in exports,
traceability queries, checks, graph references, and generated wiki pages like
records created by the existing CLI.

The wiki and validating coverage scan consume canonical typed bindings alongside
scanner-discovered bindings. Runtime results remain separate and are queried
with `sdk verification-runs`; durable relationships are queried with
`sdk verification-bindings`. Stale analysis treats a changed typed verification
path as disturbed evidence without executing the callback.

## Compile-time result

The useful guarantee is ordinary TypeScript referential integrity. The valid
fixture imports `expiry` and typechecks. A second fixture renames the export to
`shareLinkExpiry` but leaves the import unchanged; `tsc` fails with TS2305.
This proves only that the verification code refers to a declared rule handle.
It does not prove that the callback tests the right production behaviour.

## Coexistence with existing bindings

The `@provenance/rules` identity helpers, scanner patterns, Rust attributes,
and comment directives are unchanged. Typed declarations create the same
canonical Rule records those bindings cite. A codebase may therefore keep a
scanner-recognized implementation binding while tests use imported handles.

The experiment resolved the semantic question it exposed. A Rule is an
independent behavioural obligation, so a typed declaration may materialize a
valid Rule before any production function realizes it. `#[rule]` and the
equivalent language helpers bind a primary implementation; they do not define
the Rule. A missing implementation is reported as Unimplemented. Existing
Rule records, source-citation fields, decorators, attributes, and scanner
patterns keep their shape, so retrofit and typed authoring target the same
canonical model without a data migration.

## Answers from the POC

1. `expiry.verify("local-key", callback)` is more natural than a repeated Rule
   ID marker for tests. The string identifies the test relationship, not the
   Rule. The imported Rule handle provides that referential integrity.
   Imports, rename, autocomplete, navigation, and find-references all work in
   the TypeScript toolchain. The explicit `defineSpec` / `apply(spec)` split is
   also easier to reason about than persistence triggered by module import or
   the first test. It still adds an apply step and a Rust binary prerequisite.
2. The façade remains small. It builds a desired-state document, freezes typed
   handles, invokes four commands, wraps callbacks, and serializes errors.
   Reconciliation, canonical IDs, source-kind mapping, ownership checks, graph
   writes, and evidence validation remain in Rust.
3. One-shot child processes are sufficient for the protocol POC. They do not
   deliver the promised install-and-forget UX; package-supplied binary discovery
   and lifecycle management are still needed. Nothing here justifies gRPC.
4. Typed declarations coexist with external state by refusing takeover,
   retaining omissions and edges, and preserving fields outside the façade.
   Full lifecycle semantics remain unsolved by design.
5. The operations are portable: declare a source, requirement, or rule; apply
   desired state; begin a verification; run a language callback; complete the
   verification. No operation depends on a TypeScript-only runtime concept.

The next decision is how durable verification relationships and runtime
evidence should shape CI policy. Typed bindings already join coverage, stale
detection, wiki views, and semantic change plans. More languages are not the
next task.
