use crate::output::OutputFormat;
use camino::Utf8PathBuf;
use provenance_core::ScopeId;
use provenance_store::{layout::ProvenanceLayout, state_store::StateStore};
use serde::Serialize;

#[derive(Serialize, serde::Deserialize)]
pub struct ScopeExport {
    pub scope: String,
    pub sources: Vec<provenance_core::Source>,
    #[serde(default)]
    pub domains: Vec<provenance_core::Domain>,
    pub requirements: Vec<provenance_core::Requirement>,
    #[serde(default)]
    pub boundaries: Vec<provenance_core::Boundary>,
    #[serde(default)]
    pub topics: Vec<provenance_core::Topic>,
    #[serde(default)]
    pub questions: Vec<provenance_core::Question>,
    pub resolutions: Vec<provenance_core::Resolution>,
    pub rules: Vec<provenance_core::Rule>,
    #[serde(default)]
    pub services: Vec<provenance_core::Service>,
    #[serde(default)]
    pub service_bindings: Vec<provenance_core::ServiceBinding>,
    pub edges: Vec<provenance_core::Edge>,
    pub threads: Vec<provenance_core::Thread>,
    pub messages: Vec<provenance_core::Message>,
    #[serde(default)]
    pub contributions: Vec<provenance_core::Contribution>,
    #[serde(default)]
    pub synthesis_packets: Vec<provenance_core::SynthesisPacket>,
    #[serde(default)]
    pub proposal_cards: Vec<provenance_core::ProposalCard>,
    #[serde(default)]
    pub promotion_decisions: Vec<provenance_core::PromotionDecisionRecord>,
}

pub fn export_scope(repo: Utf8PathBuf, scope: String) -> anyhow::Result<ScopeExport> {
    let scope_id = ScopeId::new(scope.clone())?;
    let store = StateStore::new(ProvenanceLayout::new(repo));
    let snapshot = store.scope_snapshot(&scope_id)?;
    Ok(ScopeExport {
        scope,
        sources: snapshot.sources,
        domains: snapshot.domains,
        requirements: snapshot.requirements,
        boundaries: snapshot.boundaries,
        topics: snapshot.topics,
        questions: snapshot.questions,
        resolutions: snapshot.resolutions,
        rules: snapshot.rules,
        services: snapshot.services,
        service_bindings: snapshot.service_bindings,
        edges: snapshot.edges,
        threads: snapshot.threads,
        messages: snapshot.messages,
        contributions: snapshot.contributions,
        synthesis_packets: snapshot.synthesis_packets,
        proposal_cards: snapshot.proposals,
        promotion_decisions: snapshot.promotion_decisions,
    })
}

pub(super) fn render_export(
    format: OutputFormat,
    exported: &ScopeExport,
) -> anyhow::Result<String> {
    match format {
        OutputFormat::Json => Ok(format!("{}\n", serde_json::to_string_pretty(exported)?)),
        OutputFormat::Jsonl => {
            let mut out = String::new();
            for value in serde_json::to_value(exported)?.as_object().unwrap().values() {
                if let Some(records) = value.as_array() {
                    for record in records {
                        out.push_str(&serde_json::to_string(record)?);
                        out.push('\n');
                    }
                }
            }
            Ok(out)
        }
        OutputFormat::Markdown => Ok(format!(
            "# Provenance Export\n\n- Scope: {}\n- Sources: {}\n- Domains: {}\n- Requirements: {}\n- Boundaries: {}\n- Topics: {}\n- Questions: {}\n- Resolutions: {}\n- Rules: {}\n- Services: {}\n- Service bindings: {}\n- Edges: {}\n- Proposals: {}\n",
            exported.scope,
            exported.sources.len(),
            exported.domains.len(),
            exported.requirements.len(),
            exported.boundaries.len(),
            exported.topics.len(),
            exported.questions.len(),
            exported.resolutions.len(),
            exported.rules.len(),
            exported.services.len(),
            exported.service_bindings.len(),
            exported.edges.len(),
            exported.proposal_cards.len()
        )),
        OutputFormat::Toon => Ok(format!(
            "scope: {}\nsources: {}\ndomains: {}\nrequirements: {}\nboundaries: {}\ntopics: {}\nquestions: {}\nresolutions: {}\nrules: {}\nservices: {}\nservice_bindings: {}\nedges: {}\nproposals: {}\n",
            exported.scope,
            exported.sources.len(),
            exported.domains.len(),
            exported.requirements.len(),
            exported.boundaries.len(),
            exported.topics.len(),
            exported.questions.len(),
            exported.resolutions.len(),
            exported.rules.len(),
            exported.services.len(),
            exported.service_bindings.len(),
            exported.edges.len(),
            exported.proposal_cards.len()
        )),
        OutputFormat::Table => Ok(format!(
            "scope\tsources\tdomains\trequirements\tboundaries\ttopics\tquestions\tresolutions\trules\tservices\tservice_bindings\tedges\tproposals\n{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            exported.scope,
            exported.sources.len(),
            exported.domains.len(),
            exported.requirements.len(),
            exported.boundaries.len(),
            exported.topics.len(),
            exported.questions.len(),
            exported.resolutions.len(),
            exported.rules.len(),
            exported.services.len(),
            exported.service_bindings.len(),
            exported.edges.len(),
            exported.proposal_cards.len()
        )),
    }
}

pub(super) fn handle(
    repo: Utf8PathBuf,
    scope: String,
    format: OutputFormat,
    output: Option<Utf8PathBuf>,
) -> anyhow::Result<()> {
    let exported = export_scope(repo, scope)?;
    let rendered = render_export(format, &exported)?;
    if let Some(output_path) = output {
        std::fs::write(output_path, rendered)?;
    } else {
        print!("{rendered}");
    }
    Ok(())
}
