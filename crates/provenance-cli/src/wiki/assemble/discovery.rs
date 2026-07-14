use crate::handlers::ScopeExport;
use crate::wiki::model::{
    PageId, PageKind, PageLink, RequirementPage, RulePage, SearchEntry, SearchIndexPage, Topic,
    TopicGroup, TopicIndexPage,
};
use std::collections::{BTreeMap, BTreeSet};

struct DiscoveryRequirement<'a> {
    id: &'a str,
    domain_id: Option<&'a str>,
    link: PageLink,
    statement: &'a str,
}

struct DiscoveryRule<'a> {
    link: PageLink,
    statement: &'a str,
    requirement_ids: Vec<&'a str>,
}

enum RequirementTopic<'a> {
    Domain(&'a str),
    Unassigned,
    MissingRequirement,
}

struct RequirementTopics<'a>(BTreeMap<&'a str, Option<&'a str>>);

impl<'a> RequirementTopics<'a> {
    fn new(requirements: &'a [DiscoveryRequirement<'a>]) -> Self {
        Self(
            requirements
                .iter()
                .map(|requirement| (requirement.id, requirement.domain_id))
                .collect(),
        )
    }

    fn get(&self, requirement_id: &str) -> RequirementTopic<'a> {
        match self.0.get(requirement_id) {
            Some(Some(domain_id)) => RequirementTopic::Domain(domain_id),
            Some(None) => RequirementTopic::Unassigned,
            None => RequirementTopic::MissingRequirement,
        }
    }
}

pub(super) fn build_discovery_pages(
    state: &ScopeExport,
    requirements: &[RequirementPage],
    rules: &[RulePage],
) -> (TopicIndexPage, SearchIndexPage) {
    let requirements: Vec<_> = requirements
        .iter()
        .map(|page| DiscoveryRequirement {
            id: &page.id.record_id,
            domain_id: page.domain_id.as_deref(),
            link: page_link(&page.id, &page.title),
            statement: &page.statement,
        })
        .collect();
    let rules: Vec<_> = rules
        .iter()
        .map(|page| DiscoveryRule {
            link: page_link(&page.id, &page.title),
            statement: &page.statement,
            requirement_ids: page
                .requirements
                .iter()
                .map(|link| link.target.record_id.as_str())
                .collect(),
        })
        .collect();
    let topics = topic_index(state, &requirements, &rules);
    let search = search_index(&state.scope, &requirements, &rules);
    (topics, search)
}

fn search_index(
    scope: &str,
    requirements: &[DiscoveryRequirement<'_>],
    rules: &[DiscoveryRule<'_>],
) -> SearchIndexPage {
    let entries = requirements
        .iter()
        .map(|requirement| SearchEntry {
            link: requirement.link.clone(),
            kind: PageKind::Requirement,
            statement: requirement.statement.to_string(),
        })
        .chain(rules.iter().map(|rule| SearchEntry {
            link: rule.link.clone(),
            kind: PageKind::Rule,
            statement: rule.statement.to_string(),
        }))
        .collect();
    SearchIndexPage {
        scope: scope.to_string(),
        title: "Search requirements and rules".to_string(),
        entries,
    }
}

fn topic_index(
    state: &ScopeExport,
    requirements: &[DiscoveryRequirement<'_>],
    rules: &[DiscoveryRule<'_>],
) -> TopicIndexPage {
    let mut groups = Vec::new();
    let mut positions = BTreeMap::new();
    for domain in &state.domains {
        positions.insert(domain.id.as_str().to_string(), groups.len());
        groups.push(TopicGroup {
            topic: Topic::Defined {
                id: domain.id.as_str().to_string(),
                name: domain.name.clone(),
                description: domain.description.clone(),
            },
            requirements: Vec::new(),
            rules: Vec::new(),
        });
    }

    let missing_ids: BTreeSet<String> = requirements
        .iter()
        .filter_map(|requirement| requirement.domain_id)
        .filter(|id| !positions.contains_key(*id))
        .map(str::to_string)
        .collect();
    for id in missing_ids {
        positions.insert(id.clone(), groups.len());
        groups.push(TopicGroup {
            topic: Topic::Missing { id },
            requirements: Vec::new(),
            rules: Vec::new(),
        });
    }

    let requirement_topics = RequirementTopics::new(requirements);
    let needs_unassigned = requirements
        .iter()
        .any(|requirement| requirement.domain_id.is_none())
        || rules.iter().any(|rule| {
            rule.requirement_ids.is_empty()
                || rule
                    .requirement_ids
                    .iter()
                    .any(|id| !matches!(requirement_topics.get(id), RequirementTopic::Domain(_)))
        });
    let unassigned_position = needs_unassigned.then(|| {
        let position = groups.len();
        groups.push(TopicGroup {
            topic: Topic::Unassigned,
            requirements: Vec::new(),
            rules: Vec::new(),
        });
        position
    });

    for requirement in requirements {
        let position = requirement
            .domain_id
            .map_or(unassigned_position, |id| positions.get(id).copied());
        if let Some(position) = position {
            groups[position].requirements.push(requirement.link.clone());
        }
    }
    for rule in rules {
        let mut group_positions = BTreeSet::new();
        for requirement_id in &rule.requirement_ids {
            match requirement_topics.get(requirement_id) {
                RequirementTopic::Domain(id) => {
                    if let Some(position) = positions.get(id) {
                        group_positions.insert(*position);
                    }
                }
                RequirementTopic::Unassigned | RequirementTopic::MissingRequirement => {
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

    TopicIndexPage {
        scope: state.scope.clone(),
        title: "Topics by domain".to_string(),
        groups,
    }
}

fn page_link(id: &PageId, title: &str) -> PageLink {
    PageLink {
        target: id.clone(),
        title: title.to_string(),
    }
}
