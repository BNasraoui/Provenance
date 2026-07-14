---
name: provenance-swarm-backtrace
description: Reverse-engineer candidate requirements from an existing codebase with a multi-agent swarm. Use when the user wants to extract, mine, backtrace, or reverse-engineer requirements or rules from existing code, bootstrap a Provenance graph from a legacy system, or asks "what must be true for this code to be correct". Lands unrefuted results as asserted proposals and contested results as proposed against a commit-pinned source — never as active requirements.
---

# Swarm backtrace

Charting in reverse (docs/shaping.md, "Relationship to the swarm backtrace" — canonical;
where this file diverges, that document wins). Agents partition an existing codebase,
extract candidate requirements — *what must be true for this code to be correct* — dedup
keeping **all** evidence sites, challenge every candidate, and land everything as
proposals with the codebase (pinned to a commit) as the source. Every proposal begins proposed;
unrefuted results receive separate immutable assertion records.

## Ground rules

1. **Proposals only.** Every candidate lands with `promotion_state=proposed`. An unrefuted
   candidate may also land with an evidence-backed assertion record. Neither is an active
   requirement or pre-accepted. Extracted claims describe *current
   behavior*; the code may be wrong — that's half the point. The human confirms
   "intentional" or discovers surprises via the shaping loop, question by question.
2. **Pin the commit.** All evidence is meaningless against a moving target. Record the
   exact commit in the Source; if the target repo changes mid-run, the run is against
   the pinned commit, not HEAD.
3. **Evidence discipline** (the output contract, docs/shaping.md): every claim cites
   typed evidence. For this code-backtrace workflow, code evidence must include
   `file_path` + `line`; other evidence types do not require file locations. Speculation is
   explicitly marked `unsupported`/`exploratory`; uncertainty is rated with a rationale.
4. **Only the orchestrator lands this run.** State uses sorted JSONL shards under
   `.provenance/state/`. Concurrent mutations of one shard serialize through an advisory
   lock, preventing lost updates, and atomic shard replacement gives readers a complete old
   or new shard; this is shard-level protection, not a multi-command transaction. Subagents
   still return structured findings so the orchestrator can deduplicate, validate, and land
   one coherent run through the CLI.

## Pipeline

Run stages 1 and the landing inline; fan out stages 2 and 4 with the Agent tool
(one Agent call per partition/candidate-batch, launched in a single message so they
run concurrently). Stage 3 is a genuine barrier — it needs every extractor's output.

### 1. Scout (inline)

- Confirm the Provenance manifest exists (`.provenance/state/manifest.json`); if not:
  `provenance init --path . --scope <scope> --path-prefix .`
- Pin the target: `git -C <target> rev-parse HEAD`.
- Create the codebase Source, commit-pinned:

  ```sh
  provenance sources create --scope <scope> \
    --id source_codebase_<repo-slug>_<short-sha> \
    --name "<repo> @ <short-sha>" \
    --source-type system_state \
    --reference "git:<repo>@<full-sha>" \
    --commit-pin "<full-sha>" \
    --format json
  ```

  `<repo-slug>` must use only lowercase letters, digits, `_`, and `-`.

  `system_state` is the type for observed system behavior; `project_artifact` also
  exists — use it only when backtracing docs/specs rather than running code.
- Partition the codebase by module/subsystem (directory tree, crate/package boundaries,
  `wc -l` for sizing). Write the partition manifest — name, paths, approximate size,
  entry points — you will need it verbatim for the completeness check in stage 5.

**Partition sizing:** one agent context per partition. A partition an agent cannot read
substantially within its context produces shallow, hedged candidates — split it. A
partition of three files starves the agent of cross-file behavior — merge it. Aim for a
coherent subsystem (auth, billing, sync protocol), roughly 2–10k lines; cut along
dependency seams, not alphabetically. Shared/core code read by everything can be its own
partition *and* listed as background reading for the others.

### 2. Extract (parallel, one agent per partition)

Each extractor reads its partition and returns candidate requirements. Prompt each with:

- Its partition paths, the pinned commit, and the anchor question: *what must be true
  for this code to be correct?*
- **The requirement test: a requirement survives a rewrite of the code; an
  implementation detail does not.** "Sessions expire after 30 minutes of inactivity"
  survives; "session TTL is stored in Redis with key prefix `sess:`" does not.
- Output shape (JSON in the final message, no store writes): per candidate — a
  statement, `file:line` evidence sites (several where behavior spans files), a
  confidence score 0.0–1.0 with one-line rationale, and open questions. Anything the
  agent suspects but cannot ground in a line of code goes in a separate
  `speculation` list, never mixed into candidates.

**A good candidate statement** describes externally observable behavior or an
invariant, in domain language, testable without reading the source:
- Good: "A shift cannot be published without an assigned worker."
- Good: "Sync retries are capped at 5 with exponential backoff."
- Bad (code structure): "`ShiftPublisher` validates via `WorkerAssignmentGuard`."
- Bad (restated code): "The `MAX_RETRIES` constant is 5."

### 3. Dedup / merge (barrier — wait for all extractors)

The same requirement arrives phrased differently from different partitions. Cluster by
meaning, not wording. For each cluster:

- Write one merged statement (the sharpest phrasing, or a new one). Run it through the
  `provenance-grounded-writing` skill's climbing test first — merging partition candidates into one
  statement is exactly where capability-list language creeps in.
