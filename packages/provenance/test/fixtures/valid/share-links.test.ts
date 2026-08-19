import { shareLinks } from "./provenance.spec.js";

void shareLinks.requirements.sharing.rules.expiry.verify("share-link-expiry", () => undefined);

// A test can state the file it runs in, whatever the runtime reports.
void shareLinks.requirements.sharing.rules.expiry.verify(
  "share-link-expiry",
  () => undefined,
  import.meta,
);
void shareLinks.requirements.sharing.rules.expiry.verify(
  "share-link-expiry",
  () => undefined,
  { file: import.meta.url, method: "property" },
);
