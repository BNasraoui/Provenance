import { defineSpec } from "../../../dist/index.js";
import {
  localExpiry,
  policy,
  preserveRule,
  sharedAuthentication,
  sharing,
} from "./helpers.js";

const provenance = defineSpec("composed");
const guide = policy(provenance);
const shares = sharing(provenance, guide);
export const expiry = preserveRule(localExpiry(shares));
const authenticated = sharedAuthentication(provenance);

export default provenance.build(shares.rules(expiry, authenticated));
void expiry.verify("composed-expiry", () => undefined);
