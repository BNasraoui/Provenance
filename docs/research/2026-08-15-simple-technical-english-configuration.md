# ASD-STE100 checks for Provenance statements

**Beads:** `provenance-0ss`, `provenance-l3m`<br>
**Date:** 2026-08-15<br>
**Status:** The technical spike is complete. Work on the first checks has started.<br>
**Baseline:** `002d6bf`

## Writing note

This report uses short sentences and direct terms.
It follows ASD-STE100 Issue 9 where the available terms permit this.
It does not claim certified conformance.
It does not include the ASD-STE100 dictionary or reproduce the standard.

## Result

Provenance should check Requirement and Rule statements with ASD-STE100 Issue 9.
The check should be on by default for new and changed statements.
ASD-STE100 must be the source of the language rules.
Provenance must not define a substitute language.

The first release should have these limits:

- Check descriptive text in `requirement.statement` and `rule.statement`.
- Use one Rust checker for all write paths.
- Start with rules that give exact results.
- Reject an exact violation before a new graph write.
- Do not add custom language rules.
- Do not add an extension system.
- Do not claim full ASD-STE100 conformance.

This is a good product direction.
It can stop clear syntax faults early.
It can also give all tools the same result.
It cannot prove that natural-language text is clear, correct, or complete.

## What “deterministic” means

A deterministic check gives the same result for the same input.
The input includes the text, the ASD-STE100 issue, and the checker version.

There are three different claims:

1. The check always gives the same result.
2. Each strict finding is a real violation of the cited rule.
3. No findings means that the text conforms to all of ASD-STE100.

Provenance can meet the first claim.
It can meet the second claim for a small set of exact rules.
It cannot meet the third claim.

| Type of check | Result | Use |
| --- | --- | --- |
| Exact character or fixed-form check | Repeatable and provable | Write gate |
| Grammar parse | Repeatable but can be wrong | Review message |
| Meaning or clarity check | Needs human judgment | Human review |

A parser can make the same wrong choice each time.
Repeatable output does not prove correct analysis.

## First exact rule

The first implementation uses ASD-STE100 Issue 9 Rule 8.1.
It reports each semicolon in descriptive text.

This rule has useful exact properties:

- The input is only a string.
- Each semicolon gives one finding.
- Each finding has an exact UTF-8 byte span.
- Finding order follows source order.
- The check does not need a dictionary or parser.
- The check does not use a network, clock, locale, or language model.

The test suite tries every short string from a finite test alphabet.
It also tests Unicode text, many semicolons, stable order, and stable JSON.
This gives strong evidence for the implemented Rule 8.1 behavior.

This evidence applies only to Rule 8.1.
It does not apply to all ASD-STE100 rules.

## Other possible strict checks

Some other Issue 9 checks can become exact.
They need their full counting or matching rules before they can reject a write.

Possible checks include these items:

- recognized contractions under Rule 4.2
- permitted uses of `shall` and `should` under Rule 1.1 and Part 2
- the 25-word limit for descriptive sentences under Rule 6.3
- word-count behavior under Rules 8.4 to 8.7

Do not use a simple word counter for Rule 6.3.
The standard defines how some hyphenated terms, numbers, identifiers, quotations, and other spans count.
An unclear Rule 8.6 span must not cause a strict finding.

Do not report several sentences as a violation by itself.
Rule 6.6 permits as many as six sentences in a paragraph.

Do not make these checks strict from simple expressions:

- passive voice
- `-ing` forms
- noun clusters
- articles
- imperatives
- phrasal verbs

These checks need a grammar parse or a choice about meaning.
They should start as review messages.

## Compile-time checks

True language type checks are not the main solution.
Provenance text can come from TypeScript, the CLI, imports, JSONL files, or hand edits.
A TypeScript type can see only some string literals.
A Rust procedural macro cannot see text that is not a Rust literal.

The viable first check is a required build step:

```text
typed source -> SDK plan -> Rust ASD-STE100 checker -> result or build failure
                         -> SDK apply -> same check -> graph write
```

The SDK `plan` process is the earliest shared Rust step for typed declarations.
A build can run it before graph generation.
This gives compile-time behavior in the build process.
It is not a proof in the TypeScript type system.

The synchronous TypeScript `.statement()` builder cannot call Rust by itself.
Do not copy the ASD-STE100 rules into TypeScript.
That would create two checkers that can disagree.

For feedback while a person or agent types, add an editor or language-server adapter later.
That adapter must call the same Rust checker.
This work belongs to `provenance-l3m.5`.

## Write-time checks

The graph write is the final control.
The checker must run before canonical graph state changes.

Provenance has several write paths:

1. Direct `StateStore` Requirement and Rule creation.
2. Typed SDK `plan` and `apply` reconciliation.
3. Staged import and state replacement.
4. Git JSONL merge and manual file changes.

The first direct gate is complete.
It rejects Rule 8.1 findings before Requirement or Rule creation.
Its tests compare every canonical state file before and after rejection.
The tests also include Rule relationship edges.

Typed reconciliation must use a narrower rule.
It must check these resources:

