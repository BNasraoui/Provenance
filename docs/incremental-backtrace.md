# Incremental backtrace and evidence review

The incremental path is a cheap gate in front of the adversarial backtrace workflow. It
does not replace the extraction and refutation stages in
`skills/provenance-swarm-backtrace/SKILL.md`, and it never changes promotion state.

## Command

```sh
provenance stale --repo . --scope default --format json
provenance evidence-review --base <revision> --head <candidate> --format json
```

`stale` reports resolution staleness using its original result shape. `evidence-review`
has a separate lifecycle and report. Without `--base`, each evidence site is compared
from its single, explicitly owned source `commit_pin` to `--head` (default `HEAD`).
Multi-source or unresolved ownership is diagnosed and rejected rather than guessed.
Revisions are resolved to commits before Git reads them. `--base` uses the supplied
revision directly as the old side of every diff; it does not calculate a merge base.
CI can supply a merge-base SHA when that is the desired common range. The override
supersedes each per-source comparison pin (while `source_revision` still reports that
pin) and permits an otherwise unpinned source to participate.

Both commands expose equivalent policy flags, but apply them to their own result type:

- `--min-age-days N` requires the approved resolution (`stale`) or pinned source commit
  (`evidence-review`) to be at least `N` days old.
- `--rule-severities high,critical` retains requirement evidence and resolutions with a
  downstream rule at one of those severities.
- `--min-downstream-rules N` requires at least `N` downstream rules after severity
  filtering.

Without requirement/rule filters, normal proposed `requirement_candidate` proposals
targeting their commit-pinned Source retain proposal-owned evidence and report no
fabricated `requirement_id`. Filters that inspect downstream requirements or rules
necessarily retain only evidence with canonical or explicit Requirement ownership.

Review dates are compared with the actual UTC day. Superseded approved resolutions use
their `superseded_by` field; the report no longer relies on an impossible
requirement-to-resolution `supersedes` edge or a fixed 2099 date.

## Cheap gate and re-verification

1. Collect repository-relative `file_path` values from proposal and contribution
   evidence in the selected scope.
2. Ask Git for a NUL-delimited, rename-aware name-status diff, then intersect its old and
   new paths with the evidence path set. If none intersect, no evidence content is read.
3. Group citations by range and path, read each base/head blob once, and search the head
   version for each exact non-empty recorded line.
4. Report one of:
   - `verified`: one exact match remains at the same path and line;
   - `moved`: one exact match exists at a different line or renamed path;
   - `vanished`: no exact match exists;
   - `unverifiable`: the pinned line is unavailable/blank or has multiple matches.

Only affected citations appear in `evidence`. This makes the common no-intersection case
proportional to Git's path diff rather than to repository contents.

Accepted proposals are the ratification boundary. When one of their citations vanishes,
the report adds a contradiction review item only if ownership names a target requirement
or an accepted promotion decision names a canonical requirement. Proposal IDs are never
serialized as requirement IDs. This is intentionally worded as lost support requiring
review: exact-line disappearance cannot prove a semantic contradiction by itself.

## Deliberate limits

- The command is read-only. It does not rewrite line numbers, regenerate contributions,
  land replacement proposals, or make promotion decisions.
- Exact-line matching does not recognize reformatted or semantically equivalent code.
  Ambiguous duplicate lines remain `unverifiable` rather than guessing.
- Absolute paths, parent traversal, missing commit pins, missing Git objects, and
  non-UTF-8 Git paths are not silently normalized. Unpinned owners are diagnosed; invalid
  revisions and operational Git failures stop the command. An explicit absent path is a
  typed evidence state and is not confused with a Git failure.
- Evidence currently has line locations but no stored content hash or symbol identity.
  A later semantic verifier can consume this report to rerun only the affected candidate
  batches while preserving all evidence sites and contested findings.
