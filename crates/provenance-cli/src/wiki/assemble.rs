//! Assembles the wiki page model from Provenance state.
//!
//! Pure joins over the scope export: edges are matched against record
//! vectors by stable id, in record order, so output is deterministic for a
//! given state. Every hole found on the way (dangling references, missing
//! sources, resolved requirements without rules, orphaned records) becomes
//! a gap notice or an orphan entry instead of being dropped.

use crate::handlers::ScopeExport;
use crate::wiki::links::{detect_remote_url, EvidenceRef, LinkResolver};
use crate::wiki::model::{
    CorpusCounts, DecisionSection, EvidenceThread, FieldNote, GapKind, GapNotice, IndexEntry,
    InputCitation, LineageEntry, OrphanReport, PageId, PageKind, PageLink, RequirementPage,
    ResolutionPage, RuleCard, RulePage, ScopeIndexPage, SourceCitation, SourcePage, WikiCorpus,
};
use camino::Utf8PathBuf;
use provenance_core::{
    Edge, EdgeType, Message, NodeType, Requirement, RequirementStatus, Resolution, ResolutionInput,
    ResolutionStatus, Rule, Source, StableId, Thread,
};
use std::collections::BTreeSet;

/// Loads the scope's state from disk and assembles the wiki corpus, using
/// the repo's `origin` remote (if any) to build evidence links.
pub fn load_corpus(repo: Utf8PathBuf, scope: String) -> anyhow::Result<WikiCorpus> {
    let remote_url = detect_remote_url(repo.as_std_path());
    let state = crate::handlers::export_scope(repo, scope)?;
    let resolver = LinkResolver::new(remote_url.as_deref());
    Ok(build_corpus(&state, &resolver))
}

/// Assembles the wiki corpus from already-loaded scope state.
pub fn build_corpus(state: &ScopeExport, resolver: &LinkResolver) -> WikiCorpus {
    let assembler = Assembler { state, resolver };
    WikiCorpus {
        scope: state.scope.clone(),
        index: assembler.index_page(),
        requirements: state
            .requirements
            .iter()
            .map(|requirement| assembler.requirement_page(requirement))
            .collect(),
        resolutions: state
            .resolutions
            .iter()
            .map(|resolution| assembler.resolution_page(resolution))
            .collect(),
        rules: state
            .rules
            .iter()
            .map(|rule| assembler.rule_page(rule))
            .collect(),
        sources: state
            .sources
            .iter()
            .map(|source| assembler.source_page(source))
            .collect(),
    }
}

fn requirement_link(requirement: &Requirement) -> PageLink {
    PageLink {
        target: PageId::new(PageKind::Requirement, requirement.id.as_str()),
        title: requirement.statement.clone(),
    }
}

fn resolution_link(resolution: &Resolution) -> PageLink {
    PageLink {
        target: PageId::new(PageKind::Resolution, resolution.id.as_str()),
        title: resolution.title.clone(),
    }
}

fn rule_link(rule: &Rule) -> PageLink {
    PageLink {
        target: PageId::new(PageKind::Rule, rule.id.as_str()),
        title: rule_title(rule),
    }
}

fn rule_title(rule: &Rule) -> String {
    rule.name.clone().unwrap_or_else(|| rule.rule_code.clone())
}

const fn node_type_word(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Source => "source",
        NodeType::Requirement => "requirement",
        NodeType::Resolution => "resolution",
        NodeType::Rule => "rule",
        NodeType::Topic => "topic",
        NodeType::Question => "question",
    }
}

fn source_link(source: &Source) -> PageLink {
    PageLink {
        target: PageId::new(PageKind::Source, source.id.as_str()),
        title: source.name.clone(),
    }
}

struct Assembler<'a> {
    state: &'a ScopeExport,
    resolver: &'a LinkResolver,
}

