use crate::{
    cli::{IdeationArtifactKind, SchemaCommand},
    output,
};
use serde_json::{json, Value};

pub(super) fn handle(command: SchemaCommand) -> anyhow::Result<()> {
    match command {
        SchemaCommand::Show { artifact, format } => {
            output::print(format, &schema_for(artifact))?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(super) fn schema_for(artifact: IdeationArtifactKind) -> Value {
    let schema = match artifact {
        IdeationArtifactKind::Contribution => json!({
            "title": "Contribution",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "schema_version",
                "scope_id",
                "id",
                "target",
                "participant_slot",
                "stance",
                "strongest_finding",
                "evidence_references",
                "material_claims",
                "risks",
                "objections",
                "challenges",
                "suggested_artifact_changes",
                "unsupported_recommendations",
                "uncertainty",
                "open_questions"
            ],
            "properties": {
                "schema_version": {"const": 1},
                "scope_id": {"$ref": "#/$defs/scopeId"},
                "id": {"$ref": "#/$defs/stableId"},
                "target": {"$ref": "#/$defs/ideationTarget"},
                "participant_slot": {"type": "string", "minLength": 1},
                "stance": {"enum": ["support", "oppose", "mixed", "needs_more_evidence"]},
                "strongest_finding": {"type": "string"},
                "evidence_references": {"type": "array", "items": {"$ref": "#/$defs/evidenceReference"}},
                "material_claims": {"type": "array", "items": {"$ref": "#/$defs/materialClaim"}},
                "risks": {"type": "array", "items": {"type": "string"}},
                "objections": {"type": "array", "items": {"type": "string"}},
                "challenges": {"type": "array", "items": {"$ref": "#/$defs/claimChallenge"}},
                "suggested_artifact_changes": {"type": "array", "items": {"$ref": "#/$defs/suggestedArtifactChange"}},
                "unsupported_recommendations": {"type": "array", "items": {"$ref": "#/$defs/unsupportedRecommendation"}},
                "uncertainty": {"$ref": "#/$defs/uncertainty"},
                "open_questions": {"type": "array", "items": {"type": "string"}}
            }
        }),
        IdeationArtifactKind::SynthesisPacket => json!({
            "title": "SynthesisPacket",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "schema_version",
                "scope_id",
                "id",
                "target",
                "summary",
                "consensus",
                "contested_claims",
                "minority_objections",
                "evidence_gaps",
                "unsupported_speculation",
                "open_questions",
                "suggested_artifacts",
                "required_human_decisions"
            ],
            "properties": {
                "schema_version": {"const": 1},
                "scope_id": {"$ref": "#/$defs/scopeId"},
                "id": {"$ref": "#/$defs/stableId"},
                "target": {"$ref": "#/$defs/ideationTarget"},
                "summary": {"type": "string"},
                "consensus": {"type": "array", "items": {"$ref": "#/$defs/consensusFinding"}},
                "contested_claims": {"type": "array", "items": {"$ref": "#/$defs/contestedClaim"}},
                "minority_objections": {"type": "array", "items": {"$ref": "#/$defs/minorityObjection"}},
                "evidence_gaps": {"type": "array", "items": {"$ref": "#/$defs/evidenceGap"}},
                "unsupported_speculation": {"type": "array", "items": {"$ref": "#/$defs/unsupportedSpeculation"}},
                "open_questions": {"type": "array", "items": {"type": "string"}},
                "suggested_artifacts": {"type": "array", "items": {"$ref": "#/$defs/suggestedArtifact"}},
                "required_human_decisions": {"type": "array", "items": {"$ref": "#/$defs/requiredHumanDecision"}}
            }
        }),
        IdeationArtifactKind::Proposal => json!({
            "title": "ProposalCard",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "schema_version",
                "scope_id",
                "id",
                "proposal_key",
                "proposal_type",
                "title",
                "summary",
                "traceability",
                "promotion_state"
            ],
            "properties": {
                "schema_version": {"const": 1},
                "scope_id": {"$ref": "#/$defs/scopeId"},
                "id": {"$ref": "#/$defs/stableId"},
                "proposal_key": {"type": "string", "minLength": 1},
                "proposal_type": {"enum": ["requirement_candidate", "rule_candidate", "resolution_candidate", "source_gap", "question"]},
                "title": {"type": "string"},
                "summary": {"type": "string"},
                "confidence": {"type": "number", "minimum": 0.0, "maximum": 1.0},
                "traceability": {"$ref": "#/$defs/proposalTraceability"},
                "promotion_state": {"enum": ["proposed", "accepted", "rejected", "deferred", "duplicate", "superseded"]},
                "duplicate_of": {"$ref": "#/$defs/stableId"},
                "superseded_by": {"$ref": "#/$defs/stableId"}
            }
        }),
    };

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "artifact": artifact.name(),
        "schema": schema,
        "$defs": common_defs()
    })
}

