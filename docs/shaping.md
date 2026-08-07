# Shaping

*How requirements get defined in Provenance: a turn-based loop over the graph.*

This is the canonical statement of the shaping design. The beads that implement it
(`provenance-d2t`, `provenance-y3o`, `provenance-ekd`, `provenance-kud`, `provenance-qho`)
summarize parts of this document; where they diverge, this document wins.

## Lineage

The shape is a deliberate hybrid of three sources, each contributing a different layer:

| Layer | Source | Contributes | Silent on |
|---|---|---|---|
| **Judgment** — what does "shaped enough" mean? | Shape Up (Ryan Singer, basecamp.com/shapeup) | Appetite, no-gos, rabbit holes, fixed-time-variable-scope, shaping as distinct from building | Execution substrate, persistence |
| **Memory** — what survives? | Statesman Provenance (the Convex original) | Typed graph, evidence discipline, promotion gates, traceability into code | Session mechanics |
| **Execution** — how does a session run? | Wayfinder (Matt Pocock, github.com/mattpocock/skills) | Turn structure, fog of war, claim/handoff, method-typed units, map-as-index | Residue, evidence typing, "done" criteria |

Each is strongest where the other two are silent. The hybrid also adds things none of the
three have: a graph-computed frontier, fan-out landing, stance tournaments with
empirically-grounded caveats, and a loop that continues past "shaped" into rules,
annotated code, coverage, and review triggers.

## The map

