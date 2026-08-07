# Wiki homepage / scope index research

**Bead:** `provenance-c9m.12`<br>
**Date:** 2026-08-07<br>
**Status:** research complete; direction not ratified<br>
**Baseline:** `origin/main` at `5718016`, after PRs #42 and #43

This document is design evidence, not a homepage specification. It compares three
deliberately incompatible first-screen priorities and recommends one for a later human
disposition. No option is a decision until that disposition happens.

## Research question and boundary

What should a reader see first on a scope with 228 requirements, 165 resolutions, 576
rules, 7 sources, and enough gaps to overwhelm a literal list?

The homepage should route readers into the corpus. It should not duplicate the complete
domain or search indexes, become a defect backlog, or hide integrity problems. This work
does not change the structured data contract or production renderer.

## Evidence collected

### Current implementation

The current page model is assembled in
`crates/provenance-cli/src/wiki/assemble/pages/index.rs`:

- `11-30`: every requirement without a parent edge becomes a root row;
- `31-68`: orphan rules, resolutions, and sources become complete link lists;
- `69-80`: totals and every index-eligible gap are attached to the page.

`crates/provenance-cli/src/wiki/render/pages/index.rs` then establishes this hierarchy:

1. root requirements (`13-49`);
2. orphaned records (`50-65`);
3. one margin card per gap (`67-71`);
4. corpus totals after all gap cards (`72-101`).

The snapshot at
`crates/provenance-cli/src/wiki/snapshots/provenance__wiki__render__tests__snapshot_scope_index_page.snap`
faithfully shows that structure, but its hand-built title (`Provenance atlas — default`)
does not match the assembled title, which is the raw scope string at
`assemble/pages/index.rs:69-72`.

PR #42 added two real discovery destinations. `/domains/` groups requirements and rules,
including inherited and multi-domain membership
(`crates/provenance-cli/src/wiki/assemble/discovery.rs:78-174`). `/search/` indexes all
requirements and rules into the page and ranks them offline
(`crates/provenance-cli/src/wiki/render/pages/search.rs:21-48` and
`crates/provenance-cli/src/wiki/theme/search.js:8-44`). Resolutions and sources are in
neither discovery index. These destinations are currently small global-nav links, rather
than the homepage's primary calls to action.

### Representative corpus and browser exercise

A synthetic repository matched the issue scale exactly:

| Record | Count | Fixture shape |
|---|---:|---|
| Requirements | 228 | 12 roots, 216 refinements, 12 domains |
| Resolutions | 165 | each connected to a requirement |
| Rules | 576 | each produced by a resolution |
| Sources | 7 | all referenced |
| Homepage gaps | 42 | requirements without source references |

Statements were intentionally realistic sentence length. Requirements were distributed
across 12 domains; the first 42 omitted source references; all other records were linked
to avoid manufacturing unrelated orphan lists. The fixture was imported through the real
CLI and served with `provenance wiki serve`, so assembly, rendering, CSS, routes, and the
HTTP server were all exercised.

The exact fixture generator is retained beside the screenshots as
[`generate-scale-fixture.py`](assets/2026-08-07-wiki-homepage-scope-index/generate-scale-fixture.py).
From the repository root, reproduce the corpus and live site with:

```sh
mkdir -p target/homepage-research
python3 docs/research/assets/2026-08-07-wiki-homepage-scope-index/generate-scale-fixture.py \
  target/homepage-research/state.json
cargo run -q -p provenance-cli -- init \
  --path target/homepage-research/repo --scope homepage-scale --path-prefix .
cargo run -q -p provenance-cli -- import \
  --repo target/homepage-research/repo --scope homepage-scale \
  --input target/homepage-research/state.json --format json
cargo run -q -p provenance-cli -- wiki serve \
  --repo target/homepage-research/repo --scope homepage-scale --port 5185
```

With the server running, the byte/item measurement was:

```sh
python3 - <<'PY'
import re, urllib.request
for path in "/", "/domains/", "/search/":
    body = urllib.request.urlopen("http://127.0.0.1:5185" + path).read()
    print(path, len(body), len(re.findall(rb"<li(?:\s|>)", body)),
          len(re.findall(b"citation gap", body)))
PY
```

Observed response sizes and repeated items were:

| Route | HTML bytes | Repeated corpus items |
|---|---:|---:|
| `/` | 16,555 | 12 root rows, 42 gap cards |
| `/domains/` | 71,927 | 804 requirement/rule list items |
| `/search/` | 295,635 | 804 search entries |

