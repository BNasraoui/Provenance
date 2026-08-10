use super::super::build_corpus;
use super::fixtures::{edge, empty_state, requirement, rule, sid};
use crate::wiki::links::LinkResolver;
use crate::wiki::model::DomainState;
use provenance_core::{Domain, EdgeType, NodeType, RequirementStatus, SchemaVersion};
use provenance_macros::verifies;

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
#[verifies("rule_wiki_homepage_search_coverage", examples)]
fn discovery_indexes_requirement_and_rule_titles_and_statements() {
    let mut state = empty_state();
    state.requirements = vec![requirement(
        "req_invoice",
        "Invoices shall identify the participant",
        RequirementStatus::Active,
        vec![],
    )];
    state.rules = vec![rule("rule_invoice", Some("Group invoices"))];

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
    assert_eq!(
        corpus.search.coverage,
        "Search covers requirements and rules."
    );
}

#[test]
fn homepage_and_search_use_a_real_indexed_title_as_the_search_example() {
    let mut state = empty_state();
    state.requirements = vec![requirement(
        "req_invoice",
        "Invoices shall identify the participant",
        RequirementStatus::Active,
        vec![],
    )];
    let corpus = build_corpus(&state, &LinkResolver::new(None));

    let homepage = crate::wiki::render::render_index("default", &corpus.index);
    let search = crate::wiki::render::render_search("default", &corpus.search);

    for html in [homepage, search] {
        assert!(
            html.contains("placeholder=\"e.g. Invoices shall identify the participant\""),
            "{html}"
        );
        assert!(!html.contains("invoice participant"), "{html}");
    }
}

#[test]
#[verifies("rule_domain_attribution", examples)]
fn domains_group_rules_through_canonical_requirement_relationships() {
    let mut state = empty_state();
    state.domains = vec![domain("domain_default", "Invoicing")];
    state.requirements = vec![requirement(
        "req_invoice",
        "Invoices shall identify the participant",
        RequirementStatus::Active,
        vec![],
    )];
    state.rules = vec![rule("rule_invoice", Some("Group invoices"))];
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
#[verifies("rule_domain_attribution", examples)]
fn domains_group_children_and_rules_by_their_root_requirement_domain() {
    let mut state = empty_state();
    state.domains = vec![domain("domain_default", "Invoicing")];
    let root = requirement(
        "req_root",
        "Invoices shall identify the participant",
        RequirementStatus::Active,
        vec![],
    );
    let mut child = requirement(
        "req_child",
        "Invoice lines shall identify the participant",
        RequirementStatus::Active,
        vec![],
    );
    child.domain_id = None;
    state.requirements = vec![root, child];
    state.rules = vec![rule("rule_invoice", Some("Group invoices"))];
    state.edges = vec![
        edge(
            EdgeType::RefinesInto,
            (NodeType::Requirement, "req_root"),
            (NodeType::Requirement, "req_child"),
        ),
        edge(
            EdgeType::Produces,
            (NodeType::Requirement, "req_child"),
            (NodeType::Rule, "rule_invoice"),
        ),
    ];

    let corpus = build_corpus(&state, &LinkResolver::new(None));
    let group = &corpus.domains.groups[0];

    assert_eq!(corpus.domains.groups.len(), 1);
    assert_eq!(
        group
            .requirements
            .iter()
            .map(|link| link.target.record_id.as_str())
            .collect::<Vec<_>>(),
        vec!["req_root", "req_child"]
    );
    assert_eq!(group.rules[0].target.record_id, "rule_invoice");
}

#[test]
#[verifies("rule_domain_attribution", examples)]
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
        rule("rule_missing", Some("Missing rule")),
        rule("rule_unassigned", Some("Unassigned rule")),
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
fn domains_without_authored_groups_render_flat_records_with_statements() {
    let mut state = empty_state();
    let mut requirement = requirement(
        "req_invoice",
        "Invoices shall identify the participant",
        RequirementStatus::Active,
        vec![],
    );
    requirement.domain_id = None;
    state.requirements = vec![requirement];
    state.rules = vec![rule("rule_invoice", Some("Group invoices"))];

    let corpus = build_corpus(&state, &LinkResolver::new(None));
    let html = crate::wiki::render::render_domains("default", &corpus.domains);

    assert!(html.contains("No domains have been authored"), "{html}");
    assert!(html.contains(">All requirements</h2>"), "{html}");
    assert!(html.contains(">All rules</h2>"), "{html}");
    assert!(
        html.contains("Invoices shall identify the participant"),
        "{html}"
    );
    assert!(
        html.contains("Claim items shall be grouped by participant"),
        "{html}"
    );
    assert!(!html.contains(">Unassigned</h2>"), "{html}");
    assert!(html.contains("0 groups"), "{html}");
}

#[test]
fn missing_domain_classification_does_not_link_to_an_absent_group() {
    let mut state = empty_state();
    let mut requirement = requirement(
        "req_invoice",
        "Invoices shall identify the participant",
        RequirementStatus::Active,
        vec![],
    );
    requirement.domain_id = Some(sid("domain_missing"));
    state.requirements = vec![requirement];

    let corpus = build_corpus(&state, &LinkResolver::new(None));
    let requirement = crate::wiki::render::render_requirement("default", &corpus.requirements[0]);

    assert!(
        requirement.contains(">domain_missing</span>"),
        "{requirement}"
    );
    assert!(
        !requirement.contains("href=\"/domains/#domain-domain_missing\""),
        "{requirement}"
    );
}

#[test]
fn missing_domain_classification_links_when_the_gap_group_is_rendered() {
    let mut state = empty_state();
    state.domains = vec![domain("domain_authored", "Authored")];
    let mut requirement = requirement(
        "req_invoice",
        "Invoices shall identify the participant",
        RequirementStatus::Active,
        vec![],
    );
    requirement.domain_id = Some(sid("domain_missing"));
    state.requirements = vec![requirement];

    let corpus = build_corpus(&state, &LinkResolver::new(None));
    let requirement = crate::wiki::render::render_requirement("default", &corpus.requirements[0]);
    let domains = crate::wiki::render::render_domains("default", &corpus.domains);

    assert!(
        requirement.contains("href=\"/domains/#domain-domain_missing\""),
        "{requirement}"
    );
    assert!(
        domains.contains("id=\"domain-domain_missing\""),
        "{domains}"
    );
}

#[test]
fn empty_scope_still_has_discovery_pages() {
    let corpus = build_corpus(&empty_state(), &LinkResolver::new(None));
    assert!(corpus.domains.groups.is_empty());
    assert!(corpus.search.entries.is_empty());
}
