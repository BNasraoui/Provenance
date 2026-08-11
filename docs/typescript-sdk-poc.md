# TypeScript SDK proof of concept

## Outcome

The typed surface works with a small façade and a one-shot child-process
protocol. TypeScript owns pure declaration construction, immutable handles,
callback execution, and error serialization. The Rust engine assigns IDs,
validates and reconciles records, writes canonical state and graph edges, and
stores verification outcomes.

The package interface is:

```ts
import { defineSpec } from "provenance";

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
`expiry.verify(callback)`. The callback never crosses the process seam. Node
runs it between Rust-backed begin and complete commands. On failure, the SDK
sends a serialized error and rethrows the exact value caught from the callback.

## Process protocol

The SDK launches the CLI for each operation and exchanges one JSON document on
stdin/stdout:

- `provenance sdk apply` reconciles one complete declaration document.
- `provenance sdk begin-verification` checks the rule and creates a running
  evidence record.
- `provenance sdk complete-verification` records passed or failed.
- `provenance sdk verification-runs` queries that evidence, optionally by rule.

No daemon, socket, native addon, FFI object graph, or callback bridge is used
in this POC. Each verification uses two short-lived Rust processes. The package
still needs a `provenance` binary on `PATH` or in `PROVENANCE_BIN`; bundling and
supervising a platform binary remains required before the intended
`npm install` experience is complete.

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

Each run carries a validated canonical rule ID, method, producer, optional
call-site path and symbol, status, timestamps, and an optional serialized
error. A typed handle starts the run with its declaration address; Rust resolves
that address to the canonical ID and rejects an unapplied declaration before
Node executes the callback. Keeping runs in the existing derived cache prevents
every local test from dirtying Git-tracked canonical state. Declarations
themselves remain in `.provenance/state` and therefore appear in exports,
traceability queries, checks, graph references, and generated wiki pages like
records created by the existing CLI.

The current wiki Verification section still reads static coverage reports;
it does not render cached callback outcomes in this POC. Runtime results are
queried with `sdk verification-runs`. A durable Test-to-Rule binding and its
integration with existing stale, coverage, and wiki semantics remain separate
work from transient test runs.

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

The experiment exposed a semantic question rather than silently resolving
it. The current domain glossary says a Rule is carried by its deciding
function. A typed declaration can materialize a Rule record before any
function binding exists. The wiki and coverage scan report that state
honestly as unbound. Making typed claims first-class would require deciding
whether this is a valid Rule, a draft Rule, or a requirement whose Rule is
still unwritten. This POC does not change the glossary.

## Answers from the POC

1. `expiry.verify()` is more natural than a repeated string marker for tests.
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

The next decision is whether the authoring and navigation gain justifies
resolving the Rule semantic question and integrating runtime evidence into
coverage/wiki views. More languages are not the next task.
