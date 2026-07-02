use super::{CreateResolutionInput, CreateRuleInput, StateStore};
use crate::{jsonl, shards};
use provenance_core::{EdgeType, NodeType, Resolution, Rule, SchemaVersion};

impl StateStore {
    pub fn create_resolution(&self, input: CreateResolutionInput) -> anyhow::Result<Resolution> {
        let CreateResolutionInput {
            scope_id,
            id,
            title,
            requirement_id,
            position,
            rationale,
            status,
            context,
            enforcement,
            confidence,
            origin_thread,
            origin_message,
        } = input;
        if let Some(requirement_id) = &requirement_id {
            anyhow::ensure!(
                self.list_requirements(&scope_id)?
                    .iter()
                    .any(|requirement| &requirement.id == requirement_id),
                "requirement does not exist"
            );
        }
        let mut records = self.list_resolutions(&scope_id)?;
        let resolution = Resolution {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id: id.clone(),
            title,
            position,
            rationale,
            status,
            context,
            enforcement,
            confidence,
            review_on: None,
            review_triggers: serde_json::json!([]),
            origin_thread,
            origin_message,
        };
        anyhow::ensure!(
            !records.iter().any(|record| record.id == resolution.id),
            "resolution already exists"
        );
        records.push(resolution.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(&shards::resolutions_path(&self.layout, &scope_id), &records)?;
        if let Some(requirement_id) = requirement_id {
            self.add_edge(
                scope_id.clone(),
                EdgeType::Needs,
                NodeType::Requirement,
                requirement_id.clone(),
                NodeType::Resolution,
                id.clone(),
            )?;
            self.add_edge(
                scope_id,
                EdgeType::Resolves,
                NodeType::Resolution,
                id,
                NodeType::Requirement,
                requirement_id,
            )?;
        }
        Ok(resolution)
    }

    pub fn create_rule(&self, input: CreateRuleInput) -> anyhow::Result<Rule> {
        let CreateRuleInput {
            scope_id,
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
        } = input;
        if let Some(requirement_id) = &requirement_id {
            anyhow::ensure!(
                self.list_requirements(&scope_id)?
                    .iter()
                    .any(|requirement| &requirement.id == requirement_id),
                "requirement does not exist"
            );
        }
        if let Some(resolution_id) = &resolution_id {
            anyhow::ensure!(
                self.list_resolutions(&scope_id)?
                    .iter()
                    .any(|resolution| &resolution.id == resolution_id),
                "resolution does not exist"
            );
        }
        let mut records = self.list_rules(&scope_id)?;
        let rule = Rule {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id: id.clone(),
            rule_code,
            name,
            description,
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
            expression: serde_json::json!({}),
            inputs: serde_json::json!([]),
        };
        anyhow::ensure!(
            !records.iter().any(|record| record.id == rule.id),
            "rule already exists"
        );
        records.push(rule.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(&shards::rules_path(&self.layout, &scope_id), &records)?;
        if let Some(requirement_id) = requirement_id {
            self.add_edge(
                scope_id.clone(),
                EdgeType::Produces,
                NodeType::Requirement,
                requirement_id,
                NodeType::Rule,
                id.clone(),
            )?;
        }
        if let Some(resolution_id) = resolution_id {
            self.add_edge(
                scope_id,
                EdgeType::Produces,
                NodeType::Resolution,
                resolution_id,
                NodeType::Rule,
                id,
            )?;
        }
        Ok(rule)
    }
}
