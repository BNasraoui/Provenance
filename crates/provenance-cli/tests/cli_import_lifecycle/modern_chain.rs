use super::support::{create_system_source, export_scope, import_scope, init_repo, write_json};
use std::path::PathBuf;

#[test]
fn import_cannot_omit_existing_modern_lifecycle_chain() {
    let fixture = ModernLifecycleFixture::new();
    let mut missing_assertion = fixture.lifecycle.clone();
    missing_assertion["assertion_records"] = serde_json::json!([]);
    missing_assertion["dispositions"] = serde_json::json!([]);
    fixture
        .import_value("missing-assertion.json", &missing_assertion)
        .failure()
        .stderr(predicates::str::contains(
            "qualifying proposal proposal_a requires an assertion",
        ));

    fixture.import_lifecycle().success();
    fixture.assert_asserted_evidence_is_immutable();

    let mut omitted = fixture.lifecycle.clone();
    omitted["proposal_cards"] = serde_json::json!([]);
    omitted["assertion_records"] = serde_json::json!([]);
    omitted["dispositions"] = serde_json::json!([]);
    fixture
        .import_value("omitted.json", &omitted)
        .failure()
        .stderr(predicates::str::contains("immutable proposal proposal_a"));
}

#[test]
fn import_rejects_duplicate_evidence_record_ids() {
    let fixture = ModernLifecycleFixture::new();
    let cases = [
        (
            "contributions",
            "strongest_finding",
            "duplicate immutable contribution id contribution_a",
        ),
        (
            "synthesis_packets",
            "summary",
            "duplicate immutable synthesis packet id synthesis_a",
        ),
    ];

    for (records, divergent_field, expected) in cases {
        let mut duplicate = fixture.lifecycle.clone();
        let mut divergent = duplicate[records][0].clone();
        divergent[divergent_field] = serde_json::json!("Divergent");
        duplicate[records].as_array_mut().unwrap().push(divergent);

        fixture
            .import_value(&format!("duplicate-{records}.json"), &duplicate)
            .failure()
            .stderr(predicates::str::contains(expected));
    }
}

#[test]
fn import_rejects_assertion_with_rejected_or_deferred_disposition() {
    let fixture = ModernLifecycleFixture::new();
    for decision in ["rejected", "deferred"] {
        let mut lifecycle = fixture.lifecycle.clone();
        lifecycle["dispositions"][0]["decision"] = serde_json::json!(decision);
        fixture
            .import_value(&format!("{decision}-asserted.json"), &lifecycle)
            .failure()
            .stderr(predicates::str::contains(
                "cannot be asserted after disposition",
            ));
    }
}

struct ModernLifecycleFixture {
    _directory: tempfile::TempDir,
    root: PathBuf,
    repo: PathBuf,
    lifecycle: serde_json::Value,
}

impl ModernLifecycleFixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().to_path_buf();
        let repo = root.join("repo");
        init_repo(&repo, Some("reviewer"));
        create_system_source(&repo);
        let baseline = root.join("baseline.json");
        export_scope(&repo, &baseline).success();
        let mut lifecycle: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&baseline).unwrap()).unwrap();
        add_modern_lifecycle(&mut lifecycle);
        Self {
            _directory: directory,
            root,
            repo,
            lifecycle,
        }
    }

    fn import_lifecycle(&self) -> assert_cmd::assert::Assert {
        self.import_value("lifecycle.json", &self.lifecycle)
    }

    fn import_value(&self, name: &str, value: &serde_json::Value) -> assert_cmd::assert::Assert {
        let input = self.root.join(name);
        write_json(&input, value);
        import_scope(&self.repo, &input)
    }

    fn assert_asserted_evidence_is_immutable(&self) {
        let mut changed_contribution = self.lifecycle.clone();
        changed_contribution["contributions"][0]["strongest_finding"] =
            serde_json::json!("Rewritten");
        self.import_value("changed-contribution.json", &changed_contribution)
            .failure()
            .stderr(predicates::str::contains(
                "contribution contribution_a is referenced by an assertion and cannot be replaced",
            ));

        let mut retargeted_claim = self.lifecycle.clone();
        retargeted_claim["contributions"][0]["id"] = serde_json::json!("contribution_b");
        self.import_value("retargeted-claim.json", &retargeted_claim)
            .failure()
            .stderr(predicates::str::contains(
                "contribution contribution_a is referenced by an assertion and cannot be replaced",
            ));

        let mut changed_synthesis = self.lifecycle.clone();
        changed_synthesis["synthesis_packets"][0]["summary"] = serde_json::json!("Rewritten");
        self.import_value("changed-synthesis.json", &changed_synthesis)
            .failure()
            .stderr(predicates::str::contains(
                "synthesis packet synthesis_a is referenced by an assertion and cannot be replaced",
            ));
    }
}

fn add_modern_lifecycle(lifecycle: &mut serde_json::Value) {
    lifecycle["contributions"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "contribution_a",
        "target": {"artifact_type": "source", "artifact_id": "source_a"},
        "participant_slot": "reviewer", "stance": "support", "strongest_finding": "Observed",
        "evidence_references": [{"reference_id": "evidence_a", "evidence_type": "source", "summary": "Pinned"}],
        "material_claims": [{"claim_id": "claim_a", "statement": "Observed", "evidence_type": "source", "evidence_reference_ids": ["evidence_a"]}],
        "risks": [], "objections": [], "challenges": [], "suggested_artifact_changes": [],
        "unsupported_recommendations": [], "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
    }]);
    lifecycle["synthesis_packets"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "synthesis_a",
        "target": {"artifact_type": "source", "artifact_id": "source_a"}, "summary": "Adjudicated",
        "consensus": [], "contested_claims": [], "minority_objections": [], "evidence_gaps": [],
        "unsupported_speculation": [], "open_questions": [],
        "suggested_artifacts": [{"proposal_id": "proposal_a", "proposal_key": "proposal-a", "proposal_type": "requirement_candidate", "summary": "Candidate", "origin_participant_slots": ["reviewer"]}],
        "required_human_decisions": []
    }]);
    lifecycle["proposal_cards"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "proposal_a", "proposal_key": "proposal-a",
        "proposal_type": "requirement_candidate", "title": "Candidate", "summary": "Candidate",
        "traceability": {"target": {"artifact_type": "source", "artifact_id": "source_a"}, "source_ids": ["source_a"], "evidence_references": [], "supporting_claim_ids": ["claim_a"]},
        "promotion_state": "proposed"
    }]);
    lifecycle["assertion_records"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "assertion_a", "proposal_id": "proposal_a",
        "synthesis_packet_id": "synthesis_a", "supporting_claim_ids": ["claim_a"]
    }]);
    lifecycle["dispositions"] = serde_json::json!([{
        "schema_version": 1, "scope_id": "default", "id": "disposition_a", "proposal_id": "proposal_a",
        "decision": "accepted", "rationale": "Reviewed", "actor": {"identity_type": "human", "id": "reviewer"}
    }]);
}
