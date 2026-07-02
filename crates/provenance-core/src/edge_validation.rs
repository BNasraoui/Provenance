use crate::{EdgeType, NodeType};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EdgeValidationError {
    #[error("invalid {edge_type:?} edge endpoint: {from:?} -> {to:?}")]
    InvalidEdgeEndpoint {
        edge_type: EdgeType,
        from: NodeType,
        to: NodeType,
    },
}

pub fn validate_edge_endpoint(
    edge_type: EdgeType,
    from: NodeType,
    to: NodeType,
) -> Result<(), EdgeValidationError> {
    let valid = match edge_type {
        EdgeType::References => from == NodeType::Source && to == NodeType::Requirement,
        EdgeType::RefinesInto => from == NodeType::Requirement && to == NodeType::Requirement,
        EdgeType::DependsOn | EdgeType::Contradicts | EdgeType::Supersedes => {
            from == NodeType::Requirement && to == NodeType::Requirement
        }
        EdgeType::Needs => from == NodeType::Requirement && to == NodeType::Resolution,
        EdgeType::Resolves | EdgeType::Spawns => {
            from == NodeType::Resolution && to == NodeType::Requirement
        }
        EdgeType::Produces => {
            (from == NodeType::Resolution || from == NodeType::Requirement) && to == NodeType::Rule
        }
    };
    if valid {
        Ok(())
    } else {
        Err(EdgeValidationError::InvalidEdgeEndpoint {
            edge_type,
            from,
            to,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_requirement_reference_is_valid() {
        validate_edge_endpoint(
            EdgeType::References,
            NodeType::Source,
            NodeType::Requirement,
        )
        .unwrap();
    }

    #[test]
    fn rejects_source_to_rule_reference_edge() {
        let err = validate_edge_endpoint(EdgeType::References, NodeType::Source, NodeType::Rule)
            .unwrap_err();
        assert!(matches!(
            err,
            EdgeValidationError::InvalidEdgeEndpoint { .. }
        ));
    }

    #[test]
    fn accepts_phase_three_traceability_edges() {
        validate_edge_endpoint(EdgeType::Needs, NodeType::Requirement, NodeType::Resolution)
            .unwrap();
        validate_edge_endpoint(
            EdgeType::Resolves,
            NodeType::Resolution,
            NodeType::Requirement,
        )
        .unwrap();
        validate_edge_endpoint(EdgeType::Produces, NodeType::Resolution, NodeType::Rule).unwrap();
        validate_edge_endpoint(EdgeType::Produces, NodeType::Requirement, NodeType::Rule).unwrap();
    }

    #[test]
    fn rejects_structural_shaping_records_as_explicit_edges() {
        assert!(
            validate_edge_endpoint(EdgeType::DependsOn, NodeType::Topic, NodeType::Topic).is_err()
        );
        assert!(validate_edge_endpoint(
            EdgeType::DependsOn,
            NodeType::Question,
            NodeType::Question,
        )
        .is_err());
    }
}
