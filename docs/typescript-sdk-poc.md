# TypeScript SDK proof of concept

## Outcome

The typed surface works with a small façade and a one-shot child-process
protocol. TypeScript owns declarations, handles, callback execution, and error
serialization. The Rust engine assigns IDs, validates and reconciles records,
writes canonical state and graph edges, and stores verification outcomes.

The package interface is:

```ts
const authority = source("linear:ABC-123", {
  kind: "linear",
  name: "Linear ABC-123",
  url: "https://linear.app/example/issue/ABC-123",
});

const sharing = requirement("sharing", {
  statement: "Users can securely share documentation",
  sources: [authority],
});

export const expiry = sharing.rule("expiry", {
  statement: "Share links must expire within 30 days",
});

await apply();
```

Tests import `expiry` and call `expiry.verify(callback)`. The callback never
crosses the process seam. Node runs it between Rust-backed begin and complete
commands. On failure, the SDK sends a serialized error and rethrows the exact
value caught from the callback.

## Process protocol

The SDK launches the CLI for each operation and exchanges one JSON document on
stdin/stdout:

- `provenance sdk apply` reconciles one complete declaration document.
- `provenance sdk begin-verification` checks the rule and creates a running
  evidence record.
- `provenance sdk complete-verification` records passed or failed.
- `provenance sdk verification-runs` queries that evidence, optionally by rule.

No daemon, socket, native addon, FFI object graph, or callback bridge is
needed. Each verification uses two short-lived Rust processes. That is enough
to test the interface; batching or a persistent child remains a performance
option, not an architectural requirement.

## Identity and ownership

Declaration keys are not TypeScript variable names. Renaming:

```ts
export const expiry = sharing.rule("expiry", ...);
```

to:

```ts
export const shareLinkExpiry = sharing.rule("expiry", ...);
```

leaves the canonical ID unchanged. A key that is already a valid Provenance
stable ID is reused as-is. An explicit `id` can name an existing stable ID.
Other keys, such as `linear:ABC-123`, receive a deterministic Rust-generated
ID. Handles learn the returned canonical ID after a successful apply.

Sources, requirements, and rules carry optional `declared_by` metadata.
Apply may create a missing record or update a record with the same owner. It
refuses to take over an unowned record or one owned by another integration,
and performs that check before writing. Omitted declarations and graph edges
are retained. Fields outside the small TypeScript interface are preserved;
source references declared by the spec are added rather than replacing
external references.

This is deliberately not a full lifecycle engine. The POC has no adoption,
pruning, rename/move, deletion, or ownership-transfer operation. Changing a
rule's parent adds the new relationship and does not delete the old one.

## Verification evidence

Callback results are volatile run evidence, so they live under:

```text
.provenance/cache/scopes/<scope>/verification-runs.jsonl
```

Each run carries a validated canonical rule ID, method, producer, optional
call-site path and symbol, status, timestamps, and an optional serialized
error. Keeping runs in the existing derived cache prevents every local test
from dirtying Git-tracked canonical state. Declarations themselves remain in
`.provenance/state` and therefore appear in exports, traceability queries,
checks, graph references, and generated wiki pages like records created by
the existing CLI.

The current wiki Verification section still reads static coverage reports;
it does not render cached callback outcomes in this POC. Runtime results are
queried with `sdk verification-runs`. Joining those two evidence views is a
follow-up only if the typed UX proves worth keeping.

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
   the TypeScript toolchain. It does add an apply step and a Rust binary
   prerequisite.
2. The façade remains small. It captures declarations, holds returned IDs,
   invokes four commands, wraps callbacks, and serializes errors. Reconciliation,
   ID rules, source-kind mapping, ownership checks, graph writes, and evidence
   validation remain in Rust.
3. One-shot child processes are sufficient for the POC. There is no evidence
   yet that a daemon, socket, or gRPC would improve the model.
4. Typed declarations coexist with external state by refusing takeover,
   retaining omissions and edges, and preserving fields outside the façade.
   Full lifecycle semantics remain unsolved by design.
5. The operations are portable: declare a source, requirement, or rule; apply
   desired state; begin a verification; run a language callback; complete the
   verification. No operation depends on a TypeScript-only runtime concept.

The next decision is whether the authoring and navigation gain justifies
resolving the Rule semantic question and integrating runtime evidence into
coverage/wiki views. More languages are not the next task.