The large destination pages demonstrate why the homepage should link to, not inline,
their complete indexes. Search's initial document repeats title and statement data in
attributes and visible text. Domain inheritance can repeat a record across groups.

Browser evidence:

- [Desktop, Firefox headless, 1440×900](assets/2026-08-07-wiki-homepage-scope-index/current-desktop-1440x900.png)
- [Mobile, Firefox headless, 390×844](assets/2026-08-07-wiki-homepage-scope-index/current-mobile-390x844.png)

```sh
firefox --headless --window-size 1440,900 \
  --screenshot "$PWD/docs/research/assets/2026-08-07-wiki-homepage-scope-index/current-desktop-1440x900.png" \
  http://127.0.0.1:5185/
firefox --headless --window-size 390,844 \
  --screenshot "$PWD/docs/research/assets/2026-08-07-wiki-homepage-scope-index/current-mobile-390x844.png" \
  http://127.0.0.1:5185/
```

At 1440×900, nine gap cards occupy the visible right rail while the useful record totals
remain below it. At 390×844, the margin stacks after the main column, so neither gaps nor
totals are visible before the reader traverses the root list. The scope label wraps in the
header, and the first viewport contains only root rows. The responsive CSS explains this:
the 220px margin stacks below the main column at 860px
(`crates/provenance-cli/src/wiki/theme/provenance-wiki.css:340-345,560-570`).

### Coverage limits in current tests

The renderer snapshot has one root and one gap. Discovery assembly tests top out at three
requirements and two rules; the search DOM test uses three entries; the CLI wiki fixture
uses two requirements and one resolution, rule, and source. These tests verify semantics,
but none protects the homepage's information hierarchy at corpus scale.

## Reader jobs, ordered by landing-page urgency

1. **Find a known thing.** Reach a requirement or rule from words or an identifier.
2. **Orient in an unfamiliar scope.** Learn what areas exist and choose a promising area.
3. **Understand the corpus shape.** See scale and the top-level requirement structure
   without reading an inventory.
4. **Judge trustworthiness.** Learn whether gaps are exceptional or systemic and reach
   actionable detail.
5. **Trace an argument.** Follow requirement → decision → rule → source on detail pages.

Jobs 1-3 decide whether a new reader can enter the corpus. Job 4 must stay visible but
must not consume the route-finding surface. Job 5 belongs to record pages once a reader
has selected an entry point.

## Current failure modes

- **Diagnostics outrank navigation.** Every gap receives more visual enclosure than a
  domain or search entry point; totals come after all gaps.
- **Responsive order reverses meaning.** On mobile, root inventory comes first and scope
  health arrives after it, rather than becoming a compact summary near the title.
- **Root rows are an accidental taxonomy.** They reflect graph topology, not necessarily
  recognizable reader topics. Long requirement statements make poor navigation labels.
- **Counts lack a clear promise.** Root counts combine direct and derived relationships
  differently, while no link explains or expands them.
- **Discovery is present but visually subordinate.** “Domains” and “Search” are small nav
  links despite satisfying the two most urgent reader jobs.
- **Discovery has deliberate blind spots.** Resolutions and sources can only be found by
  graph traversal or orphan lists. Homepage copy must not imply universal search.
- **Naive reuse does not scale.** Inlining domain groups would add 804 list items;
  inlining search would add a 296KB corpus payload before interaction.
- **Scale regressions are untested.** Existing snapshots cannot fail when a new gap class
  turns the homepage into another multi-thousand-pixel rail.

## Candidate measurable constraints for disposition

These are proposed cross-direction acceptance criteria, not ratified requirements. The
human disposition should accept, modify, or reject them along with the direction. The
selectors below are suggested test contracts so each statement has an observable result.

1. At 1440×900, scope identity, search, domains, record totals, and a corpus-health summary
   are visible without scrolling: the rectangles for `h1`, either `[role=search]` or
   `a[href='/search/']`, `a[href='/domains/']`, `[data-corpus-counts]`, and
   `[data-corpus-health]` end at or above `window.innerHeight`.
2. At 390×844, search and one browse action appear before any repeated record list; no
   horizontal overflow occurs. Assert the search and domains-link rectangles precede
   `[data-homepage-list]`, and `document.documentElement.scrollWidth <= window.innerWidth`.
