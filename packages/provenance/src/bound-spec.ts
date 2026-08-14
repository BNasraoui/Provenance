import {
  createAuthoringContext,
  BoundRequirement,
  BoundRule,
  BoundSource,
  type RuleVerifier,
} from "./bound-declarations.js";
import { buildBoundSpec } from "./bound-materialize.js";
import {
  fluentSpec,
  type FluentRequirement,
  type FluentRule,
  type FluentSource,
  type FluentSpec,
} from "./fluent-spec.js";
import type { SpecHandle } from "./spec.js";

export { BoundRequirement, BoundRule, BoundSource } from "./bound-declarations.js";

export class SpecAuthoring<SpecKey extends string> {
  readonly key: SpecKey;
  readonly #context: ReturnType<typeof createAuthoringContext>;

  constructor(key: SpecKey, verify: RuleVerifier) {
    this.#context = createAuthoringContext(key, verify);
    this.key = key;
    Object.freeze(this);
  }

  source<const Key extends string>(key: Key): BoundSource<SpecKey, Key> {
    return BoundSource.create(this.#context, key);
  }

  requirement<const Key extends string>(key: Key): BoundRequirement<SpecKey, Key> {
    return BoundRequirement.create(this.#context, key);
  }

  rule<const Key extends string>(key: Key): BoundRule<SpecKey, Key, undefined> {
    return BoundRule.shared(this.#context, key);
  }

  build<const Requirements extends readonly BoundRequirement<SpecKey, any>[]>(
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
  return new SpecAuthoring(key, verify);
}
