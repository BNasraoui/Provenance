---
date: 2026-08-18
git_commit: 6b7c161cf82f01563935c99d32b4c51d32ac1913
branch: codex/simple-technical-english-spike
topic: "Session Handoff: ASD-STE100 checks and the Issue 9 dictionary importer"
tags: [handoff, ste100, writing-standard, dictionary, licensing]
status: in-progress
---

# Handoff: ASD-STE100 Issue 9 in Provenance

Written for an agent with no prior context. The work ran in Codex threads from
2026-08-14 to 2026-08-18. Those agents ran out of quota mid-flight.

## READ THIS BEFORE TOUCHING ANYTHING

The branch is a **moving target**. A Codex agent was still editing the worktree
at `/home/ben/.codex/worktrees/ada04291-7d5f-4753-8c88-4c4b67d2d949/provenance`
at 15:59 on 2026-08-18. The Codex weekly quota resets on Thu 20 Aug 2026 at
16:53 local, so those agents may resume without warning.

**Do not work `codex/simple-technical-english-spike` concurrently.** Do not
commit in that worktree. Read it, then either agree a handover with Ben or work
in a fresh worktree off the branch tip. Two agents on one branch will destroy
each other's work.

The state below is commit `6b7c161` plus the uncommitted delta as of
2026-08-18 16:00.

## What this is for

Ben wants requirement and rule prose checked against a real writing standard
while it is being written, and rejected no later than the graph write. The
trigger was concrete. A research report described the existing 110 Requirement
and Rule statements as a "corpus", and the agent had to flag the word as
project jargon:

> "even the fact you had to flag that you needed to use words like corpus to
> describe Provenance is evidence enough to me that something like this should
> be on by default. Stupid fucking word"

He then sharpened it when the agent misread him:

> "My point was that the fact something was allowed to be called corpus is
> evidence enough that this is necessary. What does corpus relate to in
> provenance?"

The answer was nothing. "Corpus" hid three concrete things: the 110 existing
statements, a labelled test set, and a held-out test set. That is exactly the
vagueness the checker exists to stop.

## What Ben ratified

1. **Use ASD-STE100 Issue 9 itself. Do not invent a dialect.**
   > "We are going to use the simple technical english spec explicitly, not some
   > custom bespoke language ruleset we invent. We may offer a way for people to
   > extend this, but this is not critical."

   Findings cite real ASD-STE100 rule numbers. Later extensions may supplement
   the standard but never replace it. The first release has no extension
   mechanism. An earlier draft proposing a Provenance-owned `provenance-simple-v1`
   profile and a generic `provenance-writing` crate was deleted.

2. **Check while writing; enforce at graph write.**
   > "I'd like some kind of compile time check on syntax in our requirements
   > that violates it. Maybe that's only practical at graph generation... ideally
   > it operates at compile time and flags stuff as agents or humans are
   > writing."

   True TypeScript type-level enforcement was rejected as not worth it. The
   closest useful compile-time boundary is graph compilation. One Rust checker
   serves author-time feedback, the SDK, the CLI and every write gate.

3. **On by default, for new or changed statements only.** Unchanged legacy text
   never blocks unrelated work. 43 of the 110 existing statements trip at least
   one mechanical check, so a retroactive gate would have been a migration
   project.

4. **Strict only where the standard is exact and data-free.** Anything needing a
   meaning choice returns no violation rather than a false rejection. Malformed
   or nested quotes make quote-dependent checks indeterminate instead of
   exposing questionable text.

5. **Dictionary enforcement is wanted.** The agent recommended shipping the
   data-free checks and closing the rights question. Ben overruled it:
   > "No I want dictionary enforcement."

