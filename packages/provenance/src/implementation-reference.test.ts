import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  parseCallSites,
  resolveImplementationReferenceAt,
} from "./implementation-reference.js";

function fixture(source: string): string {
  const directory = mkdtempSync(join(tmpdir(), "provenance-implementation-reference-"));
  writeFileSync(
    join(directory, "runtime.ts"),
    "export function startWorkflow(): void {}\nexport function stopWorkflow(): void {}\nexport class WorkflowRunner {}\n",
  );
  const spec = join(directory, "provenance.spec.ts");
  writeFileSync(spec, source);
  return spec;
}

test("Node and Bun stack frames preserve a usable source location", () => {
  assert.deepEqual(
    parseCallSites(
      "Error\n    at FluentRule.implementedBy (file:///project/spec.js:9:12)\n    at file:///project/app.js:3:7",
    ),
    [
      { file: "/project/spec.js", line: 9, column: 12 },
      { file: "/project/app.js", line: 3, column: 7 },
    ],
  );
  assert.deepEqual(
    parseCallSites("Error\n    at implementedBy (/project/sdk.ts:4:3)\n    at /project/spec.ts:11"),
    [
      { file: "/project/sdk.ts", line: 4, column: 3 },
      { file: "/project/spec.ts", line: 11 },
    ],
  );
});

test("a named import resolves to its source module and exported symbol", () => {
  const spec = fixture(`
import { startWorkflow as start } from "./runtime.js";
rule("start").implementedBy(start);
`);

  const reference = resolveImplementationReferenceAt({ file: spec, line: 3 });

  assert.equal(reference.file, join(spec, "../runtime.ts"));
  assert.equal(reference.symbol, "startWorkflow");
});

test("a namespace member resolves to its source module and exported symbol", () => {
  const spec = fixture(`
import * as runtime from "./runtime.js";
rule("start").implementedBy(runtime.startWorkflow);
`);

  const reference = resolveImplementationReferenceAt({ file: spec, line: 3 });

  assert.equal(reference.file, join(spec, "../runtime.ts"));
  assert.equal(reference.symbol, "startWorkflow");
});

test("a directly imported class resolves to its exported symbol", () => {
  const spec = fixture(`
import { WorkflowRunner as Runner } from "./runtime.js";
rule("start").implementedBy(Runner);
`);

  const reference = resolveImplementationReferenceAt({ file: spec, line: 3 });

  assert.equal(reference.file, join(spec, "../runtime.ts"));
  assert.equal(reference.symbol, "WorkflowRunner");
});

test("dynamic implementation expressions are rejected instead of reflected", () => {
  const cases = [
    "enabled ? runtime.startWorkflow : runtime.stopWorkflow",
    "chooseImplementation()",
    'runtime["startWorkflow"]',
    "() => undefined",
    "new runtime.WorkflowRunner()",
    "runner.run",
    "localImplementation",
  ];
  for (const expression of cases) {
    const spec = fixture(`
import * as runtime from "./runtime.js";
declare const enabled: boolean;
declare function chooseImplementation(): typeof runtime.startWorkflow;
declare const runner: { run(): void };
declare function localImplementation(): void;
rule("start").implementedBy(${expression});
`);
    assert.throws(
      () => resolveImplementationReferenceAt({ file: spec, line: 7 }),
      /direct named import or namespace member/i,
    );
  }
});

test("an ambiguous or unreadable call site is rejected", () => {
  const spec = fixture(`
import * as runtime from "./runtime.js";
rule("one").implementedBy(runtime.startWorkflow); rule("two").implementedBy(runtime.stopWorkflow);
`);
  assert.throws(
    () => resolveImplementationReferenceAt({ file: spec, line: 3 }),
    /more than one implementedBy call/i,
  );
  assert.throws(
    () => resolveImplementationReferenceAt({ file: `${spec}.missing`, line: 1 }),
    /could not read implementedBy call site/i,
  );
});
