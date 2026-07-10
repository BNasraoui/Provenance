use super::{initialized_store, seeded_requirement_store};
use crate::state_store::{AddSourceReferenceInput, CreateRequirementInput, CreateSourceInput};
use provenance_core::{EdgeType, RequirementStatus, SourceType, StableId};

#[test]
fn source_requirement_records_are_written_deterministically() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_source(CreateSourceInput {
            scope_id: scope.clone(),
            id: StableId::new("source_schads").unwrap(),
            name: "SCHADS Award".into(),
            source_type: SourceType::Policy,
            url: None,
            reference: None,
            commit_pin: None,
            effective_date: None,
            review_date: None,
            superseded_by: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    store
        .create_requirement(CreateRequirementInput {
            scope_id: scope.clone(),
            id: StableId::new("req_overtime").unwrap(),
            statement: "Overtime".into(),
            description: None,
            status: RequirementStatus::Active,
            domain_id: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    store
        .add_source_reference(AddSourceReferenceInput {
            scope_id: scope.clone(),
            source_id: StableId::new("source_schads").unwrap(),
            requirement_id: StableId::new("req_overtime").unwrap(),
            clause: None,
        })
        .unwrap();
    assert_eq!(
        store.list_sources(&scope).unwrap()[0].id.as_str(),
        "source_schads"
    );
    assert_eq!(
        store.list_edges().unwrap()[0].edge_type,
        EdgeType::References
    );
}

#[test]
fn concurrent_source_creates_preserve_all_records() {
    let (_dir, store, scope) = initialized_store();

    for index in 0..200 {
        store
            .create_source(CreateSourceInput {
                scope_id: scope.clone(),
                id: StableId::new(format!("source_seed_{index:03}")).unwrap(),
                name: format!("Seed {index:03}"),
                source_type: SourceType::Policy,
                url: None,
                reference: None,
                commit_pin: None,
                effective_date: None,
                review_date: None,
                superseded_by: None,
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();
    }

    let writer_count = 16;
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(writer_count));
    let mut handles = Vec::new();
    for index in 0..writer_count {
        let store = store.clone();
        let scope = scope.clone();
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            store
                .create_source(CreateSourceInput {
                    scope_id: scope,
                    id: StableId::new(format!("source_concurrent_{index:03}")).unwrap(),
                    name: format!("Concurrent {index:03}"),
                    source_type: SourceType::Policy,
                    url: None,
                    reference: None,
                    commit_pin: None,
                    effective_date: None,
                    review_date: None,
                    superseded_by: None,
                    origin_thread: None,
                    origin_message: None,
                })
                .unwrap();
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let sources = store.list_sources(&scope).unwrap();
    assert_eq!(sources.len(), 200 + writer_count);
    for index in 0..writer_count {
        assert!(sources
            .iter()
            .any(|source| { source.id.as_str() == format!("source_concurrent_{index:03}") }));
    }
}

#[test]
fn requirement_fog_is_set_and_cleared_as_free_text() {
    let (_dir, store, scope) = seeded_requirement_store();
    let requirement_id = StableId::new("req_overtime").unwrap();

    let updated = store
        .set_requirement_fog(
            &scope,
            &requirement_id,
            Some("something about public holidays and sleepovers".into()),
        )
        .unwrap();
    assert_eq!(
        updated.fog.as_deref(),
        Some("something about public holidays and sleepovers")
    );
    assert_eq!(
        store.list_requirements(&scope).unwrap()[0].fog.as_deref(),
        Some("something about public holidays and sleepovers")
    );

    let cleared = store
        .set_requirement_fog(&scope, &requirement_id, None)
        .unwrap();
    assert_eq!(cleared.fog, None);
    assert!(store
        .set_requirement_fog(&scope, &StableId::new("req_missing").unwrap(), None)
        .unwrap_err()
        .to_string()
        .contains("requirement does not exist"));
}
