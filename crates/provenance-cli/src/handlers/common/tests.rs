use super::{boundary_source_ref, canonical_artifact, complete_flag_set, resolution_inputs};
use provenance_macros::verifies;

/// Every count a flag family is exhausted over. Zero, one and two cover the
/// whole shape of the decision: a family nobody used, a family naming one
/// record, and a family naming several. Three would only repeat two.
const COUNTS: [usize; 3] = [0, 1, 2];

/// Every count a flag that cannot repeat is exhausted over: clap hands these
/// three families an `Option`, so absent and present is the whole domain.
const PRESENCE: [usize; 2] = [0, 1];

/// Independent restatement of the rule: a family names as many records as its
/// busiest flag, a flag naming a required field has to fill every one of them,
/// and a flag naming an optional field has to fill all of them or none. It
/// takes the record count from the largest count in the whole family rather
/// than from the first required flag, so it is a second reading of the rule
/// and not a copy of `complete_flag_set`.
fn every_record_gets_every_required_field(required: &[usize], optional: &[usize]) -> bool {
    let records = required.iter().chain(optional).copied().max().unwrap_or(0);
    required.iter().all(|&count| count == records)
        && optional.iter().all(|&count| count == 0 || count == records)
}

fn repeated(value: &str, count: usize) -> Vec<String> {
    (0..count).map(|_| value.to_string()).collect()
}

fn given(count: usize, value: &str) -> Option<String> {
    (count > 0).then(|| value.to_string())
}

#[test]
#[verifies("rule_flag_sets_complete", exhaustion)]
fn three_required_flags_match_the_oracle_on_every_count_triple() {
    for types in COUNTS {
        for references in COUNTS {
            for summaries in COUNTS {
                let required = [types, references, summaries];
                assert_eq!(
                    complete_flag_set(&required, &[]),
                    every_record_gets_every_required_field(&required, &[]),
                    "disagreed with the oracle on required counts {required:?}"
                );
            }
        }
    }
}

#[test]
#[verifies("rule_flag_sets_complete", exhaustion)]
fn two_required_flags_match_the_oracle_on_every_count_pair() {
    for artifact_type in COUNTS {
        for artifact_id in COUNTS {
            let required = [artifact_type, artifact_id];
            assert_eq!(
                complete_flag_set(&required, &[]),
                every_record_gets_every_required_field(&required, &[]),
                "disagreed with the oracle on required counts {required:?}"
            );
        }
    }
}

#[test]
#[verifies("rule_flag_sets_complete", exhaustion)]
fn an_optional_flag_matches_the_oracle_on_every_count_pair() {
    for source_id in COUNTS {
        for clause in COUNTS {
            let (required, optional) = ([source_id], [clause]);
            assert_eq!(
                complete_flag_set(&required, &optional),
                every_record_gets_every_required_field(&required, &optional),
                "disagreed with the oracle on required {required:?} optional {optional:?}"
            );
        }
    }
}

#[test]
#[verifies("rule_flag_sets_complete", exhaustion)]
fn resolution_inputs_accepts_exactly_the_complete_count_triples() {
    for types in COUNTS {
        for references in COUNTS {
            for summaries in COUNTS {
                let built = resolution_inputs(
                    repeated("technical", types),
                    repeated("docs/shaping.md", references),
                    repeated("why it was decided", summaries),
                );
                assert_eq!(
                    built.is_ok(),
                    complete_flag_set(&[types, references, summaries], &[]),
                    "counts {types}/{references}/{summaries} were judged differently \
                     from the rule"
                );
                if let Ok(inputs) = built {
                    assert_eq!(inputs.len(), types);
                }
            }
        }
    }
}

#[test]
#[verifies("rule_flag_sets_complete", exhaustion)]
fn canonical_artifact_accepts_exactly_the_complete_flag_pairs() {
    for artifact_type in PRESENCE {
        for artifact_id in PRESENCE {
            let built = canonical_artifact(
                given(artifact_type, "requirement"),
                given(artifact_id, "req_billing"),
            );
            assert_eq!(
                built.is_ok(),
                complete_flag_set(&[artifact_type, artifact_id], &[]),
                "flags {artifact_type}/{artifact_id} were judged differently from the rule"
            );
        }
    }
}

