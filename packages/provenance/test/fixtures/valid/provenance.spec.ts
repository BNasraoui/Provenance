import { defineSpec, requirement, rule, source } from "@quality-sh/provenance";
import { createShareLink } from "./runtime.js";

export const shareLinks = defineSpec("share-links")
  .requirements(
    requirement("sharing")
      .statement("Users can securely share documentation")
      .description("Controls for links shared outside the organization")
      .from(
        source("sharing-policy")
          .name("Sharing policy")
          .document("docs/sharing-policy.md"),
      )
      .rules(
        rule("expiry")
          .statement("Share links must expire within 30 days")
          .implementedBy(createShareLink),
      ),
  )
  .build();

void shareLinks.handles.requirements.sharing.rules.expiry;
void shareLinks.sources["sharing-policy"];

// @ts-expect-error only declared handles are available
void shareLinks.requirements.sharing.rules.revocation;
