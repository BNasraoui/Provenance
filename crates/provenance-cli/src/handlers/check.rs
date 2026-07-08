use crate::cli::Status;
use crate::output::{self, OutputFormat};
use camino::Utf8PathBuf;
use provenance_core::{
    edge_validation::validate_edge_endpoint, ArtifactLink, ArtifactLinkTargetType, IdeationTarget,
    IdeationTargetType, NodeType, ScopeId, StableId,
};
use provenance_store::{layout::ProvenanceLayout, state_store::StateStore};
use std::collections::BTreeSet;

#[allow(clippy::too_many_lines)]
pub(super) fn check(repo: Utf8PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let store = StateStore::new(ProvenanceLayout::new(repo));
    let manifest = store.manifest()?;
    anyhow::ensure!(
        !manifest.scopes.is_empty(),
        "manifest must contain at least one scope"
    );
    let manifest_scopes: BTreeSet<_> = manifest
        .scopes
        .iter()
        .map(|scope| scope.id.as_str().to_string())
        .collect();

    let mut index = CheckIndex::default();
    let mut dangling = Vec::new();
    for scope in &manifest.scopes {
        let scope_id = &scope.id;
        let sources = store.list_sources(scope_id)?;
        let domains = store.list_domains(scope_id)?;
        let requirements = store.list_requirements(scope_id)?;
        let boundaries = store.list_boundaries(scope_id)?;
        let topics = store.list_topics(scope_id)?;
        let questions = store.list_questions(scope_id)?;
        let resolutions = store.list_resolutions(scope_id)?;
        let rules = store.list_rules(scope_id)?;
        let services = store.list_services(scope_id)?;
        let service_bindings = store.list_service_bindings(scope_id)?;
        let threads = store.list_threads(scope_id)?;
        let messages = store.list_messages(scope_id)?;
        let contributions = store.list_contributions(scope_id)?;
        let synthesis_packets = store.list_synthesis_packets(scope_id)?;
        let proposal_cards = store.list_proposal_cards(scope_id)?;
        let promotion_decisions = store.list_promotion_decisions(scope_id)?;

        for source in &sources {
            index.add_node(&source.scope_id, "source", &source.id);
        }
        for domain in &domains {
            index.add_node(&domain.scope_id, "domain", &domain.id);
        }
        for requirement in &requirements {
            index.add_node(&requirement.scope_id, "requirement", &requirement.id);
        }
        for boundary in &boundaries {
            index.add_node(&boundary.scope_id, "boundary", &boundary.id);
        }
        for topic in &topics {
            index.add_node(&topic.scope_id, "topic", &topic.id);
        }
        for question in &questions {
            index.add_node(&question.scope_id, "question", &question.id);
        }
        for resolution in &resolutions {
            index.add_node(&resolution.scope_id, "resolution", &resolution.id);
        }
        for rule in &rules {
            index.add_node(&rule.scope_id, "rule", &rule.id);
        }
        for service in &services {
            index.add_node(&service.scope_id, "service", &service.id);
        }
        for thread in &threads {
            index.add_node(&thread.scope_id, "thread", &thread.id);
        }
        for message in &messages {
            index.add_node(&message.scope_id, "message", &message.id);
        }
        for proposal in &proposal_cards {
            index.add_node(&proposal.scope_id, "proposal", &proposal.id);
        }

        for source in &sources {
            if let Some(superseded_by) = &source.superseded_by {
                check_scoped_reference(
                    &index,
                    &mut dangling,
                    scope_id,
                    &format!("source {}", source.id.as_str()),
                    "superseded_by",
                    "source",
                    superseded_by,
                );
            }
            check_origin_references(
                &index,
                &mut dangling,
                scope_id,
                &format!("source {}", source.id.as_str()),
                source.origin_thread.as_ref(),
                source.origin_message.as_ref(),
            );
        }
        for requirement in &requirements {
            if let Some(domain_id) = &requirement.domain_id {
                check_scoped_reference(
                    &index,
                    &mut dangling,
                    scope_id,
                    &format!("requirement {}", requirement.id.as_str()),
                    "domain",
                    "domain",
                    domain_id,
                );
            }
            for source_ref in &requirement.source_refs {
                check_scoped_reference(
                    &index,
                    &mut dangling,
                    scope_id,
                    &format!("requirement {}", requirement.id.as_str()),
                    "source_ref",
                    "source",
                    &source_ref.source_id,
                );
            }
            check_origin_references(
                &index,
                &mut dangling,
                scope_id,
                &format!("requirement {}", requirement.id.as_str()),
                requirement.origin_thread.as_ref(),
                requirement.origin_message.as_ref(),
            );
        }
        for boundary in &boundaries {
            check_scoped_reference(
                &index,
                &mut dangling,
                scope_id,
                &format!("boundary {}", boundary.id.as_str()),
                "requirement",
                "requirement",
                &boundary.requirement_id,
            );
            if let Some(source_ref) = &boundary.source_ref {
                check_scoped_reference(
                    &index,
                    &mut dangling,
                    scope_id,
                    &format!("boundary {}", boundary.id.as_str()),
                    "source_ref",
                    "source",
                    &source_ref.source_id,
                );
            }
        }
        for topic in &topics {
            check_scoped_reference(
                &index,
                &mut dangling,
                scope_id,
                &format!("topic {}", topic.id.as_str()),
                "requirement",
                "requirement",
                &topic.requirement_id,
            );
            check_artifact_links(
                &index,
                &mut dangling,
                scope_id,
                &format!("topic {}", topic.id.as_str()),
                &topic.links,
            );
        }
        for question in &questions {
            check_scoped_reference(
                &index,
                &mut dangling,
                scope_id,
                &format!("question {}", question.id.as_str()),
                "topic",
                "topic",
                &question.topic_id,
            );
            check_scoped_reference(
                &index,
                &mut dangling,
                scope_id,
                &format!("question {}", question.id.as_str()),
                "requirement",
                "requirement",
                &question.requirement_id,
            );
            if let Some(resolution_id) = &question.resolution_id {
                check_scoped_reference(
                    &index,
                    &mut dangling,
                    scope_id,
                    &format!("question {}", question.id.as_str()),
                    "resolution",
                    "resolution",
                    resolution_id,
                );
            }
            check_artifact_links(
                &index,
                &mut dangling,
                scope_id,
                &format!("question {}", question.id.as_str()),
                &question.links,
            );
        }
        for resolution in &resolutions {
            if let Some(superseded_by) = &resolution.superseded_by {
                check_scoped_reference(
                    &index,
                    &mut dangling,
                    scope_id,
                    &format!("resolution {}", resolution.id.as_str()),
                    "superseded_by",
                    "resolution",
                    superseded_by,
                );
            }
            check_origin_references(
                &index,
                &mut dangling,
                scope_id,
                &format!("resolution {}", resolution.id.as_str()),
                resolution.origin_thread.as_ref(),
                resolution.origin_message.as_ref(),
            );
        }
        for rule in &rules {
            check_origin_references(
                &index,
                &mut dangling,
                scope_id,
                &format!("rule {}", rule.id.as_str()),
                rule.origin_thread.as_ref(),
                rule.origin_message.as_ref(),
            );
        }
        for binding in &service_bindings {
            check_scoped_reference(
                &index,
                &mut dangling,
                scope_id,
                &format!("service binding {}", binding.id.as_str()),
                "rule",
                "rule",
                &binding.rule_id,
            );
            check_scoped_reference(
                &index,
                &mut dangling,
                scope_id,
                &format!("service binding {}", binding.id.as_str()),
                "service",
                "service",
                &binding.service_id,
            );
        }
        for thread in &threads {
            check_scoped_reference(
                &index,
                &mut dangling,
                scope_id,
                &format!("thread {}", thread.id.as_str()),
                "parent",
                node_type_name(thread.parent.node_type),
                &thread.parent.node_id,
            );
        }
        for message in &messages {
            check_scoped_reference(
                &index,
                &mut dangling,
                scope_id,
                &format!("message {}", message.id.as_str()),
                "thread",
                "thread",
                &message.thread_id,
            );
        }
        for contribution in &contributions {
            check_ideation_target(
                &index,
                &mut dangling,
                scope_id,
                &format!("contribution {}", contribution.id.as_str()),
                &contribution.target,
            );
        }
        for synthesis_packet in &synthesis_packets {
            check_ideation_target(
                &index,
                &mut dangling,
                scope_id,
                &format!("synthesis packet {}", synthesis_packet.id.as_str()),
                &synthesis_packet.target,
            );
        }
        for proposal in &proposal_cards {
            check_ideation_target(
                &index,
                &mut dangling,
                scope_id,
                &format!("proposal {}", proposal.id.as_str()),
                &proposal.traceability.target,
            );
            for source_id in &proposal.traceability.source_ids {
                check_scoped_reference(
                    &index,
                    &mut dangling,
                    scope_id,
                    &format!("proposal {}", proposal.id.as_str()),
                    "source_id",
                    "source",
                    source_id,
                );
            }
            if let Some(duplicate_of) = &proposal.duplicate_of {
                check_scoped_reference(
                    &index,
                    &mut dangling,
                    scope_id,
                    &format!("proposal {}", proposal.id.as_str()),
                    "duplicate_of",
                    "proposal",
                    duplicate_of,
                );
            }
            if let Some(superseded_by) = &proposal.superseded_by {
                check_scoped_reference(
                    &index,
                    &mut dangling,
                    scope_id,
                    &format!("proposal {}", proposal.id.as_str()),
                    "superseded_by",
                    "proposal",
                    superseded_by,
                );
            }
        }
        for promotion_decision in &promotion_decisions {
            check_scoped_reference(
                &index,
                &mut dangling,
                scope_id,
                &format!("promotion decision {}", promotion_decision.id.as_str()),
                "proposal",
                "proposal",
                &promotion_decision.proposal_id,
            );
        }
    }

    for edge in store.list_edges()? {
        if !manifest_scopes.contains(edge.scope_id.as_str()) {
            dangling.push(format!(
                "edge {} is in unknown scope {}",
                edge.id.as_str(),
                edge.scope_id.as_str()
            ));
        }
        if let Err(error) = validate_edge_endpoint(edge.edge_type, edge.from_type, edge.to_type) {
            dangling.push(format!(
                "edge {} has invalid endpoint: {error}",
                edge.id.as_str()
            ));
        }
        if !index.has_global_node(edge.from_type, &edge.from_id) {
            dangling.push(format!(
                "edge {} has dangling reference: from {} {}",
                edge.id.as_str(),
                node_type_name(edge.from_type),
                edge.from_id.as_str()
            ));
        }
        if !index.has_global_node(edge.to_type, &edge.to_id) {
            dangling.push(format!(
                "edge {} has dangling reference: to {} {}",
                edge.id.as_str(),
                node_type_name(edge.to_type),
                edge.to_id.as_str()
            ));
        }
    }

    anyhow::ensure!(
        dangling.is_empty(),
        "dangling reference(s):\n- {}",
        dangling.join("\n- ")
    );
    output::print(format, &Status { status: "ok" })
}

