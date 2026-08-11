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
  requirement: string;
  statement: string;
  name?: string;
  description?: string;
}

export interface TypedSpecDocument {
  schema_version: 1;
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
  id: string;
  state: ReconcileState;
}

export interface ApplyResult {
  declared_by: string;
  created: number;
  updated: number;
  unchanged: number;
  resources: ReconciledResource[];
}

export interface VerificationRun {
  id: string;
  rule_id: string;
  status: "running" | "passed" | "failed";
}