impl<'a> Assembler<'a> {
    fn edges(&self) -> impl Iterator<Item = &'a Edge> {
        let scope = self.state.scope.as_str();
        self.state
            .edges
            .iter()
            .filter(move |edge| edge.scope_id.as_str() == scope)
    }

    /// Matches an edge by its full identity: type, kind, and id on both
    /// ends. Stable ids are not namespaced per record kind, so matching on
    /// `from_id`/`to_id` alone would let a `Source` and a `Resolution` that
    /// happen to share an id get cross-wired; `from_type`/`to_type` must be
    /// checked too.
    fn edge_exists(
        &self,
        edge_type: EdgeType,
        from_type: NodeType,
        from_id: &StableId,
        to_type: NodeType,
        to_id: &StableId,
    ) -> bool {
        self.edges().any(|edge| {
            edge.edge_type == edge_type
                && edge.from_type == from_type
                && edge.from_id == *from_id
                && edge.to_type == to_type
                && edge.to_id == *to_id
        })
    }

    fn find_requirement(&self, id: &StableId) -> Option<&'a Requirement> {
        self.state
            .requirements
            .iter()
            .find(|requirement| requirement.id == *id)
    }

    fn find_source(&self, id: &StableId) -> Option<&'a Source> {
        self.state.sources.iter().find(|source| source.id == *id)
    }

    /// Whether a thread's parent record still exists. Topics and questions
    /// are not modeled as wiki pages at all, so they are not this check's
    /// concern; every other kind must resolve to a real record.
    fn thread_parent_exists(&self, thread: &Thread) -> bool {
        let id = &thread.parent.node_id;
        match thread.parent.node_type {
            NodeType::Requirement => self.find_requirement(id).is_some(),
            NodeType::Resolution => self.state.resolutions.iter().any(|r| r.id == *id),
            NodeType::Rule => self.state.rules.iter().any(|rule| rule.id == *id),
            NodeType::Source => self.find_source(id).is_some(),
            NodeType::Topic | NodeType::Question => true,
        }
    }

    fn parent_of(&self, requirement_id: &StableId) -> Option<&'a Requirement> {
        let mut parent_ids: Vec<&StableId> = self
            .edges()
            .filter(|edge| {
                edge.edge_type == EdgeType::RefinesInto
                    && edge.from_type == NodeType::Requirement
                    && edge.to_type == NodeType::Requirement
                    && edge.to_id == *requirement_id
            })
            .map(|edge| &edge.from_id)
            .collect();
        parent_ids.sort_by_key(|id| id.as_str());
        parent_ids
            .into_iter()
            .find_map(|id| self.find_requirement(id))
    }

    fn has_parent_edge(&self, requirement_id: &StableId) -> bool {
        self.edges().any(|edge| {
            edge.edge_type == EdgeType::RefinesInto
                && edge.from_type == NodeType::Requirement
                && edge.to_type == NodeType::Requirement
                && edge.to_id == *requirement_id
        })
    }

    fn resolving_resolutions(&self, requirement_id: &StableId) -> Vec<&'a Resolution> {
        self.state
            .resolutions
            .iter()
            .filter(|resolution| {
                self.edge_exists(
                    EdgeType::Resolves,
                    NodeType::Resolution,
                    &resolution.id,
                    NodeType::Requirement,
                    requirement_id,
                )
            })
            .collect()
    }

    fn produced_rules_for_requirement(&self, requirement_id: &StableId) -> Vec<&'a Rule> {
        let resolution_ids: BTreeSet<&str> = self
            .resolving_resolutions(requirement_id)
            .into_iter()
            .map(|resolution| resolution.id.as_str())
            .collect();
        self.state
            .rules
            .iter()
            .filter(|rule| {
                self.edges().any(|edge| {
                    edge.edge_type == EdgeType::Produces
                        && edge.to_type == NodeType::Rule
                        && edge.to_id == rule.id
                        && ((edge.from_type == NodeType::Requirement
                            && edge.from_id == *requirement_id)
                            || (edge.from_type == NodeType::Resolution
                                && resolution_ids.contains(edge.from_id.as_str())))
                })
            })
            .collect()
    }

    fn produced_rules_for_resolution(&self, resolution_id: &StableId) -> Vec<&'a Rule> {
        self.state
            .rules
            .iter()
            .filter(|rule| {
                self.edge_exists(
                    EdgeType::Produces,
                    NodeType::Resolution,
                    resolution_id,
                    NodeType::Rule,
                    &rule.id,
                )
            })
            .collect()
    }

    fn lineage(&self, requirement: &'a Requirement) -> Vec<LineageEntry> {
        let mut chain = vec![requirement];
        let mut visited: BTreeSet<&str> = BTreeSet::from([requirement.id.as_str()]);
        let mut current = requirement;
        while let Some(parent) = self.parent_of(&current.id) {
            if !visited.insert(parent.id.as_str()) {
                break;
            }
            chain.push(parent);
            current = parent;
        }
        chain.reverse();
        let last = chain.len() - 1;
        chain
            .into_iter()
            .enumerate()
            .map(|(index, entry)| LineageEntry {
                link: requirement_link(entry),
                is_current: index == last,
            })
            .collect()
    }

    fn input_citation(&self, input: &ResolutionInput) -> InputCitation {
        InputCitation {
            input_type: input.input_type.clone(),
            summary: input.summary.clone(),
            reference: self.resolver.resolve(&input.reference),
        }
    }

    fn decision_section(&self, resolution: &Resolution) -> DecisionSection {
        DecisionSection {
            link: resolution_link(resolution),
            status: resolution.status.clone(),
            position: resolution.position.clone(),
            rationale: resolution.rationale.clone(),
            context: resolution.context.clone(),
            enforcement: resolution.enforcement.clone(),
            confidence: resolution.confidence,
            inputs: resolution
                .inputs
                .iter()
                .map(|input| self.input_citation(input))
                .collect(),
            made_by: resolution.made_by.clone(),
            approved_by: resolution.approved_by.clone(),
            approved_at: resolution.approved_at,
        }
    }

    fn rule_evidence(&self, rule: &Rule) -> Vec<EvidenceRef> {
        rule.source_document
            .as_ref()
            .map(|document| {
                vec![self
                    .resolver
                    .resolve_document(document, rule.source_section.as_deref(), None)]
            })
            .unwrap_or_default()
    }

    fn rule_card(&self, rule: &Rule) -> RuleCard {
        RuleCard {
            link: rule_link(rule),
            rule_code: rule.rule_code.clone(),
            name: rule.name.clone(),
            statement: rule.statement.clone(),
            status: rule.status.clone(),
            severity: rule.severity.clone(),
            modality: rule.modality.clone(),
            evidence: self.rule_evidence(rule),
        }
    }

    fn source_reference_link(&self, source: &Source) -> Option<EvidenceRef> {
        source.reference.as_ref().map(|reference| {
            self.resolver
                .resolve_at(reference, source.commit_pin.as_deref())
        })
    }

    fn source_citation(&self, source: &Source, clause: Option<String>) -> SourceCitation {
        SourceCitation {
            link: source_link(source),
            source_type: source.source_type.clone(),
            clause,
            reference: self.source_reference_link(source),
        }
    }

    /// Joins a requirement's inline source refs and `references` edges into
    /// citations, reporting dangling refs as gaps instead of dropping them.
    fn requirement_sources(
        &self,
        requirement: &Requirement,
    ) -> (Vec<SourceCitation>, Vec<GapNotice>) {
        let mut citations = Vec::new();
        let mut gaps = Vec::new();
        let mut cited: BTreeSet<&str> = BTreeSet::new();
        for reference in &requirement.source_refs {
            match self.find_source(&reference.source_id) {
                Some(source) => {
                    if cited.insert(source.id.as_str()) {
                        citations.push(self.source_citation(source, reference.clause.clone()));
                    }
                }
                None => gaps.push(GapNotice {
                    kind: GapKind::DanglingReference,
                    detail: format!(
                        "source ref points at missing source {}",
                        reference.source_id.as_str()
                    ),
                }),
            }
        }
        for source in &self.state.sources {
            if cited.contains(source.id.as_str()) {
                continue;
            }
            let edge = self.edges().find(|edge| {
                edge.edge_type == EdgeType::References
                    && edge.from_type == NodeType::Source
                    && edge.from_id == source.id
                    && edge.to_type == NodeType::Requirement
                    && edge.to_id == requirement.id
            });
            if let Some(edge) = edge {
                cited.insert(source.id.as_str());
                citations.push(self.source_citation(source, edge.label.clone()));
            }
        }
        (citations, gaps)
    }

    fn evidence_thread(&self, thread: &Thread) -> EvidenceThread {
        let mut messages: Vec<&Message> = self
            .state
            .messages
            .iter()
            .filter(|message| message.thread_id == thread.id)
            .collect();
        messages.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.as_str().cmp(b.id.as_str()))
        });
        EvidenceThread {
            thread_id: thread.id.as_str().to_string(),
            parent_type: thread.parent.node_type,
            parent_id: thread.parent.node_id.as_str().to_string(),
            status: thread.status.clone(),
            messages: messages
                .into_iter()
                .map(|message| FieldNote {
                    message_id: message.id.as_str().to_string(),
                    role: message.role.clone(),
                    created_at: message.created_at,
                    body: message.body.clone(),
                    refs: self.resolver.annotate(&message.body),
                })
                .collect(),
        }
    }

    fn threads_for(&self, node_type: NodeType, node_id: &StableId) -> Vec<EvidenceThread> {
        self.state
            .threads
            .iter()
            .filter(|thread| {
                thread.parent.node_type == node_type && thread.parent.node_id == *node_id
            })
            .map(|thread| self.evidence_thread(thread))
            .collect()
    }

    fn requirement_page(&self, requirement: &'a Requirement) -> RequirementPage {
        let resolving = self.resolving_resolutions(&requirement.id);
        let decisions: Vec<DecisionSection> = resolving
            .iter()
            .map(|resolution| self.decision_section(resolution))
            .collect();
        let produced_rules: Vec<RuleCard> = self
            .produced_rules_for_requirement(&requirement.id)
            .into_iter()
            .map(|rule| self.rule_card(rule))
            .collect();
        let (sources, mut gaps) = self.requirement_sources(requirement);
        if sources.is_empty() {
            gaps.push(GapNotice {
                kind: GapKind::MissingSourceRefs,
                detail: "no source refs recorded on this requirement".to_string(),
            });
        }
        let resolved = requirement.status == RequirementStatus::Resolved;
        if resolved && decisions.is_empty() {
            gaps.push(GapNotice {
                kind: GapKind::NoResolvingDecision,
                detail: "resolved requirement has no resolving decision".to_string(),
            });
        }
        if (resolved || !decisions.is_empty()) && produced_rules.is_empty() {
            gaps.push(GapNotice {
                kind: GapKind::NoProducedRules,
                detail: "resolved requirement has no downstream rule".to_string(),
            });
        }
        let mut threads = self.threads_for(NodeType::Requirement, &requirement.id);
        for resolution in &resolving {
            threads.extend(self.threads_for(NodeType::Resolution, &resolution.id));
        }
        RequirementPage {
            id: PageId::new(PageKind::Requirement, requirement.id.as_str()),
            title: requirement.statement.clone(),
            status: requirement.status.clone(),
            statement: requirement.statement.clone(),
            description: requirement.description.clone(),
            fog: requirement.fog.clone(),
            domain_id: requirement
                .domain_id
                .as_ref()
                .map(|id| id.as_str().to_string()),
            back_link: self.parent_of(&requirement.id).map(requirement_link),
            lineage: self.lineage(requirement),
            decisions,
            produced_rules,
            children: self
                .state
                .requirements
                .iter()
                .filter(|child| {
                    self.edge_exists(
                        EdgeType::RefinesInto,
                        NodeType::Requirement,
                        &requirement.id,
                        NodeType::Requirement,
                        &child.id,
                    )
                })
                .map(requirement_link)
                .collect(),
            sources,
            gaps,
            threads,
        }
    }

    fn resolution_page(&self, resolution: &'a Resolution) -> ResolutionPage {
        let resolves: Vec<PageLink> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| {
                self.edge_exists(
                    EdgeType::Resolves,
                    NodeType::Resolution,
                    &resolution.id,
                    NodeType::Requirement,
                    &requirement.id,
                )
            })
            .map(requirement_link)
            .collect();
        let spawned: Vec<PageLink> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| {
                self.edge_exists(
                    EdgeType::Spawns,
                    NodeType::Resolution,
                    &resolution.id,
                    NodeType::Requirement,
                    &requirement.id,
                )
            })
            .map(requirement_link)
            .collect();
        let produced_rules: Vec<RuleCard> = self
            .produced_rules_for_resolution(&resolution.id)
            .into_iter()
            .map(|rule| self.rule_card(rule))
            .collect();
        let mut gaps = Vec::new();
        if resolves.is_empty() {
            gaps.push(GapNotice {
                kind: GapKind::OrphanResolution,
                detail: "resolution does not resolve any requirement".to_string(),
            });
        }
        if resolution.status == ResolutionStatus::Approved && produced_rules.is_empty() {
            gaps.push(GapNotice {
                kind: GapKind::NoProducedRules,
                detail: "approved resolution produced no rules".to_string(),
            });
        }
        let superseded_by = resolution.superseded_by.as_ref().and_then(|id| {
            let successor = self
                .state
                .resolutions
                .iter()
                .find(|candidate| candidate.id == *id);
            if successor.is_none() {
                gaps.push(GapNotice {
                    kind: GapKind::DanglingReference,
                    detail: format!("superseded by missing resolution {}", id.as_str()),
                });
            }
            successor.map(resolution_link)
        });
        ResolutionPage {
            id: PageId::new(PageKind::Resolution, resolution.id.as_str()),
            title: resolution.title.clone(),
            status: resolution.status.clone(),
            position: resolution.position.clone(),
            rationale: resolution.rationale.clone(),
            context: resolution.context.clone(),
            enforcement: resolution.enforcement.clone(),
            confidence: resolution.confidence,
            inputs: resolution
                .inputs
                .iter()
                .map(|input| self.input_citation(input))
                .collect(),
            made_by: resolution.made_by.clone(),
            approved_by: resolution.approved_by.clone(),
            approved_at: resolution.approved_at,
            review_on: resolution.review_on.clone(),
            superseded_by,
            resolves,
            spawned,
            produced_rules,
            gaps,
            threads: self.threads_for(NodeType::Resolution, &resolution.id),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn rule_page(&self, rule: &'a Rule) -> RulePage {
        let producing_resolutions: Vec<&Resolution> = self
            .state
            .resolutions
            .iter()
            .filter(|resolution| {
                self.edge_exists(
                    EdgeType::Produces,
                    NodeType::Resolution,
                    &resolution.id,
                    NodeType::Rule,
                    &rule.id,
                )
            })
            .collect();
        let producing_requirements: Vec<&Requirement> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| {
                self.edge_exists(
                    EdgeType::Produces,
                    NodeType::Requirement,
                    &requirement.id,
                    NodeType::Rule,
                    &rule.id,
                )
            })
            .collect();
        let produced_by: Vec<PageLink> = producing_resolutions
            .iter()
            .copied()
            .map(resolution_link)
            .chain(producing_requirements.iter().copied().map(requirement_link))
            .collect();
        let mut requirement_ids: BTreeSet<&str> = producing_requirements
            .iter()
            .map(|requirement| requirement.id.as_str())
            .collect();
        for resolution in &producing_resolutions {
            for requirement in &self.state.requirements {
                if self.edge_exists(
                    EdgeType::Resolves,
                    NodeType::Resolution,
                    &resolution.id,
                    NodeType::Requirement,
                    &requirement.id,
                ) {
                    requirement_ids.insert(requirement.id.as_str());
                }
            }
        }
        let upstream_requirements: Vec<&Requirement> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| requirement_ids.contains(requirement.id.as_str()))
            .collect();
        let sources: Vec<PageLink> = self
            .state
            .sources
            .iter()
            .filter(|source| {
                upstream_requirements.iter().any(|requirement| {
                    self.edge_exists(
                        EdgeType::References,
                        NodeType::Source,
                        &source.id,
                        NodeType::Requirement,
                        &requirement.id,
                    ) || requirement
                        .source_refs
                        .iter()
                        .any(|reference| reference.source_id == source.id)
                })
            })
            .map(source_link)
            .collect();
        let mut gaps = Vec::new();
        if produced_by.is_empty() {
            gaps.push(GapNotice {
                kind: GapKind::OrphanRule,
                detail: "no resolution or requirement produces this rule".to_string(),
            });
        }
        RulePage {
            id: PageId::new(PageKind::Rule, rule.id.as_str()),
            title: rule_title(rule),
            rule_code: rule.rule_code.clone(),
            statement: rule.statement.clone(),
            description: rule.description.clone(),
            status: rule.status.clone(),
            severity: rule.severity.clone(),
            modality: rule.modality.clone(),
            rule_type: rule.rule_type.clone(),
            confidence: rule.confidence,
            extraction_method: rule.extraction_method.clone(),
            source_document: rule.source_document.clone(),
            source_section: rule.source_section.clone(),
            evidence: self.rule_evidence(rule),
            produced_by,
            requirements: upstream_requirements
                .into_iter()
                .map(requirement_link)
                .collect(),
            sources,
            gaps,
            threads: self.threads_for(NodeType::Rule, &rule.id),
        }
    }

    fn source_page(&self, source: &'a Source) -> SourcePage {
        let referenced_requirements: Vec<PageLink> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| {
                self.edge_exists(
                    EdgeType::References,
                    NodeType::Source,
                    &source.id,
                    NodeType::Requirement,
                    &requirement.id,
                ) || requirement
                    .source_refs
                    .iter()
                    .any(|reference| reference.source_id == source.id)
            })
            .map(requirement_link)
            .collect();
        let mut gaps = Vec::new();
        if referenced_requirements.is_empty() {
            gaps.push(GapNotice {
                kind: GapKind::UnreferencedSource,
                detail: "no requirement references this source".to_string(),
            });
        }
        let superseded_by = source.superseded_by.as_ref().and_then(|id| {
            let successor = self.find_source(id);
            if successor.is_none() {
                gaps.push(GapNotice {
                    kind: GapKind::DanglingReference,
                    detail: format!("superseded by missing source {}", id.as_str()),
                });
            }
            successor.map(source_link)
        });
        SourcePage {
            id: PageId::new(PageKind::Source, source.id.as_str()),
            title: source.name.clone(),
            source_type: source.source_type.clone(),
            url: source.url.clone(),
            reference: self.source_reference_link(source),
            commit_pin: source.commit_pin.clone(),
            effective_date: source.effective_date,
            review_date: source.review_date,
            superseded_by,
            referenced_requirements,
            gaps,
            threads: self.threads_for(NodeType::Source, &source.id),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn index_page(&self) -> ScopeIndexPage {
        let roots: Vec<IndexEntry> = self
            .state
            .requirements
            .iter()
            .filter(|requirement| !self.has_parent_edge(&requirement.id))
            .map(|requirement| IndexEntry {
                link: requirement_link(requirement),
                status: requirement.status.clone(),
                children: self
                    .edges()
                    .filter(|edge| {
                        edge.edge_type == EdgeType::RefinesInto
                            && edge.from_type == NodeType::Requirement
                            && edge.from_id == requirement.id
                    })
                    .count(),
                resolutions: self.resolving_resolutions(&requirement.id).len(),
                rules: self.produced_rules_for_requirement(&requirement.id).len(),
            })
            .collect();
        let mut gaps = Vec::new();
        for requirement in &self.state.requirements {
            let resolved = requirement.status == RequirementStatus::Resolved;
            let decisions = self.resolving_resolutions(&requirement.id).len();
            if resolved && decisions == 0 {
                gaps.push(GapNotice {
                    kind: GapKind::NoResolvingDecision,
                    detail: format!(
                        "requirement {} is resolved but has no resolving decision",
                        requirement.id.as_str()
                    ),
                });
            }
            if (resolved || decisions > 0)
                && self
                    .produced_rules_for_requirement(&requirement.id)
                    .is_empty()
            {
                gaps.push(GapNotice {
                    kind: GapKind::NoProducedRules,
                    detail: format!(
                        "requirement {} has no downstream rule",
                        requirement.id.as_str()
                    ),
                });
            }
        }
        // A thread's parent record can be deleted or renamed after the
        // thread was recorded. threads_for() is only ever queried with the
        // id of a record that was actually found, so a thread like this
        // would otherwise be dropped with no trace instead of becoming a
        // gap like every other kind of dangling reference.
        for thread in &self.state.threads {
            if !self.thread_parent_exists(thread) {
                gaps.push(GapNotice {
                    kind: GapKind::DanglingReference,
                    detail: format!(
                        "thread {} points at missing {} {}",
                        thread.id.as_str(),
                        node_type_word(thread.parent.node_type),
                        thread.parent.node_id.as_str()
                    ),
                });
            }
        }
        let orphans = OrphanReport {
            rules: self
                .state
                .rules
                .iter()
                .filter(|rule| {
                    !self.edges().any(|edge| {
                        edge.edge_type == EdgeType::Produces
                            && edge.to_type == NodeType::Rule
                            && edge.to_id == rule.id
                    })
                })
                .map(rule_link)
                .collect(),
            resolutions: self
                .state
                .resolutions
                .iter()
                .filter(|resolution| {
                    !self.edges().any(|edge| {
                        edge.edge_type == EdgeType::Resolves
                            && edge.from_type == NodeType::Resolution
                            && edge.from_id == resolution.id
                    })
                })
                .map(resolution_link)
                .collect(),
            sources: self
                .state
                .sources
                .iter()
                .filter(|source| {
                    !self.edges().any(|edge| {
                        edge.edge_type == EdgeType::References
                            && edge.from_type == NodeType::Source
                            && edge.from_id == source.id
                    }) && !self.state.requirements.iter().any(|requirement| {
                        requirement
                            .source_refs
                            .iter()
                            .any(|reference| reference.source_id == source.id)
                    })
                })
                .map(source_link)
                .collect(),
        };
        ScopeIndexPage {
            id: PageId::new(PageKind::ScopeIndex, self.state.scope.as_str()),
            scope: self.state.scope.clone(),
            title: self.state.scope.clone(),
            counts: CorpusCounts {
                sources: self.state.sources.len(),
                requirements: self.state.requirements.len(),
                resolutions: self.state.resolutions.len(),
                rules: self.state.rules.len(),
            },
            roots,
            gaps,
            orphans,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::ScopeExport;
    use crate::wiki::links::LinkResolver;
    use crate::wiki::model::{CorpusCounts, GapKind, PageKind};
    use camino::Utf8PathBuf;
    use provenance_core::{
        Edge, EdgeType, Message, MessageRole, NodeType, Requirement, RequirementStatus, Resolution,
        ResolutionInput, ResolutionInputType, ResolutionStatus, Rule, RuleModality, RuleSeverity,
        RuleStatus, SchemaVersion, ScopeId, Source, SourceReference, SourceType, StableId, Thread,
        ThreadParent, ThreadStatus,
    };

    fn sid(value: &str) -> StableId {
        StableId::new(value).unwrap()
    }

    fn scope_id() -> ScopeId {
        ScopeId::new("default").unwrap()
    }

    fn requirement(
        id: &str,
        statement: &str,
        status: RequirementStatus,
        source_refs: Vec<SourceReference>,
    ) -> Requirement {
        Requirement {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            statement: statement.to_string(),
            description: None,
            fog: None,
            status,
            domain_id: None,
            source_refs,
            origin_thread: None,
            origin_message: None,
        }
    }

    fn resolution(id: &str, title: &str, inputs: Vec<ResolutionInput>) -> Resolution {
        Resolution {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            title: title.to_string(),
            position: "Adopt the split".to_string(),
            rationale: "Atomicity equals drift detectability".to_string(),
            status: ResolutionStatus::Approved,
            context: Some("Codebase scan".to_string()),
            enforcement: Some("Specification".to_string()),
            confidence: Some(0.97),
            inputs,
            made_by: Some("Ben Nasraoui".to_string()),
            approved_by: Some("Ben Nasraoui".to_string()),
            approved_at: Some(1_745_000_000),
            superseded_by: None,
            review_on: None,
            review_triggers: serde_json::json!([]),
            origin_thread: None,
            origin_message: None,
        }
    }

    fn rule(id: &str, rule_code: &str, name: Option<&str>) -> Rule {
        Rule {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            rule_code: rule_code.to_string(),
            name: name.map(str::to_string),
            description: None,
            statement: "Claim items shall be grouped by participant".to_string(),
            status: RuleStatus::Active,
            severity: RuleSeverity::High,
            rule_type: None,
            modality: Some(RuleModality::Obligation),
            confidence: None,
            extraction_method: None,
            source_document: Some("src/UseCase.php".to_string()),
            source_section: Some("59-69".to_string()),
            origin_thread: None,
            origin_message: None,
            expression: serde_json::json!({}),
            inputs: serde_json::json!([]),
        }
    }

    fn source(id: &str, name: &str) -> Source {
        Source {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            name: name.to_string(),
            source_type: SourceType::Document,
            url: None,
            reference: None,
            commit_pin: None,
            effective_date: None,
            review_date: None,
            superseded_by: None,
            origin_thread: None,
            origin_message: None,
        }
    }

    fn edge(edge_type: EdgeType, from: (NodeType, &str), to: (NodeType, &str)) -> Edge {
        Edge {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: Edge::stable_id(edge_type, from.0, &sid(from.1), to.0, &sid(to.1)).unwrap(),
            edge_type,
            from_type: from.0,
            from_id: sid(from.1),
            to_type: to.0,
            to_id: sid(to.1),
            label: None,
        }
    }

    fn thread(id: &str, parent: (NodeType, &str), created_at: i64) -> Thread {
        Thread {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            parent: ThreadParent {
                node_type: parent.0,
                node_id: sid(parent.1),
            },
            status: ThreadStatus::Active,
            created_at,
        }
    }

    fn message(id: &str, thread_id: &str, body: &str, created_at: i64) -> Message {
        Message {
            schema_version: SchemaVersion(1),
            scope_id: scope_id(),
            id: sid(id),
            thread_id: sid(thread_id),
            role: MessageRole::Assistant,
            body: body.to_string(),
            created_at,
            ai_metadata: None,
        }
    }

    fn empty_state() -> ScopeExport {
        ScopeExport {
            scope: "default".to_string(),
            sources: vec![],
            domains: vec![],
            requirements: vec![],
            boundaries: vec![],
            topics: vec![],
            questions: vec![],
            resolutions: vec![],
            rules: vec![],
            services: vec![],
            service_bindings: vec![],
            edges: vec![],
            threads: vec![],
            messages: vec![],
            contributions: vec![],
            synthesis_packets: vec![],
            proposal_cards: vec![],
            promotion_decisions: vec![],
        }
    }

    #[allow(clippy::too_many_lines)]
    fn fixture_state() -> ScopeExport {
        let mut state = empty_state();
        state.sources = vec![
            {
                let mut schads = source("source_schads", "SCHADS Award mapping");
                schads.reference = Some("docs/award.md".to_string());
                schads.commit_pin = Some("abc1234".to_string());
                schads
            },
            source("source_unused", "Unused API spec"),
        ];
        state.requirements = vec![
            requirement(
                "req_root",
                "Platform shall manage invoicing",
                RequirementStatus::Active,
                vec![],
            ),
            requirement(
                "req_child",
                "SaveInvoice shall split claim items",
                RequirementStatus::Resolved,
                vec![SourceReference {
                    source_id: sid("source_schads"),
                    clause: Some("clause 10.3".to_string()),
                }],
            ),
            requirement(
                "req_stuck",
                "Rostering shall respect awards",
                RequirementStatus::Resolved,
                vec![SourceReference {
                    source_id: sid("source_missing"),
                    clause: None,
                }],
            ),
        ];
        state.resolutions = vec![
            resolution(
                "res_split",
                "Per-portion split",
                vec![ResolutionInput {
                    input_type: ResolutionInputType::Technical,
                    reference: "src/UseCase.php:59-69".to_string(),
                    summary: "Codebase scan".to_string(),
                }],
            ),
            resolution("res_orphan", "Detached decision", vec![]),
        ];
        state.rules = vec![
            rule(
                "rule_001",
                "SAH-INV-001",
                Some("Invoices grouped by participant"),
            ),
            rule("rule_orphan", "SAH-INV-999", None),
        ];
        state.edges = vec![
            edge(
                EdgeType::RefinesInto,
                (NodeType::Requirement, "req_root"),
                (NodeType::Requirement, "req_child"),
            ),
            edge(
                EdgeType::Resolves,
                (NodeType::Resolution, "res_split"),
                (NodeType::Requirement, "req_child"),
            ),
            edge(
                EdgeType::Produces,
                (NodeType::Resolution, "res_split"),
                (NodeType::Rule, "rule_001"),
            ),
            edge(
                EdgeType::Produces,
                (NodeType::Requirement, "req_child"),
                (NodeType::Rule, "rule_001"),
            ),
            edge(
                EdgeType::References,
                (NodeType::Source, "source_schads"),
                (NodeType::Requirement, "req_child"),
            ),
            edge(
                EdgeType::Spawns,
                (NodeType::Resolution, "res_split"),
                (NodeType::Requirement, "req_stuck"),
            ),
        ];
        state.threads = vec![
            thread("thr_req_child", (NodeType::Requirement, "req_child"), 10),
            thread("thr_res_split", (NodeType::Resolution, "res_split"), 20),
        ];
        state.messages = vec![
            message("msg_scoping", "thr_req_child", "Scoping note", 1),
            message(
                "msg_guard",
                "thr_res_split",
                "Guard at src/UseCase.php:153-156 confirmed by testCreateGapInvoiceOnly.",
                2,
            ),
        ];
        state
    }

    fn fixture_corpus() -> crate::wiki::model::WikiCorpus {
        let resolver = LinkResolver::new(Some("git@github.com:visualcare/vc-api.git"));
        build_corpus(&fixture_state(), &resolver)
    }

    fn gap_kinds(gaps: &[crate::wiki::model::GapNotice]) -> Vec<GapKind> {
        gaps.iter().map(|gap| gap.kind).collect()
    }

    #[test]
    fn build_corpus_on_a_truly_empty_scope_is_honestly_empty() {
        let resolver = LinkResolver::new(None);
        let corpus = build_corpus(&empty_state(), &resolver);
        assert!(corpus.requirements.is_empty());
        assert!(corpus.resolutions.is_empty());
        assert!(corpus.rules.is_empty());
        assert!(corpus.sources.is_empty());
        assert!(corpus.index.roots.is_empty());
        assert!(corpus.index.gaps.is_empty());
        assert!(corpus.index.orphans.is_empty());
        assert_eq!(corpus.index.counts, CorpusCounts::default());
    }

    #[test]
    fn index_lists_root_requirements_with_counts() {
        let corpus = fixture_corpus();
        let roots: Vec<&str> = corpus
            .index
            .roots
            .iter()
            .map(|entry| entry.link.target.record_id.as_str())
            .collect();
        assert_eq!(roots, vec!["req_root", "req_stuck"]);
        let root = &corpus.index.roots[0];
        assert_eq!(root.children, 1);
        assert_eq!(root.resolutions, 0);
        assert_eq!(root.rules, 0);
        assert_eq!(corpus.index.counts.sources, 2);
        assert_eq!(corpus.index.counts.requirements, 3);
        assert_eq!(corpus.index.counts.resolutions, 2);
        assert_eq!(corpus.index.counts.rules, 2);
    }

    #[test]
    fn index_reports_scope_gaps_and_orphans() {
        let corpus = fixture_corpus();
        let kinds = gap_kinds(&corpus.index.gaps);
        assert_eq!(
            kinds,
            vec![GapKind::NoResolvingDecision, GapKind::NoProducedRules]
        );
        assert!(corpus
            .index
            .gaps
            .iter()
            .all(|gap| gap.detail.contains("req_stuck")));
        let orphan_ids = |links: &[crate::wiki::model::PageLink]| {
            links
                .iter()
                .map(|link| link.target.record_id.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(orphan_ids(&corpus.index.orphans.rules), vec!["rule_orphan"]);
        assert_eq!(
            orphan_ids(&corpus.index.orphans.resolutions),
            vec!["res_orphan"]
        );
        assert_eq!(
            orphan_ids(&corpus.index.orphans.sources),
            vec!["source_unused"]
        );
    }

    #[test]
    fn index_reports_a_gap_for_a_thread_whose_parent_record_is_gone() {
        // A thread whose parent has been deleted/renamed is never matched
        // by any page's threads_for() lookup (those only ever query ids of
        // records that were found), so it would otherwise be dropped
        // without a trace instead of becoming a gap notice like every
        // other kind of dangling reference.
        let mut state = empty_state();
        state.requirements = vec![requirement(
            "req_child",
            "SaveInvoice shall split claim items",
            RequirementStatus::Active,
            vec![],
        )];
        state.threads = vec![thread(
            "thr_ghost",
            (NodeType::Resolution, "res_missing"),
            10,
        )];
        let resolver = LinkResolver::new(None);
        let corpus = build_corpus(&state, &resolver);
        let dangling = corpus
            .index
            .gaps
            .iter()
            .find(|gap| gap.kind == GapKind::DanglingReference)
            .expect("a dangling thread parent should be reported as a gap");
        assert!(dangling.detail.contains("thr_ghost"));
        assert!(dangling.detail.contains("res_missing"));
    }

    fn requirement_page<'a>(
        corpus: &'a crate::wiki::model::WikiCorpus,
        id: &str,
    ) -> &'a crate::wiki::model::RequirementPage {
        corpus
            .requirements
            .iter()
            .find(|page| page.id.record_id == id)
            .unwrap()
    }

    #[test]
    fn requirement_page_assembles_lineage_decision_rules_and_sources() {
        let corpus = fixture_corpus();
        let page = requirement_page(&corpus, "req_child");

        let back = page.back_link.as_ref().unwrap();
        assert_eq!(back.target.record_id, "req_root");

        let lineage: Vec<(&str, bool)> = page
            .lineage
            .iter()
            .map(|entry| (entry.link.target.record_id.as_str(), entry.is_current))
            .collect();
        assert_eq!(lineage, vec![("req_root", false), ("req_child", true)]);

        assert_eq!(page.decisions.len(), 1);
        let decision = &page.decisions[0];
        assert_eq!(decision.link.target.record_id, "res_split");
        assert_eq!(decision.link.target.kind, PageKind::Resolution);
        assert_eq!(decision.position, "Adopt the split");
        assert_eq!(decision.inputs.len(), 1);
        assert_eq!(
            decision.inputs[0].reference.href.as_deref(),
            Some("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L59-L69")
        );

        assert_eq!(
            page.produced_rules.len(),
            1,
            "direct and via-resolution rules deduplicate"
        );
        let card = &page.produced_rules[0];
        assert_eq!(card.rule_code, "SAH-INV-001");
        assert_eq!(card.evidence.len(), 1);
        assert_eq!(card.evidence[0].label, "src/UseCase.php:59-69");
        assert_eq!(
            card.evidence[0].href.as_deref(),
            Some("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L59-L69")
        );

        assert_eq!(page.sources.len(), 1);
        assert_eq!(page.sources[0].link.target.record_id, "source_schads");
        assert_eq!(page.sources[0].clause.as_deref(), Some("clause 10.3"));

        assert!(page.gaps.is_empty());
    }

    #[test]
    fn requirement_page_borrows_decision_threads_and_annotates_bodies() {
        let corpus = fixture_corpus();
        let page = requirement_page(&corpus, "req_child");
        let thread_ids: Vec<&str> = page
            .threads
            .iter()
            .map(|thread| thread.thread_id.as_str())
            .collect();
        assert_eq!(thread_ids, vec!["thr_req_child", "thr_res_split"]);
        assert_eq!(page.threads[1].parent_type, NodeType::Resolution);
        let note = &page.threads[1].messages[0];
        assert_eq!(note.refs.len(), 2);
        assert_eq!(note.refs[0].label, "src/UseCase.php:153-156");
        assert_eq!(note.refs[1].label, "testCreateGapInvoiceOnly");
    }

    #[test]
    fn requirement_page_flags_missing_sources() {
        let corpus = fixture_corpus();
        let page = requirement_page(&corpus, "req_root");
        assert_eq!(gap_kinds(&page.gaps), vec![GapKind::MissingSourceRefs]);
        let children: Vec<&str> = page
            .children
            .iter()
            .map(|link| link.target.record_id.as_str())
            .collect();
        assert_eq!(children, vec!["req_child"]);
    }

    #[test]
    fn requirement_page_flags_dangling_refs_and_frontier_gaps() {
        let corpus = fixture_corpus();
        let page = requirement_page(&corpus, "req_stuck");
        assert!(page.sources.is_empty());
        let kinds = gap_kinds(&page.gaps);
        assert!(kinds.contains(&GapKind::DanglingReference));
        assert!(kinds.contains(&GapKind::MissingSourceRefs));
        assert!(kinds.contains(&GapKind::NoResolvingDecision));
        assert!(kinds.contains(&GapKind::NoProducedRules));
        let dangling = page
            .gaps
            .iter()
            .find(|gap| gap.kind == GapKind::DanglingReference)
            .unwrap();
        assert!(dangling.detail.contains("source_missing"));
    }

    #[test]
    fn requirement_page_does_not_treat_a_same_id_record_of_another_kind_as_a_resolving_decision() {
        // A Resolution and a Source share the stable id "dup_id". The only
        // Resolves edge on file is authored for the source (not the
        // resolution), so it must not be mistaken for a real resolving
        // decision just because the ids match.
        let mut state = empty_state();
        state.requirements = vec![requirement(
            "req_child",
            "SaveInvoice shall split claim items",
            RequirementStatus::Active,
            vec![],
        )];
        state.resolutions = vec![resolution("dup_id", "Decoy resolution", vec![])];
        state.sources = vec![source("dup_id", "Decoy source")];
        state.edges = vec![edge(
            EdgeType::Resolves,
            (NodeType::Source, "dup_id"),
            (NodeType::Requirement, "req_child"),
        )];
        let resolver = LinkResolver::new(None);
        let corpus = build_corpus(&state, &resolver);
        let page = requirement_page(&corpus, "req_child");
        assert!(
            page.decisions.is_empty(),
            "resolution 'dup_id' has no real Resolves edge and must not appear as a decision"
        );
    }

    fn resolution_page<'a>(
        corpus: &'a crate::wiki::model::WikiCorpus,
        id: &str,
    ) -> &'a crate::wiki::model::ResolutionPage {
        corpus
            .resolutions
            .iter()
            .find(|page| page.id.record_id == id)
            .unwrap()
    }

    #[test]
    fn resolution_page_links_requirements_rules_and_spawned_work() {
        let corpus = fixture_corpus();
        let page = resolution_page(&corpus, "res_split");
        assert_eq!(page.resolves.len(), 1);
        assert_eq!(page.resolves[0].target.record_id, "req_child");
        assert_eq!(page.spawned.len(), 1);
        assert_eq!(page.spawned[0].target.record_id, "req_stuck");
        assert_eq!(page.produced_rules.len(), 1);
        assert_eq!(page.produced_rules[0].rule_code, "SAH-INV-001");
        assert!(page.gaps.is_empty());
        assert_eq!(page.threads.len(), 1);
        assert_eq!(page.threads[0].thread_id, "thr_res_split");
    }

    #[test]
    fn resolution_page_flags_orphaned_and_ruleless_decisions() {
        let corpus = fixture_corpus();
        let page = resolution_page(&corpus, "res_orphan");
        assert_eq!(
            gap_kinds(&page.gaps),
            vec![GapKind::OrphanResolution, GapKind::NoProducedRules]
        );
    }

    fn rule_page<'a>(
        corpus: &'a crate::wiki::model::WikiCorpus,
        id: &str,
    ) -> &'a crate::wiki::model::RulePage {
        corpus
            .rules
            .iter()
            .find(|page| page.id.record_id == id)
            .unwrap()
    }

    #[test]
    fn rule_page_traces_back_to_requirements_and_sources() {
        let corpus = fixture_corpus();
        let page = rule_page(&corpus, "rule_001");
        assert_eq!(page.title, "Invoices grouped by participant");
        let produced_by: Vec<&str> = page
            .produced_by
            .iter()
            .map(|link| link.target.record_id.as_str())
            .collect();
        assert_eq!(produced_by, vec!["res_split", "req_child"]);
        assert_eq!(page.requirements.len(), 1);
        assert_eq!(page.requirements[0].target.record_id, "req_child");
        assert_eq!(page.sources.len(), 1);
        assert_eq!(page.sources[0].target.record_id, "source_schads");
        assert_eq!(page.evidence.len(), 1);
        assert_eq!(
            page.evidence[0].href.as_deref(),
            Some("https://github.com/visualcare/vc-api/blob/HEAD/src/UseCase.php#L59-L69")
        );
        assert!(page.gaps.is_empty());
    }

    #[test]
    fn rule_page_flags_orphan_rules_and_falls_back_to_the_rule_code() {
        let corpus = fixture_corpus();
        let page = rule_page(&corpus, "rule_orphan");
        assert_eq!(page.title, "SAH-INV-999");
        assert_eq!(gap_kinds(&page.gaps), vec![GapKind::OrphanRule]);
        assert!(page.produced_by.is_empty());
    }

    fn source_page<'a>(
        corpus: &'a crate::wiki::model::WikiCorpus,
        id: &str,
    ) -> &'a crate::wiki::model::SourcePage {
        corpus
            .sources
            .iter()
            .find(|page| page.id.record_id == id)
            .unwrap()
    }

    #[test]
    fn source_page_lists_referencing_requirements_and_pins_links() {
        let corpus = fixture_corpus();
        let page = source_page(&corpus, "source_schads");
        assert_eq!(page.referenced_requirements.len(), 1);
        assert_eq!(
            page.referenced_requirements[0].target.record_id,
            "req_child"
        );
        assert_eq!(
            page.reference.as_ref().unwrap().href.as_deref(),
            Some("https://github.com/visualcare/vc-api/blob/abc1234/docs/award.md")
        );
        assert!(page.gaps.is_empty());
    }

    #[test]
    fn source_page_flags_unreferenced_sources() {
        let corpus = fixture_corpus();
        let page = source_page(&corpus, "source_unused");
        assert_eq!(gap_kinds(&page.gaps), vec![GapKind::UnreferencedSource]);
        assert!(page.referenced_requirements.is_empty());
    }

    #[test]
    fn load_corpus_reads_state_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let layout = provenance_store::layout::ProvenanceLayout::new(repo.clone());
        provenance_store::jsonl::write_jsonl_atomic(
            &provenance_store::shards::requirements_path(&layout, &scope_id()),
            &[requirement(
                "req_root",
                "Platform shall manage invoicing",
                RequirementStatus::Active,
                vec![],
            )],
        )
        .unwrap();

        let corpus = load_corpus(repo, "default".to_string()).unwrap();
        assert_eq!(corpus.scope, "default");
        assert_eq!(corpus.requirements.len(), 1);
        assert_eq!(corpus.index.roots.len(), 1);
        assert_eq!(corpus.index.counts.requirements, 1);
    }
}