6. **Do not wait for STEMG to publish a machine-readable format.**
   > "I don't think we should rely on stemg to give us a reusable format to use."

   There is none. The ASD FAQ states PDF is the only distribution format. Part 2
   starts on PDF page 129, the A–Z entries run pages 149 to 433, and Issue 9
   states 875 approved and 1,274 unapproved words. The popular
   `AminBlg/SimpleEnglish` skill (~4.2K installs) does **not** contain the
   dictionary — it is a prompt plus a small hand-written table and a regex
   linter. It cannot be a source.

7. **Local import, no redistribution.** Provenance ships original checking code.
   It never commits, packages, mirrors or publishes the PDF or anything
   extracted from it. Only the issue number and digests go in project
   configuration.

## The licensing and consent decision

This is the highest-value part of the record. Get it right.

**The constraint.** ASD owns Issue 9. Receiving the document grants no reuse or
publication rights. Reproduction in whole or part needs written ASD authority.
Special reuse rights exist for listed aerospace bodies, defence bodies,
authorities and universities; Provenance is not clearly covered. ASD also says
third-party checkers are neither endorsed nor certified. So Provenance must not
ship the PDF, extracted word lists, definitions, alternatives, tables, examples,
substantial rule text, ASD logos, or any claim of approval or compliance.

**"Ship it and apologise later" was rejected.** Ben floated it. The agent
refused: an open-source release cannot be recalled — Git history, registries,
forks and caches keep the data — and Provenance cannot grant users rights it
does not hold. Ben accepted that.

**Ben's acquisition policy:**
> "If it's being installed in interactive mode we prompt them to go fill out the
> form and download it. If it's an agent doing it we just download it for them.
> We should make sure to keep that config or autoimport it into project so that
> way we're courteous and don't spam their DL link on thousands of CI runs. We
> want to be as respectful as is practical to the stewards of the standard."

The direct asset URL `https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf`
currently returns the full 434-page PDF from ASD's own server. It is not linked
from their download page, which asks for a completed request form. Treat it as
an undocumented endpoint that may vanish.

**The Fable review.** Ben asked for an independent review of the fairness
question. Verdict: **conditional go**. The local-import design is fair to STEMG.
Its one objection was that "an agent is running" does not itself justify
bypassing the form: the form collects name, country, organisation, field and
new-user status, which STEMG says supports its records and statistics. Fable
proposed making the form the normal path for people **and** agents, with direct
download only behind explicit authorisation such as `--download-official-copy`.

**Ben's ruling overrode exactly that one point:**
> "Agreed except for the form by default. Dumb agents will definitely get stuck."

So the controlling decision is:

- Interactive onboarding sends a person to the official request form and imports
  the PDF they select.
- **Agent onboarding downloads the official PDF automatically** when the project
  has no imported dictionary. This is deliberate and was chosen against the
  reviewer's advice. Do not "fix" it back to form-first.
- Everything else in the Fable verdict was accepted.

The accepted safeguards, now recorded as graph Rules:

- Only onboarding may use the network. Checks, builds, tests and CI never
  download. CI uses a preloaded file or a persistent cache.
- One shared machine-wide cache, a per-asset download lock, a bounded retry
  count, and an identifiable Provenance User-Agent.
- If ASD removes the asset, fall back to the request form. Do not search for or
  scrape a replacement path — recorded as
  `boundary_ste_no_replacement_asset_search`.
- Never commit, mirror or redistribute the PDF or extracted data.
- Attribute ASD as owner and STEMG as maintenance group. Link to the request
  page first. No ASD or STEMG logos.
- Say "checks implemented parts of Issue 9". Never "compliant", "certified",
  "endorsed", or "passes STE".
- Report approved, unapproved, unknown and uncertain uses separately. An
  approved word can still carry a restricted meaning or part of speech, so a
  word match cannot settle those cases.
- Store the source digest, extracted-data digest, issue number and extractor
  version. Give users the STEMG change-form link for suspected dictionary
  defects.

