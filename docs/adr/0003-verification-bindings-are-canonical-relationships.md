# ADR 0003: Verification bindings are canonical relationships

## Status

Accepted.

## Context

A callback run can say that one execution passed or failed, but it cannot say
which test is intended to verify a Rule across executions. File names, line
numbers, and callback names all change too easily to provide that identity.
The generic graph also has no Test node, and adding one would commit every
language integration to a Test lifecycle that the POC does not need.

## Decision

Store a first-class Verification binding beside the canonical graph. Its stable
identity is the verification owner, canonical Rule ID, and an explicit
owner-local key. Method, repository path, and symbol are updateable facts. A
callback run is volatile evidence that cites the binding and snapshots its
execution context.

Do not add a stored `verified` status. Coverage derives Unverified from the
absence of both scanner-discovered sites and canonical typed bindings. Do not
add Test to the generic node or edge ontology yet.

## Consequences

Repeated runs reuse one relationship, while moves and method changes do not
silently create new identities. Typed bindings can participate in exports,
checks, coverage, stale analysis, queries, and wiki output without turning
test outcomes into Git-tracked state. Authors must supply one local key to
`verify`, which is the cost of stable identity.
