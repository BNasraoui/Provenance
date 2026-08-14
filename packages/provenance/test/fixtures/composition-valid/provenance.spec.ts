import { defineSpec } from "../../../dist/index.js";
import {
  bindClass,
  localExpiry,
  policy,
  preserveRule,
  sharedAuthentication,
  sharing,
} from "./helpers.js";
import { WorkflowRunner } from "../implemented-by-class-valid/runtime.js";

const provenance = defineSpec("composed");
const guide = policy(provenance);
const shares = sharing(provenance, guide);
export const expiry = bindClass(preserveRule(localExpiry(shares)), WorkflowRunner);
const authenticated = sharedAuthentication(provenance);

export default provenance.build(shares.rules(expiry, authenticated));
void expiry.verify("composed-expiry", () => undefined);