#[test]
#[verifies("rule_flag_sets_complete", exhaustion)]
fn boundary_source_ref_accepts_exactly_the_complete_flag_pairs() {
    for source_id in PRESENCE {
        for clause in PRESENCE {
            let built = boundary_source_ref(
                given(source_id, "src_contract"),
                given(clause, "section 4.2"),
            );
            assert_eq!(
                built.is_ok(),
                complete_flag_set(&[source_id], &[clause]),
                "flags {source_id}/{clause} were judged differently from the rule"
            );
        }
    }
}

#[test]
#[verifies("rule_flag_sets_complete", examples)]
fn a_short_resolution_input_family_names_all_three_flags() {
    let error = resolution_inputs(
        repeated("technical", 2),
        repeated("docs/shaping.md", 1),
        repeated("why it was decided", 2),
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "--input-type, --input-reference, and --input-summary must be provided the same number of times"
    );
}

#[test]
#[verifies("rule_flag_sets_complete", examples)]
fn a_complete_resolution_input_family_builds_one_input_per_repetition() {
    let inputs = resolution_inputs(
        repeated("technical", 2),
        repeated("docs/shaping.md", 2),
        repeated("why it was decided", 2),
    )
    .unwrap();
    assert_eq!(inputs.len(), 2);
    assert_eq!(inputs[0].reference, "docs/shaping.md");
    assert_eq!(inputs[0].summary, "why it was decided");
}

#[test]
#[verifies("rule_flag_sets_complete", examples)]
fn a_canonical_artifact_id_without_its_type_names_both_flags() {
    let error = canonical_artifact(None, Some("req_billing".to_string())).unwrap_err();
    assert_eq!(
        error.to_string(),
        "--canonical-artifact-type and --canonical-artifact-id must be provided together"
    );
}

#[test]
#[verifies("rule_flag_sets_complete", examples)]
fn a_canonical_artifact_type_without_its_id_names_both_flags() {
    let error = canonical_artifact(Some("requirement".to_string()), None).unwrap_err();
    assert_eq!(
        error.to_string(),
        "--canonical-artifact-type and --canonical-artifact-id must be provided together"
    );
}

#[test]
#[verifies("rule_flag_sets_complete", examples)]
fn a_complete_canonical_artifact_family_builds_the_artifact() {
    let artifact = canonical_artifact(
        Some("requirement".to_string()),
        Some("req_billing".to_string()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(artifact.artifact_id.as_str(), "req_billing");
}

#[test]
#[verifies("rule_flag_sets_complete", examples)]
fn a_source_clause_without_its_source_id_names_the_missing_flag() {
    let error = boundary_source_ref(None, Some("section 4.2".to_string())).unwrap_err();
    assert_eq!(error.to_string(), "--source-clause requires --source-id");
}

#[test]
#[verifies("rule_flag_sets_complete", examples)]
fn a_source_id_stands_alone_because_its_clause_is_optional() {
    let reference = boundary_source_ref(Some("src_contract".to_string()), None)
        .unwrap()
        .unwrap();
    assert_eq!(reference.source_id.as_str(), "src_contract");
    assert_eq!(reference.clause, None);
}

#[test]
#[verifies("rule_flag_sets_complete", examples)]
fn a_complete_source_reference_family_keeps_the_clause() {
    let reference = boundary_source_ref(
        Some("src_contract".to_string()),
        Some("section 4.2".to_string()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(reference.clause.as_deref(), Some("section 4.2"));
}

#[test]
#[verifies("rule_flag_sets_complete", examples)]
fn an_unused_family_names_no_record_at_all() {
    assert!(resolution_inputs(Vec::new(), Vec::new(), Vec::new())
        .unwrap()
        .is_empty());
    assert!(canonical_artifact(None, None).unwrap().is_none());
    assert!(boundary_source_ref(None, None).unwrap().is_none());
}

/// The flag family being complete says three flags were given the same number
/// of times, not that any of them said anything. Content is a separate
/// question, decided by `rule_resolution_input_content`, and the create path
/// asks it before it builds a record.
#[test]
#[verifies("rule_resolution_input_content", examples)]
fn a_complete_family_of_blank_flags_still_builds_no_input() {
    let error = resolution_inputs(
        repeated("technical", 1),
        repeated("  ", 1),
        repeated("why it was decided", 1),
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "resolution input reference must not be blank"
    );
}
