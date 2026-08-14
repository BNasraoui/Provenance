import type {
  RequirementDeclaration,
  SourceDeclaration,
} from "../../../dist/index.js";

export function attachSource<
  const Spec extends string,
  const RequirementKey extends string,
  const SourceKey extends string,
>(
  requirement: RequirementDeclaration<Spec, RequirementKey>,
  source: SourceDeclaration<Spec, SourceKey>,
): RequirementDeclaration<Spec, RequirementKey> {
  return requirement.from(source);
}
