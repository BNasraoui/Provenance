use super::super::load_corpus;
use super::fixtures::*;
use camino::Utf8PathBuf;
use provenance_core::RequirementStatus;

#[test]
fn load_corpus_reads_state_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let repo = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let layout = provenance_store::layout::ProvenanceLayout::new(repo.clone());
    provenance_store::jsonl::write_jsonl_atomic(
        &provenance_store::shards::requirements_path(&layout, &scope_id()),
        &[requirement(
            "req_root",
            "Platform shall manage invoicing",
            RequirementStatus::Active,
            vec![],
        )],
    )
    .unwrap();

    let corpus = load_corpus(repo, "default".to_string()).unwrap();
    assert_eq!(corpus.scope, "default");
    assert_eq!(corpus.requirements.len(), 1);
    assert_eq!(corpus.index.roots.len(), 1);
    assert_eq!(corpus.index.counts.requirements, 1);
}
