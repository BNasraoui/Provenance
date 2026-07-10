use super::super::common::model::{enum_names, PROMOTION_STATES, PROPOSAL_TYPES};
use serde_json::{json, Value};

pub(in crate::handlers::schema) fn schema() -> Value {
    json!({
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
            "proposal_type": {"enum": enum_names(&PROPOSAL_TYPES)},
            "title": {"type": "string"},
            "summary": {"type": "string"},
            "confidence": {"type": "number", "minimum": 0.0, "maximum": 1.0},
            "traceability": {"$ref": "#/$defs/proposalTraceability"},
            "promotion_state": {"enum": enum_names(&PROMOTION_STATES)},
            "duplicate_of": {"$ref": "#/$defs/stableId"},
            "superseded_by": {"$ref": "#/$defs/stableId"}
        }
    })
}
