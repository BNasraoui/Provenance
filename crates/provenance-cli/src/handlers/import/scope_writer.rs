use super::ScopeExport;
use camino::Utf8PathBuf;
use provenance_core::{Edge, ScopeId};
use provenance_store::layout::ProvenanceLayout;

pub(super) fn write_scope(
    layout: &ProvenanceLayout,
    scope_id: &ScopeId,
    exported: &ScopeExport,
) -> anyhow::Result<()> {
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::sources_path(layout, scope_id),
        &exported.sources,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::domains_path(layout, scope_id),
        &exported.domains,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::requirements_path(layout, scope_id),
        &exported.requirements,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::boundaries_path(layout, scope_id),
        &exported.boundaries,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::topics_path(layout, scope_id),
        &exported.topics,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::questions_path(layout, scope_id),
        &exported.questions,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::resolutions_path(layout, scope_id),
        &exported.resolutions,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::rules_path(layout, scope_id),
        &exported.rules,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::services_path(layout, scope_id),
        &exported.services,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::service_bindings_path(layout, scope_id),
        &exported.service_bindings,
    )?;
    write_edges(layout, scope_id, exported)?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::threads_path(layout, scope_id),
        &exported.threads,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::messages_path(layout, scope_id),
        &exported.messages,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::contributions_path(layout, scope_id),
        &exported.contributions,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::synthesis_packets_path(layout, scope_id),
        &exported.synthesis_packets,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::proposal_cards_path(layout, scope_id),
        &exported.proposal_cards,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::assertion_records_path(layout, scope_id),
        &exported.assertion_records,
    )?;
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::dispositions_path(layout, scope_id),
        &exported.dispositions,
    )?;
    Ok(())
}

fn write_edges(
    layout: &ProvenanceLayout,
    scope_id: &ScopeId,
    exported: &ScopeExport,
) -> anyhow::Result<()> {
    let edge_path = provenance_store::shards::edges_path(layout);
    let mut edges = read_edge_values(layout)?;
    edges.retain(|(edge, _)| edge.scope_id != *scope_id);
    edges.extend(
        exported
            .edges
            .iter()
            .map(|edge| Ok((edge.clone(), serde_json::to_value(edge)?)))
            .collect::<anyhow::Result<Vec<_>>>()?,
    );
    edges.sort_by(|(left, _), (right, _)| left.id.as_str().cmp(right.id.as_str()));
    let edges = edges
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    remove_edge_shards(layout)?;
    provenance_store::jsonl::write_jsonl_atomic(&edge_path, &edges)
}

fn read_edge_values(layout: &ProvenanceLayout) -> anyhow::Result<Vec<(Edge, serde_json::Value)>> {
    if !layout.edges_dir().exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(layout.edges_dir())? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                anyhow::anyhow!("edge shard path is not UTF-8: {}", path.display())
            })?;
            if path.extension() == Some("jsonl") {
                paths.push(path);
            }
        }
    }
    paths.sort();

    let mut records = Vec::new();
    for path in paths {
        for (line_index, line) in std::fs::read_to_string(&path)?.lines().enumerate() {
            let value: serde_json::Value = serde_json::from_str(line).map_err(|error| {
                anyhow::anyhow!(
                    "failed to parse edge shard {} line {}: {error}",
                    path,
                    line_index + 1
                )
            })?;
            let edge = serde_json::from_value(value.clone()).map_err(|error| {
                anyhow::anyhow!(
                    "failed to parse edge shard {} line {}: {error}",
                    path,
                    line_index + 1
                )
            })?;
            records.push((edge, value));
        }
    }
    Ok(records)
}

fn remove_edge_shards(layout: &ProvenanceLayout) -> anyhow::Result<()> {
    if !layout.edges_dir().exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(layout.edges_dir())? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(std::ffi::OsStr::to_str) == Some("jsonl")
        {
            std::fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}
