use crate::wiki::links::{EvidenceRef, LinkResolver};
use crate::wiki::model::{
    CorpusCounts, DecisionSection, EvidenceThread, FieldNote, GapKind, GapNotice, IndexEntry,
    InputCitation, LineageEntry, OrphanReport, PageId, PageKind, PageLink, RecordKind,
    RequirementPage, ResolutionPage, RuleCard, RulePage, ScopeIndexPage, SearchEntry,
    SearchIndexPage, SourceCitation, SourcePage, Topic, TopicGroup, TopicIndexPage, WikiCorpus,
};
use provenance_core::{
    MessageRole, NodeType, RequirementStatus, ResolutionInputType, ResolutionStatus, RuleModality,
    RuleSeverity, RuleStatus, RuleType, SourceType, ThreadStatus,
};

pub(super) const REMOTE: &str = "git@github.com:exampleorg/ex-api.git";

pub(super) fn link(kind: PageKind, id: &str, title: &str) -> PageLink {
    let kind = match kind {
        PageKind::Requirement => RecordKind::Requirement,
        PageKind::Resolution => RecordKind::Resolution,
        PageKind::Rule => RecordKind::Rule,
        PageKind::Source => RecordKind::Source,
        PageKind::ScopeIndex | PageKind::TopicIndex | PageKind::SearchIndex => {
            panic!("singleton pages cannot be record links")
        }
    };
    PageLink {
        target: PageId::new(kind, id),
        title: title.to_string(),
    }
}

pub(super) fn colliding_requirement_links() -> Vec<PageLink> {
    vec![
        link(
            PageKind::Requirement,
            "req_sah_participant_budget_summary_shall_pro",
            "Participant budget summary shall pro-rate services",
        ),
        link(
            PageKind::Requirement,
            "req_sah_participant_budget_summary_shall_pro_2",
            "Participant budget summary shall pro-rate services",
        ),
    ]
}

pub(super) fn unique_requirement_links() -> Vec<PageLink> {
    vec![
        link(
            PageKind::Requirement,
            "req_budget_split",
            "Budget portions shall reconcile",
        ),
        link(
            PageKind::Requirement,
            "req_zero_suppression",
            "Zero claim items shall be suppressed",
        ),
    ]
}

pub(super) fn rule_card(resolver: &LinkResolver) -> RuleCard {
    RuleCard {
        link: link(
            PageKind::Rule,
            "rule_sah_inv_016",
            "Suppress line emission for fully zero claim items",
        ),
        rule_code: "SAH-INV-016".to_string(),
        name: Some("Suppress line emission for fully zero claim items".to_string()),
        statement: "If a claim item's participant, government, and gap portions are all <= 0 \
                    after markup, no invoice lines shall be emitted for that claim item."
            .to_string(),
        status: RuleStatus::Active,
        severity: RuleSeverity::High,
        modality: Some(RuleModality::Prohibition),
        evidence: vec![resolver.resolve("src/UseCase.php:153-156")],
    }
}

pub(super) fn field_note(resolver: &LinkResolver) -> FieldNote {
    let body = "Per-portion guard at src/UseCase.php:211-233.\n\
                Confirmed by testCreateGapInvoiceOnly."
        .to_string();
    let refs = resolver.annotate(&body);
    FieldNote {
        message_id: "msg_000001".to_string(),
        role: MessageRole::Assistant,
        created_at: 1_714_780_800_000,
        body,
        refs,
    }
}

pub(super) fn resolution_thread(resolver: &LinkResolver) -> EvidenceThread {
    EvidenceThread {
        thread_id: "thr_resolution_res_split_0".to_string(),
        parent_type: NodeType::Resolution,
        parent_id: "res_split".to_string(),
        status: ThreadStatus::Active,
        messages: vec![field_note(resolver)],
    }
}

pub(super) fn decision(resolver: &LinkResolver) -> DecisionSection {
    DecisionSection {
        link: link(
            PageKind::Resolution,
            "res_split",
            "SaveInvoice per-portion split & $0 suppression extraction",
        ),
        status: ResolutionStatus::Approved,
        position: "Adopt these as 7 rules. Severity high.".to_string(),
        rationale: "Atomicity here = drift detectability.".to_string(),
        context: Some("Codebase scan of UseCase.php identified 7 patterns.".to_string()),
        enforcement: Some("Specification".to_string()),
        confidence: Some(0.97),
        inputs: vec![InputCitation {
            input_type: ResolutionInputType::Technical,
            summary: "Codebase scan — SaveInvoice use case.".to_string(),
            reference: resolver.resolve("src/UseCase.php:59-69"),
        }],
        made_by: Some("Ben Nasraoui".to_string()),
        approved_by: Some("Ben Nasraoui".to_string()),
        approved_at: Some(1_776_470_400_000),
    }
}

