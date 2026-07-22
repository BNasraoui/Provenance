use serde_json::{json, Value};

pub(in crate::handlers::schema) fn reference_schema() -> Value {
    json!({
        "title": "GraphReference",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "schema_version", "reference_id", "repository_id", "store_path",
            "scope_id", "commit", "graph_digest"
        ],
        "properties": {
            "schema_version": {"const": 1},
            "reference_id": {"type": "string", "pattern": "^grf1_[0-9a-f]{64}$"},
            "repository_id": {"type": "string", "pattern": "^git1_[0-9a-f]{64}$"},
            "store_path": {"const": ".provenance/state"},
            "scope_id": {"$ref": "#/$defs/scopeId"},
            "commit": {"type": "string", "pattern": "^[0-9a-f]{40,64}$"},
            "graph_digest": {"type": "string", "pattern": "^sha256:[0-9a-f]{64}$"},
            "correlation": {
                "type": "object",
                "additionalProperties": false,
                "required": ["system", "key"],
                "properties": {
                    "system": {"type": "string", "minLength": 1},
                    "key": {"type": "string", "minLength": 1}
                }
            }
        }
    })
}

pub(in crate::handlers::schema) fn export_schema() -> Value {
    let families = [
        "sources",
        "domains",
        "requirements",
        "boundaries",
        "topics",
        "questions",
        "resolutions",
        "rules",
        "services",
        "service_bindings",
        "edges",
    ];
    let mut graph_properties = serde_json::Map::new();
    graph_properties.insert("schema_version".into(), json!({"const": 1}));
    graph_properties.insert("scope".into(), json!({"type": "object"}));
    for family in families {
        graph_properties.insert(
            family.into(),
            json!({"type": "array", "items": {"type": "object"}}),
        );
    }
    json!({
        "title": "GraphReferenceExactExport",
        "type": "object",
        "additionalProperties": false,
        "required": ["schema_version", "operation", "reference_id", "graph"],
        "properties": {
            "schema_version": {"const": 1},
            "operation": {"const": "exact-export"},
            "reference_id": {"type": "string", "pattern": "^grf1_[0-9a-f]{64}$"},
            "graph": {
                "type": "object",
                "additionalProperties": false,
                "required": [
                    "schema_version", "scope", "sources", "domains", "requirements",
                    "boundaries", "topics", "questions", "resolutions", "rules",
                    "services", "service_bindings", "edges"
                ],
                "properties": graph_properties
            }
        }
    })
}
