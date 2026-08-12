import { defineSpec } from "provenance";

const spec = defineSpec("share-links", ({ requirement }) => {
  const sharing = requirement("sharing", {
    statement: "Users can securely share documentation",
  });
  const expiry = sharing.rule("expiry", {
    statement: "Share links expire within 30 days",
  });

  return { expiry };
});

export const { expiry } = spec.handles;
