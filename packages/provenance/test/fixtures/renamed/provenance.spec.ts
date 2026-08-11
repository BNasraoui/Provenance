import { requirement } from "provenance";

const sharing = requirement("sharing", {
  statement: "Users can securely share documentation",
});

export const shareLinkExpiry = sharing.rule("expiry", {
  statement: "Share links expire within 30 days",
});
