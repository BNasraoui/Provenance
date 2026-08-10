use super::super::*;
use super::fixtures::*;
use crate::state_store::{
    CreateRequirementInput, CreateResolutionInput, CreateRuleInput, StateStore,
};
use provenance_core::{
    NodeType, RequirementStatus, ResolutionStatus, RuleSeverity, RuleStatus, ScopeId,
};

#[test]
fn impact_reports_hop_distance_and_direction() {
    let (_dir, layout, scope) = seeded_layout();
    let impact = analyze_impact(
        &layout,
        &scope,
        NodeType::Source,
        &sid("source_schads"),
        ImpactOptions {
            max_hops: 3,
            follow_indirect: true,
        },
    )
    .unwrap();
    let rule = impact
        .nodes
        .iter()
        .find(|node| node.id == "rule_schads_pay_001")
        .unwrap();
    assert_eq!(rule.hop_distance, 2);
    assert_eq!(rule.direction, ImpactDirection::Downstream);
}

#[test]
fn stale_report_is_empty_for_unapproved_fixture() {
    let (_dir, layout, scope) = seeded_layout();
    assert!(find_stale(&layout, &scope).unwrap().is_empty());
}

#[test]
fn health_counts_rules_with_complete_traceability() {
    let (_dir, layout, scope) = seeded_layout();
    let health = coverage_health(&layout, &scope).unwrap();
    assert_eq!(health.rules.total, 1);
    assert_eq!(health.rules.with_complete_traceability, 1);
    assert_eq!(health.gaps.total, 0);
}

/// Seeds a requirement, the decision that settles it, and a rule both are
/// recorded as producing. No source is attached to the requirement.
fn seed_unsourced_chain(layout: &ProvenanceLayout, scope: &ScopeId) {
    let store = StateStore::new(layout.clone());
    create_requirement(&store, scope, "req_unsourced", RequirementStatus::Active);
    store
        .create_resolution(CreateResolutionInput {
            scope_id: scope.clone(),
            id: sid("res_unsourced"),
            title: "Unsourced decision".into(),
            requirement_id: Some(sid("req_unsourced")),
            position: "Adopt".into(),
            rationale: "Settles the requirement".into(),
            status: ResolutionStatus::Proposed,
            context: None,
            enforcement: None,
            confidence: None,
            inputs: Vec::new(),
            made_by: None,
            approved_by: None,
            approved_at: None,
            superseded_by: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    store
        .create_rule(CreateRuleInput {
            scope_id: scope.clone(),
            id: sid("rule_unsourced"),
            rule_code: "UNSOURCED-001".into(),
            name: None,
            description: None,
            requirement_id: Some(sid("req_unsourced")),
            resolution_id: Some(sid("res_unsourced")),
            statement: "A rule with no source behind it".into(),
            status: RuleStatus::Active,
            severity: RuleSeverity::High,
            rule_type: None,
            modality: None,
            confidence: None,
            extraction_method: None,
            source_document: None,
            source_section: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
}

/// The source leg of the orphan report has to reach the requirement that
/// produces this rule. Another sourced requirement in the scope, however
/// well traced, says nothing about this one.
#[test]
fn orphan_report_wants_a_source_behind_the_producing_requirement() {
    let (_dir, layout, scope) = seeded_layout();
    seed_unsourced_chain(&layout, &scope);

    let orphans = orphan_rules(&layout, &scope).unwrap();
    let orphan_ids: Vec<&str> = orphans
        .iter()
        .map(|orphan| orphan.rule_id.as_str())
        .collect();
    assert_eq!(orphan_ids, vec!["rule_unsourced"]);
    assert_eq!(orphans[0].missing, vec!["source".to_string()]);

    // Both producers are recorded, so the gap report leaves the rule alone;
    // the two readers differ only over the source leg.
    let gaps = find_gaps(&layout, &scope).unwrap();
    assert!(!gaps
        .iter()
        .any(|gap| gap.kind == GapKind::OrphanRule && gap.node_id == "rule_unsourced"));
    assert_eq!(
        coverage_health(&layout, &scope)
            .unwrap()
            .rules
            .with_complete_traceability,
        1
    );
}

/// A rule produced by its requirement alone is orphaned in both reports, and
/// the orphan report names the decision as the missing end.
#[test]
fn orphan_report_wants_both_producers() {
    let (_dir, layout, scope) = empty_layout();
    let store = StateStore::new(layout.clone());
    create_source(&store, &scope, "source_anchor");
    create_requirement(&store, &scope, "req_half", RequirementStatus::Active);
    attach_source(&store, &scope, "req_half", "source_anchor");
    store
        .create_rule(CreateRuleInput {
            scope_id: scope.clone(),
            id: sid("rule_half"),
            rule_code: "HALF-001".into(),
            name: None,
            description: None,
            requirement_id: Some(sid("req_half")),
            resolution_id: None,
            statement: "A rule with no decision behind it".into(),
            status: RuleStatus::Active,
            severity: RuleSeverity::High,
            rule_type: None,
            modality: None,
            confidence: None,
            extraction_method: None,
            source_document: None,
            source_section: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();

    let orphans = orphan_rules(&layout, &scope).unwrap();
    assert_eq!(orphans.len(), 1);
    assert_eq!(orphans[0].rule_id, "rule_half");
    assert_eq!(orphans[0].missing, vec!["resolution".to_string()]);
    assert!(find_gaps(&layout, &scope).unwrap().iter().any(|gap| {
        gap.kind == GapKind::OrphanRule
            && gap.node_id == "rule_half"
            && gap.reason == "no resolution produces this rule"
    }));
}

#[test]
#[provenance_macros::verifies("rule_graph_gaps", examples)]
fn gaps_flag_requirements_without_domain_id_but_not_requirements_with_one() {
    let (_dir, layout, scope) = seeded_layout();
    StateStore::new(layout.clone())
        .create_requirement(CreateRequirementInput {
            scope_id: scope.clone(),
            id: sid("req_missing_domain"),
            statement: "Rostering rules need a domain".into(),
            description: None,
            status: RequirementStatus::Active,
            domain_id: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    let gaps = find_gaps(&layout, &scope).unwrap();
    assert!(gaps.iter().any(|gap| gap.kind == GapKind::MissingDomainId
        && gap.requirement_id.as_deref() == Some("req_missing_domain")
        && gap.reason.contains("domain_id")));
    assert!(!gaps.iter().any(|gap| gap.kind == GapKind::MissingDomainId
        && gap.requirement_id.as_deref() == Some("req_schads_overtime")
        && gap.reason.contains("domain_id")));
}
