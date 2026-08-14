import { defineSpec } from "../../../dist/index.js";

const provenance = defineSpec("lifecycles");
const sharing = provenance.requirement("sharing").statement("Shares expire");
const sessions = provenance.requirement("sessions").statement("Sessions expire");
const expiry = sharing.rule("expiry").statement("Share links expire");

provenance.build(sharing.rules(expiry), sessions.rules(expiry));
