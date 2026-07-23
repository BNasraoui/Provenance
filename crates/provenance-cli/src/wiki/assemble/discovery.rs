use crate::handlers::ScopeExport;
use crate::wiki::model::{
    DomainGroup, DomainIndexPage, DomainState, PageId, PageLink, RequirementPage, RulePage,
    SearchEntry, SearchIndexPage,
};
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

    let missing_ids = requirements
        .iter()
        .filter_map(|requirement| requirement.domain_id)
        .filter(|id| !positions.contains_key(*id))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    for id in missing_ids {
        positions.insert(id.clone(), groups.len());
        groups.push(DomainGroup {
            state: DomainState::Missing { id },
            requirements: Vec::new(),
            rules: Vec::new(),
        });
    }

    let requirement_domains = requirements
        .iter()
        .map(|requirement| (requirement.id, requirement.domain_id))
        .collect::<BTreeMap<_, _>>();
    let needs_unassigned = requirements
        .iter()
        .any(|requirement| requirement.domain_id.is_none())
        || rules.iter().any(|rule| {
            rule.requirement_ids.is_empty()
                || rule
                    .requirement_ids
                    .iter()
                    .any(|id| !matches!(requirement_domains.get(id), Some(Some(_))))
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
        let position = requirement
            .domain_id
            .and_then(|id| positions.get(id).copied())
            .or_else(|| unassigned_position.filter(|_| requirement.domain_id.is_none()));
        if let Some(position) = position {
            groups[position].requirements.push(requirement.link.clone());
        }
    }
    for rule in rules {
        let mut group_positions = BTreeSet::new();
        for requirement_id in &rule.requirement_ids {
            match requirement_domains.get(requirement_id) {
                Some(Some(domain_id)) => {
                    if let Some(position) = positions.get(*domain_id) {
                        group_positions.insert(*position);
                    }
                }
                Some(None) | None => {
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

fn page_link(id: &PageId, title: &str) -> PageLink {
    PageLink {
        target: id.clone(),
        title: title.to_string(),
    }
}