**This decision is ratified in the graph and NOT implemented in code.** No
downloader, no onboarding command, no cache, no configuration, no user agent,
no attribution text exists. `grep` finds no network code in `crates/`. Rules
`rule_ste_dictionary_agent_acquisition`, `rule_ste_dictionary_interactive_acquisition`
and `rule_ste_dictionary_not_distributed` have no implementation bindings. This
is the largest block of agreed-but-unbuilt work on the branch, tracked as
`provenance-l3m.14`.

One human task falls out of it: `provenance-l3m.16`, notify STEMG about the
feature and its automatic agent download before the first dictionary-enabled
release. Development does not wait for their answer.

## What has shipped

Eleven PRs merged **into the spike branch**, all green on Linux, macOS and
Windows:

| PR | What |
|---|---|
| #94 | Distribution-rights research recorded in the graph, licensing left to a human |
| #96 | `provenance-ste100` crate; Rule 8.1 semicolon checker, exhaustively tested |
| #97 | Default write gate on direct Requirement and Rule creation |
| #98 | Typed SDK Plan diagnostics and atomic Apply rejection |
| #99 | Repository-independent author-time `check-statement` preflight |
| #101 | Import, dry-run and JSONL merge gates; informational `check` against Git HEAD |
| #102 | Rule 4.2 contracted verb forms |
| #103 | Rule 6.3 descriptive sentence length, using the Rules 8.4–8.7 count method |
| #104 | Quoted-text protection shared by Rules 4.2, 6.3 and 8.1 |
| #106 | Full Issue 9 audit for further exact, data-free checks |
| #107 | Rule 6.6 paragraph sentence limit |

Enforced today: semicolons (8.1), contractions (4.2), descriptive sentence
length (6.3), paragraph length (6.6), and quoted-text protection across all of
them. Every gate calls the same Rust checker; there is no duplicated rule logic
and no copied dictionary.

The #106 audit is the important negative result: **Rule 6.6 was the last exact,
data-free strict check available for plain statement strings.** Everything
remaining needs licensed word data, parsed structure, or a human meaning choice.
Rule 4.3 would first need typed list input. That is why the dictionary importer
is the only way forward.

`AGENTS.md` on this branch now requires ASD-STE100 Issue 9 for technical prose —
graph records, docs, comments, beads, PRs, commits and handoffs — and warns that
a clean automated report does not prove conformance. That section does not exist
on `main`.

Report: `docs/research/2026-08-15-simple-technical-english-configuration.md`,
itself rewritten in the standard.

## Exact current state

**Branch:** `codex/simple-technical-english-spike`
**Worktree:** `/home/ben/.codex/worktrees/ada04291-7d5f-4753-8c88-4c4b67d2d949/provenance`
**HEAD:** `6b7c161` "wip(ste100): snapshot the PDF dictionary importer",
one commit ahead of `origin/codex/simple-technical-english-spike` (`d56874b`).

Ben made `6b7c161` himself with `--no-verify`, because the pre-commit hook runs
`cargo clippy --workspace --all-targets -- -D warnings` and the work in progress
does not pass it. **Clippy is not clean on this branch.** Treat the commit as a
safety snapshot, not a checkpoint.

**Uncommitted in that worktree:**

```
 M .beads/interactions.jsonl                            <- Ben's. Do not touch.
 M crates/provenance-ste100/src/dictionary/layout.rs
 M crates/provenance-ste100/src/dictionary/mod.rs
 M crates/provenance-ste100/src/dictionary/parse.rs
 M crates/provenance-ste100/tests/dictionary_import.rs
```

The crate:

```
crates/provenance-ste100/src/
  lib.rs  contracted_verbs.rs  paragraph.rs  protected_spans.rs
  sentence.rs  word_count.rs
  dictionary/{mod,pdf,layout,parse,digest}.rs
crates/provenance-ste100/tests/
  rule_4_2.rs  rule_6_3.rs  rule_6_6.rs  rule_8_1.rs
  quoted_text_protection.rs  dictionary_import.rs
```

