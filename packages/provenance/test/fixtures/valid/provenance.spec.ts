import { defineSpec, requirement, rule, source } from "@quality-sh/provenance";

const policy = source("policy").document("docs/policy.md").name("Security policy");
const expiryDeclaration = rule("expiry").statement("Share links expire within 30 days");
const sharingDeclaration = requirement("sharing")
  .statement("Users can securely share documentation")
  .description("Share-link lifecycle requirements")
  .from(policy)
  .rules(expiryDeclaration);
const spec = defineSpec("share-links").sources(policy).requirements(sharingDeclaration).build();

export const sharing = spec.handles.requirements.sharing;
export const expiry = sharing.rules.expiry;

// @ts-expect-error only declared handles are available
void sharing.rules.revocation;
