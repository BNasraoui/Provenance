use crate::cli::policy::RulesCommand;
use crate::output;
use provenance_core::{RuleModality, RuleSeverity, RuleStatus, RuleType, ScopeId, StableId};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{CreateRuleInput, StateStore},
};

pub(super) fn handle(command: RulesCommand) -> anyhow::Result<()> {
    match command {
        RulesCommand::Create {
            repo,
            scope,
            id,
            rule_code,
            name,
            description,
            requirement_id,
            resolution_id,
            statement,
            status,
            severity,
            rule_type,
            modality,
            confidence,
            extraction_method,
            source_document,
            source_section,
            origin_thread,
            origin_message,
            format,
        } => {
            let rule =
                StateStore::new(ProvenanceLayout::new(repo)).create_rule(CreateRuleInput {
                    scope_id: ScopeId::new(scope)?,
                    id: StableId::new(id)?,
                    rule_code,
                    name,
                    description,
                    requirement_id: requirement_id.map(StableId::new).transpose()?,
                    resolution_id: resolution_id.map(StableId::new).transpose()?,
                    statement,
                    status: RuleStatus::parse(&status)?,
                    severity: RuleSeverity::parse(&severity)?,
                    rule_type: rule_type.map(|value| RuleType::parse(&value)).transpose()?,
                    modality: modality
                        .map(|value| RuleModality::parse(&value))
                        .transpose()?,
                    confidence,
                    extraction_method,
                    source_document,
                    source_section,
                    origin_thread: origin_thread.map(StableId::new).transpose()?,
                    origin_message: origin_message.map(StableId::new).transpose()?,
                })?;
            output::print(format, &rule)?;
        }
    }
    Ok(())
}
