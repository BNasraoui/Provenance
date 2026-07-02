ALTER TABLE requirements ADD COLUMN domain_id TEXT;

CREATE TABLE IF NOT EXISTS domains (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_domains_name ON domains(scope_id, name);

CREATE TABLE IF NOT EXISTS services (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    owner TEXT,
    repository TEXT,
    environment TEXT,
    tier TEXT,
    external_id TEXT,
    status TEXT NOT NULL,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_services_name ON services(scope_id, name);
CREATE INDEX IF NOT EXISTS idx_services_status ON services(scope_id, status);

CREATE TABLE IF NOT EXISTS service_bindings (
    scope_id TEXT NOT NULL,
    id TEXT NOT NULL,
    rule_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    binding_type TEXT NOT NULL,
    PRIMARY KEY (scope_id, id)
);
CREATE INDEX IF NOT EXISTS idx_service_bindings_rule ON service_bindings(scope_id, rule_id);
CREATE INDEX IF NOT EXISTS idx_service_bindings_service ON service_bindings(scope_id, service_id);
CREATE INDEX IF NOT EXISTS idx_service_bindings_unique ON service_bindings(scope_id, rule_id, service_id, binding_type);
