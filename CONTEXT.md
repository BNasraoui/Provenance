# Domain Glossary

## Domain

A reader-facing taxonomy classification for requirements. A derived rule belongs to each Domain of its upstream requirements through the canonical graph relationships.

## Rule

A statement produced by a decision, carried in code by the function that decides it. The function is bound to the rule's graph record by a `#[rule("rule_id")]` marker. Where a type makes the violating value unbuildable, the type carries the rule instead and its construction is the proof.

## Verification

Evidence that a rule holds, carried by a `#[verifies("rule_id", method)]` marker on a test or a type. The method is one of `exhaustion`, `property`, `examples`, `conformance`, `construction`, or `proof`. Exhaustion over a finite domain is proof, not a sample; `proof` names a machine-checked proof outside the test runner, bridged by the marked site.

## Enforcement

The live path: the running code that rejects a violation. Verification is evidence about that code; enforcement is the code itself.

## Unverified

An active rule with no verification marker anywhere in the scanned tree. It is absence, not a stored field: `provenance coverage scan --path . --validate-rules` derives it at scan time and reports it, and no shard records it.

## Evidence site

A source line carrying a rule binding, verification binding, or provenance annotation. Its file path and line number remain its human-readable coordinate.

## Evidence anchor

The enclosing symbol and content identity recorded alongside an Evidence site's coordinate. A later scan resolves the anchor before deciding whether the site is Unchanged, New, Moved, or Gone; these states are derived report findings, not canonical graph state.

## Evidence path

A repository path the graph makes evidentiary: an Evidence site citing a known Rule, or a code path named by a Source that a Requirement references. A diff can leave the path Untouched, Touch it so re-verification is wanted, Move its durable anchor, or leave it Gone. These are report findings; running the gate performs no review or re-extraction.

## Topic

A persisted, claimable shaping work area attached to a requirement. A Topic is not a reader taxonomy classification.

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

## External action correlation

An optional immutable association between a Disposition and one action owned by another system. Its identity is the exact system, external scope, action kind, and stable key tuple; equal keys in different systems, scopes, or kinds are distinct. It is audit context, not Disposition identity or workflow state.

## Declaration owner

The integration URI allowed to reconcile a Source, Requirement, or Rule definition carrying the same owner. It grants no authority over other records, the whole graph, or facts the declaration does not state.

## Declaration address

An owner-local hierarchical identity for one typed declaration. Equal child keys under different parents have distinct addresses. The address is not the canonical Stable ID.

## Commit-then-issue

The handoff in which canonical graph changes are committed before a graph reference is issued, so issuance does not create new canonical state.

## Proposal

An immutable modern candidate definition. It is always authored as `proposed`; assertion and disposition records derive its effective state without rewriting it.

## Proposal demand

A bounded occasion to consult undisposed Proposals because current work names an exact changed evidence path or an explicit typed Territory. Proposal demand is not a global review queue.

## Territory

The typed artifacts explicitly claimed by current shaping work: a Topic, its anchor Requirement, and its declared artifact links. Similar names or graph proximity do not expand Territory.

## Assertion

Immutable evidence that one proposal passed unblocked adjudication using positive, uniquely owned evidence. Proposal lineage names assertion IDs, not mutable proposal state.

## Disposition

The sole immutable authority for `accepted`, `rejected`, or `deferred`. Its actor ID is a repository-allowlisted audit attestation under repository and CLI access, not proof of cryptographic or human identity.

## Ratification through action

Acceptance recorded when a human action resolves the relevant problem and produces an existing canonical artifact. The immutable Disposition names the artifact, may correlate the external action that produced it, and preserves the Proposal definition unchanged.

## Frozen legacy terminal

A pre-lifecycle proposal row whose terminal definition is covered by the compiled, versioned shipped-v1 fingerprint. It remains readable but cannot be asserted, disposed again, replaced, or used as authority for new lifecycle ingress.