#[allow(clippy::too_many_lines)]
fn common_defs() -> Value {
    json!({
        "stableId": {
            "type": "string",
            "pattern": "^[a-z0-9_-]+$",
            "description": "StableId: lowercase ASCII letters, digits, '_' or '-'."
        },
        "scopeId": {
            "type": "string",
            "pattern": "^[a-z0-9_-]+$",
            "description": "ScopeId: lowercase ASCII letters, digits, '_' or '-'."
        },
        "stableIdArray": {
            "type": "array",
            "items": {"$ref": "#/$defs/stableId"}
        },
        "ideationTarget": {
            "type": "object",
            "additionalProperties": false,
            "required": ["artifact_type", "artifact_id"],
            "properties": {
                "artifact_type": {"enum": ["source", "requirement", "resolution", "rule", "question"]},
                "artifact_id": {"$ref": "#/$defs/stableId"}
            }
        },
        "evidenceReference": {
            "type": "object",
            "additionalProperties": false,
            "required": ["reference_id", "evidence_type", "summary"],
            "properties": {
                "reference_id": {"$ref": "#/$defs/stableId"},
                "evidence_type": {"enum": ["source", "artifact", "unsupported", "exploratory"]},
                "summary": {"type": "string"},
                "file_path": {"type": "string"},
                "line": {"type": "integer", "minimum": 1}
            }
        },
        "materialClaim": {
            "type": "object",
            "additionalProperties": false,
            "required": ["claim_id", "statement", "evidence_type", "evidence_reference_ids"],
            "properties": {
                "claim_id": {"$ref": "#/$defs/stableId"},
                "statement": {"type": "string"},
                "evidence_type": {"enum": ["source", "artifact", "unsupported", "exploratory"]},
                "evidence_reference_ids": {"$ref": "#/$defs/stableIdArray"},
                "confidence": {"type": "number", "minimum": 0.0, "maximum": 1.0}
            }
        },
        "claimChallenge": {
            "type": "object",
            "additionalProperties": false,
            "required": ["claim_id", "objection"],
            "properties": {
                "claim_id": {"$ref": "#/$defs/stableId"},
                "objection": {"type": "string"}
            }
        },
        "suggestedArtifactChange": {
            "type": "object",
            "additionalProperties": false,
            "required": ["artifact_type", "change_type", "supporting_claim_ids", "summary"],
            "properties": {
                "artifact_type": {"enum": ["source", "requirement", "resolution", "rule", "question"]},
                "artifact_id": {"$ref": "#/$defs/stableId"},
                "change_type": {"enum": ["create", "update", "supersede", "link"]},
                "supporting_claim_ids": {"$ref": "#/$defs/stableIdArray"},
                "summary": {"type": "string"}
            }
        },
        "unsupportedRecommendation": {
            "type": "object",
            "additionalProperties": false,
            "required": ["recommendation", "marker"],
            "properties": {
                "recommendation": {"type": "string"},
                "marker": {"enum": ["unsupported", "exploratory"]}
            }
        },
        "uncertainty": {
            "type": "object",
            "additionalProperties": false,
            "required": ["level", "rationale"],
            "properties": {
                "level": {"enum": ["low", "medium", "high"]},
                "rationale": {"type": "string"}
            }
        },
        "consensusFinding": {
            "type": "object",
            "additionalProperties": false,
            "required": ["statement", "supporting_participant_slots", "evidence_reference_ids"],
            "properties": {
                "statement": {"type": "string"},
                "supporting_participant_slots": {"type": "array", "items": {"type": "string"}},
                "evidence_reference_ids": {"$ref": "#/$defs/stableIdArray"}
            }
        },
        "contestedClaim": {
            "type": "object",
            "additionalProperties": false,
            "required": ["claim_id", "statement", "supporting_participant_slots", "opposing_participant_slots", "evidence_quality"],
            "properties": {
                "claim_id": {"$ref": "#/$defs/stableId"},
                "statement": {"type": "string"},
                "supporting_participant_slots": {"type": "array", "items": {"type": "string"}},
                "opposing_participant_slots": {"type": "array", "items": {"type": "string"}},
                "evidence_quality": {"enum": ["strong", "mixed", "weak", "unsupported"]}
            }
        },
        "minorityObjection": {
            "type": "object",
            "additionalProperties": false,
            "required": ["participant_slot", "objection", "evidence_reference_ids"],
            "properties": {
                "participant_slot": {"type": "string"},
                "objection": {"type": "string"},
                "evidence_reference_ids": {"$ref": "#/$defs/stableIdArray"}
            }
        },
        "evidenceGap": {
            "type": "object",
            "additionalProperties": false,
            "required": ["question", "needed_evidence_type", "blocking_promotion"],
            "properties": {
                "question": {"type": "string"},
                "needed_evidence_type": {"enum": ["source", "artifact", "unsupported", "exploratory"]},
                "blocking_promotion": {"type": "boolean"}
            }
        },
        "unsupportedSpeculation": {
            "type": "object",
            "additionalProperties": false,
            "required": ["statement", "originating_participant_slots", "marker"],
            "properties": {
                "statement": {"type": "string"},
                "originating_participant_slots": {"type": "array", "items": {"type": "string"}},
                "marker": {"enum": ["unsupported", "exploratory"]}
            }
        },
        "suggestedArtifact": {
            "type": "object",
            "additionalProperties": false,
            "required": ["proposal_key", "proposal_type", "summary", "origin_participant_slots"],
            "properties": {
                "proposal_key": {"type": "string", "minLength": 1},
                "proposal_type": {"enum": ["requirement_candidate", "rule_candidate", "resolution_candidate", "source_gap", "question"]},
                "summary": {"type": "string"},
                "origin_participant_slots": {"type": "array", "items": {"type": "string"}}
            }
        },
        "requiredHumanDecision": {
            "type": "object",
            "additionalProperties": false,
            "required": ["decision_key", "prompt", "blocks_promotion"],
            "properties": {
                "decision_key": {"$ref": "#/$defs/stableId"},
                "prompt": {"type": "string"},
                "blocks_promotion": {"type": "boolean"}
            }
        },
        "proposalTraceability": {
            "type": "object",
            "additionalProperties": false,
            "required": ["target", "source_ids", "evidence_references", "supporting_claim_ids"],
            "properties": {
                "target": {"$ref": "#/$defs/ideationTarget"},
                "source_ids": {"$ref": "#/$defs/stableIdArray"},
                "evidence_references": {"type": "array", "items": {"$ref": "#/$defs/evidenceReference"}},
                "supporting_claim_ids": {"$ref": "#/$defs/stableIdArray"}
            }
        }
    })
}
