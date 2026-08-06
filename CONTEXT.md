# Domain Glossary

## Graph reference

An immutable identification of one canonical graph scope at one pinned repository commit. Its identity includes the repository, canonical store, scope, commit, and graph content.

## Pinned commit

The complete Git commit identity from which a graph reference is read. A pinned read is independent of later working-tree changes.

## Exact export

The canonical graph content recovered for a graph reference from its pinned commit.

## Relevant canonical state

The selected scope declaration and graph records that contribute to that scope. Collaboration history and derived data are not canonical graph state.

## External correlation

An optional association between a graph reference and an identifier owned by another system. It does not participate in graph-reference identity.

## Commit-then-issue

The handoff in which canonical graph changes are committed before a graph reference is issued, so issuance does not create new canonical state.

## Proposal

An immutable modern candidate definition. It is always authored as `proposed`; assertion and disposition records derive its effective state without rewriting it.

## Assertion

Immutable evidence that one proposal passed unblocked adjudication using positive, uniquely owned evidence. Proposal lineage names assertion IDs, not mutable proposal state.

## Disposition

The sole immutable authority for `accepted`, `rejected`, or `deferred`. Its actor ID is a repository-allowlisted audit attestation under repository and CLI access, not proof of cryptographic or human identity.

## Frozen legacy terminal

A pre-lifecycle proposal row whose terminal definition is covered by the compiled, versioned shipped-v1 fingerprint. It remains readable but cannot be asserted, disposed again, replaced, or used as authority for new lifecycle ingress.
