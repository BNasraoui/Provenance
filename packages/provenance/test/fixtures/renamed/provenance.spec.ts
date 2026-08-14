import { defineSpec } from "@quality-sh/provenance";

const provenance = defineSpec("share-links");
const sharing = provenance
  .requirement("sharing")
  .statement("Users can securely share documentation");
export const shareLinkExpiry = sharing
  .rule("expiry")
  .statement("Share links expire within 30 days");

export default provenance.build(sharing.rules(shareLinkExpiry));
