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
const rustTarget = targetFor(process.platform, process.arch);
const binaryName = process.platform === "win32" ? "provenance.exe" : "provenance";
const builtBinary = join(repositoryRoot, "target", "debug", binaryName);

execFileSync(process.execPath, [
  join(packageRoot, "scripts", "package-engine.js"),
  "--target", rustTarget,
  "--binary", builtBinary,
  "--out", stagedEngine,
  "--version", version,
]);
npm(["pack", stagedEngine, "--pack-destination", temporary]);
npm(["pack", packageRoot, "--pack-destination", temporary]);

const archives = readdirSync(temporary).filter((name) => name.endsWith(".tgz"));
const mainArchive = archiveNamed(archives, `quality-sh-provenance-${version}.tgz`);
const engineArchive = archives.find((name) => name !== mainArchive);
assert.ok(engineArchive, "platform engine archive was not created");
const engineManifest = JSON.parse(readFileSync(join(stagedEngine, "package.json"), "utf8"));

const application = join(temporary, "application");
mkdirSync(application);
writeFileSync(join(application, "package.json"), JSON.stringify({ private: true, type: "module" }));
npm(
  [
    "install",
    "--offline",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    join(temporary, mainArchive),
    join(temporary, engineArchive),
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
`);
writeFileSync(join(application, "verify.mjs"), `
import { apply, defineSpec, plan, requirement, rule } from "@quality-sh/provenance";
import { startWorkflow } from "./runtime.mjs";
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

const typedStart = rule("typed-start")
  .statement("Installed typed specs resolve imported production implementations")
  .implementedBy(startWorkflow);
const typedRequirement = requirement("typed-implementation")
  .statement("Installed typed specs retain implementation links")
  .rules(typedStart);
const typedSpec = defineSpec("packed-implemented-by")
  .requirements(typedRequirement)
  .build();
const typedResult = await apply(typedSpec);
if (typedResult.implementation_bindings?.[0]?.file !== "runtime.mjs") {
  throw new Error("implementedBy did not survive the packed install");
}
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
assert.equal(runs.length, 1);
assert.equal(runs[0].status, "passed");

function archiveNamed(archives, expected) {
  const archive = archives.find((name) => name === expected);
  assert.ok(archive, `missing ${expected}; found ${archives.join(", ")}`);
  return archive;
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
