---
date: 2026-08-18
git_commit: 7dfb3c77a46e515ff6106169ebba908786e36c93
branch: main
topic: "Session Handoff: Typed TypeScript SDK — requirements as code"
tags: [handoff, typescript-sdk, requirements-as-code, lifecycle, plan]
status: in-progress
---

# Handoff: typed TypeScript SDK

Written for an agent with no prior context. The work ran in Codex threads from
2026-08-11 to 2026-08-18. Those agents ran out of quota mid-flight. Everything
described here as merged is on `origin/main` at `7dfb3c7`.

**Read first:** `docs/typescript-sdk-poc.md`, `packages/provenance/README.md`,
`docs/adr/0002`–`0005`, `AGENTS.md`.

## What this is for

Provenance already records traceability outside production code: tree-sitter
scanning plus `#[rule]` / `#[verifies]` attributes bind code to graph records.
That is the retrofit path and it stays.

This line of work adds a second, deeper path. A developer writes requirements
and rules as ordinary typed TypeScript values, imports those values in tests,
and lets the TypeScript compiler enforce the reference. Rename or delete the
rule and the build breaks. No string literal to rot.

The point is not another authoring format. Two properties carry the value:

1. **Compile-time referential integrity.** `expiry.verify(...)` is a real
   program reference. `@verifies("rule_expiry")` is a string that silently dies.
2. **`plan` as a requirements-aware diff.** Change a requirement statement and
   Provenance can say which rules, implementations, tests and evidence now
   deserve attention. Ben identified this as the strongest product idea in the
   whole thread: "Oh so it highlights the diff as requirements change?
   Interesting" then "That's actually very useful. Send it."

The governing boundaries, restated by Ben several times:

- «Languages own syntax. Rust owns semantics.» The npm package is a façade. No
  Provenance domain rule may be reimplemented in TypeScript.
- «Provenance describes the system without requiring the production system to
  become Provenance-aware.» Deleting Provenance must not break the application.
  This is why the spec points at production code through `.implementedBy(...)`
  rather than production code carrying decorators.
- Typed specs own only what they declare. The database holds accumulated
  knowledge from humans, scanners, Linear, CI and agents.

## What Ben ratified

These are settled. Do not reopen them without him.

1. **A Rule is an independent atomic behavioural obligation.** It may exist
   before any implementation; that state is *unimplemented*, not invalid.
   `#[rule("id")]` binds the primary implementation and no longer defines the
   Rule. This reversed the older repo contract and required a graph, scanner,
   coverage and wiki migration. Recorded in `docs/adr/0002`.
2. **No daemon yet.** Ben's bar was install ergonomics, not architecture: "The
   main thing is that if I run npm install I get the same behavior on any
   platform and I don't need to install any other external deps. I should not
   need the cli preinstalled." One-shot child processes stay behind an internal
   transport interface. A daemon must earn its keep through measured startup or
   query cost (`provenance-o1b.6`).
3. **npm scope `@quality-sh/provenance`.** `provenance`, `@quality` and
   `@provenance` are all taken. Ben has claimed the npm organisation. He has
   *not* yet claimed the matching `quality-sh` GitHub org, PyPI org, ghcr
   namespace or Homebrew tap.
4. **Do not publish.** Direct quote: "I dont want to publish something until I
   am happy with it. Stop asking to publish things." Treat this as standing.
   Note the cost of it under *Blocked work* below, but do not ask again.
5. **A Rule may refine several Requirements.** Ben: "What if rules are reused
   across requirements? They almost certainly will be." Consequences that are
   now implemented: the declaration address is spec-scoped, not parent-scoped;
   reusing the same declaration value means one shared Rule with two edges;
   separate Rules come from a factory function creating separate values.
6. **Fluent immutable DSL, one expression.** Ben rejected PR #92 outright: "92
   is not what we agreed upon. That isn't fluent it's declaring a bunch of
   separate vars." The replacement is PR #100, cleaned further in PR #105.
7. **Adoption and move-in support for pre-existing records is out of scope.**
   Ben: "2. Imo is unnecessary. We are still pre 1.0."
8. **Retire in place, never hard delete.** Omitting a declaration retires it and
   keeps its history and StableId. Reintroducing it reuses that ID. ADRs 0004
   and 0005.
9. **`verify()` stays but is not the future.** Ben: "I am not married to verify
   needing to exist... I am just happy to infer from call sites and reverse
   engineer it from a test runner's results," then "keep what we have for
   verify, I guess but this isn't essential and there's a better story here
   eventually. Let's put it on the backburner." Do not spend design time here.

## What Ben rejected

- `Fragment` / `defineFragment` / `RequirementGroup` / `Module` / `include()`.
  He killed the first on sight: "What is a fragment here? That is new vocabulary
  we're introducing." Ordinary TypeScript functions compose specs. Do not invent
  a noun without behaviour behind it.
