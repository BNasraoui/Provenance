import { defineSpec, requirement, rule } from "../../../dist/index.js";

const provenance = defineSpec("lifecycles");
const policy = provenance.source("policy").document("docs/policy.md");
const sharing = provenance
  .requirement("sharing")
  .statement("Shares expire")
  .from(policy);
const sessions = provenance
  .requirement("sessions")
  .statement("Sessions expire")
  .from(policy);

export const shareExpiry = sharing
  .rule("expiry")
  .statement("Share links expire");
export const authenticatedExpiry = provenance
  .rule("authenticated-expiry")
  .statement("Authenticated access expires");

export const spec = provenance.build(
  sharing.rules(shareExpiry, authenticatedExpiry),
  sessions.rules(authenticatedExpiry),
);

shareExpiry.verify("share-expiry", () => undefined);

const legacyRule = rule("legacy-expiry").statement("Legacy shares expire");
const legacyRequirement = requirement("legacy-sharing")
  .statement("Legacy shares expire")
  .rules(legacyRule);
defineSpec("legacy").requirements(legacyRequirement).build();
