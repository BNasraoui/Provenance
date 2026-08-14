# Simplified Technical English configuration research

**Bead:** `provenance-0ss`<br>
**Date:** 2026-08-15<br>
**Status:** Research is complete. ASD-STE100 is the selected standard.<br>
**Baseline:** `ef8269843b9c9d1f2246a4cc7cd54825e72a4f37`

## Conformance note

This report follows ASD-STE100 Issue 9 writing rules where possible.
It uses short sentences, active voice, and consistent terms.
It does not include the official ASD-STE100 dictionary.
Some software terms are project-approved technical terms for this report.
Thus, this report does not claim certified ASD-STE100 conformance.

## Result

Provenance can check Requirement and Rule syntax against ASD-STE100 Issue 9.
The check can give repeatable results for mechanical rules.
It can also give useful review messages for syntax checks.
A person must review checks about meaning and clarity.

ASD-STE100 must define the rules and vocabulary.
Provenance must not define a replacement writing standard.
Apply the check first to Requirement statements and Rule statements.

The strict tier must contain only proven violations.
A parser can produce review findings after tests show good precision.
Parser findings must not fail CI during the first release.

Do not use TypeScript or Rust type checks as the main control.
Use `provenance writing check --strict` as the build gate.
Run this command before a build and in CI.
Also run the same analyzer from SDK `plan` and `apply` operations.
Authoring tools must use this analyzer to show findings while a person or agent writes.
Graph generation must run the analyzer before it changes graph state.

Do not include the ASD-STE100 dictionary, examples, or rule text without permission and legal review.
Issue 9 states that ASD owns the document and restricts its reproduction.
ASD also states that software cannot check all rules.

## Scope of the research

This research answers three questions.

1. How repeatable can the checks be?
2. Can Provenance check the text at compile time?
3. What is a safe implementation path?

This research starts with canonical Requirement statements and Rule statements.
It does not include source quotations, names, citations, messages, fog, rationales, or descriptions.
These fields preserve evidence or explain context and choices.

## Meaning of deterministic verification

The term deterministic verification can describe three different promises.

### Promise 1: Repeatable execution

The same text, standard issue, terms, and analyzer version give the same findings.
Provenance can make this promise.

### Promise 2: Correct violations

Each strict finding identifies a real violation of the selected ASD-STE100 rule.
Provenance can make this promise for a small mechanical tier.

### Promise 3: Complete conformance

No findings mean that the text follows all vocabulary, grammar, meaning, and clarity rules.
Provenance cannot make this promise for natural-language text.

A deterministic parser can make the same incorrect analysis each time.
Repeatable execution does not make each finding correct.

## External evidence

ASD published ASD-STE100 Issue 9 in January 2025.
Issue 9 contains 53 writing rules in nine sections.
It also contains a controlled dictionary.

The rules separate procedural text from descriptive text.
Provenance statements usually describe obligations.
They do not usually give maintenance work steps.
Thus, the 25-word descriptive limit is the better limit.

The following sources set a clear limit on automated checks:

