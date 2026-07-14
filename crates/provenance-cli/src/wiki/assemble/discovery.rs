use crate::handlers::ScopeExport;
use crate::wiki::model::{
    PageId, PageKind, PageLink, RequirementPage, RulePage, SearchEntry, SearchIndexPage,
    TopicGroup, TopicIndexPage,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn build_discovery_pages(
    state: &ScopeExport,
    requirements: &[RequirementPage],
    rules: &[RulePage],
) -> (TopicIndexPage, SearchIndexPage) {
    let topics = topic_index(state, requirements, rules);
    let search = search_index(&state.scope, requirements, rules);
    (topics, search)
}

fn search_index(
    scope: &str,
    requirements: &[RequirementPage],
    rules: &[RulePage],
) -> SearchIndexPage {
    let entries = requirements
        .iter()
        .map(|page| SearchEntry {
            link: page_link(&page.id, &page.title),
            kind: PageKind::Requirement,
            statement: page.statement.clone(),
        })
        .chain(rules.iter().map(|page| SearchEntry {
            link: page_link(&page.id, &page.title),
            kind: PageKind::Rule,
            statement: page.statement.clone(),
        }))
        .collect();
    SearchIndexPage {
        id: PageId::new(PageKind::SearchIndex, scope),
        scope: scope.to_string(),
        title: "Search requirements and rules".to_string(),
        entries,
    }
}

fn topic_index(
    state: &ScopeExport,
    requirements: &[RequirementPage],
    rules: &[RulePage],
) -> TopicIndexPage {
    let mut groups = Vec::new();
    let mut positions = BTreeMap::new();
    for domain in &state.domains {
        positions.insert(domain.id.as_str().to_string(), groups.len());
        groups.push(TopicGroup {
            domain_id: Some(domain.id.as_str().to_string()),
            anchor: format!("domain-{}", domain.id.as_str()),
            name: domain.name.clone(),
            description: domain.description.clone(),
            missing: false,
            requirements: Vec::new(),
            rules: Vec::new(),
        });
    }

    let missing_ids: BTreeSet<String> = requirements
        .iter()
        .filter_map(|page| page.domain_id.clone())
        .filter(|id| !positions.contains_key(id))
        .collect();
    for id in missing_ids {
        positions.insert(id.clone(), groups.len());
        groups.push(TopicGroup {
            anchor: format!("domain-{id}"),
            name: format!("Missing domain: {id}"),
            domain_id: Some(id),
            description: None,
            missing: true,
            requirements: Vec::new(),
            rules: Vec::new(),
        });
    }

    let needs_unassigned = requirements.iter().any(|page| page.domain_id.is_none())
        || rules.iter().any(|page| {
            page.requirements.is_empty()
                || page
                    .requirements
                    .iter()
                    .any(|link| requirement_domain(requirements, &link.target.record_id).is_none())
        });
    let unassigned_position = needs_unassigned.then(|| {
        let position = groups.len();
        groups.push(TopicGroup {
            domain_id: None,
            anchor: "domain-unassigned".to_string(),
            name: "Unassigned".to_string(),
            description: Some(
                "Records with no domain_id, or rules with no domain-backed requirement provenance."
                    .to_string(),
            ),
            missing: true,
            requirements: Vec::new(),
            rules: Vec::new(),
        });
        position
    });

    for page in requirements {
        let position = page
            .domain_id
            .as_ref()
            .map_or(unassigned_position, |id| positions.get(id).copied());
        if let Some(position) = position {
            groups[position]
                .requirements
                .push(page_link(&page.id, &page.title));
        }
    }
    for page in rules {
        let mut group_positions = BTreeSet::new();
        for requirement in &page.requirements {
            match requirement_domain(requirements, &requirement.target.record_id) {
                Some(id) => {
                    if let Some(position) = positions.get(id) {
                        group_positions.insert(*position);
                    }
                }
                None => {
                    if let Some(position) = unassigned_position {
                        group_positions.insert(position);
                    }
                }
            }
        }
        if page.requirements.is_empty() {
            if let Some(position) = unassigned_position {
                group_positions.insert(position);
            }
        }
        for position in group_positions {
            groups[position]
                .rules
                .push(page_link(&page.id, &page.title));
        }
    }

    TopicIndexPage {
        id: PageId::new(PageKind::TopicIndex, &state.scope),
        scope: state.scope.clone(),
        title: "Topics by domain".to_string(),
        groups,
    }
}

fn requirement_domain<'a>(
    requirements: &'a [RequirementPage],
    requirement_id: &str,
) -> Option<&'a str> {
    requirements
        .iter()
        .find(|page| page.id.record_id == requirement_id)
        .and_then(|page| page.domain_id.as_deref())
}

fn page_link(id: &PageId, title: &str) -> PageLink {
    PageLink {
        target: id.clone(),
        title: title.to_string(),
    }
}
