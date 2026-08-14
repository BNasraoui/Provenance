import { defineSpec } from "@quality-sh/provenance";

const provenance = defineSpec("share-links");
const policy = provenance
  .source("policy")
  .document("docs/policy.md")
  .name("Security policy");
const sharing = provenance
  .requirement("sharing")
  .statement("Users can securely share documentation")
  .description("Share-link lifecycle requirements")
  .from(policy);
export const expiry = sharing.rule("expiry").statement("Share links expire within 30 days");

export default provenance.build(sharing.rules(expiry));
