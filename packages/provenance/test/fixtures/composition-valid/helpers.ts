import type {
  RequirementDeclaration,
  RuleDeclaration,
  SourceDeclaration,
  SpecAuthoring,
} from "../../../dist/index.js";

export function policy<const Spec extends string>(
  author: SpecAuthoring<Spec>,
): SourceDeclaration<Spec, "policy"> {
  return author.source("policy").name("Security policy").document("docs/policy.md");
}

export function sharing<const Spec extends string, const SourceKey extends string>(
  author: SpecAuthoring<Spec>,
  source: SourceDeclaration<Spec, SourceKey>,
): RequirementDeclaration<Spec, "sharing"> {
  return author.requirement("sharing").statement("Shares expire").from(source);
}

export function localExpiry<const Spec extends string, const RequirementKey extends string>(
  requirement: RequirementDeclaration<Spec, RequirementKey>,
): RuleDeclaration<Spec, "expiry", RequirementKey> {
  return requirement.rule("expiry").statement("Share links expire");
}

export function sharedAuthentication<const Spec extends string>(
  author: SpecAuthoring<Spec>,
): RuleDeclaration<Spec, "authenticated", undefined> {
  return author.rule("authenticated").statement("Authenticated access is required");
}

export function preserveRule<
  const Spec extends string,
  const Key extends string,
  const RequirementKey extends string | undefined,
>(
  declaration: RuleDeclaration<Spec, Key, RequirementKey>,
): RuleDeclaration<Spec, Key, RequirementKey> {
  return declaration.id(`existing-${declaration.key}`);
}

export function bindClass<
  const Spec extends string,
  const Key extends string,
  const RequirementKey extends string | undefined,
>(
  declaration: RuleDeclaration<Spec, Key, RequirementKey>,
  target: abstract new (...args: never[]) => unknown,
): RuleDeclaration<Spec, Key, RequirementKey> {
  return declaration.implementedBy(target);
}