- a created Requirement or Rule
- an updated Requirement or Rule when `statement` changed

It must not check every declaration in the submitted typed document.
Typed documents send unchanged statements again.
An old statement must not block an unrelated field change.

The typed check must run after in-memory reconciliation.
It must run before the first Apply write.
Plan and Apply must use the same ordered diagnostic list.

Import, merge, and manual-edit checks are separate work in `provenance-l3m.7`.

## Diagnostic data

Each machine-readable diagnostic should include these fields:

- declaration address or graph record identity
- resource kind
- `field: statement`
- `standard: ASD-STE100`
- `issue: 9`
- cited ASD rule
- finding type
- exact UTF-8 byte span
- direct correction message

The checker must sort findings by stable source identity and source span.
The same input must give byte-for-byte equal JSON on each supported platform.

## Configuration

Do not build a general writing-rule configuration system for the first release.
Do not build a `provenance-writing` rules engine.

Use a fixed `provenance-ste100` Rust crate.
Its first public interface is:

```rust
pub fn check_descriptive(text: &str) -> Report;
```

The standard and issue are part of the report.
The checker contains only ASD-STE100 behavior.

A later repository setting can control enforcement or a migration baseline.
It must not change the meaning of ASD-STE100 rules.
An optional future extension can add more checks.
It must supplement ASD-STE100 and must not replace it.

## Existing text and migration

Do not make old text block all new work.
Apply the default write gate to created statements and changed statements.
Leave an unchanged statement alone during an unrelated update.

A repository-wide check can report old findings.
The project can then correct or record a baseline for them.
The open graph question `question_ste_graph_failure_policy` must decide when a repository scan changes from warning to failure.
No implementation agent should answer that question for the user.

## Rights and product claims

The official sources do not give Provenance a clear public right to distribute the ASD-STE100 dictionary or extracted rule data.
Free download access is not a general redistribution license.

The relevant official sources are:

- [ASD-STE100 Issue 9](https://asd-web-be-prod.azurewebsites.net/media/wunhmi5y/asd-ste100-issue-9.pdf)
- [STEMG software guidance](https://www.asd-ste100.org/STEsoftware.html)
- [STEMG FAQ](https://www.asd-ste100.org/STE_faq.html)
- [STEMG download page](https://www.asd-ste100.org/STE_downloads.html#article02-2l)

Until the project has sufficient written rights:

- Write original code for individually cited exact rules.
- Do not ship the PDF.
- Do not ship extracted word lists, tables, examples, or rule text.
- Link users to the official download page.
- Do not use ASD or STEMG logos.
- Do not claim ASD approval, certification, or full conformance.
- Keep built-in official vocabulary checks blocked.

The human choice remains open:

- Option A: ship original exact checks without copied standard data.
- Option B: wait for written rights or a licensed integration before more ASD-STE100 work.

The graph question is `question_ste_distribution_rights`.
The proposed position is `prop_res_ste_data_free_until_licensed`.
It is not an approved Resolution.

## Implementation path

### Step 1: Pure checker

Use the separate `provenance-ste100` crate.
Keep it free of graph identity and enforcement policy.
Add one cited graph Rule for each implemented ASD behavior.
Bind production code with `#[rule]`.
Bind meaningful tests with `#[verifies]`.

Status: complete for Rule 8.1 in PR 96.

### Step 2: Direct graph writes

Call the checker below the CLI handlers and before direct state mutation.
Keep the typed checker report in the error.
Return structured data to the caller.
Prove that rejection leaves graph records and edges unchanged.

Status: complete in PR 97.

### Step 3: Typed plan and apply

Reconcile in memory.
Check only created or statement-changed Requirement and Rule resources.
Make Plan and Apply use one diagnostic result.
Reject Apply before records, edges, or bindings change.

Status: in progress under `provenance-l3m.4`.

### Step 4: Live author feedback

Map declaration addresses and spans back to source locations.
Call the Rust checker from an editor or language-server process.
Do not add a TypeScript copy of the rules.

Status: planned under `provenance-l3m.5`.

### Step 5: Other graph paths

Check staged import before state replacement.
Check merged and hand-edited JSONL before acceptance.
Use the same checker and diagnostic format.

Status: planned under `provenance-l3m.7`.

## Test plan

Each exact rule needs tests for its own input domain.
Use exhaustion when the finite domain is small.
Use properties for larger mechanical domains.
Use examples for integration and graph-state behavior.

Each write path needs these tests:

- valid text keeps current behavior
- exact violation reports all required fields
- Unicode spans are correct
- many findings have stable order
- rejection leaves all affected state unchanged
- Plan and Apply report the same findings
- unchanged old text does not block an unrelated update
- Linux, macOS, and Windows give the same result

Every production behavior must have a graph Rule before the code lands.

## Decision

Continue the implementation.

Use ASD-STE100 Issue 9 directly.
Keep exact checks small and well tested.
Make the graph write gate authoritative.
Use SDK plan as the first build-time check.
Add live editor feedback after the shared path works.
Do not create a second language standard inside Provenance.