`import_dictionary(&[u8]) -> Result<DictionaryImport, DictionaryImportError>` is
the whole public seam: bytes in, verified import or typed failure out. PDF
parsing, entry reconstruction, count checks and digests stay behind it. The PDF
library is an adapter, pinned at `pdf_oxide =0.3.77` with default features off
and only `legacy-crypto` enabled — no renderer, OCR, GPU or system fonts. The
crypto feature is needed because the official PDF is encrypted with an empty
user password, and without it text extraction misreads encrypted streams as
corrupt compression.

`DictionaryImportIdentity` carries issue, `source_sha256`, `data_sha256` and
`extractor_version`. `DictionaryEntry` carries headword, word forms, part of
speech, status, approved meaning or alternatives, STE example and optional
non-STE example.

**Tests.** Synthetic-PDF cases pass: rejects non-PDF input, rejects a PDF
without Issue 9 identity, rejects an incomplete dictionary, and imports a
complete positioned dictionary deterministically. The real-document test
`imports_the_local_official_issue_9_pdf` is `#[ignore]` and reads the PDF path
from `ASD_STE100_ISSUE9_PDF`. No PDF is committed and none should be.

**Layout traps already found and paid for.** The text stream merges the two
example columns on some rows, so cells must be reconstructed by coordinates, not
stream order. A column midpoint split the part-of-speech token from its
headword, so boundaries use the next column's actual start. A five-point hanging
indent attached an alternative in column 2 to the wrong headword in column 1.
Either PDF y-axis direction is normalised before rows are built.

### The unresolved problem the work stopped on

Raw row counts from the official PDF are **878 approved and 1,314 unapproved**.
Issue 9 states **875 and 1,274**. The difference is repeated headwords under
different parts of speech, not stray pages — widening the column offset did not
change the totals, which rules out another column leak.

The uncommitted delta is the investigation into which unit the standard counts:

- `mod.rs` renames `EXPECTED_*_ENTRIES` to `EXPECTED_*_HEADWORDS` and switches
  validation to **distinct lowercase headwords**, while still counting and
  reporting headword/part-of-speech pairs, distinct word forms, raw rows and
  order violations in the failure message. `PartOfSpeech` gained `Ord` for the
  pair set.
- `parse.rs` joins headwords wrapped across lines with a trailing hyphen, and
  treats the prior line as the entry start in that case.
- `layout.rs` widens the column boundary offset from 10 to 20 points.

Counting spellings alone is too coarse; raw rows are too fine when one
word/part-of-speech entry spans several sense rows. **The count unit is not
settled.** That is where a new agent picks up.

### Integration debt

The spike branch is **47 commits ahead of and 20 behind `origin/main`**, with no
open PR to `main`. `main` has no `provenance-ste100` crate and no STE section in
`AGENTS.md`. Every one of the eleven PRs above merged into the spike branch, not
into `main`. This gap grows every day the branch stays open.

## Open questions for Ben

1. **How does the spike branch land on `main`?** One large PR, or split by
   slice? Nothing is on `main` after four days of merged work. This needs a
   decision soon.
2. **Should the importer accept counts that disagree with the printed totals?**
   If distinct headwords do not land on 875 and 1,274, the choice is to relax the
   check to a structural one, or to keep it strict and reject the real PDF.
   Deciding this decides whether the importer can ever pass on the official
   document.
3. **Where does the normalised index live, and what shape is it?** The design
   says an operating-system private application-data directory with only issue
   number, parser version and digest in the project. None of it is built, and
   `provenance-l3m.13`'s acceptance criteria require a "reusable local index"
   that does not exist yet.
4. **Nothing else.** Do not reopen the download policy. Ben ruled on it.

## Next step

Finish `provenance-l3m.13` — but first, settle the count unit. Concretely:

1. Coordinate with Ben about the branch. Confirm no Codex agent is live before
   you touch it.
