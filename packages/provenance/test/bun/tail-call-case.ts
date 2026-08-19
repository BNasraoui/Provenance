// Run by tail-call.test.ts through `bun test`, once per shape. The verify call
// sits in the tail position of the test callback, which is the one shape Bun
// cannot report: JavaScriptCore replaces the calling frame with the called one,
// so the SDK sees a stack of its own frames and nothing else.
//
// This file is not named `*.test.ts` on purpose: it only runs through an
// explicit path, so a plain `bun test` sweep never picks it up.
import { test } from "bun:test";

import { configure, defineSpec } from "../../dist/index.js";

configure({ engine: process.env.PROVENANCE_TEST_ENGINE, repository: import.meta.dir });

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

if (process.env.PROVENANCE_STATED_FILE === "1") {
  test("stated file", () =>
    spec.handles.expiry.verify("share-link-expiry", () => undefined, import.meta));
} else {
  test("no stated file", () => spec.handles.expiry.verify("share-link-expiry", () => undefined));
}
