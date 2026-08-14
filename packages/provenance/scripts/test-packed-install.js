import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("..", import.meta.url));
const repositoryRoot = fileURLToPath(new URL("../../..", import.meta.url));
const npmCli = process.env.npm_execpath;
assert.ok(npmCli, "run this test through npm so its CLI path is known");
const version = JSON.parse(readFileSync(join(packageRoot, "package.json"), "utf8")).version;
const temporary = mkdtempSync(join(tmpdir(), "provenance-packed-install-"));
process.once("exit", () => rmSync(temporary, { recursive: true, force: true }));
const stagedEngine = join(temporary, "engine-package");
const isolatedCache = join(temporary, "npm-cache");
const rustTarget = targetFor(process.platform, process.arch);
const binaryName = process.platform === "win32" ? "provenance.exe" : "provenance";
const builtBinary = join(repositoryRoot, "target", "debug", binaryName);
const typescriptRoot = join(packageRoot, "node_modules", "typescript");
const typescriptManifest = JSON.parse(
  readFileSync(join(typescriptRoot, "package.json"), "utf8"),
);

execFileSync(process.execPath, [
  join(packageRoot, "scripts", "package-engine.js"),
  "--target", rustTarget,
  "--binary", builtBinary,
  "--out", stagedEngine,
  "--version", version,
]);
npm(["pack", stagedEngine, "--pack-destination", temporary]);
npm(["pack", packageRoot, "--pack-destination", temporary]);
npm(["pack", typescriptRoot, "--pack-destination", temporary]);

const archives = readdirSync(temporary).filter((name) => name.endsWith(".tgz"));
const mainArchive = archiveNamed(archives, `quality-sh-provenance-${version}.tgz`);
const engineManifest = JSON.parse(readFileSync(join(stagedEngine, "package.json"), "utf8"));
const engineArchive = archiveNamed(archives, archiveName(engineManifest));
const typescriptArchive = archiveNamed(archives, archiveName(typescriptManifest));

const application = join(temporary, "application");
mkdirSync(application);
writeFileSync(join(application, "package.json"), JSON.stringify({ private: true, type: "module" }));
npm(
  [
    "install",
    "--offline",
    "--cache",
    isolatedCache,
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    join(temporary, mainArchive),
    join(temporary, engineArchive),
    join(temporary, typescriptArchive),
  ],
  { cwd: application },
);

