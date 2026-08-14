import { defineSpec } from "@quality-sh/provenance";
import { startWorkflow } from "./runtime.js";

const provenance = defineSpec("workflow-runtime");
export const start = provenance
  .rule("start")
  .statement("Accepted workflows start")
  .implementedBy(startWorkflow);
