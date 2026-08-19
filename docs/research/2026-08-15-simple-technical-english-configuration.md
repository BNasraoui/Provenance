# ASD-STE100 checks for Provenance statements

**Beads:** `provenance-0ss`, `provenance-l3m`, `provenance-l3m.12`<br>
**Date:** 2026-08-18<br>
**Status:** The shared checker and all planned graph-write paths are merged. The standards audit is complete.<br>
**Baseline:** `ca8f873`

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

## Current merged checks

The current checker implements strict findings for these rules:

- Rule 8.1 semicolons, with the quotation condition in Rule 8.6.
- A sound subset of Rule 4.2 contracted verb forms.
- The Rule 6.3 limit for descriptive sentences.
- The counting behavior in Rules 8.4 through 8.7 that Rule 6.3 needs.

These changes are merged through PR 104 at `ca8f873`.
The same Rust report now controls direct writes, typed SDK plan and apply, staged import, merged JSONL, and manual-change reports.
Manual-change findings remain informational.

The Rule 8.1 check has these exact properties:

- The input is only a string.
- Each semicolon gives one finding.
- Each finding has an exact UTF-8 byte span.
- Finding order follows source order.
- The check does not need a dictionary or parser.
- The check does not use a network, clock, locale, or language model.

The test suite tries every short string from a finite test alphabet.
It also tests Unicode text, many semicolons, stable order, and stable JSON.
This gives strong evidence for the implemented Rule 8.1 behavior.

This evidence applies only to the implemented Rule 8.1 behavior.
It does not apply to all ASD-STE100 rules.

## Standards audit of remaining descriptive rules

The audit reviewed all 53 Part 1 rules in the official Issue 9 PDF.
It then removed the procedural rules in Section 5 and the safety-instruction rules in Section 7 from this statement-field scope.
Section 5 applies only when a Rule statement is an instruction.
Section 7 applies only when a field is a designated warning or caution.

The classifications below apply to plain `requirement.statement` and `rule.statement` strings.
They do not apply to a future structured document model.
The locators are printed Issue 9 page numbers.

### Exact

Rule 6.6 is the only remaining data-free check that is ready for a strict implementation.
It limits a descriptive paragraph to six sentences (Rule 6.6, page 1-6-7).
The existing exact-or-indeterminate sentence model can support this check without a grammar guess.
The grounded implementation Bead is `provenance-l3m.11`.

The test boundary must be narrow:

- Check only paragraph spans that the input preserves.
- Emit one finding only when an exact sentence segmentation has more than six sentences.
- Cover the complete half-open UTF-8 paragraph span.
- Treat a heading as outside the paragraph.
- Do not count list fragments as paragraph sentences only because Rule 8.4 counts list text separately for sentence length.
- Return no strict Rule 6.6 finding for an abbreviation, decimal, identifier, quotation, or other boundary that the scanner cannot segment exactly.
- Test zero through six sentences, seven sentences, several paragraphs, Unicode spans, protected punctuation, stable order, and indeterminate boundaries.

Rule 4.3 has data-free structural parts for a vertical list (pages 1-4-4 through 1-4-6).
For example, its introduction and item punctuation have mechanical constraints.
It is not ready for this plain-string checker because the input does not identify a vertical list or its items.
Do not infer that structure from a regular expression.
If a later host supplies typed list nodes, audit the structural subset again.

### Partial

These rules have a possible sound subset, but the complete rule needs syntax, terminology, meaning, or document context:

- Rule 1.7, technical nouns used as verbs (page 1-1-11).
- Rule 1.9, length and quality of technical nouns (page 1-1-12).
- Rule 1.13, technical verbs used as nouns (pages 1-1-16 through 1-1-17).
- Rules 2.1 and 2.2, multi-word nouns and long official names (pages 1-2-1 through 1-2-4).
- Rules 3.2 and 3.4, permitted verb constructions (pages 1-3-2 through 1-3-3).
- Rule 3.6, active voice and its unknown-agent exception (pages 1-3-5 through 1-3-8).
- Rule 4.2, omitted sentence components and contraction forms outside the merged finite matcher (page 1-4-3).
- Rule 4.3, list selection, attachment, and the plain-string structural problem (pages 1-4-4 through 1-4-6).
- Rule 8.6, protected formulas, names, titles, labels, and other special spans that plain text does not identify (pages 1-8-5 through 1-8-8).

A partial checker must report only a proved subset.
For example, a parsed passive clause does not prove that its omitted agent is known.
A word that ends in `-ing` does not prove a Rule 3.5 violation.
Four adjacent words do not prove a Rule 2.1 noun cluster.

### Indeterminate

These rules need a decision about meaning, discourse, grammar, technical accuracy, or external project context:

- Rules 1.8, 1.10, and 1.11, approved project terms, audience-specific jargon, and names for the same item (pages 1-1-11 through 1-1-13).
- Rule 4.1, clear and accurate descriptive prose with one topic (pages 1-4-1 through 1-4-2).
- Rules 4.2, 4.4, and 4.5, omitted components, topic connections, and applicable articles (pages 1-4-3 through 1-4-9).
- Rules 6.1 and 6.2, information order and repeated key terms (pages 1-6-1 through 1-6-4).
- The qualitative part of Rule 6.3, ease of understanding (page 1-6-4).
- Rules 6.4 and 6.5, paragraph topic structure (pages 1-6-5 through 1-6-6).
- Rules 8.2 and 8.3, meaningful hyphens and permitted uses of parentheses (pages 1-8-2 through 1-8-4).
- Rules 9.1 and 9.4, meaning-preserving rewrites and consistent wording (pages 1-9-1 through 1-9-5 and 1-9-8 through 1-9-9).

The official STEMG tools page also says that a checker cannot determine whether the first sentence is the topic sentence.
These items can support human review, but they must not become strict checks from surface grammar guesses.

### Rights-blocked

These rules need official vocabulary, forms, meanings, categories, exceptions, or other extracted standard data:

- Rules 1.1 through 1.6, approved words, parts of speech, meanings, forms, and technical-noun categories (pages 1-1-1 through 1-1-10).
- Rule 1.12, technical-verb categories and approved alternatives (pages 1-1-13 through 1-1-16).
- Rule 1.14, official spelling and directive exceptions (page 1-1-17).
- Rules 3.1, 3.3, 3.5, and 3.7, approved verb forms, participles, `-ing` exceptions, and approved action verbs (pages 1-3-1 through 1-3-5 and page 1-3-9).
- Rules 9.2 and 9.3, approved meanings, parts of speech, and phrasal-verb exceptions (pages 1-9-5 through 1-9-8).

Some rights-blocked rules also need a parser or project termbase.
Written rights to the data would remove only the distribution block.
They would not make the linguistic analysis exact.

Section 9 also contains eight general recommendations on pages 1-9-9 through 1-9-13.
Issue 9 says that they are not STE rules.
They must not produce strict conformance findings.

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

Import, merge, and manual-edit checks are complete.
Imports and accepted merges use the changed-statement gate.
Manual edits produce an informational report.

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

Status: complete for Rules 4.2, 6.3, 8.1, and the related Rules 8.4 through 8.7 through PR 104.

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

Status: complete in PR 98.

### Step 4: Live author feedback

Map declaration addresses and spans back to source locations.
Call the Rust checker from an editor or language-server process.
Do not add a TypeScript copy of the rules.

Status: the shared `sdk check-statement` preflight is complete in PR 99.
An editor adapter remains optional later work.

### Step 5: Other graph paths

Check staged import before state replacement.
Check merged and hand-edited JSONL before acceptance.
Use the same checker and diagnostic format.

Status: complete in PR 101.

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
