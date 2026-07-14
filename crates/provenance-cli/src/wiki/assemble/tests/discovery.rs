use super::super::build_corpus;
use super::fixtures::*;
use crate::wiki::links::LinkResolver;
use crate::wiki::model::Topic;
use provenance_core::{Domain, RequirementStatus, SchemaVersion};

fn domain(id: &str, name: &str) -> Domain {
    Domain {
        schema_version: SchemaVersion(1),
        scope_id: scope_id(),
        id: sid(id),
        name: name.to_string(),
        description: None,
        color: None,
    }
}

#[test]
fn discovery_indexes_requirement_and_rule_titles_and_statements() {
    let corpus = fixture_corpus();
    let entries = &corpus.search.entries;
    assert_eq!(
        entries.len(),
        corpus.requirements.len() + corpus.rules.len()
    );

    let rule = entries
        .iter()
        .find(|entry| entry.link.target.record_id == "rule_001")
        .unwrap();
    assert_eq!(rule.link.title, "Invoices grouped by participant");
    assert_eq!(
        rule.statement,
        "Claim items shall be grouped by participant"
    );
}

#[test]
fn topics_group_requirements_and_their_rules_under_defined_domains() {
    let mut state = fixture_state();
    state.domains = vec![domain("domain_default", "Invoicing")];
    let corpus = build_corpus(&state, &LinkResolver::new(None));

    let group = &corpus.topics.groups[0];
    assert_eq!(
        group.topic,
        Topic::Defined {
            id: "domain_default".to_string(),
            name: "Invoicing".to_string(),
            description: None,
        }
    );
    assert!(group
        .requirements
        .iter()
        .any(|link| link.target.record_id == "req_child"));
    assert!(group
        .rules
        .iter()
        .any(|link| link.target.record_id == "rule_001"));
}

#[test]
fn topics_surface_missing_and_unassigned_domain_data_without_dropping_records() {
    let mut state = empty_state();
    state.domains = vec![domain("domain_empty", "Empty domain")];
    let mut dangling = requirement(
        "req_dangling",
        "Dangling domain requirement",
        RequirementStatus::Active,
        vec![],
    );
    dangling.domain_id = Some(sid("domain_missing"));
    let mut unassigned = requirement(
        "req_unassigned",
        "Unassigned requirement",
        RequirementStatus::Active,
        vec![],
    );
    unassigned.domain_id = None;
    state.requirements = vec![dangling, unassigned];
    state.rules = vec![rule("rule_orphan", "ORPH-001", Some("Detached rule"))];

    let corpus = build_corpus(&state, &LinkResolver::new(None));
    assert_eq!(corpus.topics.groups.len(), 3);
    assert!(corpus.topics.groups[0].requirements.is_empty());
    assert!(corpus.topics.groups[0].rules.is_empty());

    let missing = corpus
        .topics
        .groups
        .iter()
        .find(|group| matches!(&group.topic, Topic::Missing { id } if id == "domain_missing"))
        .unwrap();
    assert_eq!(missing.requirements[0].target.record_id, "req_dangling");

    let unassigned = corpus
        .topics
        .groups
        .iter()
        .find(|group| group.topic == Topic::Unassigned)
        .unwrap();
    let requirement_ids: Vec<&str> = unassigned
        .requirements
        .iter()
        .map(|link| link.target.record_id.as_str())
        .collect();
    assert_eq!(requirement_ids, vec!["req_unassigned"]);
    assert_eq!(unassigned.rules[0].target.record_id, "rule_orphan");
}

#[test]
fn discovery_pages_are_empty_but_present_for_an_empty_scope() {
    let corpus = build_corpus(&empty_state(), &LinkResolver::new(None));
    assert!(corpus.search.entries.is_empty());
    assert!(corpus.topics.groups.is_empty());
}
