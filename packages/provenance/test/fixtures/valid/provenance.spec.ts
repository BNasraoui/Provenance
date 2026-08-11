import { requirement } from "provenance";

export const sharing = requirement("sharing", {
  statement: "Users can securely share documentation",
});

export const expiry = sharing.rule("expiry", {
  statement: "Share links expire within 30 days",
});
