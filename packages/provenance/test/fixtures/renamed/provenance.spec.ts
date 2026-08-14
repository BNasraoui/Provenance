import { defineSpec, requirement, rule } from "@quality-sh/provenance";

const expiryDeclaration = rule("share-link-expiry").statement("Share links expire within 30 days");
const sharingDeclaration = requirement("sharing")
  .statement("Users can securely share documentation")
  .rules(expiryDeclaration);
export const shareLinks = defineSpec("share-links").requirements(sharingDeclaration).build();