pub(super) fn requirement_fixture() -> RequirementPage {
    let resolver = LinkResolver::new(Some(REMOTE));
    RequirementPage {
        id: PageId::new(RecordKind::Requirement, "req_saveinvoice_split"),
        title: "SaveInvoice shall split each claim item into portions".to_string(),
        status: RequirementStatus::Discovery,
        statement: "Grouping by participant_ref with per-portion positive-amount guards."
            .to_string(),
        description: None,
        fog: None,
        domain_id: Some("dom_invoicing".to_string()),
        back_link: Some(link(
            PageKind::Requirement,
            "req_sah",
            "Support at Home (SAH)",
        )),
        lineage: vec![
            LineageEntry {
                link: link(PageKind::Requirement, "req_platform", "ExampleOrg platform"),
                is_current: false,
            },
            LineageEntry {
                link: link(PageKind::Requirement, "req_sah", "Support at Home (SAH)"),
                is_current: false,
            },
            LineageEntry {
                link: link(
                    PageKind::Requirement,
                    "req_saveinvoice_split",
                    "SaveInvoice shall split each claim item into portions",
                ),
                is_current: true,
            },
        ],
        decisions: vec![decision(&resolver)],
        produced_rules: vec![rule_card(&resolver)],
        children: vec![link(
            PageKind::Requirement,
            "req_gap_lines",
            "Gap lines shall be suppressed when zero",
        )],
        siblings: vec![],
        sources: vec![SourceCitation {
            link: link(PageKind::Source, "source_schads", "SCHADS Award mapping"),
            source_type: SourceType::Document,
            clause: Some("clause 10.3".to_string()),
            reference: Some(resolver.resolve_at("docs/award.md", Some("abc1234"))),
        }],
        gaps: vec![],
        threads: vec![resolution_thread(&resolver)],
    }
}

pub(super) fn gappy_requirement_fixture() -> RequirementPage {
    RequirementPage {
        id: PageId::new(RecordKind::Requirement, "req_stuck"),
        title: "Rostering shall respect awards".to_string(),
        status: RequirementStatus::Resolved,
        statement: "Rostering shall respect awards.".to_string(),
        description: None,
        fog: Some("Which award clauses apply is still unclear.".to_string()),
        domain_id: None,
        back_link: None,
        lineage: vec![LineageEntry {
            link: link(
                PageKind::Requirement,
                "req_stuck",
                "Rostering shall respect awards",
            ),
            is_current: true,
        }],
        decisions: vec![],
        produced_rules: vec![],
        children: vec![],
        siblings: vec![],
        sources: vec![],
        gaps: vec![
            GapNotice {
                kind: GapKind::DanglingReference,
                detail: "source ref points at source_missing, which does not exist".to_string(),
            },
            GapNotice {
                kind: GapKind::MissingSourceRefs,
                detail: "no source refs recorded on this requirement".to_string(),
            },
            GapNotice {
                kind: GapKind::NoResolvingDecision,
                detail: "resolved with no resolving decision".to_string(),
            },
        ],
        threads: vec![],
    }
}

pub(super) fn resolution_fixture() -> ResolutionPage {
    let resolver = LinkResolver::new(Some(REMOTE));
    ResolutionPage {
        id: PageId::new(RecordKind::Resolution, "res_split"),
        title: "SaveInvoice per-portion split & $0 suppression extraction".to_string(),
        status: ResolutionStatus::Approved,
        position: "Adopt these as 7 rules. Severity high.".to_string(),
        rationale: "Atomicity here = drift detectability.".to_string(),
        context: Some("Codebase scan of UseCase.php identified 7 patterns.".to_string()),
        enforcement: Some("Specification".to_string()),
        confidence: Some(0.97),
        inputs: vec![InputCitation {
            input_type: ResolutionInputType::Technical,
            summary: "Codebase scan — SaveInvoice use case.".to_string(),
            reference: resolver.resolve("src/UseCase.php:59-69"),
        }],
        made_by: Some("Ben Nasraoui".to_string()),
        approved_by: Some("Ben Nasraoui".to_string()),
        approved_at: Some(1_776_470_400_000),
        review_on: Some("2026-10-01".to_string()),
        superseded_by: None,
        resolves: vec![link(
            PageKind::Requirement,
            "req_saveinvoice_split",
            "SaveInvoice shall split each claim item into portions",
        )],
        spawned: vec![],
        produced_rules: vec![rule_card(&resolver)],
        gaps: vec![],
        threads: vec![resolution_thread(&resolver)],
    }
}

