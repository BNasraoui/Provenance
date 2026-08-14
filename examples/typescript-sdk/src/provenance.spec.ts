import { defineSpec } from "@quality-sh/provenance";
import { createShareLink } from "./share-links.js";

const provenance = defineSpec("share-links");
const policy = provenance
  .source("sharing-policy")
  .name("Sharing policy")
  .document("docs/sharing-policy.md");
const sharing = provenance
  .requirement("sharing")
  .statement("Users can securely share documentation")
  .description("Controls for links shared outside the organization")
  .from(policy);
export const expiry = sharing
  .rule("expiry")
  .statement("Share links must expire within 30 days")
  .implementedBy(createShareLink);

const spec = provenance.build(sharing.rules(expiry));

export default spec;