const localEngine = join(
  application,
  "node_modules",
  ...engineManifest.name.split("/"),
  "bin",
  binaryName,
);
execFileSync(localEngine, ["init", "--path", application, "--scope", "default", "--path-prefix", "."]);
writeFileSync(join(application, "runtime.mjs"), `
export function startWorkflow() {}
export class WorkflowRunner {
  static constructions = 0;
  constructor() { WorkflowRunner.constructions += 1; }
}
`);
writeFileSync(join(application, "helpers.ts"), `
import type {
  RequirementDeclaration,
  RuleDeclaration,
  SourceDeclaration,
  SpecAuthoring,
} from "@quality-sh/provenance";

export function guide<const Spec extends string>(
  author: SpecAuthoring<Spec>,
): SourceDeclaration<Spec, "guide"> {
  return author.source("guide").name("Packed SDK guide").document("README.md");
}

export function installed<const Spec extends string>(
  author: SpecAuthoring<Spec>,
  source: SourceDeclaration<Spec, "guide">,
): RequirementDeclaration<Spec, "installed"> {
  return author.requirement("installed")
    .statement("The packed SDK exposes spec-bound TypeScript declarations")
    .description("Exercises emitted fluent metadata types")
    .from(source);
}

export function invocation<
  const Spec extends string,
  const RequirementKey extends string,
>(
  requirement: RequirementDeclaration<Spec, RequirementKey>,
): RuleDeclaration<Spec, "invocation", RequirementKey> {
  return requirement.rule("invocation")
    .statement("A direct Rule handle remains typed across module boundaries");
}
`);
writeFileSync(join(application, "consumer.ts"), `
import { defineSpec } from "@quality-sh/provenance";
import { guide, installed, invocation as declareInvocation } from "./helpers.js";

const provenance = defineSpec("packed-typescript-consumer");
const packedGuide = guide(provenance);
const packedInstallation = installed(provenance, packedGuide);
export const invocation = declareInvocation(packedInstallation);
export const spec = provenance.build(packedInstallation.rules(invocation));

void invocation.verify("packed-consumer", () => undefined);
`);
writeFileSync(join(application, "tsconfig.json"), JSON.stringify({
  compilerOptions: {
    module: "NodeNext",
    moduleResolution: "NodeNext",
    target: "ES2022",
    strict: true,
    noEmit: true,
    skipLibCheck: true,
  },
  include: ["*.ts"],
}));
execFileSync(process.execPath, [
  join(application, "node_modules", "typescript", "bin", "tsc"),
  "-p", join(application, "tsconfig.json"),
], { cwd: application, stdio: "pipe" });
writeFileSync(join(application, "verify.mjs"), `
import { apply, defineSpec, plan, requirement, rule, source } from "@quality-sh/provenance";
import { startWorkflow, WorkflowRunner } from "./runtime.mjs";
const spec = defineSpec("packed-install", ({ requirement }) => {
  const installed = requirement("installed", {
    statement: "The installed SDK invokes its package-supplied engine"
  });
  const invocation = installed.rule("invocation", {
    statement: "Typed SDK operations reach the packaged Rust engine"
  });
  return { installed, invocation };
});
const preview = await plan(spec);
if (preview.created !== 2) throw new Error("plan did not reach the packaged engine");
const result = await apply(spec);
if (result.created !== 2) throw new Error("apply did not reach the packaged engine");
await spec.handles.invocation.verify("packed-install", () => undefined, {
  file: "verify.mjs",
  symbol: "packedInstall"
});

const typedSpec = defineSpec("packed-implemented-by")
  .requirements(
    requirement("typed-implementation")
      .statement("Installed typed specs retain implementation links")
      .from(source("packed-guide").name("Packed guide").document("README.md"))
      .rules(
        rule("typed-start")
          .statement("Installed typed specs resolve imported production implementations")
          .implementedBy(startWorkflow),
        rule("typed-runner")
          .statement("Installed typed specs resolve imported production classes")
          .implementedBy(WorkflowRunner),
      ),
  )
  .build();
const typedResult = await apply(typedSpec);
if (typedResult.implementation_bindings?.some(
  ({ file, symbol }) => file === "runtime.mjs" && symbol === "startWorkflow"
) !== true) {
  throw new Error("implementedBy did not survive the packed install: " + JSON.stringify(typedResult));
}
if (typedResult.implementation_bindings?.some(
  ({ file, symbol }) => file === "runtime.mjs" && symbol === "WorkflowRunner"
) !== true) {
  throw new Error("class implementedBy did not survive the packed install: " + JSON.stringify(typedResult));
}
if (WorkflowRunner.constructions !== 0) {
  throw new Error("implementedBy constructed the packed class target");
}
if (typedResult.resources?.some(({ kind }) => kind === "source") !== true) {
  throw new Error("Requirement.from Source was not collected by build");
}
await typedSpec.requirements["typed-implementation"].rules["typed-start"].verify(
  "packed-direct-rule",
  () => undefined,
  { file: "verify.mjs", symbol: "packedDirectRule" },
);
`);

const { PROVENANCE_BIN: _removed, ...environment } = process.env;
execFileSync(process.execPath, [join(application, "verify.mjs")], {
  cwd: application,
  env: { ...environment, PATH: "" },
  stdio: "pipe",
});
const runs = JSON.parse(execFileSync(localEngine, [
  "sdk", "verification-runs", "--repo", application, "--scope", "default", "--format", "json",
], { encoding: "utf8" }));
assert.equal(runs.length, 2);
assert.deepEqual(runs.map(({ status }) => status), ["passed", "passed"]);

function archiveNamed(archives, expected) {
  const archive = archives.find((name) => name === expected);
  assert.ok(archive, `missing ${expected}; found ${archives.join(", ")}`);
  return archive;
}

function archiveName(manifest) {
  return `${manifest.name.replace(/^@/, "").replace("/", "-")}-${manifest.version}.tgz`;
}

function npm(args, options = {}) {
  execFileSync(process.execPath, [npmCli, ...args], { ...options, stdio: "pipe" });
}

function targetFor(platform, arch) {
  const key = `${platform}-${arch}`;
  const targets = {
    "darwin-arm64": "aarch64-apple-darwin",
    "darwin-x64": "x86_64-apple-darwin",
    "linux-x64": "x86_64-unknown-linux-gnu",
    "win32-x64": "x86_64-pc-windows-msvc",
  };
  const target = targets[key];
  if (target === undefined) throw new Error(`packed install test does not support ${key}`);
  return target;
}