A shaping effort is anchored by a **requirement** — any requirement, at any depth.
Requirements are recursive, so a feature-sized idea ("provenance docs shareable via a
short-lived link"), a compliance program ("be SCHADS compliant"), and a sub-effort under
either all get the identical loop. There is no separate map/epic/project concept.

The map is an **index, not a store**. A session loads it low-res — the anchor requirement,
decisions so far (resolutions, gisted one line each), the fog, and the frontier — and zooms
into full artifact bodies on demand. A decision lives in exactly one place: its resolution.

### Fog of war

The anchor requirement carries a deliberately **unstructured** fog section: the dim view of
decisions and investigations you can tell are coming but cannot yet phrase sharply.

The test: **question when the question can be stated precisely (even if blocked); fog when
it can't.** Never pre-slice fog into question-sized pieces — one patch of fog may graduate
into several questions, or none, once the frontier reaches it.

Fog is the anti-over-modeling guard. A rich schema tempts agents into speculative
node-minting; fog gives the not-yet-sharp a home that isn't forty premature Question nodes.

## Invocation

Two modes. Every session ends with a handoff.

### Chart

A loose idea arrives. One session's work — do **not** also resolve questions.

1. Create (or select) the anchor requirement.
2. Grill to surface:
   - **Appetite** (Shape Up): how much time is this worth? Recorded up front, re-checked
     as requirements accumulate.
   - **Boundaries** (the no-gos): constraints on the solution space, each with a source
     ref where one exists (e.g. a privacy policy clause).
   - **Fog**: everything sensed but not yet sharp.
   - **First questions**: each minted with a `resolution_method` (see below), sized to
     one agent session, wired with blocking edges where order matters.
3. Handoff.

### Work

The turn loop, against an anchor requirement:

1. **Prime** — load the map low-res. The frontier is *computed from graph semantics*,
   not hand-wired: requirements with no source edge, unresolved `contradicts` pairs,
   requirements that can't produce a rule, open questions and unexplored topics.
2. **Claim** — at method-dependent granularity (see table below). Claiming happens
   before any work so concurrent sessions skip claimed items.
3. **Resolve and land as you go** — each question is recorded the moment it resolves,
   before the dialogue moves on. Expect fan-out (see below).
4. **Graduate fog** the answers made sharp; update or kill questions the decisions
   invalidated.
5. **Stop and hand off** — the turn ends at the first stop condition:
   context budget nearing its threshold, human done, a fork/verify spawn
   (two-phase boundary), or the topic drained.

### Invariants

"One question per session" is deliberately **not** the rule — grill turns are cheap to
continue and expensive to restart, and answers don't respect question boundaries. The
real invariants:

1. Every decision is recorded the moment it resolves — never carry a
   resolved-but-unrecorded decision in conversation state. If the session dies mid-topic,
   everything answered so far is already in the graph.
2. The session never outruns its context budget.
3. The agent never proceeds past a decision the human hasn't ratified.
4. The map is consistent at handoff.

## Resolution methods

Questions are typed by the **verb that resolves them** — chosen when the question is
minted, telling the next session what to do. This complements the content-noun typing
(source/requirement/resolution/rule) that tells the auditor what an artifact *is*.

| Method | What it is | Claim granularity |
|---|---|---|
| `grill` | One-question-at-a-time dialogue with the human; a recommended answer offered each time. The default. | A whole **topic**, drained question by question |
| `prototype` | Fork tournament — competing concrete artifacts to react to (below) | Single **question** (two-phase) |
| `research` | Reading sources, docs, codebases; summary linked as evidence | Single **question**, but incidental answers are recorded on whatever questions they actually resolve |
| `verify` | Adversarial refuters on an expensive-if-wrong claim (regulatory / SCHADS tier) | Single **question** (two-phase) |
| `task` | Human-world work — provisioning, signups, data moves. The answer records resulting facts later work depends on | The task |

## Landing fan-out

One answer routinely lands as **multiple graph mutations**, not a bare answered-question.
Worked example — anchor: *"Provenance docs shareable via short-lived link"*; question:
*"How long is short-lived?"* (method: grill). The answer lands as:

- a **Resolution**: "7-day default, configurable ceiling" — position + rationale;
- which **produces** a **Rule**: "share links MUST expire ≤ 30 days" (severity: high);
- and **spawns** a **Requirement**: "expiry configuration";
- and **graduates fog**: "access auditing" is now statable as a question
  ("should link accesses be logged as provenance events?").

Landing an answer as a bare answered-question when it implies resolutions, rules, or
requirements **starves the graph**. The dialogue itself persists as a Thread on the
requirement; candidate artifacts land as **proposals** for human promotion (see below).

Every statement landed this way — resolution, rule, requirement, or source — follows the
specificity discipline in the `provenance-grounded-writing` skill before it lands, not only at the leaf.

## Fork tournaments (`prototype`)

A prototype's job is to **raise the fidelity of the discussion** — a cheap, concrete
artifact the human reacts to. The artifact is a conversation move, not a deliverable;
the tournament serves the dialogue, and its primary output is the human's extracted
reactions plus the recorded decision.

**Phase 1** (turn N — agent work, parallelizable, no human needed): at a genuine design
fork, spawn N agents with **stance-based personas**. A stance is a value system + quality
bar + exit criterion ("reduce until it fails the 5-second test" vs. "disclose
progressively by altitude") — **not a character**. Each produces a competing artifact
opening with a design-principles manifesto. Artifacts land as proposals linked to the
question; the question's status is set to `blocked_on_human` until the human disposal
turn.

**Phase 2** (turn N+1 — human disposal, the promotion gate with a clock): the human
reacts, picks a winner, grafts ideas from runners-up. The decision lands as a Resolution
referencing the competing proposals; losers are marked rejected/superseded; grafted ideas
stay traceable to their origin proposal.

**Empirical caveats** (from the Statesman provenance scoping record): same-model hats
*converge* on central insights — the Steve and Jony canvas prototypes independently
arrived at "territory, not pipeline." Don't expect independence from personas alone;
engineer real divergence via **evidence partition** (each participant grounded in
different material) and **task framing** (advocate vs. refuter). What personas reliably
do provide: they break sycophancy, license ruthlessness and taste, and force articulated
values before output. A tournament at forks — not a standing committee on everything.

## The output contract

All producers — the shaping dialogue, tournaments, verifiers, and the swarm backtrace —
land their candidates through the same durable shapes (ported from the Convex
`ideationRuns` design; see `provenance-ekd`):

- **Contributions**: claims cite typed evidence; speculation is explicitly marked
  `unsupported`/`exploratory`; uncertainty is rated with rationale.
- **Synthesis**: consensus, contested claims, and minority objections are kept separate —
  never averaged; evidence gaps can block promotion; required human decisions are explicit.
- **Proposals**: typed candidates (`requirement_candidate`, `rule_candidate`,
  `source_gap`, …) with traceability. Definitions are immutable and always authored
  `proposed`. A positive, unblocked **Assertion** derives `asserted`; an immutable
  **Disposition** is the sole authority for `accepted`, `rejected`, or `deferred`.
  Lineage uses assertion IDs so it cannot silently retarget when proposal views change.

### Dispose on demand

The proposal shard is evidence-backed context, not a batch-review inbox. Undisposed
proposals surface when current work enters territory they already name:

- a diff touches an exact repository-relative `file_path` in the proposal's evidence;
- a shaping session claims the proposal's target topic, the topic's anchor requirement,
  or one of the topic's explicit artifact links; or
- an external work item such as a bug explicitly identifies one of those typed targets.

Use `provenance proposals surface --changed-path <path>` for changed files. Topic claims
surface their matching proposals in the claim result. Integrations that already know a
typed territory can use `provenance proposals surface --target-type <type> --target-id
<id>`; Provenance does not infer territory from issue text, titles, graph proximity, or
similar filenames.

Review all proposals together only when a small set jointly blocks the work: competing
fork proposals, explicitly contested synthesis claims, or conflicting backtrace findings.
Do not turn the complete backtrace output into a disposal queue.

When a human action both resolves the problem and produces an existing canonical artifact
(source, requirement, resolution, or rule), record one accepted disposition with
that human actor and `canonical_artifact`. The action is then the ratification evidence;
do not require a second ceremonial review. A commit or bug ID alone is not a canonical
artifact in the current model and must not be disguised as one. The current writer records
but does not verify that artifact reference, so create and verify the canonical record first.

Both `proposed` and `asserted` remain undisposed and surface only on exact path or typed
territory demand. `provenance prime` does not globally consult proposals. Dispositions
require a repository-allowlisted actor ID; this is an audit attestation under repository
and CLI access, not cryptographic identity verification.

The run-status machinery of the original (queued/running/failed_retryable…) is *not*
ported — skills and workflows own execution; only durable outputs enter state.

## Relationship to the swarm backtrace

The backtrace (`provenance-qho`) is charting in reverse: agents partition an existing
codebase, extract candidate requirements ("what must be true for this code to be
correct"), dedup keeping *all* evidence sites, and land everything as `proposed` — never
`active` — with the codebase source carrying a validated `commit_pin`. Proposals and
material claims may carry `0.0`-`1.0` confidence scores. Its output becomes consultable
context and surfaces for disposition only when later work enters its explicit territory,
apart from small contested or conflicting sets that already block a decision.

## Beyond "shaped"

Shape Up ends at the bet; wayfinder ends at "the way is clear." This loop continues:
resolutions produce rules, rules are annotated in code (`@provenance` markers), the
scanner reports coverage, and review triggers fire when sources change or rules fail.
Fog to enforcement, one graph.
