# CLI

Common local workflow:

```sh
provenance init --path . --scope default --path-prefix .
provenance sources create --scope default --id source_policy --name "Policy"
provenance domains create --scope default --id domain_policy --name "Policy"
provenance requirements create --scope default --id req_policy --statement "Follow policy" --domain-id domain_policy
provenance services create --scope default --id service_api --name "api" --status active
provenance service-bindings create --scope default --rule-id rule_policy --service-id service_api --binding-type enforces
provenance materialize --format json
provenance export --scope default --format json --output provenance-export.json
provenance check --format json
```

Agent-facing commands support JSON output for deterministic parsing.

Shaping turn-state commands: `questions create` requires `--method` (grill, prototype, research, verify, or task); `topics claim/release/close` and `questions claim/release/answer` manage claim state (claiming an already-claimed item fails and reports the holder; closing a topic or answering a question clears its claim); `requirements fog set/show/clear` manages the deliberately unstructured fog text on an anchor requirement.

Creation commands accept enriched v1 metadata for cloud-imported projects. Examples: `sources create --source-type legislation --reference "Department guidance" --effective-date 1714521600000 --review-date 1717200000000 --superseded-by source_2025`, `requirements create --status discovery --description "Research note" --domain-id domain_policy`, `resolutions create --status draft --confidence 0.9 --context "Code scan" --input-type regulatory --input-reference "Program manual" --input-summary "Reviewed rules" --made-by "Analyst" --approved-by "Approver" --approved-at 1714780800000 --superseded-by res_2025`, `rules create --status draft --rule-type business --modality obligation --source-document path --source-section "lines 1-3"`, and `services create --environment production --tier critical --external-id backstage:component/api`.
