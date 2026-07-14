use super::graph::{EdgeType, NodeType};

#[test]
fn topic_and_question_are_thread_parent_node_types_but_not_edge_endpoints() {
    assert_eq!(NodeType::parse("topic").unwrap(), NodeType::Topic);
    assert_eq!(NodeType::parse("question").unwrap(), NodeType::Question);
    assert!(crate::edge_validation::validate_edge_endpoint(
        EdgeType::DependsOn,
        NodeType::Topic,
        NodeType::Topic,
    )
    .is_err());
}
