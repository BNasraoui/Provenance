//! Verification of `rule_export_strips_collaboration`: a graph that leaves
//! the repository carries no trace of who was talking or who claimed what.

use super::*;

/// One collaboration field on one record kind: the domain of the
/// exhaustion proof below. One variant per `visit_field!` line in
/// `visit_collaboration_fields`; `visited_fields` below ties the two
/// together, so a field added to the walk fails this test until it joins
/// the chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollaborationSite {
    SourceOriginThread,
    SourceOriginMessage,
    RequirementOriginThread,
    RequirementOriginMessage,
    TopicClaimedBy,
    TopicClaimedAt,
    QuestionClaimedBy,
    QuestionClaimedAt,
    ResolutionOriginThread,
    ResolutionOriginMessage,
    RuleOriginThread,
    RuleOriginMessage,
}

// Built from an exhaustive match, in the order the walk visits the fields,
// so adding a CollaborationSite variant fails compilation until the new
// site joins the chain.
fn all_sites() -> Vec<CollaborationSite> {
    let mut all = vec![CollaborationSite::SourceOriginThread];
    while let Some(next) = match all.last().unwrap() {
        CollaborationSite::SourceOriginThread => Some(CollaborationSite::SourceOriginMessage),
        CollaborationSite::SourceOriginMessage => Some(CollaborationSite::RequirementOriginThread),
        CollaborationSite::RequirementOriginThread => {
            Some(CollaborationSite::RequirementOriginMessage)
        }
        CollaborationSite::RequirementOriginMessage => Some(CollaborationSite::TopicClaimedBy),
        CollaborationSite::TopicClaimedBy => Some(CollaborationSite::TopicClaimedAt),
        CollaborationSite::TopicClaimedAt => Some(CollaborationSite::QuestionClaimedBy),
        CollaborationSite::QuestionClaimedBy => Some(CollaborationSite::QuestionClaimedAt),
        CollaborationSite::QuestionClaimedAt => Some(CollaborationSite::ResolutionOriginThread),
        CollaborationSite::ResolutionOriginThread => {
            Some(CollaborationSite::ResolutionOriginMessage)
        }
        CollaborationSite::ResolutionOriginMessage => Some(CollaborationSite::RuleOriginThread),
        CollaborationSite::RuleOriginThread => Some(CollaborationSite::RuleOriginMessage),
        CollaborationSite::RuleOriginMessage => None,
    } {
        all.push(next);
    }
    all
}

const fn site_field(site: CollaborationSite) -> &'static str {
    match site {
        CollaborationSite::SourceOriginThread
        | CollaborationSite::RequirementOriginThread
        | CollaborationSite::ResolutionOriginThread
        | CollaborationSite::RuleOriginThread => "origin_thread",
        CollaborationSite::SourceOriginMessage
        | CollaborationSite::RequirementOriginMessage
        | CollaborationSite::ResolutionOriginMessage
        | CollaborationSite::RuleOriginMessage => "origin_message",
        CollaborationSite::TopicClaimedBy | CollaborationSite::QuestionClaimedBy => "claimed_by",
        CollaborationSite::TopicClaimedAt | CollaborationSite::QuestionClaimedAt => "claimed_at",
    }
}

fn thread_id() -> StableId {
    StableId::new("thread_private").unwrap()
}

fn message_id() -> StableId {
    StableId::new("message_private").unwrap()
}

