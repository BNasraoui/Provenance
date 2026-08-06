use crate::handlers::ScopeExport;
use crate::wiki::model::{
    DomainGroup, DomainIndexPage, DomainState, PageId, PageLink, RequirementPage, RulePage,
    SearchEntry, SearchIndexPage,
};
use provenance_core::{EdgeType, NodeType};
use std::collections::{BTreeMap, BTreeSet};

struct RequirementRecord<'a> {
    id: &'a str,
    domain_id: Option<&'a str>,
    link: PageLink,
    statement: &'a str,
}

struct RuleRecord<'a> {
    link: PageLink,
    statement: &'a str,
    requirement_ids: Vec<&'a str>,
}

pub(super) fn build_discovery_pages(
    state: &ScopeExport,
    requirements: &[RequirementPage],
    rules: &[RulePage],
) -> (DomainIndexPage, SearchIndexPage) {
    let requirements = requirements
        .iter()
        .map(|page| RequirementRecord {
            id: &page.id.record_id,
            domain_id: page.domain_id.as_deref(),
            link: page_link(&page.id, &page.title),
            statement: &page.statement,
        })
        .collect::<Vec<_>>();
    let rules = rules
        .iter()
        .map(|page| RuleRecord {
            link: page_link(&page.id, &page.title),
            statement: &page.statement,
            requirement_ids: page
                .requirements
                .iter()
                .map(|link| link.target.record_id.as_str())
                .collect(),
        })
        .collect::<Vec<_>>();

    (
        domain_index(state, &requirements, &rules),
        search_index(&state.scope, &requirements, &rules),
    )
}

fn search_index(
    scope: &str,
    requirements: &[RequirementRecord<'_>],
    rules: &[RuleRecord<'_>],
) -> SearchIndexPage {
    let entries = requirements
        .iter()
        .map(|requirement| SearchEntry {
            link: requirement.link.clone(),
            statement: requirement.statement.to_string(),
        })
        .chain(rules.iter().map(|rule| SearchEntry {
            link: rule.link.clone(),
            statement: rule.statement.to_string(),
        }))
        .collect();
    SearchIndexPage {
        scope: scope.to_string(),
        title: "Search requirements and rules".to_string(),
        entries,
    }
}

fn domain_index(
    state: &ScopeExport,
    requirements: &[RequirementRecord<'_>],
    rules: &[RuleRecord<'_>],
) -> DomainIndexPage {
    let mut groups = Vec::new();
    let mut positions = BTreeMap::new();
    for domain in &state.domains {
        positions.insert(domain.id.as_str().to_string(), groups.len());
        groups.push(DomainGroup {
            state: DomainState::Defined {
                id: domain.id.as_str().to_string(),
                name: domain.name.clone(),
                description: domain.description.clone(),
            },
            requirements: Vec::new(),
            rules: Vec::new(),
        });
    }

    let requirement_domains = effective_requirement_domains(state, requirements);
    let missing_ids = requirement_domains
        .values()
        .flatten()
        .filter(|id| !positions.contains_key(*id))
        .cloned()
        .collect::<BTreeSet<_>>();
    for id in missing_ids {
        positions.insert(id.clone(), groups.len());
        groups.push(DomainGroup {
            state: DomainState::Missing { id },
            requirements: Vec::new(),
            rules: Vec::new(),
        });
    }

    let needs_unassigned = requirement_domains.values().any(BTreeSet::is_empty)
        || rules.iter().any(|rule| {
            rule.requirement_ids.is_empty()
                || rule
                    .requirement_ids
                    .iter()
                    .any(|id| requirement_domains.get(*id).is_none_or(BTreeSet::is_empty))
        });
    let unassigned_position = needs_unassigned.then(|| {
        let position = groups.len();
        groups.push(DomainGroup {
            state: DomainState::Unassigned,
            requirements: Vec::new(),
            rules: Vec::new(),
        });
        position
    });

    for requirement in requirements {
        if let Some(domain_ids) = requirement_domains.get(requirement.id) {
            for domain_id in domain_ids {
                if let Some(position) = positions.get(domain_id) {
                    groups[*position]
                        .requirements
                        .push(requirement.link.clone());
                }
            }
            if domain_ids.is_empty() {
                if let Some(position) = unassigned_position {
                    groups[position].requirements.push(requirement.link.clone());
                }
            }
        }
    }
    for rule in rules {
        let mut group_positions = BTreeSet::new();
        for requirement_id in &rule.requirement_ids {
            match requirement_domains.get(*requirement_id) {
                Some(domain_ids) if !domain_ids.is_empty() => {
                    for domain_id in domain_ids {
                        if let Some(position) = positions.get(domain_id) {
                            group_positions.insert(*position);
                        }
                    }
                }
                Some(_) | None => {
                    if let Some(position) = unassigned_position {
                        group_positions.insert(position);
                    }
                }
            }
        }
        if rule.requirement_ids.is_empty() {
            if let Some(position) = unassigned_position {
                group_positions.insert(position);
            }
        }
        for position in group_positions {
            groups[position].rules.push(rule.link.clone());
        }
    }

    DomainIndexPage {
        scope: state.scope.clone(),
        title: "Requirements and rules by domain".to_string(),
        groups,
    }
}

fn effective_requirement_domains(
    state: &ScopeExport,
    requirements: &[RequirementRecord<'_>],
) -> BTreeMap<String, BTreeSet<String>> {
    let records = requirements
        .iter()
        .map(|requirement| (requirement.id, requirement))
        .collect::<BTreeMap<_, _>>();
    let mut parents = BTreeMap::<&str, Vec<&str>>::new();
    for edge in &state.edges {
        if edge.edge_type == EdgeType::RefinesInto
            && edge.from_type == NodeType::Requirement
            && edge.to_type == NodeType::Requirement
        {
            parents
                .entry(edge.to_id.as_str())
                .or_default()
                .push(edge.from_id.as_str());
        }
    }

    requirements
        .iter()
        .map(|requirement| {
            let mut domains = BTreeSet::new();
            let mut visited = BTreeSet::new();
            let mut pending = vec![requirement.id];
            while let Some(id) = pending.pop() {
                if !visited.insert(id) {
                    continue;
                }
                if let Some(record) = records.get(id) {
                    if let Some(domain_id) = record.domain_id {
                        domains.insert(domain_id.to_string());
                    }
                }
                if let Some(parent_ids) = parents.get(id) {
                    pending.extend(parent_ids.iter().copied());
                }
            }
            (requirement.id.to_string(), domains)
        })
        .collect()
}

fn page_link(id: &PageId, title: &str) -> PageLink {
    PageLink {
        target: id.clone(),
        title: title.to_string(),
    }
}