#[derive(Default)]
struct CheckIndex {
    global_nodes: BTreeSet<(String, String)>,
    scoped_nodes: BTreeSet<(String, String, String)>,
}

impl CheckIndex {
    fn add_node(&mut self, scope_id: &ScopeId, node_type: &str, id: &StableId) {
        let node_type = node_type.to_string();
        let id = id.as_str().to_string();
        self.global_nodes.insert((node_type.clone(), id.clone()));
        self.scoped_nodes
            .insert((scope_id.as_str().to_string(), node_type, id));
    }

    fn has_global_node(&self, node_type: NodeType, id: &StableId) -> bool {
        self.global_nodes.contains(&(
            node_type_name(node_type).to_string(),
            id.as_str().to_string(),
        ))
    }

    fn has_scoped_node(&self, scope_id: &ScopeId, node_type: &str, id: &StableId) -> bool {
        self.scoped_nodes.contains(&(
            scope_id.as_str().to_string(),
            node_type.to_string(),
            id.as_str().to_string(),
        ))
    }
}

fn check_origin_references(
    index: &CheckIndex,
    dangling: &mut Vec<String>,
    scope_id: &ScopeId,
    owner: &str,
    origin_thread: Option<&StableId>,
    origin_message: Option<&StableId>,
) {
    if let Some(origin_thread) = origin_thread {
        check_scoped_reference(
            index,
            dangling,
            scope_id,
            owner,
            "origin_thread",
            "thread",
            origin_thread,
        );
    }
    if let Some(origin_message) = origin_message {
        check_scoped_reference(
            index,
            dangling,
            scope_id,
            owner,
            "origin_message",
            "message",
            origin_message,
        );
    }
}