- **Keep ALL evidence sites from every duplicate — never pick one winner.** Multiple
  independent sites are the strongest signal the behavior is intentional; discarding
  them destroys exactly the information the human needs.
- Carry the highest-context confidence rationale; note disagreement between extractors
  as a contested point for stage 4.
- Assign each merged candidate a stable `proposal_key`
  (`backtrace/<partition>/<slug>`, or `backtrace/cross/<slug>` for merged
  cross-partition candidates) — this is the dedup identity if the run is repeated.

### 4. Adversarial pass (parallel)

Fan out refuters over the merged candidates (batch ~10–20 candidates per agent; give
each refuter the candidates *with* their evidence sites and read access to the code).
Each candidate is challenged on two questions:

1. **Requirement or implementation detail?** Apply the rewrite test again, hostilely.
2. **Does the evidence actually support it?** Re-read every cited site. A constant
   proves a value exists, not that it is enforced; find the enforcement path or demote.

Refuters return, per candidate: uphold / demote-to-detail / reclassify-as-speculation /
narrow-the-statement, with an objection string for anything not upheld. Per the output
contract: demoted-but-possibly-true claims are kept and marked `unsupported` or
`exploratory` — adversaries mark speculation, they don't delete it.

### 5. Land (inline, serial)

Everything lands through the output contract shapes (docs/shaping.md, "The output
contract"), orchestrator only. Do not stream giant JSON through shell flags. Persist the
durable run outputs first, validate them, then land the run directory.

Run directory contract:

- `<run-dir>/extractors/<partition>.json` — `<Contribution>` or
  `{"contribution": <Contribution>}` for each extractor participant slot.
- `<run-dir>/refuters/<batch>.json` — `<Contribution>` or
  `{"contribution": <Contribution>}` for each refuter participant slot.
- `<run-dir>/merge/merged.json` — `{"synthesis_packet": <SynthesisPacket>, "proposals":
  [<ProposalCard>, ...]}` for the merged candidate set.

Use the CLI schemas while assembling these files:

```sh
provenance schema show contribution --format json
provenance schema show synthesis-packet --format json
provenance schema show proposal --format json
```

`provenance validate` accepts a single full artifact record. Use it on generated records
before wrapping them, or rely on `land` to validate the run-dir files as it reads them:

```sh
provenance validate contribution --input contribution.json --format json
provenance validate synthesis-packet --input synthesis.json --format json
provenance validate proposal --input proposal.json --format json
```

The landing command reads extractor/refuter contributions plus merge outputs, validates
schema version and nested IDs, and writes contributions, synthesis packets, and proposals
serially:

```sh
provenance swarm-backtrace land --scope <scope> --run-dir <run-dir> --format json
```

Use `--replace` when intentionally re-landing a regenerated run with the same stable IDs.
It can replace matching contribution and synthesis IDs; an existing proposal is replaceable
only while it is still `proposed` and has no promotion decision. Without `--replace`,
existing contribution/synthesis/proposal IDs fail fast.

Contribution records:

- one per participant slot (each extractor and refuter);
- extractor stance is usually `support`;
- refuter stance is usually `oppose`, `mixed`, or `needs_more_evidence`;
- code evidence uses `evidence_type: "artifact"` with `file_path` and `line`;
- hunches stay in `unsupported_recommendations` objects whose `marker` is `"unsupported"`
  or `"exploratory"`.

Synthesis packet:

- one per run, targeting the codebase Source;
- consensus, contested claims, and minority objections stay separate — never averaged;
- missing-enforcement-path cases go in `evidence_gaps` with `blocking_promotion` set
  honestly;
- uncovered partitions (below) go in `open_questions`.

Proposals:

- one per surviving merged candidate;
- use `proposal_type: "requirement_candidate"` for behavioral requirements,
  `"source_gap"` for implied missing policy/spec sources, and `"question"` for candidates
  that only sharpened into a question;
- set structured `confidence` on the proposal (`0.0`-`1.0`) instead of burying the score
  in `summary`;
- keep ALL merged evidence sites in `traceability.evidence_references`;
- keep supporting claim links in `traceability.supporting_claim_ids`;
- every proposal uses `promotion_state: "proposed"`;
- emit an `assertions` record only when supporting claims exist with positive evidence and the
  adjudicating synthesis has no contested claim or promotion blocker;
- use assertion IDs in `builds_on` when a later proposal incorporates an earlier assertion;
  neither assertion nor lineage is human ratification.

**Completeness:** reconcile against the stage-1 partition manifest. Any partition with
no extractor output — agent failed, context blown, code unreadable — is **logged, never
silently skipped**: an open question in the synthesis packet ("partition X uncovered:
<reason>") and, if material, a `source_gap` proposal. A backtrace that looks complete
but silently dropped a partition is worse than one that says where it didn't look.

### 6. Hand off to shaping

The backtrace's output *feeds* the shaping loop; it does not finish anything. Tell the
human what landed (`provenance proposals list --scope <scope> --promotion-state asserted`), what's contested, and
what surprised the refuters — then the shaping loop (docs/shaping.md, "Invocation")
takes over: `provenance prime`, and the human disposes of proposals question by
question via `promotion-decisions create`. Do not make promotion decisions yourself.
