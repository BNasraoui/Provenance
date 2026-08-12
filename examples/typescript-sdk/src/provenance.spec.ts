import { defineSpec } from "@quality-sh/provenance";

const spec = defineSpec("share-links", ({ requirement, source }) => {
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
    statement: "Share links must expire within 30 days",
  });

  return { sharing, expiry };
});

export default spec;
export const { expiry, sharing } = spec.handles;
