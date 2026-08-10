DROP INDEX IF EXISTS idx_rules_code;
ALTER TABLE rules DROP COLUMN rule_code;

DROP INDEX IF EXISTS idx_service_bindings_unique;
DROP INDEX IF EXISTS idx_service_bindings_service;
DROP INDEX IF EXISTS idx_service_bindings_rule;
DROP TABLE IF EXISTS service_bindings;

DROP INDEX IF EXISTS idx_services_status;
DROP INDEX IF EXISTS idx_services_name;
DROP TABLE IF EXISTS services;
