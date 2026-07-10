use super::super::*;

#[test]
fn domain_service_records_roundtrip_without_hosted_fields() {
    let domain = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "domain_payroll",
        "name": "Payroll",
        "description": "Payroll compliance requirements",
        "color": "#3b82f6"
    });
    let requirement = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "req_overtime",
        "statement": "Overtime must be traceable",
        "status": "discovery",
        "domainId": "domain_payroll"
    });
    let service = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "service_payroll_api",
        "name": "payroll-api",
        "description": "Calculates payroll",
        "owner": "platform",
        "repository": "github.com/example/payroll-api",
        "environment": "production",
        "tier": "critical",
        "externalId": "backstage:component/payroll-api",
        "status": "active"
    });
    let service_binding = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "binding_overtime_payroll_enforces",
        "ruleId": "rule_overtime",
        "serviceId": "service_payroll_api",
        "bindingType": "enforces"
    });

    let domain: Domain = serde_json::from_value(domain).unwrap();
    let requirement: Requirement = serde_json::from_value(requirement).unwrap();
    let service: Service = serde_json::from_value(service).unwrap();
    let service_binding: ServiceBinding = serde_json::from_value(service_binding).unwrap();

    let domain = serde_json::to_value(domain).unwrap();
    let requirement = serde_json::to_value(requirement).unwrap();
    let service = serde_json::to_value(service).unwrap();
    let service_binding = serde_json::to_value(service_binding).unwrap();

    assert_eq!(domain["schema_version"], 1);
    assert_eq!(domain["id"], "domain_payroll");
    assert!(domain.get("createdBy").is_none());
    assert!(domain.get("updatedAt").is_none());
    assert_eq!(requirement["domain_id"], "domain_payroll");
    assert_eq!(service["environment"], "production");
    assert_eq!(service["tier"], "critical");
    assert_eq!(service["external_id"], "backstage:component/payroll-api");
    assert_eq!(service_binding["binding_type"], "enforces");
    assert_eq!(service_binding["rule_id"], "rule_overtime");
}
