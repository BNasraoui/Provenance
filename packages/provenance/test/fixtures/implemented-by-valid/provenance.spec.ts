import { defineSpec, requirement, rule } from "@quality-sh/provenance";
import { startWorkflow } from "./runtime.js";

const start = rule("start")
  .statement("Accepted workflows start")
  .implementedBy(startWorkflow);
const workflows = requirement("workflows")
  .statement("Accepted workflows execute")
  .rules(start);

export const spec = defineSpec("workflow-runtime").requirements(workflows).build();