- Nested object-literal configuration as the primary API shape.
- Publishing anything to npm.
- Any suggestion that a typed declaration duplicates executable business logic.
  A Rule is a claim with evidence, not a second implementation.

## The API as it stands

```ts
export const shareLinks = defineSpec("share-links")
  .requirements(
    requirement("sharing")
      .statement("Users can securely share documentation")
      .from(source("sharing-policy").document("docs/sharing-policy.md"))
      .rules(
        rule("expiry")
          .statement("Share links must expire within 30 days")
          .implementedBy(createShareLink),
      ),
  )
  .build();
```

Tests reach the rule through the built spec:

```ts
await shareLinks.requirements.sharing.rules.expiry.verify(
  "share-link-expiry",
  async () => { /* ordinary test */ },
);
```

Sources named in `.from(...)` are collected automatically and appear in the
built value's TypeScript type. `.handles` survives for compatibility but is
absent from the preferred example. The older `defineSpec(name, callback)` and
scoped-context forms still work.

`.implementedBy(target)` uses the target only for type checking. The SDK reads
its own call site, parses that source file, and accepts a named import or
`namespace.export` only. Dynamic expressions, bundled or minified specs are
rejected loudly rather than guessed at. Rust validates and stores the
repo-relative location.

## What has merged

All on `main`, all green across Linux, macOS and Windows including packaged
installs:

| PR | What |
|---|---|
| #84 | Typed SDK POC: declarations, `apply`, `verify`, hierarchical addresses, durable verification bindings, positional binding key, first `plan` |
| #85 | Self-contained install: `sdk info` handshake, project discovery, per-platform engine packages, hermetic packed-install proof |
| #89 | Coverage scan skips `node_modules`, `target`, `.git` |
| #90 | Fluent `.name()` and `.description()` |
| #91 | Wiki renders typed implementation bindings |
| #93 | `implementedBy` accepts exported classes |
| #95 | Public declaration composition types |
| #100 | One-expression fluent spec example and nested typed access (replaced the rejected #92) |
| #105 | Cleaned the preferred example |
| #108 | Typed declaration lifecycle: retire, move, conflict, structured `plan` states |
| #109 | Removing `.implementedBy(...)` retires the binding instead of leaving it active |

Also merged along the way: the Rule semantic migration (graph health, coverage,
scanner, wiki), comment-directive bindings classified as real implementations,
scan-territory guard, cross-platform path canonicalisation.

## Exact current state

- `origin/main` is `7dfb3c7`, immediately after PR #109 merged at 05:56 UTC on
  2026-08-18.
- **The local checkout at `/home/ben/Documents/repos/provenance` is five commits
  behind `origin/main`** (local `main` is `ef82698`). It lacks #105, #108 and
  #109. Fast-forward before doing anything, and preserve the uncommitted
  `.beads/interactions.jsonl` lines while you do.
- Branches `codex/typescript-sdk-poc`, `codex/fluent-typed-spec`,
  `codex/typed-spec-lifecycle` and `codex/implementation-binding-lifecycle` are
  all merged. Nothing on them is needed.
- Protocol version is 2. Old specs that omit the declaration address keep the
  earlier inferred behaviour.
- Nothing is published to npm.

### The workflowd dogfood is NOT gone

The last Codex agent reported "the earlier uncommitted dogfood worktree is also
gone" and started planning a rewrite. **That is wrong.** The work survives at:

```
/home/ben/Documents/repos/workflowd-provenance-typed-dogfood
branch feature/typed-provenance-dogfood (base f6c9952)
```

Uncommitted and untracked there: `provenance/qrspi.spec.ts` (177 lines),
`.provenance/`, `scripts/provenance.ts`, `test/qrspi/provenance-spec.test.ts`,
plus edits to four QRSPI test files. Check it before recreating anything.

Two caveats. The spec still uses the pre-#100 scoped-context style
(`provenance.rule(...)`, `provenance.requirement(...)`), so it needs updating to
the merged one-expression fluent API. And the SDK is linked locally, so it needs
`PROVENANCE_BIN` pointing at a development binary.

## Blocked work

**The dogfood cannot become a real workflowd PR.** Bun installs Git
dependencies but not a package inside a Git repository subdirectory, and
`@quality-sh/provenance` is unpublished. So workflowd can dogfood locally
through `bun link` but its CI cannot install the SDK. Publishing would unblock
it. Ben has forbidden publishing. Work around it locally; do not raise it again
unless he does.

## Open questions for Ben

1. **Publish or stay local?** Everything in the product loop past local
   dogfooding waits on this. State the cost once, in passing, and move on.
