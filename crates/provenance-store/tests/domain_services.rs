use provenance_core::{
    Manifest, RepoPathPrefix, RequirementStatus, RuleSeverity, RuleStatus, ScopeId, StableId,
};
use provenance_store::{
    layout::ProvenanceLayout,
    state_store::{
        CreateDomainInput, CreateRequirementInput, CreateRuleInput, CreateServiceBindingInput,
        CreateServiceInput, StateStore,
    },
};

fn seeded_store() -> (tempfile::TempDir, StateStore, ScopeId) {
    let dir = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    let scope = ScopeId::new("default").unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_string(&Manifest::default_with_scope(
            scope.clone(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    (dir, StateStore::new(layout), scope)
}

fn seed_domain_rule_and_service(store: &StateStore, scope: &ScopeId) {
    store
        .create_domain(CreateDomainInput {
            scope_id: scope.clone(),
            id: StableId::new("domain_payroll").unwrap(),
            name: "Payroll".into(),
            description: Some("Payroll compliance".into()),
            color: Some("#3b82f6".into()),
        })
        .unwrap();
    store
        .create_domain(CreateDomainInput {
            scope_id: scope.clone(),
            id: StableId::new("domain_awards").unwrap(),
            name: "Awards".into(),
            description: None,
            color: None,
        })
        .unwrap();
    store
        .create_requirement(CreateRequirementInput {
            scope_id: scope.clone(),
            id: StableId::new("req_overtime").unwrap(),
            statement: "Overtime must be traceable".into(),
            description: None,
            status: RequirementStatus::Discovery,
            domain_id: Some(StableId::new("domain_payroll").unwrap()),
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    store
        .create_rule(CreateRuleInput {
            scope_id: scope.clone(),
            id: StableId::new("rule_overtime").unwrap(),
            rule_code: "PAY-001".into(),
            name: None,
            description: None,
            requirement_id: Some(StableId::new("req_overtime").unwrap()),
            resolution_id: None,
            statement: "Pay overtime after threshold".into(),
            status: RuleStatus::Active,
            severity: RuleSeverity::High,
            rule_type: None,
            modality: None,
            confidence: None,
            extraction_method: None,
            source_document: None,
            source_section: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    store
        .create_service(CreateServiceInput {
            scope_id: scope.clone(),
            id: StableId::new("service_payroll_api").unwrap(),
            name: "payroll-api".into(),
            description: Some("Calculates payroll".into()),
            owner: Some("platform".into()),
            repository: Some("github.com/example/payroll-api".into()),
            environment: Some(provenance_core::ServiceEnvironment::Production),
            tier: Some(provenance_core::ServiceTier::Critical),
            external_id: Some("backstage:component/payroll-api".into()),
            status: provenance_core::ServiceStatus::Active,
        })
        .unwrap();
}

#[test]
fn domain_service_records_are_written_deterministically_and_validate_bindings() {
    let (_dir, store, scope) = seeded_store();

    seed_domain_rule_and_service(&store, &scope);

    let binding = store
        .create_service_binding(CreateServiceBindingInput {
            scope_id: scope.clone(),
            rule_id: StableId::new("rule_overtime").unwrap(),
            service_id: StableId::new("service_payroll_api").unwrap(),
            binding_type: provenance_core::ServiceBindingType::Enforces,
        })
        .unwrap();

    assert_eq!(
        store.list_domains(&scope).unwrap()[0].id.as_str(),
        "domain_awards"
    );
    assert_eq!(
        store.list_requirements(&scope).unwrap()[0]
            .domain_id
            .as_ref()
            .unwrap()
            .as_str(),
        "domain_payroll"
    );
    assert_eq!(
        binding.id.as_str(),
        "service_binding_rule_overtime_service_payroll_api_enforces"
    );
    assert!(store
        .create_service_binding(CreateServiceBindingInput {
            scope_id: scope,
            rule_id: StableId::new("rule_missing").unwrap(),
            service_id: StableId::new("service_payroll_api").unwrap(),
            binding_type: provenance_core::ServiceBindingType::Consumes,
        })
        .unwrap_err()
        .to_string()
        .contains("rule does not exist"));
}
