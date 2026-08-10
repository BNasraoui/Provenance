# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Bootstrap

After cloning the repository or creating a worktree, enable the committed Git
hooks from that checkout:

```bash
git config core.hooksPath .githooks
```

The pre-commit hook runs formatting, Clippy, and compile checks. Fix any
reported failure before committing. When a hook must be bypassed intentionally,
use Git's standard `git commit --no-verify` option.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work atomically
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Non-Interactive Shell Commands

**ALWAYS use non-interactive flags** with file operations to avoid hanging on confirmation prompts.

Shell commands like `cp`, `mv`, and `rm` may be aliased to include `-i` (interactive) mode on some systems, causing the agent to hang indefinitely waiting for y/n input.

**Use these forms instead:**
```bash
# Force overwrite without prompting
cp -f source dest           # NOT: cp source dest
mv -f source dest           # NOT: mv source dest
rm -f file                  # NOT: rm file

# For recursive operations
rm -rf directory            # NOT: rm -r directory
cp -rf source dest          # NOT: cp -r source dest
```

**Other commands that may prompt:**
- `scp` - use `-o BatchMode=yes` for non-interactive
- `ssh` - use `-o BatchMode=yes` to fail instead of prompting
- `apt-get` - use `-y` flag
- `brew` - use `HOMEBREW_NO_AUTO_UPDATE=1` env var

## Rules

A rule is a function — or a type whose construction is the proof. `#[rule("rule_id")]`
binds one function to one rule record in the graph: the function *is* the decision, not a
description of it and not a claim to satisfy it. Exactly one function may carry an id.

`#[verifies("rule_id", method)]` marks whatever proves the rule, with one method word:

- `exhaustion` — every input in a finite domain is tried
- `property` — generated inputs checked against a stated property
- `examples` — hand-picked cases
- `conformance` — a copy of the rule elsewhere checked against the rule function
- `construction` — a type or constraint makes violation impossible; the attribute goes on
  the type, never on a test

Both attributes come from `provenance-macros` (`use provenance_macros::rule;`,
`use provenance_macros::verifies;`). They expand to nothing and cost one argument check at
compile time; what they buy is a symbol the scanner finds and refactors carry along.

Unverified is **absence**, derived and never stored. `provenance coverage scan --path .
--scope default --validate-rules` reports it, along with unknown rule ids, a second
function claiming one id, and a `#[verifies]` with no `#[rule]` to verify. Adding
`--strict` makes any warning a non-zero exit; how strictly CI runs the scan is a per-repo
dial, not a property of a rule.

The rule record carries the binding in `--source-document` (the file) and
`--source-section` (the bare symbol) — never a line range.

Rules follow human decisions, not code shape. Do not mint one rule per function, and do
not split one decision across five rules because the match has five arms. Prose intent
lives in the requirement and the resolution. **A decision with no function is a
requirement whose rule is unwritten** — an ordinary state, not a defect. Leave it a
requirement; never create a rule record with no function behind it, and never write a
`#[verifies]` test that asserts nothing to clear a warning.

## Rule Doc Headers

The doc comment above a `#[rule("...")]` item is one short paragraph saying what
the rule decides, followed only by constraints the code cannot show for itself.
Amendment history, proof inventories, and cross-references belong in the rule's
graph record, not in the source header. `crates/provenance-cli/tests/cli_structure.rs`
enforces this mechanically: it caps how many `///` lines may sit above a
`#[rule]` attribute and fails on record-keeping phrases such as `Amended 20` and
`tracked in beads`.

<!-- BEGIN BEADS INTEGRATION -->
## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Version-controlled: Built on Dolt with cell-level merge
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update <id> --claim --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task atomically**: `bd update <id> --claim`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Auto-Sync

bd automatically syncs with git:

- Exports to `.beads/issues.jsonl` after changes (5s debounce)
- Imports from JSONL when newer (e.g., after `git pull`)
- No manual export/import needed!

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

For more details, see README.md and docs/QUICKSTART.md.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

<!-- END BEADS INTEGRATION -->