2. **Does a requirement change make evidence stale or merely "review
   required"?** The agent proposed keeping `stale` for its current meaning —
   evidence-bearing code changed — and adding a separate review-required state.
   Ben never ruled. `provenance-o1b.3` needs the answer.
3. **Verification's long-term story.** Keep explicit `verify()`, or derive
   candidate evidence from test-runner output joined to `implementedBy` links?
   Explicitly backburnered, but it decides how much ceremony the SDK carries.
4. **The remaining namespace reservations** (GitHub org, PyPI, ghcr, Homebrew
   tap). Ben said he would do them "later".

## Next step

Take `provenance-o1b.3` — turn `plan` into a semantic review surface. The last
Codex agent named it as the next slice after #109, and it is the piece that
makes the product argument. Acceptance criteria are already written in the bead:
for real changes, `plan` must show changed obligations, affected Rules,
implementation and verification sites, current evidence state and
review-required reasons; JSON stable and composable; human output readable; a
requirement-only change must not falsely claim code evidence is stale.

Question 2 above blocks the last clause. Get the ruling before you build it.

The strongest competing candidate is `provenance-cok`: a removed `rule.verify()`
wrapper leaves its durable VerificationBinding active, so a Rule keeps looking
verified after its test relationship disappeared. This is the verification twin
of the bugs #108 and #109 fixed, and it is a correctness hole, not polish. If
`o1b.3` stalls on a missing decision, take `cok` instead.

Do not add SDK surface. Do not start Python, Go or C#.

## Open beads

```
provenance-o1b     epic  Validate typed Provenance through workflowd
  o1b.1  in_progress  Dogfood typed Provenance in workflowd
  o1b.3  open         Turn plan into a semantic review surface
  o1b.4  open         Run agent-maintained Provenance experiments in workflowd
  o1b.5  open         Expose structured engine query primitives
  o1b.6  open  p3     Measure whether Provenance needs a daemon
provenance-cok   bug   Reconcile disappeared typed verification bindings
provenance-6vo   bug   Verification file capture across test runtimes (Bun needs
                       explicit { file: import.meta.path })
provenance-s23   task  Reduce typed verification ceremony
provenance-txe   p2    Present retired typed declarations in wiki and queries
provenance-dcr   bug   Ignore dependency and build trees in coverage scans
```

`provenance-dcr` is listed open but #89 appears to have fixed it. Verify before
claiming it.

## Gotchas that will bite you

**Beads is the only tracker.** No markdown TODO lists. `bd ready`,
`bd update <id> --claim`, `bd close <id> --reason "..."`, always `--json`. New
work found mid-task gets `--deps discovered-from:<parent>`. The store is Dolt,
shared, and syncs through the repo. **Never touch `.beads/interactions.jsonl` —
it carries the user's own uncommitted lines. Preserve them across any
fast-forward or rebase.**

**TDD with recorded RED/GREEN evidence.** This repo means it literally. A
compile error is *not* RED. A test that fails because the enum variant or the
command does not exist yet is a setup error. The coordinating agent rejected
work three separate times over exactly this and made workers rebuild a
compiling empty stub so the assertion could fail behaviourally. Record the
failing output and the passing output in the bead.

**Pre-commit hook.** Enable it in any new checkout:
`git config core.hooksPath .githooks`. It runs `cargo fmt --all --check`, then
`cargo clippy --workspace --all-targets -- -D warnings`, then
`cargo check --workspace --all-targets`. It is slow. `--no-verify` exists but
CI will catch you.

**No Rust file over 500 lines.** Tests included. Split by responsibility.

**Rule doc headers are capped mechanically.** One short paragraph above a
`#[rule]`. `crates/provenance-cli/tests/cli_structure.rs` fails on record-keeping
phrases such as "Amended 20" or "tracked in beads". Amendment history belongs in
the graph record.

**Write graph records before code.** Use the `provenance-grounded-writing`
skill in `skills/` for any new Requirement, Rule, Resolution or Boundary text,
and `provenance-shaping` when a product direction changes. Rules follow
behavioural obligations, not code shape. Do not mint one Rule per function.
Never write a `#[verifies]` test that asserts nothing to clear a warning.

**Cross-platform traps already paid for once.** Paths in canonical state must be
repo-relative with forward slashes regardless of the writing machine. macOS
aliases `/var` to `/private/var`, so verification and implementation files both
need canonicalising. The packed-install test must run against a cold npm cache
or it passes only on a warm developer machine.

**The slow test is not hung.** `pr_45_scale_homepage...` in the wiki suite burns
CPU for minutes. Let it finish.

**Ben's register.** Plain English, no jargon, no coined shorthand. He will say
"less clanker verbiage" and he means it. When he asks what a change does, answer
in terms of what a company gets, not what the diff touches. Surface design forks
before building.
