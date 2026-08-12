import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { chmodSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import {
  apply,
  configure,
  defineSpec,
  requirement,
  source,
} from "./index.js";

const engine = fileURLToPath(
  new URL("../../../target/debug/provenance", import.meta.url),
);

function repository(): string {
  const repo = mkdtempSync(join(tmpdir(), "provenance-ts-sdk-"));
  execFileSync(engine, [
    "init",
    "--path",
    repo,
    "--scope",
    "default",
    "--path-prefix",
    ".",
  ]);
  return repo;
}

function declareFixture(repo: string) {
  configure({
    engine,
    repository: repo,
    scope: "default",
    owner: "spec://typescript/share-links",
    verificationOwner: "ci://node-test",
  });
  const linear = source("linear:ABC-123", {
    kind: "linear",
    name: "Linear ABC-123",
    url: "https://linear.app/example/issue/ABC-123",
  });
  const sharing = requirement("sharing", {
    statement: "Users can securely share documentation",
    sources: [linear],
  });
  const expiry = sharing.rule("expiry", {
    statement: "Share links expire within 30 days",
  });
  return { expiry, sharing };
}

function engineJson(repo: string, args: string[]): unknown {
  return JSON.parse(
    execFileSync(engine, [...args, "--repo", repo, "--format", "json"], {
      encoding: "utf8",
    }),
  );
}

function recordingEngine(): {
  engine: string;
  requests: () => Array<{ command: string; input: unknown }>;
} {
  const directory = mkdtempSync(join(tmpdir(), "provenance-recording-engine-"));
  const executable = join(directory, "engine.mjs");
  const log = join(directory, "requests.jsonl");
  writeFileSync(
    executable,
    `#!/usr/bin/env node
import { appendFileSync, readFileSync } from "node:fs";
const command = process.argv[3];
const source = readFileSync(0, "utf8");
const input = source === "" ? undefined : JSON.parse(source);
appendFileSync(${JSON.stringify(log)}, JSON.stringify({ command, input }) + "\\n");
if (command === "begin-verification") {
  process.stdout.write(JSON.stringify({
    id: "run_" + input.key,
    binding_id: "verification_binding_" + input.key,
    rule_id: "rule_expiry",
    status: "running",
    commit: "0123456789abcdef",
    file: input.file,
    symbol: input.symbol,
  }));
} else {
  process.stdout.write(JSON.stringify({
    id: input.run,
    binding_id: "verification_binding_completed",
    rule_id: "rule_expiry",
    status: input.status,
  }));
}
`,
  );
  chmodSync(executable, 0o755);
  return {
    engine: executable,
    requests: () =>
      readFileSync(log, "utf8")
        .trim()
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line) as { command: string; input: unknown }),
  };
}

test("verify sends the same durable binding key on repeated runs", async () => {
  const recorder = recordingEngine();
  configure({ engine: recorder.engine, repository: repository() });
  const spec = defineSpec("share-links", ({ requirement }) => {
    const sharing = requirement("sharing", {
      statement: "Users can securely share documentation",
    });
    return {
      expiry: sharing.rule("expiry", {
        statement: "Share links expire within 30 days",
      }),
    };
  });
  const options = {
    key: "share-link-expiry",
    method: "examples",
    file: "src/share-links.test.ts",
    symbol: "checkExpiry",
  } as const;

  await spec.handles.expiry.verify(() => undefined, options);
  await spec.handles.expiry.verify(() => undefined, options);

  const begins = recorder.requests().filter(({ command }) => command === "begin-verification");
  assert.equal(begins.length, 2);
  assert.deepEqual(begins.map(({ input }) => input), [
    {
      declaration: {
        declared_by: "spec://typescript",
        address: ["share-links", "requirement", "sharing", "rule", "expiry"],
      },
      key: "share-link-expiry",
      method: "examples",
      declared_by: "ci://typescript",
      file: "src/share-links.test.ts",
      symbol: "checkExpiry",
    },
    {
      declaration: {
        declared_by: "spec://typescript",
        address: ["share-links", "requirement", "sharing", "rule", "expiry"],
      },
      key: "share-link-expiry",
      method: "examples",
      declared_by: "ci://typescript",
      file: "src/share-links.test.ts",
      symbol: "checkExpiry",
    },
  ]);
});

