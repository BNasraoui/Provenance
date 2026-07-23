# Domain Glossary

## Domain

A reader-facing taxonomy classification for requirements. A derived rule belongs to each Domain of its upstream requirements through the canonical graph relationships.

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

## Commit-then-issue

The handoff in which canonical graph changes are committed before a graph reference is issued, so issuance does not create new canonical state.
