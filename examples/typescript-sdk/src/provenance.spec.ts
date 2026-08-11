import { apply, requirement, source } from "provenance";

const linear = source("linear:ABC-123", {
  kind: "linear",
  name: "Linear ABC-123",
  url: "https://linear.app/example/issue/ABC-123",
});

export const sharing = requirement("sharing", {
  statement: "Users can securely share documentation",
  sources: [linear],
});

export const expiry = sharing.rule("expiry", {
  statement: "Share links must expire within 30 days",
});

await apply();
