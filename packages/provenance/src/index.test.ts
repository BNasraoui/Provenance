import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { apply, configure, requirement, source } from "./index.js";

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

test("typed declarations reconcile to canonical Provenance records", async () => {
  const repo = repository();
  const { expiry, sharing } = declareFixture(repo);

  const result = await apply();

  assert.equal(expiry.id, "expiry");
  assert.equal(sharing.id, "sharing");
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

test("verify records a passed Node callback against the imported rule", async () => {
  const repo = repository();
  const { expiry } = declareFixture(repo);

  let called = false;
  await expiry.verify(() => {
    called = true;
  });

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
  assert.match(runs.at(-1)?.file ?? "", /index\.test\.js$/);
});

test("verify records a failed callback and rethrows the original error", async () => {
  const repo = repository();
  const { expiry } = declareFixture(repo);
  await apply();
  const failure = new Error("expiry assertion failed");

  await assert.rejects(
    expiry.verify(async () => {
      throw failure;
    }),
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
