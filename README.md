# Provenance

Never lose the *why* behind your decisions.

Provenance is a tool for building requirements traceability, from source to requirement to rule. A rule is a function, bound to its record in the graph by `#[rule("rule_id")]`; the tests that verify it carry `#[verifies("rule_id", method)]`.

### Installation

```sh
cargo build --release -p provenance-cli --all-features
```

The binary lands at `target/release/provenance`. Put it on your PATH.

### Quick start

```sh
# set up a repo (commit .provenance/state/, ignore .provenance/cache/)
provenance init --path . --scope default --path-prefix .

# put something in the graph
provenance requirements create --scope default --id req_exports \
  --statement "Exports finish in under a minute"

# see where things stand
provenance prime
```

### Essential commands

| Command | What it does |
| --- | --- |
| `provenance prime` | Bounded low-res graph frontier; proposals surface separately when evidence or claimed territory demands them |
| `provenance check` | Validate the state files |
| `provenance materialize` | Rebuild the SQLite query cache |
| `provenance graph <requirement>` | Show the neighbourhood of a requirement |
| `provenance graph-reference issue\|show\|verify\|exact-export` | Hand off an immutable pinned graph |
| `provenance traceability <rule>` | Walk a rule back to the decision and requirement behind it |
| `provenance proposals surface --scope default --changed-path <path>` | Surface undisposed proposals when current work touches their evidence or explicit territory |
| `provenance wiki build` / `provenance wiki serve` | Build or serve the generated wiki with domain browsing and offline search |
| `provenance coverage scan --path . --validate-rules` | Check every marker against the graph and name each active rule with no verification |
| `provenance stale --since main` | Report whether a diff touched, moved, or removed any graph evidence path |
| `provenance skills install` | Install the bundled agent skills (`provenance-shaping`, `provenance-fork-tournament`, `provenance-swarm-backtrace`, `provenance-grounded-writing`) |

The repository uses the `skills/<name>/SKILL.md` layout, so the bundled skills can also
be installed through the skills.sh ecosystem with `npx skills add <owner/repo>`.

### Documentation

- [Shaping](docs/shaping.md), the refinement method and how agent sessions run it
- [CLI](docs/cli.md), the full command surface
- [State format](docs/state-format.md) and [cache](docs/cache.md), how storage works

Licensed under BUSL-1.1.
