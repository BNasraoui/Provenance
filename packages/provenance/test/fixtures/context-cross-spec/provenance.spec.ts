import { defineSpec } from "../../../dist/index.js";

const first = defineSpec("first");
const second = defineSpec("second");
const policy = first.source("policy").document("docs/policy.md");

second.requirement("sharing").statement("Shares expire").from(policy);
