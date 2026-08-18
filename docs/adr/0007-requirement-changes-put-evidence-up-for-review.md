# ADR 0007: A changed Requirement puts evidence up for review

## Status

Accepted.

## Context

`plan` reconciled rows. It could say a Requirement was updated, but not what
that meant for the Rules underneath it or for the tests already vouching for
them. A reader had to work out which evidence a reworded obligation called
into question.

Provenance already has a word for evidence that needs another look. `stale`
means the code carrying the evidence changed, and it is computed from the diff.
Reusing it here would be wrong. When only a sentence in a Requirement changes,
no file moved and no test was touched, so nothing about the code is out of
date. Saying otherwise would teach readers to distrust the word.

## Decision

Restating a Requirement puts the evidence of every Rule it produces into a
state called review required, which is distinct from stale. Only the statement
counts, because it carries the obligation.

Apply writes one record per affected Rule naming the Requirement, the field,
its value before and after, and when the change landed. The record is the
authority; `plan` reads it rather than inferring the state. Because `plan` also
holds the diff it is previewing, it reports the review that applying that diff
would raise, marked apart by having no timestamp.

A verification run arriving for a Rule after the change clears that Rule's
review. Clearing writes the run and the time onto the record and keeps the
reason. Only reviews raised before the run clear, because an earlier run cannot
speak for a later change. Nobody re-reads old test data by hand.

## Consequences

A reworded Requirement produces a list of Rules, their implementation and
verification sites, and a plain reason each one deserves attention. Rerunning
the tests for a Rule answers it. The history of why review was asked for
survives, so a later reader can see which wording change prompted a rerun.
Evidence that is out of date because the code moved keeps its own separate
report.