/// Sets one collaboration field on every record of its kind, leaving the
/// rest of the graph alone.
fn populate_site(graph: &mut GraphExport, site: CollaborationSite) {
    match site {
        CollaborationSite::SourceOriginThread => {
            for record in &mut graph.sources {
                record.origin_thread = Some(thread_id());
            }
        }
        CollaborationSite::SourceOriginMessage => {
            for record in &mut graph.sources {
                record.origin_message = Some(message_id());
            }
        }
        CollaborationSite::RequirementOriginThread => {
            for record in &mut graph.requirements {
                record.origin_thread = Some(thread_id());
            }
        }
        CollaborationSite::RequirementOriginMessage => {
            for record in &mut graph.requirements {
                record.origin_message = Some(message_id());
            }
        }
        CollaborationSite::TopicClaimedBy => {
            for record in &mut graph.topics {
                record.claimed_by = Some("workflowd-123".into());
            }
        }
        CollaborationSite::TopicClaimedAt => {
            for record in &mut graph.topics {
                record.claimed_at = Some(1_700_000_000);
            }
        }
        CollaborationSite::QuestionClaimedBy => {
            for record in &mut graph.questions {
                record.claimed_by = Some("workflowd-123".into());
            }
        }
        CollaborationSite::QuestionClaimedAt => {
            for record in &mut graph.questions {
                record.claimed_at = Some(1_700_000_000);
            }
        }
        CollaborationSite::ResolutionOriginThread => {
            for record in &mut graph.resolutions {
                record.origin_thread = Some(thread_id());
            }
        }
        CollaborationSite::ResolutionOriginMessage => {
            for record in &mut graph.resolutions {
                record.origin_message = Some(message_id());
            }
        }
        CollaborationSite::RuleOriginThread => {
            for record in &mut graph.rules {
                record.origin_thread = Some(thread_id());
            }
        }
        CollaborationSite::RuleOriginMessage => {
            for record in &mut graph.rules {
                record.origin_message = Some(message_id());
            }
        }
    }
}

/// Every field the shared walk visits, in walk order. Read off the walk
/// itself rather than listed here, so the domain of the exhaustion proof
/// is the implementation's field list and not a hand-kept copy of it.
fn visited_fields(graph: &mut GraphExport) -> Vec<&'static str> {
    let mut visited = Vec::new();
    visit_collaboration_fields(graph, &mut |name, _| visited.push(name));
    visited
}

/// The fields the walk visits that are currently set.
fn populated_fields(graph: &mut GraphExport) -> Vec<&'static str> {
    let mut populated = Vec::new();
    visit_collaboration_fields(graph, &mut |name, field| {
        if field.is_populated() {
            populated.push(name);
        }
    });
    populated
}

/// A graph holding one record of every family, every collaboration field
/// clear.
fn full_graph(scope: &ScopeId) -> GraphExport {
    graph_in_scope(scope, &all_families())
}

#[test]
#[verifies("rule_export_strips_collaboration", exhaustion)]
fn every_collaboration_field_is_rejected_and_cleared() {
    let claimed = ScopeId::new("default").unwrap();
    let sites = all_sites();

    // The domain: with one record of every family, the walk visits each
    // (record kind, field) pair exactly once, so this is the field list
    // the export and import halves share. A field added to the walk shows
    // up here and fails the proof until it joins CollaborationSite.
    assert_eq!(
        visited_fields(&mut full_graph(&claimed)),
        sites
            .iter()
            .map(|site| site_field(*site))
            .collect::<Vec<_>>(),
        "every collaboration field the shared walk visits must be covered by CollaborationSite"
    );

    for &site in &sites {
        let mut graph = full_graph(&claimed);
        populate_site(&mut graph, site);

        let Err(GraphReferenceError::Incomplete { detail }) =
            graph.validate_no_collaboration_fields()
        else {
            panic!("an exact export carrying {site:?} was accepted on import");
        };
        assert!(
            detail.contains(site_field(site)),
            "the refusal must name the {site:?} field, got: {detail}"
        );

        strip_collaboration_fields(&mut graph);
        assert!(
            populated_fields(&mut graph).is_empty(),
            "export left {site:?} set after stripping"
        );
    }
}

#[test]
#[verifies("rule_export_strips_collaboration", property)]
fn a_stripped_graph_is_always_accepted_on_import() {
    // The property: whatever collaboration state a graph starts with,
    // stripping it leaves a graph the import check accepts. Stated without
    // reference to which fields exist, so which ones are set must not
    // matter. The generator walks every subset of the field sites, over a
    // graph holding every record family and over one holding none.
    let claimed = ScopeId::new("default").unwrap();
    let sites = all_sites();
    for families in [all_families(), Vec::new()] {
        for mask in 0u32..(1u32 << sites.len()) {
            let mut graph = graph_in_scope(&claimed, &families);
            for (index, &site) in sites.iter().enumerate() {
                if mask & (1u32 << index) != 0 {
                    populate_site(&mut graph, site);
                }
            }
            strip_collaboration_fields(&mut graph);
            assert!(
                graph.validate_no_collaboration_fields().is_ok(),
                "a stripped graph was refused on import (site mask {mask:#x})"
            );
        }
    }
}