3. The homepage renders at most 20 repeated record/topic rows and at most 5 health
   categories: count `[data-homepage-row]` and `[data-health-category]` after assembling
   both the small fixture and the 228/165/576/7 fixture. Complete results live on
   dedicated routes.
4. No individual gap cards appear on the homepage. Assert zero `.citation.gap` elements,
   at most 5 `[data-health-category]` summaries, and one health-detail link. At assembly
   test level, compare the sorted detail-route gap texts with the sorted input
   `GapNotice.detail` values to prove exact preservation.
5. Homepage HTML must remain bounded by roots/domains/category summaries, not by all 976
   records (979 generated pages including the three singleton routes). Use 50KB
   uncompressed at this fixture as a proposed regression ceiling.
6. A keyboard user reaches search, domain browsing, and health detail in the first six
   Tab presses from the document body. Record `document.activeElement` after each press;
   the three controls must all appear by press six. Search has a persistent visible label
   and submit behavior without relying on placeholder text.
7. The homepage makes search coverage explicit (“requirements and rules”) until the
   index includes resolutions and sources.
8. A scale test must assemble at least 228/165/576/7 with 42 gaps and assert bounded
   homepage repetition. Desktop and 390px browser checks must assert ordering and no
   horizontal overflow.

## Fork assessment

This choice meets the repository's fork-tournament criteria:

- **Mutually exclusive:** only one organizing object can own the first viewport: query,
  domain map, or corpus health/structure.
- **Expensive to reverse:** the choice shapes assembler projections, route contracts,
  responsive order, accessibility semantics, copy, and scale snapshots that subsequent
  work will build upon.
- **Preference unknowable without artifacts:** all three satisfy the measurable floor;
  choosing among speed, orientation, and assurance is a product-value reaction.

The following are therefore disposal-ready stances, not ingredients to average together.
Each has a manifesto, a concrete low-cost wireframe, a quality bar, and an exit criterion.

## Direction A — Search launchpad

### Design-principles manifesto

The homepage is a door, not a report. Optimize the first ten seconds for forward motion.
Make the strongest cross-cutting tool physically dominant, disclose its coverage honestly,
and keep every summary compact enough that it cannot compete with the next action.

```text
┌ Provenance / payments ───────────── Domains  Health ┐
│ PAYMENTS ATLAS                                      │
│ Find a requirement or rule                          │
│ [ Search titles, statements, or IDs…            ] ↵ │
│ Searches 228 requirements + 576 rules               │
│                                                     │
│ Browse instead                                      │
│ [Identity 42] [Billing 76] [Privacy 31] [All 12 →]  │
│                                                     │
│ 7 sources · 228 requirements · 165 decisions · 576  │
│ Corpus health: 42 missing-source links       View → │
│ Top-level requirements (12)                  View → │
└─────────────────────────────────────────────────────┘
```

**Quality bar:** a reader with one remembered term reaches a result in one interaction;
an unfamiliar reader sees domains without mistaking search for universal coverage.

**Tradeoffs:** best for known-item retrieval and compact on mobile; weaker at communicating
the structure of a new scope. It depends on improving search affordance and eventually
search coverage, but does not require embedding the 804-entry search DOM on `/`.

**Exit criterion:** stop when search, domain escape hatch, totals, and health fit in the
first desktop and mobile viewports; do not add root rows to “use the space.”

## Direction B — Domain atlas

### Design-principles manifesto

The homepage is a map. Readers should recognize the territory before choosing a record.
Use the authored domain vocabulary as the dominant visual structure; make breadth,
overlap, and unclassified material legible. Search is a utility, not the composition.

```text
┌ Provenance / payments ─────────────── Search [⌕] ┐
│ 12 DOMAINS · 228 REQUIREMENTS · 576 RULES         │
│ (card counts below are illustrative)              │
│                                                   │
│ ┌ Identity ───────────┐ ┌ Billing ──────────────┐ │
│ │ 42 req · 106 rules  │ │ 76 req · 184 rules   │ │
│ │ 3 top-level paths → │ │ 2 top-level paths →  │ │
│ └─────────────────────┘ └────────────────────────┘ │
│ ┌ Privacy ────────────┐ ┌ Security ─────────────┐ │
│ │ 31 req · 82 rules   │ │ 49 req · 121 rules   │ │
│ └─────────────────────┘ └────────────────────────┘ │
│ [Show all 12 domains]   Unassigned 8   Health 42 → │
└────────────────────────────────────────────────────┘
```

