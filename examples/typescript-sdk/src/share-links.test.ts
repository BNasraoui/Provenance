import { shareLinks } from "./provenance.spec.js";
import { createShareLink } from "./share-links.js";

const THIRTY_DAYS = 30 * 24 * 60 * 60 * 1000;

await shareLinks.requirements.sharing.rules.expiry.verify("share-link-expiry", () => {
  const now = new Date();
  const link = createShareLink(now);

  if (link.expiresAt.getTime() > now.getTime() + THIRTY_DAYS) {
    throw new Error("share link expires after the 30-day limit");
  }
});
