# Rules as Code — Handoff Context

> Superseded on 2026-08-10 by the model that landed in PR #48. The reversal this
> doc argues for (fiat: a registered `defineRule` primitive, declared facts, a
> runtime) was itself overturned the next morning: a rule IS a function, bound by
> `#[rule("id")]` with no primitive and no runtime, and tests verify rather than
> enforce. Read this for the reasoning that got there — the trilemma, the codebase
> audits, the adoption analysis — not for the current design.


*Continues the "Provenance AST / Rule Verification" handoff. That document proposed rules as
contracts over ordinary production code, verified non-invasively. This session tested that
model against the actual Provenance codebase and four real application codebases, and
reversed its central principle. Read this instead of that.*

## The decision

**Rules are executable by fiat.** A Provenance rule is a registered, pure function in the
application codebase. The function does not implement the rule or claim to satisfy it — it
**is** the rule, definitionally. There is no separate prose statement the code must be
checked against.

This overturns the previous working principle ("Provenance should understand what the
software means without requiring the software to be written in Provenance"). The reversal
was deliberate, argued, and is recorded here as the decision of this session.

## Why the contract model died

The contract model (rule = machine-readable spec, bindings = claims that code satisfies it)
has an irreducible flaw: **association is nondeterministic**. "This function/WHERE
clause/schema enforces rule R" is a semantic claim no tool can verify. Verification against
a formal spec is deterministic; the binding of spec to code never is. Every system in this
space has the gap — Dafny verifies code against a spec but not the spec against intent;
Antithesis explores deterministically but humans choose the invariants.

There are only two ways to close the association gap:

1. **Fiat** — the artifact is definitionally the rule. No gap exists.
2. **Falsification** — the binding stays a claim, but a formal statement makes wrong
   bindings testable. The gap is measured, not closed.

We chose fiat. Falsification-only designs (annotations, wrappers-as-claims, comment
markers) were explored and rejected below.

## The trilemma

You can pick two:

1. **Determinism** — production behavior is the rule, by identity
2. **Non-invasiveness** — application code unchanged
3. **Coverage** — rules govern the behavior that matters

Fiat picks 1 and 3 and pays with 2. Provenance rules are therefore **invasive the way
Effect is invasive in TypeScript**: a framework that restructures how code is written and
survives only by being clearly better than not using it.

## What a rule is now

The application adopts **functional core, imperative shell**, where the core is made of
registered rule functions:

```ts
// rules/checkout.ts — this IS the rule, definitionally
export const canCheckout = defineRule("CHECKOUT-001",
  (facts: { customer: CustomerFacts; cart: CartFacts }) =>
    facts.customer.active && facts.cart.itemCount > 0
);
```

- Rules are pure over declared **facts**. The shell fetches facts and executes
  consequences; every decision routes through a rule.
- `defineRule` is an identity wrapper plus registration. No runtime engine, no evaluator.
- Rules may call rules. The dependency graph emerges (the OpenFisca property) without
  OpenFisca's runtime.
- Requirements stay prose. A requirement without an executable definition is not yet a
  rule — that is a loud, first-class state, not an error.

The nondeterminism does not vanish; it moves to the one edge built for it: **does this
oracle express what the human decided?** That is the requirement→rule link, ratified once
in review by a human — exactly what the existing assertion/disposition lifecycle records.
Everything below that edge is mechanical.

### The boundary residue

Fiat's boundary is the language runtime. Two things live outside it, permanently:

- **Substrate enforcement that cannot execute the function**: SQL WHERE clauses, DB CHECK
  and UNIQUE constraints, cross-service checks. Choice per site: pull the decision up into
  the core (pay performance), or keep a **replica** with a conformance test against the
  rule (differential testing — deterministic verdict, evidence not identity).
- **Enforcement by construction**: newtypes, constraints, transaction shapes (noscope's
  `RedactedToken`, homebot's `INSERT OR IGNORE` + UNIQUE). These are *stronger* than
  predicates — a rule check has TOCTOU races a UNIQUE constraint does not. They are
  recorded as bindings of a different kind, never rewritten into rule functions.

Identity within the runtime; evidence at the boundaries.

## "It must be better" — the adoption bar

Effect won by giving relief from an acute pain, not by having virtues. Traceability is a
virtue; nobody buys virtues. **Writing a decision as a rule must pay for itself in the
first PR that uses it.** The day-one payoffs, in order of strength:

1. **Explanation for free.** Registered pure functions over declared facts yield automatic
   decision reports: which rule, which facts, which result. "Why was this declined?" from
   a log line. For compliance domains the audit trail is itself the legal obligation
   (see funnel findings) — this is the killer feature.
2. **Tests for free.** Registering a rule generates the property-test scaffold and
   example-table harness. What noscope's process demanded by discipline, produced by
   machinery.
3. **The why in the editor.** Hover a rule → its requirement, decision, and review
   triggers when the source policy changes.
4. **Drift detection at boundaries.** Every replica carries a conformance test; CI reports
   when one of five enforcement sites changes alone.

Adoption lessons carried over from Effect:

- **Smooth gradient**: one rule in an ordinary codebase must be useful alone, or nobody
  writes rule two.
- **Virality is mechanism and cost**: rules calling rules pulls the decision layer in;
  the interop story (a rule wrapping legacy logic, ugly but honest) matters as much as
  the pure story.
- **Agent-era economics**: restructuring cost is collapsing (agents write the code) while
  the value of a verifiable decision layer rises (registered rules become the ground
  truth humans review; the rest is regenerable). This bet is sane in 2026 in a way it
  was not in 2020.

## Evidence base (what this session actually found)

### Current Provenance state (crates inspected directly)

- Rule = prose `statement` + severity/type/modality enums.
  `crates/provenance-core/src/model/artifacts.rs:392` — no semantic structure.
- **Empty socket**: `Rule.expression` (json `{}`) and `Rule.inputs` (json `[]`) are
  serialized, cached, and hashed into graph references but never written (writer hardcodes
  `{}` at `crates/provenance-store/src/state_store/rule_writers.rs:155`) and never read.
  Statesman heritage. Nothing breaks if repurposed; changing them changes pinned-graph
  hashes (correct, but schema-version the payload).
- **Rules are leaf nodes**: only `Produces` (requirement/resolution → rule) may touch a
  rule (`crates/provenance-core/src/edge_validation.rs`). No rule→rule edges — no
  composition, no dependencies. Under fiat, rules calling rules needs this opened.
- **Binding today is ephemeral**: `@provenance rule:` comment annotations
  (`crates/provenance-scanner/src/parser.rs`; `@statesman` still accepted as legacy);
  `coverage scan` prints a report and persists nothing. Coverage `confidence` is
  self-asserted in the comment.
- One persisted binding exists: `ServiceBinding` rule↔service, typed
  enforces/consumes/monitors (`crates/provenance-core/src/model/services.rs:124`) — the
  template shape for typed binding records, wrong granularity.
- "Verification" = counting: annotation presence, Produces-edge presence
  (`crates/provenance-store/src/cache/health.rs`). No test evidence, no assurance levels.

### Codebase audits (agents; workflowd, funnel, Home2Own, homebot)

Rule-shaped logic classified: (a) already a pure exported function, (b) inline needing
extraction, (c) one rule at many sites, (d) temporal/cross-call invariant.

- **Zero-refactor bind rate is 10–15%** (30% in workflowd, which already has a deliberate
  `src/domain/` layer). The clean cases exist because they were extracted for testability
  — rule-shaped code is a symptom of good engineering, not something a wrapper injects.
- **(c) dominates the load-bearing rules.** workflowd's "reviewable PR" lives at five
  sites in two languages (TS + four SQL fragments, `src/store/currentness.ts:114` etc.).
  Home2Own's rental-vs-purchase threshold exists at three sites with two different numbers
  (10 000 and 100 000) and two period labels — the drift bug a single-home rule prevents.
  funnel's consent enums diverge across DB CHECK / validator / service
  (`'inferred'` vs `'INFERRED'` vs `'implied'`).
- **(d) is where small services keep their correctness.** homebot's at-most-once and
  cursor monotonicity are SQL constraints + transaction shapes; workflowd's leases, retry
  budgets, staleness guards likewise. A temporal rule can be folded into pure
  `f(priorState, event)` — workflowd's `decidePullRequestTransition` proves it — at the
  cost of a reified state type and a 278-line module. Under fiat, temporal oracles are
  `f(trace) → legal`.
- **SQL duplication is load-bearing, not sloppy**: predicates are repeated in WHERE
  clauses because filtering in the database is the point. Centralizing evaluation into
  the app costs performance. Hence replicas + conformance, not unification.
- **funnel: the crispest legal rules have the least code.** DNCR calling hours, frequency
  caps, APP 7.2 opt-out simplicity — specified to the minute, zero enforcement sites
  (`validateCallTiming` is called and never defined). Roughly a third of documented legal
  requirements had no code at all. Also: audit writes inside consent predicates are the
  compliance obligation, not laziness — splitting decision from evidence emission is a
  real refactor the framework should absorb (decision reports).
- **workflowd already invented a proto-rule()**: `makeCurrentnessPolicy` names and reuses
  SQL predicates. The felt need is real; the carrier was SQL fragments, not functions.

### noscope — the extreme, already run

77 `NS-xxx` rules, module headers per rule, agent workflow requiring a dedicated test per
rule before code (see its `PROMPT.md`).

- The ledger works: statement → enforcement → evidence is grep-traceable; agents built
  reliably because rules were the work units.
- The best rules became **types, not predicates** (NS-058 → `RedactedToken` newtype whose
  Display/Debug can only emit redacted form). A rule can demand a mechanism that makes
  violation impossible without being that mechanism.
- Even at maximum discipline: binding stayed prose comments; the configured
  `tests/contracts` directory was never created (**if formalization isn't nearly free, it
  doesn't happen**); evidence stayed at three hand-picked examples for a ∀ rule (NS-005)
  — the exact upgrade a generated property test provides.
- Discipline held only while the enforcing process ran. Nothing checks the 77 rules today.
  Tooling must replace process: `provenance check` in CI and the editor.

### Prior art (for the registration surface)

Rules are **values you import, not interfaces you implement**. Closest analogues: Vercel
Flags SDK (`flag({ key, decide })`), Temporal/Inngest (`createFunction({ id }, handler)`),
OpenFisca (registered variables; the invasive runtime we are *not* copying). Determinism
of the id↔graph link comes from check-time validation (i18n-key/lint model: red squiggle
+ CI failure), not codegen. The runtime package stays tiny — registration and identity
only; all intelligence in the CLI/LSP.

## Implications for the Provenance model

1. **Redefine Rule** (CONTEXT.md glossary): from "normative proposition" to "executable
   definition, registered in a codebase." Prose intent lives in Requirements/Resolutions.
2. **New persisted record: rule registration** — rule id ↔ {repo, module, symbol,
   commit}. Mirror `ServiceBinding`'s shape. Replaces the ephemeral scan as the source of
   binding truth; the scanner becomes the reconciler.
3. **New persisted records: boundary bindings** — replica sites (kind: sql-fragment,
   schema-constraint, ui-check, …) each carrying/expecting a conformance test, and
   by-construction bindings (kind: type, db-constraint, transaction-shape) as
   attestations.
4. **Evidence rows** — conformance runs, generated property-test runs, proofs — as
   immutable records; assurance level derived, never stored. Reuse the
   assertion/disposition pattern (ADR 0001).
5. **Open rule→rule edges** in `edge_validation.rs` (rules call rules).
6. **The `expression` socket's role shrinks**: under fiat the function is the definition,
   so `expression` is no longer the rule's semantics. Candidate reuse: storing declared
   fact shapes (`inputs`) and optional *properties about* the rule (metamorphic/∀
   statements used to generate tests against the oracle). Contracts don't define rules
   anymore; they test them.
7. **Unbound requirement surfacing**: "requirement cannot produce a rule" already exists
   in the prime frontier; "rule registered but symbol missing" and "requirement with no
   rule and no exemption" become check failures.

## Open questions for the next session

1. **Facts.** Rules are pure over declared facts — what defines the fact vocabulary?
   Per-scope fact types in the graph? TypeScript types only? This is where OpenFisca's
   entity model earns its keep, and the hardest open design problem.
2. **The runtime package.** Name, shape, and the interop story (a rule wrapping impure
   legacy logic). How does registration reach the graph — scan of `defineRule` call
   sites, build step, or runtime export?
3. **Decision reports.** What does the shell call to get the audit record? Is emission
   part of `defineRule` or a separate `decide()` entry point?
4. **Conformance harness.** Shape of a differential test between a rule and a SQL
   replica; who owns fixture/generator data.
5. **Multi-language.** Fiat per runtime — what is a rule when the same scope spans a TS
   app and a Rust service? One rule, two registrations? Which is definitional?
6. **Migration.** The existing `@provenance` annotation layer and `ServiceBinding` —
   deprecate, or keep as the soft tier for unmigrated code?
7. **The bar.** Which of the four day-one payoffs ships first? (Argued here: decision
   reports or generated tests — the two with relief, not virtue.)
