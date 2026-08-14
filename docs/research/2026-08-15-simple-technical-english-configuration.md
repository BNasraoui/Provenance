# Simple Technical English configuration research spike

**Bead:** `provenance-0ss`<br>
**Date:** 2026-08-15<br>
**Status:** research complete; direction not ratified<br>
**Baseline:** `ef8269843b9c9d1f2246a4cc7cd54825e72a4f37`

## Outcome

This is a good idea if Provenance ships an opt-in, versioned writing profile with an
honest limit: it can give deterministic findings for mechanical rules and useful review
prompts for syntactic rules, but it cannot certify that prose is clear or fully compliant
with ASD-STE100.

The recommended first product is **not** an `ASD-STE100 compliant` switch. It is a
Provenance-owned profile, tentatively named `provenance-simple-v1`, for Requirement and
Rule statements. Its strict tier should contain only checks whose violations the tool can
prove from the text and committed configuration. Parser-based findings should remain
warnings until a labelled corpus shows that they have near-zero false positives.

True TypeScript or Rust compile-time verification is not a viable primary mechanism.
Build-time verification is viable: `provenance writing check --strict` can run before a
build, in CI, and from the existing typed-SDK plan/apply path. This covers every canonical
record, including records created outside TypeScript, and keeps one Rust-owned definition
of each check.

Do not bundle the official ASD-STE100 dictionary, examples, or rule text without written
permission or legal review. Issue 9 says ASD owns the document and restricts reproduction.
ASD also warns that tools are aids, cannot check all rules, and must not imply ASD
certification. Provenance can safely validate an original controlled-writing profile while
leaving a future licensed ASD-STE100 adapter as a separate decision.

## The question being answered

The phrase “deterministic STE verification” can make three different claims:

1. **Repeatable execution:** the same text, profile, terms, and checker version produce the
   same findings.
2. **Definitive findings:** each reported violation is in fact a violation of the selected
   writing policy.
3. **Complete conformance:** no findings means that the text complies with all writing,
   vocabulary, sense, and clarity rules.

Provenance can guarantee the first. It can guarantee the second for a deliberately small
mechanical tier. It cannot guarantee the third for natural-language STE. A deterministic
algorithm can still make the same incorrect parse every time.

This spike considers canonical Requirement and Rule statements first. It does not propose
rewriting source quotations, names, citations, discussion messages, fog, rationales, or
descriptions. Those fields either preserve outside evidence or need enough prose to explain
context and trade-offs.

## External evidence

ASD-STE100 Issue 9, published in January 2025, has 53 writing rules in nine sections plus a
controlled dictionary. The rules distinguish procedural text from descriptive text. This
matters because Provenance statements are declarative obligations, not maintenance work
steps. The 25-word descriptive limit is the nearer fit; the 20-word procedural limit and
rules about work steps, notes, and safety instructions are usually inapplicable.

The official sources set a clear ceiling on automation:

- The [Issue 9 standard](https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf)
  defines lexical, grammatical, structural, and meaning-dependent rules. Its copyright
  notice reserves reproduction except for named groups and uses.
