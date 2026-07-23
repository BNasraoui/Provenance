use super::super::build_corpus;
use super::fixtures::{edge, empty_state, requirement, rule, sid};
use crate::wiki::links::LinkResolver;
use crate::wiki::model::DomainState;
use provenance_core::{Domain, EdgeType, NodeType, RequirementStatus, SchemaVersion};

fn domain(id: &str, name: &str) -> Domain {
    Domain {
        schema_version: SchemaVersion(1),
        scope_id: super::fixtures::scope_id(),
        id: sid(id),
        name: name.to_string(),
        description: Some(format!("About {name}")),
        color: None,
    }
}

#[test]
fn discovery_indexes_requirement_and_rule_titles_and_statements() {
    let mut state = empty_state();
    state.requirements = vec![requirement(
        "req_invoice",
        "Invoices shall identify the participant",
        RequirementStatus::Active,
        vec![],
    )];
    state.rules = vec![rule("rule_invoice", "INV-1", Some("Group invoices"))];

    let corpus = build_corpus(&state, &LinkResolver::new(None));

    assert_eq!(corpus.search.entries.len(), 2);
    assert_eq!(
        corpus.search.entries[0].link.title,
        "Invoices shall identify the participant"
    );
    assert_eq!(
        corpus.search.entries[0].statement,
        "Invoices shall identify the participant"
    );
    assert_eq!(corpus.search.entries[1].link.title, "Group invoices");
    assert_eq!(
        corpus.search.entries[1].statement,
        "Claim items shall be grouped by participant"
    );
}

#[test]
fn domains_group_rules_through_canonical_requirement_relationships() {
    let mut state = empty_state();
    state.domains = vec![domain("domain_default", "Invoicing")];
    state.requirements = vec![requirement(
        "req_invoice",
        "Invoices shall identify the participant",
        RequirementStatus::Active,
        vec![],
    )];
    state.rules = vec![rule("rule_invoice", "INV-1", Some("Group invoices"))];
    state.edges = vec![edge(
        EdgeType::Produces,
        (NodeType::Requirement, "req_invoice"),
        (NodeType::Rule, "rule_invoice"),
    )];

    let corpus = build_corpus(&state, &LinkResolver::new(None));
    let group = &corpus.domains.groups[0];

    assert!(matches!(&group.state, DomainState::Defined { id, .. } if id == "domain_default"));
    assert_eq!(group.requirements[0].target.record_id, "req_invoice");
    assert_eq!(group.rules[0].target.record_id, "rule_invoice");
}

#[test]
fn domains_surface_defined_missing_and_unassigned_without_dropping_rules() {
    let mut state = empty_state();
    state.domains = vec![domain("domain_default", "Invoicing")];
    let defined = requirement("req_defined", "Defined", RequirementStatus::Active, vec![]);
    let mut missing = requirement("req_missing", "Missing", RequirementStatus::Active, vec![]);
    missing.domain_id = Some(sid("domain_missing"));
    let mut unassigned = requirement(
        "req_unassigned",
        "Unassigned",
        RequirementStatus::Active,
        vec![],
    );
    unassigned.domain_id = None;
    state.requirements = vec![defined, missing, unassigned];
    state.rules = vec![
        rule("rule_missing", "MISS-1", Some("Missing rule")),
        rule("rule_unassigned", "NONE-1", Some("Unassigned rule")),
    ];
    state.edges = vec![
        edge(
            EdgeType::Produces,
            (NodeType::Requirement, "req_missing"),
            (NodeType::Rule, "rule_missing"),
        ),
        edge(
            EdgeType::Produces,
            (NodeType::Requirement, "req_unassigned"),
            (NodeType::Rule, "rule_unassigned"),
        ),
    ];

    let corpus = build_corpus(&state, &LinkResolver::new(None));

    assert_eq!(corpus.domains.groups.len(), 3);
    assert!(matches!(
        corpus.domains.groups[0].state,
        DomainState::Defined { .. }
    ));
    assert!(matches!(
        &corpus.domains.groups[1].state,
        DomainState::Missing { id } if id == "domain_missing"
    ));
    assert!(matches!(
        corpus.domains.groups[2].state,
        DomainState::Unassigned
    ));
    assert_eq!(
        corpus.domains.groups[1].rules[0].target.record_id,
        "rule_missing"
    );
    assert_eq!(
        corpus.domains.groups[2].rules[0].target.record_id,
        "rule_unassigned"
    );
}

#[test]
fn empty_scope_still_has_discovery_pages() {
    let corpus = build_corpus(&empty_state(), &LinkResolver::new(None));
    assert!(corpus.domains.groups.is_empty());
    assert!(corpus.search.entries.is_empty());
}
