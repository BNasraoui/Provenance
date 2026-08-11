import { invokeEngine, type EngineSettings } from "./engine.js";
import { fileURLToPath } from "node:url";
import type {
  ApplyResult,
  VerificationRun,
} from "./protocol.js";
import { DeclarationRegistry } from "./registry.js";

export type VerificationMethod =
  | "exhaustion"
  | "property"
  | "examples"
  | "conformance"
  | "construction"
  | "proof";

export interface ConfigureOptions {
  engine?: string;
  repository?: string;
  scope?: string;
  owner?: string;
  verificationOwner?: string;
}

export interface SourceOptions {
  id?: string;
  name?: string;
  kind: string;
  url?: string;
  reference?: string;
}

export interface RequirementOptions {
  id?: string;
  statement: string;
  description?: string;
  sources?: SourceHandle[];
}

export interface RuleOptions {
  id?: string;
  statement: string;
  name?: string;
  description?: string;
}

export interface VerifyOptions {
  method?: VerificationMethod;
  file?: string;
  symbol?: string;
}

export interface SourceHandle {
  readonly key: string;
  readonly id: string;
}

export interface RequirementHandle {
  readonly key: string;
  readonly id: string;
  rule(key: string, options: RuleOptions): RuleHandle;
}

export interface RuleHandle {
  readonly key: string;
  readonly id: string;
  verify(callback: () => unknown | Promise<unknown>, options?: VerifyOptions): Promise<void>;
}

export type { ApplyResult } from "./protocol.js";

const registry = new DeclarationRegistry();
const moduleFile = fileURLToPath(import.meta.url);
let settings = defaults();

export function configure(options: ConfigureOptions): void {
  settings = { ...defaults(), ...options };
  registry.reset();
}

export function source(key: string, options: SourceOptions): SourceHandle {
  const handle = new DeclaredHandle(key);
  registry.addSource(
    {
      key,
      id: options.id,
      name: options.name ?? key,
      kind: options.kind,
      url: options.url,
      reference: options.reference,
    },
    handle,
  );
  return handle;
}

export function requirement(
  key: string,
  options: RequirementOptions,
): RequirementHandle {
  const handle = new Requirement(key);
  registry.addRequirement(
    {
      key,
      id: options.id,
      statement: options.statement,
      description: options.description,
      sources: (options.sources ?? []).map((source) => source.key),
    },
    handle,
  );
  return handle;
}

export async function apply(): Promise<ApplyResult> {
  const result = await invokeEngine<ApplyResult>(
    engineSettings(),
    "apply",
    registry.document(settings.owner),
  );
  registry.assign(result);
  return result;
}

class DeclaredHandle implements SourceHandle {
  #id?: string;

  constructor(readonly key: string) {}

  get id(): string {
    if (this.#id === undefined) {
      throw new Error(
        `Provenance declaration \`${this.key}\` has no canonical id until apply() succeeds`,
      );
    }
    return this.#id;
  }

  assignId(id: string): void {
    this.#id = id;
  }
}

class Requirement extends DeclaredHandle implements RequirementHandle {
  rule(key: string, options: RuleOptions): RuleHandle {
    const handle = new Rule(key);
    registry.addRule(
      {
        key,
        id: options.id,
        requirement: this.key,
        statement: options.statement,
        name: options.name,
        description: options.description,
      },
      handle,
    );
    return handle;
  }
}

class Rule extends DeclaredHandle implements RuleHandle {
  async verify(
    callback: () => unknown | Promise<unknown>,
    options: VerifyOptions = {},
  ): Promise<void> {
    const location = options.file === undefined ? callerLocation() : undefined;
    if (registry.dirty) {
      await apply();
    }
    const run = await invokeEngine<VerificationRun>(
      engineSettings(),
      "begin-verification",
      {
        rule: this.id,
        method: options.method ?? "examples",
        declared_by: settings.verificationOwner,
        file: options.file ?? location?.file,
        symbol: options.symbol,
      },
    );
    try {
      await callback();
    } catch (error) {
      try {
        await complete(run.id, "failed", serializeError(error));
      } catch {
        // The callback error is the test runner's primary failure. Preserve it.
      }
      throw error;
    }
    await complete(run.id, "passed");
  }
}

async function complete(
  run: string,
  status: "passed" | "failed",
  error?: string,
): Promise<void> {
  await invokeEngine<VerificationRun>(engineSettings(), "complete-verification", {
    run,
    status,
    error,
  });
}

function serializeError(error: unknown): string {
  if (error instanceof Error) {
    return error.stack ?? `${error.name}: ${error.message}`;
  }
  if (typeof error === "string") {
    return error;
  }
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

function callerLocation(): { file: string } | undefined {
  const stack = new Error().stack?.split("\n").slice(1) ?? [];
  for (const line of stack) {
    const openParenthesis = line.lastIndexOf("(");
    const framed = openParenthesis >= 0
      ? line.slice(openParenthesis + 1, line.endsWith(")") ? -1 : undefined)
      : line.trim().split(/\s+/).at(-1);
    const match = framed?.match(/^(.*):\d+:\d+$/);
    if (match?.[1] !== undefined) {
      const file = match[1].startsWith("file:")
        ? fileURLToPath(match[1])
        : match[1];
      if (file !== moduleFile) {
        return { file };
      }
    }
  }
  return undefined;
}

function defaults(): Required<ConfigureOptions> {
  return {
    engine: process.env.PROVENANCE_BIN ?? "provenance",
    repository: process.env.PROVENANCE_REPO ?? process.cwd(),
    scope: process.env.PROVENANCE_SCOPE ?? "default",
    owner: process.env.PROVENANCE_SPEC_OWNER ?? "spec://typescript",
    verificationOwner: process.env.PROVENANCE_VERIFICATION_OWNER ?? "ci://typescript",
  };
}

function engineSettings(): EngineSettings {
  return {
    engine: settings.engine,
    repository: settings.repository,
    scope: settings.scope,
  };
}
