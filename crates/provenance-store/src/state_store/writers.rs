use super::{AddSourceReferenceInput, CreateRequirementInput, CreateSourceInput, StateStore};
use crate::{jsonl, shards};
use provenance_core::{
    edge_validation::validate_edge_endpoint, Edge, EdgeType, NodeType, Requirement, SchemaVersion,
    ScopeId, Source, SourceReference, StableId,
};

impl StateStore {
    pub fn create_source(&self, input: CreateSourceInput) -> anyhow::Result<Source> {
        let CreateSourceInput {
            scope_id,
            id,
            name,
            source_type,
            url,
            reference,
            origin_thread,
            origin_message,
        } = input;
        let mut records = self.list_sources(&scope_id)?;
        let source = Source {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id,
            name,
            source_type,
            url,
            reference,
            origin_thread,
            origin_message,
        };
        anyhow::ensure!(
            !records.iter().any(|record| record.id == source.id),
            "source already exists"
        );
        records.push(source.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(&shards::sources_path(&self.layout, &scope_id), &records)?;
        Ok(source)
    }

    pub fn create_requirement(&self, input: CreateRequirementInput) -> anyhow::Result<Requirement> {
        let CreateRequirementInput {
            scope_id,
            id,
            statement,
            description,
            status,
            origin_thread,
            origin_message,
        } = input;
        let mut records = self.list_requirements(&scope_id)?;
        let requirement = Requirement {
            schema_version: SchemaVersion(1),
            scope_id: scope_id.clone(),
            id,
            statement,
            description,
            status,
            source_refs: Vec::new(),
            origin_thread,
            origin_message,
        };
        anyhow::ensure!(
            !records.iter().any(|record| record.id == requirement.id),
            "requirement already exists"
        );
        records.push(requirement.clone());
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        jsonl::write_jsonl_atomic(
            &shards::requirements_path(&self.layout, &scope_id),
            &records,
        )?;
        Ok(requirement)
    }

    pub fn add_source_reference(&self, input: AddSourceReferenceInput) -> anyhow::Result<Edge> {
        let AddSourceReferenceInput {
            scope_id,
            source_id,
            requirement_id,
            clause,
        } = input;
        validate_edge_endpoint(
            EdgeType::References,
            NodeType::Source,
            NodeType::Requirement,
        )?;
        anyhow::ensure!(
            self.list_sources(&scope_id)?
                .iter()
                .any(|source| source.id == source_id),
            "source does not exist"
        );
        let mut requirements = self.list_requirements(&scope_id)?;
        let requirement = requirements
            .iter_mut()
            .find(|requirement| requirement.id == requirement_id)
            .ok_or_else(|| anyhow::anyhow!("requirement does not exist"))?;
        let source_ref = SourceReference {
            source_id: source_id.clone(),
            clause,
        };
        if !requirement
            .source_refs
            .iter()
            .any(|existing| existing == &source_ref)
        {
            requirement.source_refs.push(source_ref);
            requirement.source_refs.sort_by(|a, b| {
                a.source_id
                    .as_str()
                    .cmp(b.source_id.as_str())
                    .then(a.clause.cmp(&b.clause))
            });
            requirements.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            jsonl::write_jsonl_atomic(
                &shards::requirements_path(&self.layout, &scope_id),
                &requirements,
            )?;
        }
        let mut records = self.list_edges()?;
        let edge = Edge {
            schema_version: SchemaVersion(1),
            scope_id,
            id: Edge::stable_id(
                EdgeType::References,
                NodeType::Source,
                &source_id,
                NodeType::Requirement,
                &requirement_id,
            )?,
            edge_type: EdgeType::References,
            from_type: NodeType::Source,
            from_id: source_id,
            to_type: NodeType::Requirement,
            to_id: requirement_id,
            label: None,
        };
        if !records
            .iter()
            .any(|record| record.id == edge.id && record.scope_id == edge.scope_id)
        {
            records.push(edge.clone());
        }
        records.sort_by(|a, b| {
            a.scope_id
                .as_str()
                .cmp(b.scope_id.as_str())
                .then(a.id.as_str().cmp(b.id.as_str()))
        });
        jsonl::write_jsonl_atomic(&shards::edges_path(&self.layout), &records)?;
        Ok(edge)
    }

    pub(crate) fn add_edge(
        &self,
        scope_id: ScopeId,
        edge_type: EdgeType,
        from_type: NodeType,
        from_id: StableId,
        to_type: NodeType,
        to_id: StableId,
    ) -> anyhow::Result<Edge> {
        validate_edge_endpoint(edge_type, from_type, to_type)?;
        let mut records = self.list_edges()?;
        let edge = Edge {
            schema_version: SchemaVersion(1),
            scope_id,
            id: Edge::stable_id(edge_type, from_type, &from_id, to_type, &to_id)?,
            edge_type,
            from_type,
            from_id,
            to_type,
            to_id,
            label: None,
        };
        if !records
            .iter()
            .any(|record| record.id == edge.id && record.scope_id == edge.scope_id)
        {
            records.push(edge.clone());
        }
        records.sort_by(|a, b| {
            a.scope_id
                .as_str()
                .cmp(b.scope_id.as_str())
                .then(a.id.as_str().cmp(b.id.as_str()))
        });
        jsonl::write_jsonl_atomic(&shards::edges_path(&self.layout), &records)?;
        Ok(edge)
    }
}
