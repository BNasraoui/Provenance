# Provenance

*The origin, history, and chain of reasoning behind every rule in your system.*

Every rule in your software exists for a reason. Provenance captures and preserves that
reason.

It is a CLI for refining software. The reasoning behind what you build, from the sources
that motivated it to the questions that got asked, the decisions that settled them, and the
rules that came out, is kept as a typed graph in your repo, alongside the code it explains.

```
Source → Requirement → Resolution → Rule → annotated code
```

When someone asks *"why does it work this way?"*, whether that's a new engineer, a reviewer,
or an agent picking up a session, Provenance answers with the full chain: the source, the
requirement it produced, the resolution that settled the ambiguity (position, rationale,
what informed it), the rule that codified the answer, and the code that enforces it. When a
source changes (a spec revs, a decision is reversed, an incident rewrites an assumption),
the same graph walks the other direction and tells you what's affected.

State is plain JSONL under `.provenance/state/`, committed to git, reviewed in PRs, merged
like anything else. A derived SQLite cache under `.provenance/cache/` makes queries fast;
it is disposable and never the source of truth. Every command speaks `--format json`, so
agents work the graph the same way people do.

## The data model

Four artifact types. Everything in the graph is one of these, connected by typed edges
(`references`, `refines_into`, `depends_on`, `contradicts`, `supersedes`, `needs`,
`resolves`, `spawns`, `produces`).

A **Source** is an external reference the software must answer to: a spec, a design doc,
an incident report, an integration contract, a policy. Sources don't change inside
Provenance; they change in the real world, and the graph tracks that via supersession.

A **Requirement** is the recursive unit of work: a statement about a desired state of the
system. Requirements decompose into sub-requirements, which are themselves full
requirements. There is no separate concept for initiative, epic, or story. The recursion
handles scale.

A **Resolution** records how an ambiguity was settled: the position taken, the rationale,
what informed it, how confident, and when to review it. Resolutions are independent nodes,
not fields on a requirement. One resolution can address several requirements, and a
resolution can outlive the requirement that prompted it.

A **Rule** is the terminal output: a testable constraint with a statement, classification,
severity, modality, and lifecycle, plus back-links to the research that produced it.

Refinement has working materials too, living inside requirements rather than as graph
nodes. **Topics** group **questions**, and each question is typed by the verb that resolves
it (`grill`, `prototype`, `research`, `verify`, or `task`), with claim state so concurrent
sessions don't collide. **Boundaries** are the no-gos: constraints on the solution space,
each traceable to its source. **Fog** is deliberately unstructured text for what you sense
coming but can't yet phrase sharply; a question is minted only when it can be stated
precisely. Candidate artifacts from any producer land as **proposals** awaiting explicit
human promotion, and **threads**, conversations attachable to any artifact, are the
interface humans and agents share.

## Quick start

```bash
# Create a store in your repo (state lives in .provenance/state/, commit it)
provenance init --path . --scope default --path-prefix .

# Ground a requirement in a source
provenance sources create --scope default --id src_sharing_spec \
  --name "Link Sharing Design Doc" --source-type project_artifact
provenance requirements create --scope default --id req_share_expiry \
  --statement "Shared links must be short-lived"

# Refine: topics hold questions; every question is typed by how it gets resolved
provenance topics create --scope default --id top_expiry --requirement-id req_share_expiry \
  --title "Expiry semantics"
provenance questions create --scope default --id q_link_ttl --topic-id top_expiry \
  --question "How long should a share link live?" --method grill

# Validate and build the query cache
provenance check --format json
provenance materialize --format json

# Orient (designed for agents at session start)
provenance prime
```

## The shaping loop

Provenance is built around a turn-based process for refining software with an agent. A
session loads the map low-res, claims work so concurrent sessions skip it, resolves
questions by their method, records every decision the moment it resolves, and hands off
with the graph consistent. Design forks get fork tournaments: competing stance-based
prototypes for the human to react to. Existing codebases get the swarm backtrace: agents
extract candidate requirements from the code and land them as proposals, never as fact.

The canonical design is [docs/shaping.md](docs/shaping.md). Agent skills implementing it
live in [skills/](skills/), currently `fork-tournament` and `swarm-backtrace`, with the
core shaping skill in progress.

## Annotations and coverage

Rules trace into code via comment annotations, scanned with tree-sitter:

```rust
// @provenance rule: SHARE-EXP-001
// @provenance coverage: full
fn enforce_link_expiry() { /* ... */ }
```

```bash
provenance coverage scan --format json   # where rules are (and aren't) enforced
```

## Asking questions of the graph

Every artifact links to its predecessors, and the chain is queryable in both directions:

```bash
provenance traceability <rule-id>          # why does this rule exist? (backward chain)
provenance impact --node-type source <id>  # source changed, what's affected? (forward)
provenance gaps                            # requirements with no rules, rules with no coverage
provenance stale                           # resolutions past their review date
provenance graph <requirement-id>          # the neighborhood of a requirement
provenance health                          # overall graph health
```

## Installation

From source, for now:

```bash
cargo build --release -p provenance-cli --all-features
cp target/release/provenance ~/.local/bin/
```

Prebuilt binaries and package-manager installs are planned; see the issue tracker. Skills
will install via `provenance skills install` (in progress).

## Development

Rust workspace, four crates:

- `crates/provenance-core`: domain model, edge validation
- `crates/provenance-store`: JSONL state store, shards, SQLite cache, migrations
- `crates/provenance-scanner`: annotation scanner and coverage
- `crates/provenance-cli`: the CLI; integration tests live here

Quality gates: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --all --check`. Issues are tracked with [beads](https://github.com/steveyegge/beads)
(`bd ready` to see the frontier); agent workflow conventions are in [PROMPT.md](PROMPT.md)
and [AGENTS.md](AGENTS.md).

Docs: [state format](docs/state-format.md) · [CLI](docs/cli.md) · [cache](docs/cache.md) ·
[shaping](docs/shaping.md) · [migration from Statesman](docs/migration-from-statesman.md) ·
[release](docs/release.md)

## Status

Early. The state format carries a `schema_version` and the store applies migrations, but
expect breaking changes while the shaping loop is being dogfooded.

## License

Licensed under the [Business Source License 1.1](LICENSE).