pub(super) fn rule_fixture() -> RulePage {
    let resolver = LinkResolver::new(Some(REMOTE));
    RulePage {
        id: PageId::new(RecordKind::Rule, "rule_sah_inv_016"),
        title: "Suppress line emission for fully zero claim items".to_string(),
        rule_code: "SAH-INV-016".to_string(),
        statement: "No invoice lines shall be emitted for fully zero claim items.".to_string(),
        description: None,
        status: RuleStatus::Active,
        severity: RuleSeverity::High,
        modality: Some(RuleModality::Prohibition),
        rule_type: Some(RuleType::Business),
        confidence: Some(0.92),
        extraction_method: Some("codebase_scan".to_string()),
        source_document: Some("src/UseCase.php".to_string()),
        source_section: Some("153-156".to_string()),
        evidence: vec![
            resolver.resolve("src/UseCase.php:153-156"),
            EvidenceRef {
                label: "SCHADS Award clause 10.3".to_string(),
                href: None,
            },
        ],
        produced_by: vec![link(
            PageKind::Resolution,
            "res_split",
            "SaveInvoice per-portion split & $0 suppression extraction",
        )],
        requirements: vec![link(
            PageKind::Requirement,
            "req_saveinvoice_split",
            "SaveInvoice shall split each claim item into portions",
        )],
        sources: vec![link(
            PageKind::Source,
            "source_schads",
            "SCHADS Award mapping",
        )],
        gaps: vec![],
        threads: vec![],
    }
}

pub(super) fn source_fixture() -> SourcePage {
    let resolver = LinkResolver::new(Some(REMOTE));
    SourcePage {
        id: PageId::new(RecordKind::Source, "source_schads"),
        title: "SCHADS Award mapping".to_string(),
        source_type: SourceType::Document,
        url: Some("https://example.test/award".to_string()),
        reference: Some(resolver.resolve_at("docs/award.md", Some("abc1234"))),
        commit_pin: Some("abc1234".to_string()),
        effective_date: Some(1_714_780_800_000),
        review_date: None,
        superseded_by: None,
        referenced_requirements: vec![link(
            PageKind::Requirement,
            "req_saveinvoice_split",
            "SaveInvoice shall split each claim item into portions",
        )],
        gaps: vec![],
        threads: vec![],
    }
}

pub(super) fn index_fixture() -> ScopeIndexPage {
    ScopeIndexPage {
        scope: "default".to_string(),
        title: "Provenance atlas — default".to_string(),
        counts: CorpusCounts {
            sources: 2,
            requirements: 3,
            resolutions: 1,
            rules: 1,
        },
        roots: vec![IndexEntry {
            link: link(PageKind::Requirement, "req_platform", "ExampleOrg platform"),
            status: RequirementStatus::Active,
            children: 2,
            resolutions: 1,
            rules: 1,
        }],
        gaps: vec![GapNotice {
            kind: GapKind::UnreferencedSource,
            detail: "source_unused is referenced by nothing".to_string(),
        }],
        orphans: OrphanReport {
            rules: vec![link(PageKind::Rule, "rule_orphan", "ORPH-001")],
            resolutions: vec![],
            sources: vec![link(PageKind::Source, "source_unused", "Unused API spec")],
        },
    }
}

pub(super) fn corpus_fixture() -> WikiCorpus {
    let requirement = requirement_fixture();
    let rule = rule_fixture();
    WikiCorpus {
        scope: "default".to_string(),
        index: index_fixture(),
        topics: TopicIndexPage {
            scope: "default".to_string(),
            title: "Topics by domain".to_string(),
            groups: vec![TopicGroup {
                topic: Topic::Defined {
                    id: "dom_invoicing".to_string(),
                    name: "Invoicing".to_string(),
                    description: Some("Claim and invoice settlement".to_string()),
                },
                requirements: vec![link(
                    PageKind::Requirement,
                    "req_saveinvoice_split",
                    &requirement.title,
                )],
                rules: vec![link(PageKind::Rule, "rule_sah_inv_016", &rule.title)],
            }],
        },
        search: SearchIndexPage {
            scope: "default".to_string(),
            title: "Search requirements and rules".to_string(),
            entries: vec![
                SearchEntry {
                    link: link(
                        PageKind::Requirement,
                        "req_saveinvoice_split",
                        &requirement.title,
                    ),
                    statement: requirement.statement.clone(),
                },
                SearchEntry {
                    link: link(PageKind::Rule, "rule_sah_inv_016", &rule.title),
                    statement: rule.statement.clone(),
                },
            ],
        },
        requirements: vec![requirement, gappy_requirement_fixture()],
        resolutions: vec![resolution_fixture()],
        rules: vec![rule],
        sources: vec![source_fixture()],
    }
}
