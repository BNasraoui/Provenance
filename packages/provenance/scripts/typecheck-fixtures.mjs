import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("..", import.meta.url));
const tsc = fileURLToPath(new URL("../node_modules/typescript/bin/tsc", import.meta.url));

function typecheck(fixture) {
  return spawnSync(process.execPath, [tsc, "-p", `test/fixtures/${fixture}/tsconfig.json`], {
    cwd: packageRoot,
    encoding: "utf8",
  });
}

const built = spawnSync(process.execPath, [tsc, "-p", "tsconfig.json"], {
  cwd: packageRoot,
  encoding: "utf8",
});
assert.equal(built.status, 0, built.stdout + built.stderr);

const valid = typecheck("valid");
assert.equal(valid.status, 0, valid.stdout + valid.stderr);

const contextValid = typecheck("context-valid");
assert.equal(contextValid.status, 0, contextValid.stdout + contextValid.stderr);

const contextLocalRuleMismatch = typecheck("context-local-rule-mismatch");
assert.notEqual(
  contextLocalRuleMismatch.status,
  0,
  "a requirement-local Rule unexpectedly attached to another Requirement",
);
assert.match(contextLocalRuleMismatch.stdout + contextLocalRuleMismatch.stderr, /TS2345/);

const contextCrossSpec = typecheck("context-cross-spec");
assert.notEqual(contextCrossSpec.status, 0, "a Source unexpectedly crossed spec contexts");
assert.match(contextCrossSpec.stdout + contextCrossSpec.stderr, /TS2345/);

const implementedByValid = typecheck("implemented-by-valid");
assert.equal(implementedByValid.status, 0, implementedByValid.stdout + implementedByValid.stderr);

const implementedByRemoved = typecheck("implemented-by-removed");
assert.notEqual(
  implementedByRemoved.status,
  0,
  "a removed implementation export unexpectedly typechecked",
);
assert.match(implementedByRemoved.stdout + implementedByRemoved.stderr, /TS2305/);
assert.match(implementedByRemoved.stdout + implementedByRemoved.stderr, /startWorkflow/);

const missingKey = typecheck("missing-key");
assert.notEqual(missingKey.status, 0, "verification without a key unexpectedly typechecked");
assert.match(missingKey.stdout + missingKey.stderr, /TS2554/);

const renamed = typecheck("renamed");
assert.notEqual(renamed.status, 0, "renamed export unexpectedly typechecked");
assert.match(renamed.stdout + renamed.stderr, /TS2305/);
assert.match(renamed.stdout + renamed.stderr, /expiry/);