**Quality bar:** a new reader can name the scope's major areas and enter one in under ten
seconds; missing and inherited domain membership are not disguised.

**Tradeoffs:** strongest orientation and a natural continuation of `/domains/`; weakest
when domain data is sparse, unstable, or overlapping. Domain inheritance can inflate
counts, so labels need defined counting semantics. A card grid also needs careful mobile
ordering and a hard cap rather than copying the full 72KB domain page.

**Exit criterion:** stop at one compact card per authored domain up to the row cap; if the
taxonomy cannot produce recognizable labels or honest counts, reject this direction
rather than filling cards with record lists.

## Direction C — Traceability ledger

### Design-principles manifesto

The homepage is the corpus's cover sheet. Trust begins with an honest account of what
exists, how much is connected, and where evidence is missing. Compress failures into
measures; never let diagnostics become decorative noise or conceal the healthy graph.

```text
┌ Provenance / payments ───────── Search  Domains ┐
│ CORPUS COVER SHEET                               │
│ 228 Requirements  165 Decisions  576 Rules  7 Src│
│                                                  │
│ Traceability                                     │
│ Requirements with sources       186 / 228  82%  │
│ Decisions connected             165 / 165 100%  │
│ Rules with producers            576 / 576 100%  │
│ [Inspect 42 findings →]                          │
│                                                  │
│ Top-level paths (12)                             │
│ Identity → 18 refinements   Billing → 18 …       │
└──────────────────────────────────────────────────┘
```

**Quality bar:** an auditor can distinguish corpus scale from corpus quality at a glance,
and every summary has an explainable denominator and a route to exact findings.

**Tradeoffs:** strongest for maintainers, reviewers, and assurance work; it risks framing
the wiki as an internal quality dashboard rather than reader documentation. It also
requires new aggregate health projections before the UI can promise percentages.

**Exit criterion:** stop at four explainable measures and one compact top-level-path list;
if a metric cannot link to its exact numerator records, omit it rather than inventing a
proxy.

## Comparison and unratified recommendation

| Criterion | A: Search launchpad | B: Domain atlas | C: Traceability ledger |
|---|---|---|---|
| Known-item retrieval | strongest | adequate | adequate |
| Unfamiliar-reader orientation | good | strongest | weak |
| Corpus assurance | visible summary | secondary | strongest |
| Works with incomplete taxonomy | yes | no | yes |
| New aggregate model work | low | medium | high |
| Mobile compression | strongest | medium | good |
| Primary risk | search coverage claim | taxonomy quality | maintainer-first framing |

**Recommend Direction A for human disposition.** It gives the two merged discovery routes
the prominence their reader jobs justify, has the lowest implementation cost, remains
honest about search coverage, and can satisfy all scale bounds without transferring the
804-entry search index onto the homepage. This is a recommendation, not a ratified
direction. Do not implement it until a human accepts A, B, or C and explicitly records any
grafts from the runners-up.

## Next decisions and implementation beads

1. **Decision bead:** dispose of A/B/C. Record the winner, rejected alternatives, and any
   explicit grafts; do not resolve this through an averaged “dashboard.”
2. **Corpus-health route bead (blocked on 1):** if the accepted direction includes a
   health summary, first group exact gaps by kind and expose category counts plus
   drill-down. This removes the homepage's individual-card obligation without hiding data.
3. **Homepage implementation bead (blocked on 1 and, conditionally, 2):** implement only
   the accepted first-viewport hierarchy, responsive ordering, semantics, and coverage
   copy. Keep detail routes separate.
4. **Search coverage bead:** decide whether resolutions and sources join offline search;
   measure payload and per-keystroke cost before changing the homepage's coverage claim.
5. **Scale-contract bead:** add the representative corpus fixture and bounded repetition,
   50KB HTML, first-focus-order, 1440px, and 390px assertions independently of the visual
   direction.

## Verification target for this research artifact

The evidence should remain valid if these commands pass on the baseline:

```sh
cargo test -p provenance-cli snapshot_scope_index_page
cargo test -p provenance-cli --test cli_wiki
npm run test:dom
cargo run -q -p provenance-cli -- docs check --repo . --format json
```

The first three protect the current renderer, CLI publication/serving behavior, and merged
offline search. The docs check protects this artifact's internal Markdown links.

The scale fixture itself is evidence rather than a committed test. Re-run the reproduction
commands above before implementation, then promote its assertions into the proposed
scale-contract bead.
