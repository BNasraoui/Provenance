export type DeclarationAddress = readonly string[];

export interface SourceDeclaration {
  key: string;
  id?: string;
  name: string;
  kind: string;
  url?: string;
  reference?: string;
}

export interface RequirementDeclaration {
  key: string;
  id?: string;
  statement: string;
  description?: string;
  sources: string[];
}

export interface RuleDeclaration {
  key: string;
  id?: string;
  requirement?: string;
  requirements?: string[];
  statement: string;
  name?: string;
  description?: string;
}

export interface TypedSpecDocument {
  schema_version: 1;
  spec: string;
  declared_by: string;
  sources: SourceDeclaration[];
  requirements: RequirementDeclaration[];
  rules: RuleDeclaration[];
}

export type ResourceKind = "source" | "requirement" | "rule";
export type ReconcileState = "created" | "updated" | "unchanged";

export interface ReconciledResource {
  kind: ResourceKind;
  key: string;
  parent?: string;
  address: string[];
  id: string;
  state: ReconcileState;
  changes?: FieldChange[];
}

export interface FieldChange {
  field: string;
  before: unknown;
  after: unknown;
}

export interface ApplyResult {
  declared_by: string;
  created: number;
  updated: number;
  unchanged: number;
  resources: ReconciledResource[];
}

export interface ImplementationSite {
  file: string;
  line: number;
}

export interface VerificationSite {
  key?: string;
  method: string;
  declared_by?: string;
  file: string;
  line?: number;
  symbol?: string;
}

export interface AffectedRule {
  id: string;
  implementations: ImplementationSite[];
  verifications: VerificationSite[];
}

export interface PlanResult extends ApplyResult {
  affected_rules: AffectedRule[];
}

export interface VerificationRun {
  id: string;
  binding_id: string;
  rule_id: string;
  commit?: string;
  file?: string;
  symbol?: string;
  status: "running" | "passed" | "failed";
}
