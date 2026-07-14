# Incremental backtrace and evidence review

The incremental path is a cheap gate in front of the adversarial backtrace workflow. It
does not replace the extraction and refutation stages in
`skills/provenance-swarm-backtrace/SKILL.md`, and it never changes promotion state.

## Command

```sh
provenance stale --repo . --scope default --format json
provenance stale --base <merge-base> --head <candidate> --format json
```

Without `--base`, each evidence owner is compared from the `commit_pin` of its first
traceability source to `--head` (default `HEAD`). Revisions are resolved to commits before
Git reads them. The optional base override is intended for CI, where one common range is
more useful than per-source pins.

The existing stale filters are active:

- `--min-age-days N` requires the approved resolution or pinned source commit to be at
  least `N` days old. Records without an approval time cannot satisfy a positive age
  filter.
- `--rule-severities high,critical` retains requirement evidence and resolutions with a
  downstream rule at one of those severities.
- `--min-downstream-rules N` requires at least `N` downstream rules after severity
  filtering.

Review dates are compared with the actual UTC day. Superseded approved resolutions use
their `superseded_by` field; the report no longer relies on an impossible
requirement-to-resolution `supersedes` edge or a fixed 2099 date.

## Cheap gate and re-verification

1. Collect repository-relative `file_path` values from proposal and contribution
   evidence in the selected scope.
2. Ask Git for a NUL-delimited, rename-aware name-status diff, then intersect its old and
   new paths with the evidence path set. If none intersect, no evidence content is read.
3. For each citation on an intersecting path, read the recorded line at the pinned/base
   commit and search the head version for that exact non-empty line.
4. Report one of:
   - `verified`: one exact match remains at the same path and line;
   - `moved`: one exact match exists at a different line or renamed path;
   - `vanished`: no exact match exists;
   - `unverifiable`: the pinned line is unavailable/blank or has multiple matches.

Only affected citations appear in `evidence`. This makes the common no-intersection case
proportional to Git's path diff rather than to repository contents.

Accepted `requirement_candidate` proposals are the ratification boundary. When one of
their citations vanishes, the report adds a contradiction review item. If a promotion
decision names a canonical requirement, that ID is used; otherwise a requirement target
or the accepted proposal ID identifies the ratified statement. This is intentionally
worded as lost support requiring review: exact-line disappearance cannot prove a semantic
contradiction by itself.

## Deliberate limits

- The command is read-only. It does not rewrite line numbers, regenerate contributions,
  land replacement proposals, or make promotion decisions.
- Exact-line matching does not recognize reformatted or semantically equivalent code.
  Ambiguous duplicate lines remain `unverifiable` rather than guessing.
- Absolute paths, parent traversal, missing commit pins, missing Git objects, and
  non-UTF-8 Git paths are not silently normalized. Unpinned owners are diagnosed; invalid
  revisions and Git failures stop the command.
- Evidence currently has line locations but no stored content hash or symbol identity.
  A later semantic verifier can consume this report to rerun only the affected candidate
  batches while preserving all evidence sites and contested findings.
