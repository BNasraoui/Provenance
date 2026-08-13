import type {
  RequirementDeclaration,
  RuleDeclaration,
  SourceDeclaration,
  TypedSpecDocument,
} from "./protocol.js";
import {
  registerSpec,
  type RequirementHandle,
  type RuleHandle,
  type SourceHandle,
  type SpecHandle,
  type VerifyOptions,
} from "./spec.js";

type RuleVerifier = (
  address: readonly string[],
  key: string,
  callback: () => unknown | Promise<unknown>,
  options?: VerifyOptions,
) => Promise<void>;

export class FluentSource<Key extends string = string> {
  readonly key: Key;
  readonly declaration?: SourceDeclaration;

  constructor(key: Key, declaration?: SourceDeclaration) {
    requireKey("source", key);
    this.key = key;
    this.declaration = declaration === undefined ? undefined : Object.freeze({ ...declaration });
    Object.freeze(this);
  }

  document(reference: string): FluentSource<Key> {
    requireText("document reference", reference);
    return new FluentSource(this.key, {
      key: this.key,
      name: this.key,
      kind: "document",
      reference,
    });
  }
}

export class FluentRule<Key extends string = string> {
  readonly key: Key;
  readonly text?: string;
  readonly explicitId?: string;

  constructor(key: Key, text?: string, explicitId?: string) {
    requireKey("rule", key);
    this.key = key;
    this.text = text;
    this.explicitId = explicitId;
    Object.freeze(this);
  }

  statement(text: string): FluentRule<Key> {
    requireText("rule statement", text);
    return new FluentRule(this.key, text, this.explicitId);
  }

  id(existingId: string): FluentRule<Key> {
    requireText("rule id", existingId);
    return new FluentRule(this.key, this.text, existingId);
  }
}

export class FluentRequirement<
  Key extends string = string,
  Rules extends readonly FluentRule[] = readonly [],
> {
  readonly key: Key;
  readonly text?: string;
  readonly sourceDeclarations: readonly FluentSource[];
  readonly ruleDeclarations: Rules;

  constructor(
    key: Key,
    text?: string,
    sources: readonly FluentSource[] = [],
    rules: Rules = [] as unknown as Rules,
  ) {
    requireKey("requirement", key);
    this.key = key;
    this.text = text;
    this.sourceDeclarations = Object.freeze([...sources]);
    this.ruleDeclarations = Object.freeze([...rules]) as unknown as Rules;
    Object.freeze(this);
  }

  statement(text: string): FluentRequirement<Key, Rules> {
    requireText("requirement statement", text);
    return new FluentRequirement(
      this.key,
      text,
      this.sourceDeclarations,
      this.ruleDeclarations,
    );
  }

  from(...sources: readonly FluentSource[]): FluentRequirement<Key, Rules> {
    return new FluentRequirement(
      this.key,
      this.text,
      appendByIdentity(this.sourceDeclarations, sources),
      this.ruleDeclarations,
    );
  }

  rules<const Added extends readonly FluentRule[]>(
    ...rules: Added
  ): FluentRequirement<Key, readonly [...Rules, ...Added]> {
    return new FluentRequirement(
      this.key,
      this.text,
      this.sourceDeclarations,
      appendByIdentity(this.ruleDeclarations, rules) as unknown as readonly [...Rules, ...Added],
    );
  }
}

type AnyRequirement = FluentRequirement<string, readonly FluentRule[]>;

type RuleHandles<Rules extends readonly FluentRule[]> = Readonly<{
  [Declaration in Rules[number] as Declaration["key"]]: RuleHandle;
}>;

type TypedRequirementHandle<Declaration extends AnyRequirement> = Omit<
  RequirementHandle,
  "rules"
> & {
  readonly rules: RuleHandles<Declaration["ruleDeclarations"]>;
};

export type FluentSpecHandles<
  Sources extends readonly FluentSource[],
  Requirements extends readonly AnyRequirement[],
> = Readonly<Record<string, unknown>> & {
  readonly sources: Readonly<{
    [Declaration in Sources[number] as Declaration["key"]]: SourceHandle;
  }>;
  readonly requirements: Readonly<{
    [Declaration in Requirements[number] as Declaration["key"]]: TypedRequirementHandle<Declaration>;
  }>;
};

