use crate::{
    cli::{IdeationArtifactKind, SchemaCommand},
    output,
};
use serde_json::{json, Value};

mod artifacts;
mod common;

pub(super) fn handle(command: SchemaCommand) -> anyhow::Result<()> {
    match command {
        SchemaCommand::Show { artifact, format } => {
            output::print(format, &schema_for(artifact))?;
        }
    }
    Ok(())
}

fn schema_for(artifact: IdeationArtifactKind) -> Value {
    let schema = match artifact {
        IdeationArtifactKind::Contribution => artifacts::contribution::schema(),
        IdeationArtifactKind::SynthesisPacket => artifacts::synthesis_packet::schema(),
        IdeationArtifactKind::Proposal => artifacts::proposal::schema(),
    };

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "artifact": artifact.name(),
        "schema": schema,
        "$defs": common::definitions()
    })
}

#[cfg(test)]
mod tests;
