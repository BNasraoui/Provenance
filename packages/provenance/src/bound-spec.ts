import {
  createAuthoringContext,
  BoundRequirement,
  BoundRule,
  BoundSource,
  type RuleVerifier,
} from "./bound-declarations.js";
import type {
  RequirementDeclaration,
  RuleDeclaration,
  SourceDeclaration,
  SpecAuthoring,
} from "./bound-types.js";
import { buildBoundSpec } from "./bound-materialize.js";
import {
  fluentSpec,
  type FluentRequirement,
  type FluentRule,
  type FluentSource,
  type FluentSpec,
} from "./fluent-spec.js";
import type { SpecHandle } from "./spec.js";

class BoundSpecAuthor<SpecKey extends string> implements SpecAuthoring<SpecKey> {
  readonly key: SpecKey;
  readonly #context: ReturnType<typeof createAuthoringContext>;

  constructor(key: SpecKey, verify: RuleVerifier) {
    this.#context = createAuthoringContext(key, verify);
    this.key = key;
    Object.freeze(this);
  }

  source<const Key extends string>(key: Key): SourceDeclaration<SpecKey, Key> {
    return BoundSource.create(this.#context, key);
  }

  requirement<const Key extends string>(key: Key): RequirementDeclaration<SpecKey, Key> {
    return BoundRequirement.create(this.#context, key);
  }

  rule<const Key extends string>(key: Key): RuleDeclaration<SpecKey, Key, undefined> {
    return BoundRule.shared(this.#context, key);
  }

  build<const Requirements extends readonly RequirementDeclaration<SpecKey, any>[]>(
    ...requirements: Requirements
  ): SpecHandle<Readonly<Record<string, unknown>>> {
    return buildBoundSpec(this.#context, requirements);
  }

  sources<const Added extends readonly FluentSource[]>(
    ...sources: Added
  ): FluentSpec<Added, readonly []> {
    return fluentSpec(this.key, this.#context.verify).sources(...sources);
  }

  requirements<const Added extends readonly FluentRequirement<string, readonly FluentRule[]>[]>(
    ...requirements: Added
  ): FluentSpec<readonly [], Added> {
    return fluentSpec(this.key, this.#context.verify).requirements(...requirements);
  }
}

export function authorSpec<const Key extends string>(
  key: Key,
  verify: RuleVerifier,
): SpecAuthoring<Key> {
  return new BoundSpecAuthor(key, verify);
}

export type {
  BoundRequirement,
  BoundRule,
  BoundSource,
  RequirementDeclaration,
  RuleDeclaration,
  SourceDeclaration,
  SpecAuthoring,
} from "./bound-types.js";