export class FluentSpec<
  Sources extends readonly FluentSource[] = readonly [],
  Requirements extends readonly AnyRequirement[] = readonly [],
> {
  readonly key: string;
  readonly sourceDeclarations: readonly FluentSource[];
  readonly requirementDeclarations: readonly AnyRequirement[];
  readonly #verify: RuleVerifier;

  constructor(
    key: string,
    verify: RuleVerifier,
    sources: Sources = [] as unknown as Sources,
    requirements: Requirements = [] as unknown as Requirements,
  ) {
    requireKey("spec", key);
    this.key = key;
    this.#verify = verify;
    this.sourceDeclarations = Object.freeze([...sources]) as unknown as Sources;
    this.requirementDeclarations = Object.freeze([...requirements]) as unknown as Requirements;
    Object.freeze(this);
  }

  sources<const Added extends readonly FluentSource[]>(
    ...sources: Added
  ): FluentSpec<readonly [...Sources, ...Added], Requirements> {
    return new FluentSpec(
      this.key,
      this.#verify,
      appendByIdentity(this.sourceDeclarations, sources) as unknown as readonly [
        ...Sources,
        ...Added,
      ],
      this.requirementDeclarations,
    );
  }

  requirements<const Added extends readonly AnyRequirement[]>(
    ...requirements: Added
  ): FluentSpec<Sources, readonly [...Requirements, ...Added]> {
    return new FluentSpec(
      this.key,
      this.#verify,
      this.sourceDeclarations,
      appendByIdentity(this.requirementDeclarations, requirements) as unknown as readonly [
        ...Requirements,
        ...Added,
      ],
    );
  }

  build(): SpecHandle<FluentSpecHandles<Sources, Requirements>> {
    return buildSpec(this, this.#verify);
  }
}

export function fluentSource<const Key extends string>(key: Key): FluentSource<Key> {
  return new FluentSource(key);
}

export function fluentRule<const Key extends string>(key: Key): FluentRule<Key> {
  return new FluentRule(key);
}

export function fluentRequirement<const Key extends string>(key: Key): FluentRequirement<Key> {
  return new FluentRequirement(key);
}

export function fluentSpec(key: string, verify: RuleVerifier): FluentSpec {
  return new FluentSpec(key, verify);
}

function buildSpec<
  Sources extends readonly FluentSource[],
  Requirements extends readonly AnyRequirement[],
>(
  spec: FluentSpec<Sources, Requirements>,
  verify: RuleVerifier,
): SpecHandle<FluentSpecHandles<Sources, Requirements>> {
  const sources = uniqueByKey("Source", spec.sourceDeclarations);
  const requirements = uniqueByKey("Requirement", spec.requirementDeclarations);
  const sourceSet = new Set(sources);
  const memberships = new Map<FluentRule, AnyRequirement[]>();
  const relationOwners = new Map<string, FluentRule>();

  for (const source of sources) {
    if (source.declaration === undefined) {
      throw new Error(`Source declaration \`${source.key}\` has no document definition`);
    }
  }
  for (const requirement of requirements) {
    requireText(`Requirement \`${requirement.key}\` statement`, requirement.text);
    for (const source of requirement.sourceDeclarations) {
      if (!sourceSet.has(source)) {
        throw new Error(
          `Requirement \`${requirement.key}\` references Source \`${source.key}\` not included in the spec`,
        );
      }
    }
    for (const rule of requirement.ruleDeclarations) {
      requireText(`Rule \`${rule.key}\` statement`, rule.text);
      const relation = `${requirement.key}\0${rule.key}`;
      const existing = relationOwners.get(relation);
      if (existing !== undefined && existing !== rule) {
        throw new Error(
          `distinct Rule declarations with key \`${rule.key}\` collide under Requirement \`${requirement.key}\``,
        );
      }
      relationOwners.set(relation, rule);
      const parents = memberships.get(rule);
      if (parents === undefined) memberships.set(rule, [requirement]);
      else parents.push(requirement);
    }
  }

  const addresses = new Map<FluentRule, readonly string[]>();
  const addressOwners = new Map<string, FluentRule>();
  for (const [rule, parents] of memberships) {
    const address = ruleAddress(spec.key, rule.key, parents);
    const serialized = JSON.stringify(address);
    const existing = addressOwners.get(serialized);
    if (existing !== undefined && existing !== rule) {
      throw new Error(
        `distinct Rule declarations with key \`${rule.key}\` resolve to the same declaration address`,
      );
    }
    addressOwners.set(serialized, rule);
    addresses.set(rule, address);
  }

  const ruleHandles = new Map<FluentRule, RuleHandle>();
  for (const [rule, address] of addresses) {
    ruleHandles.set(rule, makeRuleHandle(rule, address, verify));
  }
  const sourceHandles = Object.freeze(
    Object.fromEntries(
      sources.map((source) => [
        source.key,
        Object.freeze({
          kind: "source" as const,
          key: source.key,
          address: Object.freeze([spec.key, "source", source.key]),
        }),
      ]),
    ),
  );
  const requirementHandles = Object.freeze(
    Object.fromEntries(
      requirements.map((requirement) => [
        requirement.key,
        Object.freeze({
          kind: "requirement" as const,
          key: requirement.key,
          address: Object.freeze([spec.key, "requirement", requirement.key]),
          rules: Object.freeze(
            Object.fromEntries(
              requirement.ruleDeclarations.map((rule) => [rule.key, ruleHandles.get(rule)!]),
            ),
          ),
        }),
      ]),
    ),
  );
  const handles = Object.freeze({ sources: sourceHandles, requirements: requirementHandles });
  return registerSpec(
    spec.key,
    handles as FluentSpecHandles<Sources, Requirements>,
    document(spec, sources, requirements, memberships),
  );
}

function document(
  spec: FluentSpec<readonly FluentSource[], readonly AnyRequirement[]>,
  sources: readonly FluentSource[],
  requirements: readonly AnyRequirement[],
  memberships: ReadonlyMap<FluentRule, AnyRequirement[]>,
): Omit<TypedSpecDocument, "declared_by"> {
  const sourceRecords = sources.map((source) => source.declaration!);
  const requirementRecords = requirements.map<RequirementDeclaration>((requirement) => ({
    key: requirement.key,
    statement: requirement.text!,
    sources: requirement.sourceDeclarations.map(({ key }) => key).sort(),
  }));
  const ruleRecords = [...memberships].map<RuleDeclaration>(([rule, parents]) => ({
    key: rule.key,
    id: rule.explicitId,
    requirements: parents.map(({ key }) => key).sort(),
    statement: rule.text!,
  }));
  sourceRecords.sort(byKey);
  requirementRecords.sort(byKey);
  ruleRecords.sort(byKey);
  return {
    schema_version: 1,
    spec: spec.key,
    sources: sourceRecords,
    requirements: requirementRecords,
    rules: ruleRecords,
  };
}

function makeRuleHandle(
  rule: FluentRule,
  address: readonly string[],
  verify: RuleVerifier,
): RuleHandle {
  const frozenAddress = Object.freeze([...address]);
  return Object.freeze({
    kind: "rule" as const,
    key: rule.key,
    address: frozenAddress,
    verify: (
      key: string,
      callback: () => unknown | Promise<unknown>,
      options?: VerifyOptions,
    ) => verify(frozenAddress, key, callback, options),
  });
}

function ruleAddress(
  spec: string,
  key: string,
  parents: readonly AnyRequirement[],
): readonly string[] {
  return parents.length === 1
    ? [spec, "requirement", parents[0]!.key, "rule", key]
    : [spec, "rule", key];
}

function uniqueByKey<T extends { readonly key: string }>(kind: string, values: readonly T[]): T[] {
  const byKey = new Map<string, T>();
  for (const value of values) {
    const existing = byKey.get(value.key);
    if (existing !== undefined && existing !== value) {
      throw new Error(`distinct ${kind} declarations use key \`${value.key}\``);
    }
    byKey.set(value.key, value);
  }
  return [...byKey.values()];
}

function appendByIdentity<T>(existing: readonly T[], added: readonly T[]): T[] {
  const result = [...existing];
  for (const value of added) {
    if (!result.includes(value)) result.push(value);
  }
  return result;
}

function requireKey(kind: string, key: string): void {
  requireText(`${kind} key`, key);
}

function requireText(field: string, value: string | undefined): asserts value is string {
  if (value === undefined || value.trim() === "") {
    throw new Error(`${field} must not be empty`);
  }
}

function byKey(left: { key: string }, right: { key: string }): number {
  return left.key.localeCompare(right.key);
}