2. Run the ignored test against a local official PDF with
   `ASD_STE100_ISSUE9_PDF` set. The diagnostic in the uncommitted `mod.rs`
   already prints distinct headwords, headword/part pairs, distinct word forms,
   raw rows and order violations for both statuses. Read those five numbers
   against 875 and 1,274 and pick the unit that matches. That is a five-minute
   experiment and it unblocks everything.
3. Pin the answer with duplicate-headword fixtures before changing the check.
4. Get clippy clean, then replace `6b7c161` with a proper commit.
5. Then `provenance-l3m.14` (onboarding paths and safeguards) and
   `provenance-l3m.15` (enforce the dictionary in statement checks).
   `provenance-l3m.16` is Ben's to send.

Keep the importer honest: report only page and row in diagnostics, never the
copyrighted text.

## Open beads

```
provenance-l3m      epic         Enforce ASD-STE100 syntax before graph writes
  l3m.13  in_progress  p1  Import the Issue 9 dictionary from an official PDF
  l3m.14  open         p1  Add interactive and agent dictionary onboarding
  l3m.15  open         p1  Enforce the imported dictionary in statement checks
  l3m.16  open         p2  Notify STEMG before the dictionary-enabled release
```

`l3m.14`'s notes already carry the ruling: "Human ruling after Fable review: do
not make the request form the agent default. Limited agents can stop at the
form, so agent onboarding downloads automatically."

`provenance-l3m.6`, the original human rights decision, is closed. It was
resolved by `res_ste_local_dictionary_acquisition` and
`res_ste_agent_download_is_default`, with the old wait-for-a-licence position
recorded as rejected in `disp_ste_data_free_until_licensed_rejected`.

## Gotchas that will bite you

**Beads is the only tracker.** `bd ready`, `bd update <id> --claim`,
`bd close <id> --reason "..."`, always `--json`. This checkout's bd has no
`bd sync`; use the Dolt push path. **Never touch `.beads/interactions.jsonl` —
it holds the user's own uncommitted lines and every prior agent preserved them
byte for byte. Do the same.**

**TDD with recorded RED/GREEN evidence.** A compile error is not RED. The
coordinating agent rejected work three times on this branch over exactly that
mistake: a test failing because `RuleNumber::SixThree` did not exist is a setup
error, not a behavioural failure. Build a compiling empty stub, watch the
assertion fail, then implement. Record both outputs in the bead.

**Pre-commit hook.** `git config core.hooksPath .githooks` in any new checkout.
It runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --
-D warnings`, then `cargo check --workspace --all-targets`. **Clippy currently
fails on this branch**, so you will need `--no-verify` until you clean it. Say so
when you do.

**Graph Rules before production code.** Every slice on this branch added its
graph Rule first. Use the `provenance-grounded-writing` skill in `skills/` for
any new Requirement, Rule, Resolution or Boundary statement, and
`provenance-shaping` when a product direction changes. Tie each Rule to one
exact ASD-STE100 obligation. Do not write broad claims such as "checks
contractions correctly". Then bind implementation with `#[rule("id")]` and
evidence with `#[verifies("id", method)]` where method is one of `exhaustion`,
`property`, `examples`, `conformance`, `construction`, `proof`.

**The checker's own graph must follow the standard.** A reviewer rejected graph
statements on this branch for being longer than needed and for coining a project
term where the standard's plain "quoted text" was enough. Passing the
conservative checker is not the bar; the writing is.

**Banned word.** "Corpus" started this whole line of work. Do not use it. A
reviewer held a merge because a test name still contained it. "Determinate" was
also replaced with plain wording.

**No Rust file over 500 lines.** Tests included.

**Slow tests are not hung.** The wiki scale test burns CPU for minutes. Windows
CI workspace tests are always the last to finish.

**Ben's register.** Plain English, short sentences, active voice. He interrupts
rambling. Answer what a change does for a user, not what the diff touches.
