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

const valid = typecheck("valid");
assert.equal(valid.status, 0, valid.stdout + valid.stderr);

const renamed = typecheck("renamed");
assert.notEqual(renamed.status, 0, "renamed export unexpectedly typechecked");
assert.match(renamed.stdout + renamed.stderr, /TS2305/);
assert.match(renamed.stdout + renamed.stderr, /expiry/);
