import { defineSpec, requirement, rule } from "@quality-sh/provenance";

const expiryDeclaration = rule("expiry").statement("Share links expire within 30 days");
const sharingDeclaration = requirement("sharing")
  .statement("Users can securely share documentation")
  .rules(expiryDeclaration);
const spec = defineSpec("share-links").requirements(sharingDeclaration).build();

export const sharing = spec.handles.requirements.sharing;
export const shareLinkExpiry = sharing.rules.expiry;
