use super::super::*;
use super::fixtures::*;
use crate::state_store::{CreateRequirementInput, StateStore};
use provenance_core::{NodeType, RequirementStatus};

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

#[test]
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
