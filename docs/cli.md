# CLI

Common local workflow:

```sh
provenance init --path . --scope default --path-prefix .
provenance sources create --scope default --id source_policy --name "Policy"
provenance domains create --scope default --id domain_policy --name "Policy"
provenance requirements create --scope default --id req_policy --statement "Follow policy" --domain-id domain_policy
provenance edges create --scope default --type references --from-type source --from-id source_policy --to-type requirement --to-id req_policy
provenance services create --scope default --id ex_api --name "api" --status active
provenance service-bindings create --scope default --rule-id rule_policy --service-id ex_api --binding-type enforces
provenance materialize --format json
provenance export --scope default --format json --output provenance-export.json
provenance check --format json
```

Agent-facing commands support JSON output for deterministic parsing.

Skill distribution commands embed the top-level `skills/*/SKILL.md` product skills in the
binary: `provenance skills list --format json`,
`provenance skills show provenance-fork-tournament`, and
`provenance skills install [--global] [--copy] [--force] --format json`. Local installs
write canonical skill files to `.agents/skills/` and link them into `.claude/skills/`;
`--copy` writes Claude skill directories instead of symlinks. `provenance prime` reports
whether the canonical skills are installed and prints the repo-root install command;
shaping/ideation commands emit a non-blocking stderr hint when skills are missing,
suppressible with `--quiet`.

Ideation JSON flags accept inline JSON or `@path/to/payload.json`. Artifact helpers:
`provenance schema show contribution|synthesis-packet|proposal --format json` prints
canonical record schemas, and `provenance validate contribution|synthesis-packet|proposal
--input artifact.json --format json` validates full records, including nested stable IDs.
`contributions create`, `synthesis-packets create`, and `proposals create` keep duplicate
protection by default; pass `--replace` to intentionally upsert the same stable ID.
Use `proposals create --promotion-state asserted` only for an unrefuted adversarial result;
`--builds-on <proposal-id>` records provisional derivation. Asserted proposals are durable
and cannot be replaced. Consult them with
`proposals list --promotion-state asserted --format json`; `prime` renders both raw
proposals and assertions, explicitly marking assertions as not human-ratified. Accepting a
requirement, resolution, or rule candidate requires a human promotion decision rather than
direct creation in the accepted state.
Swarm backtrace runs can land durable run outputs with
`provenance swarm-backtrace land --scope <scope> --run-dir <run-dir> --format json`.

Graph edge commands: `edges create --type references|refines_into|depends_on|contradicts|supersedes|needs|resolves|spawns|produces --from-type source|requirement|resolution|rule --from-id <id> --to-type source|requirement|resolution|rule --to-id <id>`, `edges list`, and `edges delete --id <edge-id>`. Creation validates edge type/endpoints and requires both endpoint records to exist.

Shaping turn-state commands: `questions create` requires `--method` (grill, prototype, research, verify, or task); `topics claim/release/close` and `questions claim/release/answer` manage claim state (claiming an already-claimed item fails and reports the holder; closing a topic or answering a question clears its claim); `requirements fog set/show/clear` manages the deliberately unstructured fog text on an anchor requirement.

Creation commands accept enriched v1 metadata for cloud-imported projects. Examples: `sources create --source-type legislation --reference "Department guidance" --commit-pin 5e1f2a9c4b6d8e0f1234567890abcdef12345678 --effective-date 1714521600000 --review-date 1717200000000 --superseded-by source_2025`, `requirements create --status discovery --description "Research note" --domain-id domain_policy`, `resolutions create --status draft --confidence 0.9 --context "Code scan" --input-type regulatory --input-reference "Program manual" --input-summary "Reviewed rules" --made-by "Analyst" --approved-by "Approver" --approved-at 1714780800000 --superseded-by res_2025`, `rules create --status draft --rule-type business --modality obligation --source-document path --source-section "lines 1-3"`, `proposals create --confidence 0.83`, and `services create --environment production --tier critical --external-id backstage:component/api`. Confidence values must be between `0.0` and `1.0`; source commit pins must be 7-64 hexadecimal characters.