test("verify sends distinct durable binding keys from one test file", async () => {
  const recorder = recordingEngine();
  configure({ engine: recorder.engine, repository: repository() });
  const spec = defineSpec("share-links", ({ requirement }) => {
    const sharing = requirement("sharing", {
      statement: "Users can securely share documentation",
    });
    return {
      expiry: sharing.rule("expiry", {
        statement: "Share links expire within 30 days",
      }),
    };
  });

  await spec.handles.expiry.verify(() => undefined, {
    key: "maximum-expiry",
    file: "src/share-links.test.ts",
  });
  await spec.handles.expiry.verify(() => undefined, {
    key: "expired-link",
    file: "src/share-links.test.ts",
  });

  const keys = recorder.requests()
    .filter(({ command }) => command === "begin-verification")
    .map(({ input }) => (input as { key?: string }).key);
  assert.deepEqual(keys, ["maximum-expiry", "expired-link"]);
});

test("typed declarations reconcile to canonical Provenance records", async () => {
  const repo = repository();
  const { expiry, sharing } = declareFixture(repo);

  const result = await apply();

  assert.match(expiry.id, /^rule_legacy_sharing_expiry_/);
  assert.match(sharing.id, /^requirement_legacy_sharing_/);
  assert.equal(result.created, 3);
  const rule = engineJson(repo, [
    "rules",
    "show",
    "--scope",
    "default",
    "--id",
    expiry.id,
  ]) as { statement: string; declared_by: string };
  assert.equal(rule.statement, "Share links expire within 30 days");
  assert.equal(rule.declared_by, "spec://typescript/share-links");
});

test("equal local rule keys under different requirements reconcile separately", async () => {
  const repo = repository();
  configure({
    engine,
    repository: repo,
    owner: "spec://typescript/lifecycles",
  });
  const sharing = requirement("sharing", {
    statement: "Users can securely share documentation",
  });
  const shareLinkExpiry = sharing.rule("expiry", {
    statement: "Share links expire within 30 days",
  });
  const sessions = requirement("sessions", {
    statement: "User sessions are time bounded",
  });
  const sessionExpiry = sessions.rule("expiry", {
    statement: "Inactive sessions expire within 24 hours",
  });

  await apply();

  assert.notEqual(shareLinkExpiry.id, sessionExpiry.id);
});

test("defineSpec finalizes pure builders into immutable hierarchical handles", () => {
  configure({ engine: "/engine/must/not/start" });
  let escapedRequirement: { rule(key: string, options: unknown): unknown } | undefined;
  const spec = defineSpec("lifecycles", ({ requirement }) => {
    const sharing = requirement("sharing", {
      statement: "Users can securely share documentation",
    });
    escapedRequirement = sharing;
    const shareLinkExpiry = sharing.rule("expiry", {
      statement: "Share links expire within 30 days",
    });
    const sessions = requirement("sessions", {
      statement: "User sessions are time bounded",
    });
    const sessionExpiry = sessions.rule("expiry", {
      statement: "Inactive sessions expire within 24 hours",
    });
    return { sharing, shareLinkExpiry, sessions, sessionExpiry };
  });

  assert.deepEqual(spec.handles.shareLinkExpiry.address, [
    "lifecycles",
    "requirement",
    "sharing",
    "rule",
    "expiry",
  ]);
  assert.deepEqual(spec.handles.sessionExpiry.address, [
    "lifecycles",
    "requirement",
    "sessions",
    "rule",
    "expiry",
  ]);
  assert.equal(Object.isFrozen(spec), true);
  assert.equal(Object.isFrozen(spec.handles), true);
  assert.equal(Object.isFrozen(spec.handles.sharing), true);
  assert.equal(Object.isFrozen(spec.handles.shareLinkExpiry), true);
  assert.throws(
    () => escapedRequirement?.rule("late", {}),
    /finalized/i,
  );
});

