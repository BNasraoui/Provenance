import { defineSpec } from "../../../dist/index.js";
import { attachSource } from "./helpers.js";

const first = defineSpec("first");
const second = defineSpec("second");
const policy = first.source("policy").document("docs/policy.md");
const sharing = second.requirement("sharing").statement("Shares expire");

attachSource(sharing, policy);