fn check_artifact_links(
    index: &CheckIndex,
    dangling: &mut Vec<String>,
    scope_id: &ScopeId,
    owner: &str,
    links: &[ArtifactLink],
) {
    for link in links {
        let node_type = artifact_link_target_name(link.target_type);
        check_scoped_reference(
            index,
            dangling,
            scope_id,
            owner,
            "link",
            node_type,
            &link.target_id,
        );
    }
}

fn check_ideation_target(
    index: &CheckIndex,
    dangling: &mut Vec<String>,
    scope_id: &ScopeId,
    owner: &str,
    target: &IdeationTarget,
) {
    let node_type = ideation_target_type_name(target.artifact_type);
    check_scoped_reference(
        index,
        dangling,
        scope_id,
        owner,
        "target",
        node_type,
        &target.artifact_id,
    );
}

fn check_scoped_reference(
    index: &CheckIndex,
    dangling: &mut Vec<String>,
    scope_id: &ScopeId,
    owner: &str,
    relation: &str,
    target_type: &str,
    target_id: &StableId,
) {
    if !index.has_scoped_node(scope_id, target_type, target_id) {
        dangling.push(format!(
            "{owner} has dangling reference: {relation} {target_type} {}",
            target_id.as_str()
        ));
    }
}

const fn node_type_name(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Source => "source",
        NodeType::Requirement => "requirement",
        NodeType::Resolution => "resolution",
        NodeType::Rule => "rule",
        NodeType::Topic => "topic",
        NodeType::Question => "question",
    }
}

const fn artifact_link_target_name(target_type: ArtifactLinkTargetType) -> &'static str {
    match target_type {
        ArtifactLinkTargetType::Source => "source",
        ArtifactLinkTargetType::Requirement => "requirement",
        ArtifactLinkTargetType::Resolution => "resolution",
        ArtifactLinkTargetType::Rule => "rule",
    }
}

const fn ideation_target_type_name(target_type: IdeationTargetType) -> &'static str {
    match target_type {
        IdeationTargetType::Source => "source",
        IdeationTargetType::Requirement => "requirement",
        IdeationTargetType::Resolution => "resolution",
        IdeationTargetType::Rule => "rule",
        IdeationTargetType::Topic => "topic",
        IdeationTargetType::Question => "question",
        IdeationTargetType::Domain => "domain",
    }
}