test("immutable rule handles verify through an applied declaration address", async () => {
  const repo = repository();
  configure({
    engine,
    repository: repo,
    owner: "spec://typescript",
    verificationOwner: "ci://node-test",
  });
  const spec = defineSpec("share-links", ({ requirement }) => {
    const sharing = requirement("sharing", {
      statement: "Users can securely share documentation",
    });
    const expiry = sharing.rule("expiry", {
      statement: "Share links expire within 30 days",
    });
    return { sharing, expiry };
  });
  let called = false;

  await assert.rejects(
    spec.handles.expiry.verify(
      () => {
        called = true;
      },
      { key: "share-link-expiry", file: "tests/share-links.test.ts" },
    ),
    /has not been applied/i,
  );
  assert.equal(called, false);
  assert.equal("id" in spec.handles.expiry, false);

  await apply(spec);
  await spec.handles.expiry.verify(
    () => {
      called = true;
    },
    { key: "share-link-expiry", file: "tests/share-links.test.ts" },
  );

  assert.equal(called, true);
  const runs = engineJson(repo, [
    "sdk",
    "verification-runs",
    "--scope",
    "default",
  ]) as Array<{ file?: string; rule_id: string; status: string }>;
  assert.equal(runs.at(-1)?.status, "passed");
  assert.match(runs.at(-1)?.rule_id ?? "", /^rule_share-links_sharing_expiry_/);
  assert.equal(runs.at(-1)?.file, "tests/share-links.test.ts");
});

test("reapplying an address reuses the canonical id already assigned by Rust", async () => {
  const repo = repository();
  configure({ engine, repository: repo, owner: "spec://typescript" });
  const declared = (id?: string) =>
    defineSpec("share-links", ({ requirement }) => {
      const sharing = requirement("sharing", {
        statement: "Users can securely share documentation",
      });
      const expiry = sharing.rule("expiry", {
        id,
        statement: "Share links expire within 30 days",
      });
      return { sharing, expiry };
    });

  const first = await apply(declared("rule_existing_expiry"));
  const second = await apply(declared());
  const firstRule = first.resources.find((resource) => resource.kind === "rule");
  const secondRule = second.resources.find((resource) => resource.kind === "rule");

  assert.equal(firstRule?.id, "rule_existing_expiry");
  assert.equal(secondRule?.id, "rule_existing_expiry");
  assert.equal(second.created, 0);
});

test("verify records a passed Node callback against the imported rule", async () => {
  const repo = repository();
  const { expiry } = declareFixture(repo);

  let called = false;
  await expiry.verify(
    () => {
      called = true;
    },
    { key: "share-link-expiry", file: "tests/share-links.test.ts" },
  );

  assert.equal(called, true);
  const runs = engineJson(repo, [
    "sdk",
    "verification-runs",
    "--scope",
    "default",
    "--rule",
    expiry.id,
  ]) as Array<{ status: string; rule_id: string; file?: string }>;
  assert.equal(runs.at(-1)?.status, "passed");
  assert.equal(runs.at(-1)?.rule_id, expiry.id);
  assert.equal(runs.at(-1)?.file, "tests/share-links.test.ts");
});

test("verify records a failed callback and rethrows the original error", async () => {
  const repo = repository();
  const { expiry } = declareFixture(repo);
  await apply();
  const failure = new Error("expiry assertion failed");

  await assert.rejects(
    expiry.verify(
      async () => {
        throw failure;
      },
      { key: "share-link-expiry", file: "tests/share-links.test.ts" },
    ),
    (error) => error === failure,
  );

  const runs = engineJson(repo, [
    "sdk",
    "verification-runs",
    "--scope",
    "default",
    "--rule",
    expiry.id,
  ]) as Array<{ status: string; error?: string }>;
  assert.equal(runs.at(-1)?.status, "failed");
  assert.match(runs.at(-1)?.error ?? "", /expiry assertion failed/);
});