- The STEMG [software guidance](https://www.asd-ste100.org/software.html) says that no
  checker can test all rules, that accuracy varies, and that authors must decide whether a
  finding applies in context.
- Boeing's [checker description](https://www.boeing.com/company/simplified-english-checker)
  shows the scale of a mature implementation: a full syntactic parser with more than 400
  syntax rules, technical-vocabulary management, and only partial word-sense checking.
- A published [controlled-language parser study](https://aclanthology.org/2003.eamt-1.15.pdf)
  describes the same split. Sentence and paragraph length, noun clusters, auxiliaries,
  verb forms, multiple commands, and document labels are automatable; approved meaning
  needs a separate meaning-based analysis.

The evidence supports a linter with graded findings, not a compliance certificate.

## How deterministic can the checks be?

### Tier 1: strict-safe mechanical checks

These checks need no probabilistic model. With a frozen tokenizer and profile revision,
they can produce exact, stable results:

- sentence word limits for a declared text mode;
- paragraph sentence limits when the input retains paragraph structure;
- forbidden punctuation such as semicolons;
- closed lists of unambiguous contractions or forbidden modal words;
- approved spelling when the configured dialect and exception list are explicit;
- word-count treatment for hyphens, parentheses, numbers, identifiers, quotations, and
  list-introducing colons.

The last item is important. “25 words” is deterministic only after the profile defines what
counts as one word. The ASD counting rules can inform an original tokenizer contract, but
Provenance should not silently claim that a similar count is official ASD conformance.

### Tier 2: strict-safe only with committed terminology

Vocabulary membership and inflected forms are deterministic when all permitted inputs are
committed and versioned:

- the profile's general vocabulary;
- project-approved technical nouns and their plural forms;
- project-approved technical verbs and their allowed forms;
- product names, abbreviations, identifiers, and protected literals;
- preferred terms for a concept when a human has declared the equivalence.

The checker can prove “this token is absent from the configured vocabulary.” It cannot
prove that an unknown token is or is not a legitimate technical noun. An unknown term must
therefore be either a review finding or an error only in repositories that choose a closed
term registry.

This tier is still deterministic because the human decision enters as configuration. The
tool checks consistency with that decision; it does not infer the decision.

### Tier 3: repeatable parser findings, not definitive verdicts

A fixed rule-based parser can repeatably flag:

- a dictionary word used as the wrong part of speech;
- noun clusters longer than the configured limit;
- disallowed tense and auxiliary constructions;
- likely passive voice;
- `-ing` forms outside an allowed noun or modifier position;
- missing articles;
- likely imperative instructions and multiple commands;
- likely phrasal verbs;
- whether a condition appears before a command.

These findings depend on a correct token, part-of-speech, and sentence parse. Technical
terms and short requirement fragments are common sources of ambiguity. They should carry a
`review` disposition rather than a numeric confidence and should not fail CI until the
specific check has demonstrated strict-tier precision on the project's corpus.

A deterministic rule-based parser is preferable to an LLM for this tier. A fixed local
model could also be repeatable, but it would still have classification errors and would add
model-file, platform, and version controls without making the result a proof.

### Tier 4: human-only meaning and discourse checks

The checker cannot decide these claims from surface text alone:

- an approved word has the intended approved meaning;
- a term is easy to understand rather than jargon;
- two different terms name the same real item;
- a sentence is clear, gives one idea, or preserves meaning after a rewrite;
- information arrives gradually and paragraphs stay on one topic;
- a list is needed because prose is complex;
- a safety explanation names the correct risk or result;
- two actions truly occur at the same time;
- a sentence is factually correct or implements the intended decision.

An LLM can suggest review candidates or rewrites here, but its output must never enter the
strict verdict. Provenance should not store a `compliant` boolean. As with Unimplemented and
Unverified, writing conformance is derived from current text, profile, terms, and checker
version.

## Compile-time verification

| Mechanism | What it can prove | Coverage | Recommendation |
| --- | --- | --- | --- |
| TypeScript template-literal and conditional types | Simple patterns on preserved string literals, such as a forbidden semicolon or a small token union | Typed SDK literals only; no CLI, JSONL, imports, or computed strings | Do not use as the primary checker |
| TypeScript language-service plugin | Editor diagnostics over source literals | Editor only; the official plugin interface supplies language-service information and does not make `tsc` a repository policy gate | Useful later as an adapter |
| Custom TypeScript transformer or ESLint rule | Arbitrary build-time lint over TypeScript syntax | TypeScript only and easy to bypass through another graph ingress | Optional adapter, not source of truth |
| Rust procedural macro | Arbitrary checks over a Rust string literal | No current Provenance prose declaration path uses Rust literals | Not applicable |
| `provenance writing check --strict` | Every supported deterministic check over every selected canonical field | All canonical records, including hand-edited and imported JSONL | Primary gate |
| Typed SDK `plan` / `apply` preflight | The same Rust-owned checks before reconciliation | Typed declarations | Primary author feedback for the SDK |

TypeScript's type system can deconstruct literal strings with template literal types, but
the [TypeScript documentation](https://www.typescriptlang.org/docs/handbook/2/template-literal-types)
recommends ahead-of-time generation for large string unions. Recursive conditional types
also have cost and depth limits; the
[TypeScript 4.1 notes](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-1.html)
warn library authors not to ship complex recursive types that fail on realistic inputs.
The [language-service plugin setting](https://www.typescriptlang.org/tsconfig/plugins.html)
is explicitly an editor extension point.

The current SDK exposes `statement: string` in
`packages/provenance/src/spec.ts:17-29`. Preserving each literal as a generic type and
recursively tokenizing it would make the interface and error messages much larger while
still missing dynamic strings and every non-TypeScript writer. It would also duplicate the
Rust policy. A branded `SimpleEnglish` string would prove only that a helper returned the
brand, not that the compiler independently understood the prose.

The useful interpretation of “compile-time” is therefore **a required build gate**, not a
language type-system proof.

## Fit with the current Provenance design

### What exists

- The manifest currently contains scopes, path prefixes, and disposition actors only
  (`crates/provenance-core/src/model/manifest.rs:19-43`). There is no repository policy
  configuration.
- CLI Requirement and Rule creation forwards unrestricted strings to `StateStore`
  (`crates/provenance-cli/src/handlers/requirements.rs:15-41` and
  `crates/provenance-cli/src/handlers/rules.rs:78-106`).
- Typed-spec validation checks document identity and scope existence, not statement text
  (`crates/provenance-store/src/state_store/typed_specs.rs:215-239`).
- `provenance check` validates structural integrity and graph references
  (`crates/provenance-cli/src/handlers/check.rs:14-64`). It currently has binary success or
  failure rather than graded findings.
- Exact graph exports include the selected Scope and canonical graph records
  (`crates/provenance-store/src/graph_reference/projection.rs:29-89`). They do not carry
  repository validation policy.
- The grounded-writing skill already asks for one plain, specific decision and moves
  context into descriptions (`.agents/skills/provenance-grounded-writing/SKILL.md:43-78`).
  That semantic guidance remains necessary even if mechanical writing checks exist.

### Consequences

1. Writing policy should be repository configuration, not fields on Requirement or Rule.
   Adding per-record compliance state would duplicate derived truth and dirty canonical
   graph records after checker upgrades.
2. The first integration should be a separate lint command. Structural `provenance check`
   should not start rejecting a legacy corpus merely because an opt-in style profile was
   added.
3. The analyzer must sit below every ingress. TypeScript must not own its own copy of the
   rules.
4. A validation report should include the engine version, profile revision, and policy
   digest. An exact graph export need not change unless the product later promises offline
   reproduction of writing findings from the export alone.
5. The profile must complement artifact semantics. It cannot replace the Swap, Name,
   Evidence, and Climb tests, because those ask whether a statement represents a grounded
   decision rather than whether its grammar is simple.

## Proposed product shape

### Configuration

Use a committed `.provenance/writing.toml`. Keep the public interface small: select a
versioned profile, target fields, term files, and enforcement by finding class. Do not
expose 53 booleans in the first version.

```toml
schema_version = 1

[defaults]
profile = "provenance-simple-v1"
targets = ["requirement.statement", "rule.statement"]
term_files = [".provenance/technical-terms.toml"]

[defaults.enforcement]
violation = "warn"
review = "warn"

[scopes.default.enforcement]
violation = "error"
review = "warn"
```

The profile name includes a revision. A future change to tokenization, vocabulary, or a
check's meaning creates `provenance-simple-v2`; it does not silently change v1 results.

A term file should make domain decisions explicit:

```toml
[[technical_nouns]]
term = "graph reference"
forms = ["graph reference", "graph references"]

[[technical_verbs]]
term = "materialize"
forms = ["materialize", "materializes", "materialized"]

[[protected_literals]]
value = "JSONL"
```

Do not copy the ASD dictionary into this file as a built-in asset. `provenance-simple-v1`
needs an original, documented general vocabulary or can start without closed-vocabulary
enforcement.

### Findings

The machine-readable result should name a stable check, exact graph location, original
span, and whether the finding is a proven violation or a request for review:

```json
{
  "profile": "provenance-simple-v1",
  "policy_digest": "sha256:...",
  "artifact": { "scope": "default", "type": "rule", "id": "rule_example" },
  "field": "statement",
  "span": { "start": 37, "end": 38 },
  "check": "PSE-PUNCT-001",
  "disposition": "violation",
  "message": "Replace the semicolon with two sentences or move context to the description"
}
```

Findings sort by scope, artifact type, ID, field, span, and check ID. The analyzer uses no
network, clock, ambient locale, or LLM. Golden tests freeze Unicode normalization,
sentence segmentation, token counting, abbreviation handling, hyphen handling, and spans.

The tool should suggest edits but never rewrite canonical state automatically. Even a
seemingly safe rewrite can change obligation strength or move a condition.

### Module and seam

Add a deep `provenance-writing` Rust crate with one pure external interface:

```rust
pub fn analyze(policy: &CompiledWritingPolicy, site: TextSite<'_>) -> Vec<WritingFinding>;
```

`TextSite` carries artifact identity, field, text, and declared text mode. The module hides
tokenization, terminology matching, rule dispatch, and any parser adapter. Tests exercise
the same interface as callers.

Adapters extract `TextSite` values from canonical records and typed declarations:

```text
CLI create ─────┐
SDK plan/apply ─┼─> TextSite ─> provenance-writing ─> stable findings
repository scan ┘                    ↑
                              profile + term registry
```

The first implementation needs no parser dependency. For the syntax tier,
[`harper-core`](https://docs.rs/harper-core/latest/harper_core/) is the strongest current
Rust-shaped candidate: it runs in process, exposes parsed tokens and verb metadata, accepts
custom dictionaries, and is Apache-2.0. It is not evidence of STE accuracy. Benchmark it
against a small purpose-built tagger and hand-labelled Provenance text before adopting it.

[Vale](https://vale.sh/docs/styles) is a useful prototype and comparison because it has
configurable style rules and JSON output. It is a weaker production seam here: it expects
documents and markup, while Provenance already knows the exact graph record and field. An
external process would add field serialization and span remapping without solving the
meaning-dependent rules.

## Baseline corpus exercise

A read-only script checked the 53 current Requirement statements and 57 current Rule
statements at the pinned baseline. It used a deliberately simple ASCII word tokenizer and
four candidate checks: semicolon, `shall`, more than 25 words in a sentence, and more than
one sentence.

| Candidate | Requirements | Rules | Total |
| --- | ---: | ---: | ---: |
| Semicolon | 6 | 12 | 18 |
| `shall` | 4 | 0 | 4 |
| More than 25 words | 9 | 20 | 29 |
| More than one sentence | 0 | 3 | 3 |
| At least one of the above | 15 | 28 | 43 of 110 (39.1%) |

This is not an STE compliance measurement. It is a migration and product-fit measurement.
It shows:

- strict-by-default adoption would break a large part of the existing graph;
- Rule statements are more affected than Requirements;
- existing Provenance guidance deliberately uses semicolons to state boundary cases, so
  changing the profile also requires changing the grounded-writing skill and reviewing
  whether the split loses atomic meaning;
- a warning-only rollout is necessary;
- the checker needs fixture-backed tokenization before its word count can become a gate.

A naive contraction expression produced 14 hits, all from apostrophe-`s` possessives in
this corpus. A closed contraction list removed those false positives. This small result
illustrates the general rule: repeatable pattern matching is not enough; a strict check
must also have a domain in which its classification is unambiguous.

## Implementation path

### Stage 0 — name the promise and clear rights

Decide between these products before implementation:

- **Recommended:** an original `provenance-simple-v1` controlled-writing profile, with no
  claim of ASD compliance.
- **Separate future product:** an ASD-STE100 Issue 9 adapter based on permission or a
  licensed data source, with clear non-certification language.

Exit criterion: product wording and redistribution rights are reviewed. If the requirement
is “Provenance certifies ASD-STE100 compliance,” stop; the evidence says that promise is
not technically supportable.

### Stage 1 — mechanical linter

1. Add `.provenance/writing.toml` loading with closed schemas and clear errors.
2. Add `provenance-writing` with sentence segmentation, token counting, punctuation,
   modal-word, contraction, and configured-term checks.
3. Add `provenance writing check --scope ... --format table|json --strict`.
4. Target only `requirement.statement` and `rule.statement`.
5. Emit warnings by default; `--strict` fails only on `violation` findings.
6. Add a baseline/suppression mechanism before recommending it for an existing graph.

Exit criteria:

- golden fixtures give identical JSON ordering and spans on Linux, macOS, and Windows;
- the mechanical oracle suite has zero false positives and zero false negatives;
- 1,000 short statements add less than 100 ms on a release build on CI hardware;
- the current corpus has a reviewed migration report rather than an unbounded warning
  flood.

### Stage 2 — every ingress, one analyzer

1. Run the analyzer during typed `sdk plan`; return findings beside semantic changes.
2. Run the same preflight in `sdk apply` and CLI create commands.
3. Run a full scan after staged import validation and before publication when enforcement
   is `error`.
4. Add the configured writing gate to CI only after the repository has accepted or
   suppressed its baseline.

Do not duplicate checks in TypeScript. The SDK may format Rust findings and map them to
source locations, but Rust remains the source of truth.

Exit criterion: CLI, SDK, import, and direct canonical-state checks produce the same
finding for the same artifact text and policy.

### Stage 3 — syntax review tier

1. Hand-label at least 200 real Requirement and Rule statements, including technical
   terms, abbreviations, passive constructions, noun clusters, `-ing` words, and boundary
   conditions.
2. Compare a small purpose-built rule tagger with `harper-core` through an internal parser
   seam.
3. Add only checks whose messages tell the author what evidence triggered the finding.
4. Keep them as `review` findings unless each check reaches at least 95% precision on the
   held-out corpus and has no known meaning-changing auto-fix.

Low recall is acceptable for an advisory authoring aid. False CI failures are not.

### Stage 4 — editor feedback

Expose the same JSON findings through an LSP or editor adapter. For typed specs, map
artifact address and field back to the string literal. Editor support should consume the
checker; it should not define a second checker.

### Stage 5 — evaluate broader fields

Consider `boundary.statement` and `resolution.position` only after the first two fields are
useful. Keep descriptions, rationales, context, source material, and collaboration prose
outside strict checking unless a new field-specific profile is justified.

## Go/no-go decision

**Go** on a two-week implementation spike for the original, mechanical
`provenance-simple-v1` linter.

**Do not go** on full ASD-STE100 conformance, compiler-type enforcement, automatic
rewriting, or LLM-backed CI decisions.

Continue beyond the spike only if:

1. the strict subset has zero false positives on the labelled current corpus;
2. authors can resolve findings without losing thresholds, exceptions, actors, or failure
   conditions;
3. custom-term setup remains bounded and reviewable;
4. the policy/version digest makes results reproducible;
5. the warning baseline is small enough to act on after a one-time migration;
6. any ASD-branded adapter has explicit rights and avoids certification claims.

If the exact subset proves too small to change writing quality, keep the grounded-writing
skill and do not add a checker merely to count words. If the subset catches real ambiguity
without creating review noise, it earns the deeper parser stage.
