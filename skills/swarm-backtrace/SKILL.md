---
name: swarm-backtrace
description: Reverse-engineer candidate requirements from an existing codebase with a multi-agent swarm. Use when the user wants to extract, mine, backtrace, or reverse-engineer requirements or rules from existing code, bootstrap a Provenance graph from a legacy system, or asks "what must be true for this code to be correct". Lands everything as proposals (promotion_state=proposed) against a commit-pinned source — never as active requirements.
---

# Swarm backtrace

Charting in reverse (docs/shaping.md, "Relationship to the swarm backtrace" — canonical;
where this file diverges, that document wins). Agents partition an existing codebase,
extract candidate requirements — *what must be true for this code to be correct* — dedup
keeping **all** evidence sites, challenge every candidate, and land everything as
`proposed` proposals with the codebase (pinned to a commit) as the source.

## Ground rules

1. **Proposals only.** Every candidate lands with `promotion_state=proposed` — never as
   an active requirement, never pre-accepted. Extracted claims describe *current
   behavior*; the code may be wrong — that's half the point. The human confirms
   "intentional" or discovers surprises via the shaping loop, question by question.
2. **Pin the commit.** All evidence is meaningless against a moving target. Record the
   exact commit in the Source; if the target repo changes mid-run, the run is against
   the pinned commit, not HEAD.
3. **Evidence discipline** (the output contract, docs/shaping.md): every claim cites
   typed evidence with `file_path` + `line`; speculation is explicitly marked
   `unsupported`/`exploratory`; uncertainty is rated with a rationale.
4. **Only the orchestrator writes the store.** Record types share append-only JSONL
   files under `.provenance/state/`; concurrent writers can corrupt or lose records.
   Subagents *return* structured findings; the orchestrating session lands them
   serially via the CLI.

## Pipeline

Run stages 1 and the landing inline; fan out stages 2 and 4 with the Agent tool
(one Agent call per partition/candidate-batch, launched in a single message so they
run concurrently). Stage 3 is a genuine barrier — it needs every extractor's output.

### 1. Scout (inline)

- Confirm the Provenance store exists (`.provenance/state/`); if not:
  `provenance init --path . --scope <scope> --path-prefix .`
- Pin the target: `git -C <target> rev-parse HEAD`.
- Create the codebase Source, commit-pinned:

  ```sh
  provenance sources create --scope <scope> \
    --id source_codebase_<repo>_<short-sha> \
    --name "<repo> @ <short-sha>" \
    --source-type system_state \
    --reference "git:<repo>@<full-sha>" \
    --format json
  ```

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

- Write one merged statement (the sharpest phrasing, or a new one).
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

Everything through the output contract shapes (docs/shaping.md, "The output contract"),
all with `--format json`, orchestrator only:

1. **Contributions** — one per participant slot (each extractor and each refuter):

   ```sh
   provenance contributions create --scope <scope> \
     --id contrib_backtrace_<slot> \
     --target-type source --target-id source_codebase_<repo>_<short-sha> \
     --participant-slot extract_<partition> \
     --stance support \
     --strongest-finding "<one line>" \
     --evidence-json '[{"reference_id":"ev_<slug>","evidence_type":"artifact","summary":"<what this line shows>","file_path":"src/auth/session.rs","line":42}]' \
     --claims-json '[{"claim_id":"claim_<slug>","statement":"<candidate statement>","evidence_type":"artifact","evidence_reference_ids":["ev_<slug>"]}]' \
     --uncertainty-level low|medium|high \
     --uncertainty-rationale "<why>"
   ```

   Refuter contributions use `--stance oppose|mixed|needs_more_evidence`,
   `--challenges-json '[{"claim_id":"...","objection":"..."}]'`, and put speculation in
   `--unsupported-recommendations-json '[{"recommendation":"...","marker":"unsupported"}]'`.
   Code evidence is `evidence_type: "artifact"`; hunches are `"unsupported"` or
   `"exploratory"`.

2. **Synthesis packet** — one per run, targeting the Source. Consensus, contested
   claims, and minority objections stay separate — never averaged. Surprises that need
   the human go in `--required-human-decisions-json`; missing-enforcement-path cases go
   in `--evidence-gaps-json` (set `blocking_promotion` honestly). Uncovered partitions
   (below) go in `--open-questions-json`.

3. **Proposals** — one per surviving merged candidate:

   ```sh
   provenance proposals create --scope <scope> \
     --id prop_req_<slug> \
     --proposal-key backtrace/<partition>/<slug> \
     --proposal-type requirement_candidate \
     --title "<merged statement>" \
     --summary "<behavior, confidence 0.NN, intentional-or-accidental note>" \
     --target-type source --target-id source_codebase_<repo>_<short-sha> \
     --source-id source_codebase_<repo>_<short-sha> \
     --evidence-json '[<ALL merged evidence sites>]' \
     --supporting-claim-id claim_<slug> --supporting-claim-id claim_<other> \
     --promotion-state proposed
   ```

   Use `--proposal-type source_gap` for "this behavior implies a policy/award/spec we
   don't have", and `question` for candidates that only sharpened into a question.
   There is no numeric confidence field on proposals — put the score in `--summary` and
   express it structurally via the contribution's uncertainty rating.

**Completeness:** reconcile against the stage-1 partition manifest. Any partition with
no extractor output — agent failed, context blown, code unreadable — is **logged, never
silently skipped**: an open question in the synthesis packet ("partition X uncovered:
<reason>") and, if material, a `source_gap` proposal. A backtrace that looks complete
but silently dropped a partition is worse than one that says where it didn't look.

### 6. Hand off to shaping

The backtrace's output *feeds* the shaping loop; it does not finish anything. Tell the
human what landed (`provenance proposals list --scope <scope>`), what's contested, and
what surprised the refuters — then the shaping loop (docs/shaping.md, "Invocation")
takes over: `provenance prime`, and the human disposes of proposals question by
question via `promotion-decisions create`. Do not make promotion decisions yourself.
