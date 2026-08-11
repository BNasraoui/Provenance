import { defineSpec } from "provenance";

const spec = defineSpec("share-links", ({ requirement }) => {
  const sharing = requirement("sharing", {
    statement: "Users can securely share documentation",
  });
  const shareLinkExpiry = sharing.rule("expiry", {
    statement: "Share links expire within 30 days",
  });

  return { sharing, shareLinkExpiry };
});

export const { shareLinkExpiry, sharing } = spec.handles;