- The [Issue 9 standard](https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf) contains lexical, grammar, structure, and meaning rules.
- Its copyright notice restricts reproduction of the standard.
- The STEMG [software guidance](https://www.asd-ste100.org/software.html) states that no checker can test all rules.
- The guidance tells authors to decide if a finding applies in its context.
- The [Boeing checker description](https://www.boeing.com/company/simplified-english-checker) describes more than 400 syntax rules and a full parser.
- The Boeing checker does only limited word-sense checks.
- A [controlled-language parser study](https://aclanthology.org/2003.eamt-1.15.pdf) describes the same split between mechanical and meaning checks.

This evidence supports a linter with different finding types.
It does not support an automated compliance certificate.

## Determinism levels

### Level 1: Mechanical violations

These checks do not need a probabilistic model.
A fixed tokenizer, standard issue, and analyzer version can give exact and stable results.

The strict tier can check these items:

- sentence word limits for a selected text mode
- paragraph sentence limits when the input keeps paragraph structure
- forbidden punctuation, such as semicolons
- a closed list of contractions
- a closed list of forbidden modal words
- approved spelling for a selected dialect
- configured exceptions to approved spelling
- word-count rules for hyphens and parentheses
- word-count rules for numbers and identifiers
- word-count rules for quotations and list colons

The analyzer must apply the ASD-STE100 word-count rules.
Without this definition, the 25-word limit is not deterministic.

Tests must show that its result agrees with the standard.

### Level 2: Terms from committed configuration

Vocabulary checks are deterministic when all permitted inputs are versioned.

The configuration can contain these term groups:

- the official general vocabulary for the selected issue
- approved technical nouns and their plural forms
- approved technical verbs and their permitted forms
- product names, abbreviations, identifiers, and protected literals
- one preferred term for a concept

The analyzer can prove that a token is absent from the configured vocabulary.
It cannot prove that an unknown token is not a valid technical noun.

An unknown term must normally produce a review finding.
A repository can select an error only when it uses a closed term registry.

The human decision is part of the configuration.
The analyzer checks that decision consistently.
It does not make the decision.

### Level 3: Repeatable syntax review

A fixed rule-based parser can find these possible problems:

- an approved word that has the wrong part of speech
- a noun cluster that exceeds the configured limit
- a tense that the selected issue does not permit
- a complex auxiliary verb
- possible passive voice
- an `-ing` form in a position that the selected issue does not permit
- a missing article
- a possible instruction in descriptive text
- more than one possible command
- a possible phrasal verb
- a condition that occurs after a command

These checks need correct tokens, parts of speech, and sentence structure.
Technical terms and short Requirement statements can cause ambiguous results.

These checks must have the `review` disposition first.
Do not give them a numerical confidence value.
Do not use them as a CI gate before tests show strict-tier precision.

Use a deterministic rule-based parser for this level.
Do not use an LLM as the source of a strict finding.

A fixed local model can give repeatable results.
However, it can still make classification errors.
It also adds model file, platform, and version controls.

### Level 4: Human review of meaning

The analyzer cannot make these decisions from the text alone:

- An approved word has the intended approved meaning.
- A term is easy to understand.
- Two different terms identify the same item.
- A sentence contains one clear idea.
- A new sentence keeps the meaning of the original sentence.
- Information occurs in a gradual order.
- All sentences in a paragraph have one topic.
- The text needs a list.
- A safety statement identifies the correct risk or result.
- Two actions occur at the same time.
- A sentence is factually correct.
- A sentence implements the intended decision.

An LLM can suggest review items or new text.
Its output must not affect the strict result.

Do not store a `compliant` Boolean value on an artifact.
Derive findings from the current text, standard issue, terms, and analyzer version.

## Compile-time verification

### TypeScript types

TypeScript template literal types can check small patterns in string literals.
For example, a type can reject a literal that contains a semicolon.

This method checks only literals that keep their literal type.
It does not check computed strings, CLI input, JSONL, or imported records.

Recursive token types also increase compile cost and reduce error quality.
The [TypeScript documentation](https://www.typescriptlang.org/docs/handbook/2/template-literal-types) recommends generated code for large string unions.
The [TypeScript 4.1 notes](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-1.html) warn about complex recursive types.

Do not use TypeScript types as the main analyzer.

### TypeScript editor plugin

A TypeScript language-service plugin can show findings in an editor.
It does not make `tsc` a repository policy gate.
The [plugin documentation](https://www.typescriptlang.org/tsconfig/plugins.html) describes it as an editor extension point.

Use an editor plugin only as a later adapter.

### TypeScript transformer or ESLint rule

A transformer or ESLint rule can inspect TypeScript source during a build.
It checks only the TypeScript ingress.
A user can bypass it through another graph ingress.

Use it only as an optional adapter.

### Rust procedural macro

A Rust procedural macro can check a Rust string literal.
Provenance does not declare its prose in Rust literals.
Thus, this method does not apply.

### Provenance build gate

`provenance writing check --strict` can check all selected canonical fields.
It can check hand-edited records and imported JSONL records.
It can also check records that do not come from TypeScript.

This command is the recommended gate.
The SDK can call the same Rust analyzer before reconciliation.

The useful meaning of compile-time is a required build gate.
It is not a language type-system proof.

## Current Provenance design

### Existing parts

- The manifest contains scopes, path prefixes, and disposition actors.
- See `crates/provenance-core/src/model/manifest.rs:19-43`.
- The manifest does not contain repository policy configuration.
- Requirement creation sends an unrestricted string to `StateStore`.
- See `crates/provenance-cli/src/handlers/requirements.rs:15-41`.
- Rule creation also sends an unrestricted string to `StateStore`.
- See `crates/provenance-cli/src/handlers/rules.rs:78-106`.
- Typed-spec checks cover document identity and scope existence.
- See `crates/provenance-store/src/state_store/typed_specs.rs:215-239`.
- Typed-spec checks do not check statement text.
- `provenance check` checks graph structure and graph references.
- See `crates/provenance-cli/src/handlers/check.rs:14-64`.
- Exact graph exports contain the selected Scope and canonical graph records.
- See `crates/provenance-store/src/graph_reference/projection.rs:29-89`.
- These exports do not contain repository validation policy.
- The grounded-writing skill gives semantic author guidance.
- See `.agents/skills/provenance-grounded-writing/SKILL.md:43-78`.

### Design effects

Use repository configuration for the writing policy.
Do not add writing state to a Requirement or Rule.
Stored state would become incorrect after an analyzer update.

Add a separate lint command first.
Do not make `provenance check` reject all existing statements without a migration step.

Put the analyzer below every graph ingress.
Do not keep a separate rule copy in TypeScript.

Include the analyzer version, standard issue, and policy digest in each report.
An exact graph export does not need this policy in the first release.
Add it later only if exports must reproduce findings without repository configuration.

The ASD-STE100 check supports artifact semantics.
It does not replace the Swap, Name, Evidence, and Climb tests.
Those tests check the decision, not only its grammar.

## Proposed product

### Configuration file

Use a committed `.provenance/writing.toml` file.
Keep the first public interface small.
Select the standard issue, text type, target fields, technical terms, and enforcement levels.
Do not provide one switch for each ASD rule.

```toml
schema_version = 1

[defaults]
standard = "ASD-STE100"
issue = 9
text_type = "descriptive"
targets = ["requirement.statement", "rule.statement"]
technical_term_files = [".provenance/technical-terms.toml"]

[defaults.enforcement]
violation = "warn"
review = "warn"

[scopes.default.enforcement]
violation = "error"
review = "warn"
```

The standard name and issue identify the rule source.
The analyzer version identifies the implementation.
A change to the issue or analyzer changes the policy digest.

Use a term file for project term decisions.

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

Use official dictionary data only with the necessary permission.
If Provenance cannot distribute that data, use a licensed checker or user-supplied data.
Do not create a substitute general vocabulary.
Do not add custom writing rules in the first implementation.
Project technical terms remain necessary because ASD-STE100 permits approved technical terms.

### Finding format

Each result must contain these data:

- a stable check ID
- the exact graph location
- the source span
- the standard issue
- the analyzer version
- the policy digest
- the finding disposition
- a direct message

Use `violation` only when the analyzer can prove the problem.
Use `review` when a person must decide.

```json
{
  "standard": "ASD-STE100",
  "issue": 9,
  "analyzer": "provenance-writing@0.1.0",
  "policy_digest": "sha256:...",
  "artifact": { "scope": "default", "type": "rule", "id": "rule_example" },
  "field": "statement",
  "span": { "start": 37, "end": 38 },
  "check": "PSE-PUNCT-001",
  "disposition": "violation",
  "message": "Use two sentences, or move context to the description."
}
```

Sort findings by scope, artifact type, ID, field, span, and check ID.
The analyzer must not use a network, clock, ambient locale, or LLM.

Golden tests must fix these behaviors:

- Unicode normalization
- sentence boundaries
- token counts
- abbreviation rules
- hyphen rules
- source spans
- result order

The tool can suggest an edit.
It must not change canonical state automatically.
A small edit can change the strength or condition of an obligation.

### Rust module

Add a `provenance-writing` Rust crate.
Give the crate one pure public interface.

```rust
pub fn analyze(policy: &CompiledWritingPolicy, site: TextSite<'_>) -> Vec<WritingFinding>;
```

`TextSite` contains the artifact identity, field, text, and text mode.
The crate hides token rules, term checks, rule dispatch, and parser adapters.
Tests must use the same interface as all callers.

Adapters create `TextSite` values from canonical records and typed declarations.

```text
CLI create -----+
SDK plan/apply -+--> TextSite --> provenance-writing --> stable findings
repository scan +                         ^
                                  standard + technical terms
```

The first version does not need a parser dependency.

For the syntax level, [`harper-core`](https://docs.rs/harper-core/latest/harper_core/) is the best current Rust candidate.
It runs in the process and gives access to tokens and verb data.
It also accepts custom dictionaries and has an Apache-2.0 license.

These features do not prove STE accuracy.
Compare it with a small project parser and labeled Requirement and Rule statements.

[Vale](https://vale.sh/docs/styles) is useful for a prototype and comparison.
It has configurable style rules and JSON output.
However, Vale expects documents and markup.

Provenance already knows each graph record and field.
An external process adds field serialization and span mapping.
It does not solve meaning checks.

## Current statement test

A read-only script checked the current Requirement and Rule statements.
The test used the pinned baseline.
It checked 53 Requirement statements and 57 Rule statements.

The script used a simple ASCII word tokenizer.
It checked semicolons, `shall`, sentences over 25 words, and multiple sentences.

| Candidate | Requirements | Rules | Total |
| --- | ---: | ---: | ---: |
| Semicolon | 6 | 12 | 18 |
| `shall` | 4 | 0 | 4 |
| More than 25 words | 9 | 20 | 29 |
| More than one sentence | 0 | 3 | 3 |
| One or more findings | 15 | 28 | 43 of 110, or 39.1 percent |

This test did not measure STE conformance.
It measured migration cost and product fit.

The test gives these results:

- Strict default enforcement would reject much of the current graph.
- Rule statements have more findings than Requirement statements.
- Current guidance uses semicolons for some boundary cases.
- The ASD-STE100 check can require changes to the grounded-writing skill.
- Reviewers must make sure that a sentence split does not change atomic meaning.
- The first release needs warning-only enforcement.
- The tokenizer needs fixture tests before the word limit becomes a gate.

A simple contraction expression found 14 possible contractions.
All 14 results were possessive forms with apostrophe `s`.
A closed contraction list removed these false results.

This result shows an important rule.
A repeatable expression is not sufficient for a strict check.
The check must also have an unambiguous input domain.

## Implementation path

### Stage 0: Define the product promise

Use ASD-STE100 Issue 9 as the only rule source.
Obtain permission, use a licensed checker, or require user-supplied rule data.
Do not create a Provenance replacement standard.
Do not add custom rule extensions in the first implementation.

Review product words and redistribution rights.
Do not claim that an automated result certifies full conformance.
The evidence does not support that claim.

### Stage 1: Make the mechanical linter

1. Load `.provenance/writing.toml` with a closed schema and clear errors.
2. Add sentence boundaries, token counts, punctuation, modal word, contraction, and configured term checks.
3. Add `provenance writing check --scope ... --format table|json --strict`.
4. Check only `requirement.statement` and `rule.statement`.
5. Give warnings by default.
6. In strict mode, fail only for `violation` findings.
7. Add a baseline or suppression method for an existing graph.

The stage has these exit criteria:

- Golden fixtures give identical JSON order and spans on Linux, macOS, and Windows.
- The mechanical test set has no false positive or false negative result.
- A test of 1,000 short statements takes less than 100 milliseconds in a release build on CI hardware.
- The current Requirement and Rule statements have a reviewed migration report.
- The command does not produce an uncontrolled warning list.

### Stage 2: Use one analyzer at every ingress

1. Run the analyzer during typed SDK `plan` operations.
2. Return findings with semantic changes.
3. Run the same preflight during SDK `apply` operations.
4. Run the same preflight during CLI create operations.
5. Scan all staged imports before publication when enforcement is `error`.
6. Add the CI gate only after the repository accepts or suppresses the baseline.

Do not copy checks into TypeScript.
The SDK can format Rust findings and map them to source locations.
Rust remains the source of truth.

For the exit test, use the same text and policy at every ingress.
Each ingress must give the same finding.

### Stage 3: Add syntax review

1. Label at least 200 real Requirement and Rule statements.
2. Include technical terms, abbreviations, passive forms, noun clusters, `-ing` words, and boundary conditions.
3. Compare a small project parser with `harper-core`.
4. Use an internal parser interface for the comparison.
5. Add only checks that explain the evidence for the finding.
6. Keep each new check at the `review` level first.

A check can move to the strict tier only after it reaches 95 percent precision.
Use a separate test set for this measurement.
The check must not have a known meaning-changing automatic fix.

Low recall is acceptable for author guidance.
False CI failures are not acceptable.

### Stage 4: Add editor messages

Send the same JSON findings to an LSP or editor adapter.
For typed specs, map the artifact address and field to the source string.

The editor must use the shared analyzer.
It must not define a second analyzer.

### Stage 5: Review more fields

Review `boundary.statement` and `resolution.position` after the first two fields give value.

Keep descriptions, rationales, context, source material, and messages outside strict checks.
Add a field only when the selected ASD-STE100 text type applies to it.

## Decision

Start a two-week implementation spike for deterministic ASD-STE100 Issue 9 syntax checks.

Do not start these products:

- full ASD-STE100 conformance checks
- a second checker in a language type system
- automatic text changes
- LLM-based CI decisions

Continue after the spike only when all these conditions are true:

1. The strict subset has no false positives on the labeled current statements.
2. Authors can correct findings without loss of thresholds, exceptions, actors, or failure conditions.
3. Custom term setup stays small and easy to review.
4. The policy digest and version make results repeatable.
5. Authors can resolve the warning baseline after one migration.
6. An ASD-branded adapter has explicit rights and no certification claim.

Do not add the analyzer if the exact subset only counts words.
Keep the grounded-writing skill in that case.

Continue to the parser stage if the exact subset finds real ambiguity without review noise.
