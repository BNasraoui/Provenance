use super::{CreateEdgeInput, CreateResolutionInput, CreateRuleInput, StateStore};
use crate::shards;
use crate::transaction::StateTransaction;
use provenance_core::{
    validate_optional_confidence_score, EdgeType, NodeType, Resolution, Rule, SchemaVersion,
};

impl StateStore {
    pub fn create_resolution(&self, input: CreateResolutionInput) -> anyhow::Result<Resolution> {
        self.write_transaction(|transaction| self.create_resolution_locked(transaction, input))
    }

    fn create_resolution_locked(
        &self,
        transaction: &mut StateTransaction,
        input: CreateResolutionInput,
    ) -> anyhow::Result<Resolution> {
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
            inputs,
            made_by,
            approved_by,
            approved_at,
            superseded_by,
            origin_thread,
            origin_message,
        } = input;
        let confidence = validate_optional_confidence_score(confidence)?;
        if let Some(requirement_id) = &requirement_id {
            anyhow::ensure!(
                self.list_requirements_unlocked(&scope_id)?
                    .iter()
                    .any(|requirement| &requirement.id == requirement_id),
                "requirement does not exist"
            );
        }
        let path = shards::resolutions_path(&self.layout, &scope_id);
        let resolution = transaction.mutate_jsonl(&path, |records: &mut Vec<Resolution>| {
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
                inputs,
                made_by,
                approved_by,
                approved_at,
                superseded_by,
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
            Ok(resolution)
        })?;
        if let Some(requirement_id) = requirement_id {
            self.add_edge_in(
                transaction,
                CreateEdgeInput {
                    scope_id: scope_id.clone(),
                    edge_type: EdgeType::Needs,
                    from_type: NodeType::Requirement,
                    from_id: requirement_id.clone(),
                    to_type: NodeType::Resolution,
                    to_id: id.clone(),
                },
            )?;
            self.add_edge_in(
                transaction,
                CreateEdgeInput {
                    scope_id,
                    edge_type: EdgeType::Resolves,
                    from_type: NodeType::Resolution,
                    from_id: id,
                    to_type: NodeType::Requirement,
                    to_id: requirement_id,
                },
            )?;
        }
        Ok(resolution)
    }

    pub fn create_rule(&self, input: CreateRuleInput) -> anyhow::Result<Rule> {
        self.write_transaction(|transaction| self.create_rule_locked(transaction, input))
    }

    fn create_rule_locked(
        &self,
        transaction: &mut StateTransaction,
        input: CreateRuleInput,
    ) -> anyhow::Result<Rule> {
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
        let confidence = validate_optional_confidence_score(confidence)?;
        if let Some(requirement_id) = &requirement_id {
            anyhow::ensure!(
                self.list_requirements_unlocked(&scope_id)?
                    .iter()
                    .any(|requirement| &requirement.id == requirement_id),
                "requirement does not exist"
            );
        }
        if let Some(resolution_id) = &resolution_id {
            anyhow::ensure!(
                self.list_resolutions_unlocked(&scope_id)?
                    .iter()
                    .any(|resolution| &resolution.id == resolution_id),
                "resolution does not exist"
            );
        }
        let path = shards::rules_path(&self.layout, &scope_id);
        let rule = transaction.mutate_jsonl(&path, |records: &mut Vec<Rule>| {
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
            Ok(rule)
        })?;
        if let Some(requirement_id) = requirement_id {
            self.add_edge_in(
                transaction,
                CreateEdgeInput {
                    scope_id: scope_id.clone(),
                    edge_type: EdgeType::Produces,
                    from_type: NodeType::Requirement,
                    from_id: requirement_id,
                    to_type: NodeType::Rule,
                    to_id: id.clone(),
                },
            )?;
        }
        if let Some(resolution_id) = resolution_id {
            self.add_edge_in(
                transaction,
                CreateEdgeInput {
                    scope_id,
                    edge_type: EdgeType::Produces,
                    from_type: NodeType::Resolution,
                    from_id: resolution_id,
                    to_type: NodeType::Rule,
                    to_id: id,
                },
            )?;
        }
        Ok(rule)
    }
}
