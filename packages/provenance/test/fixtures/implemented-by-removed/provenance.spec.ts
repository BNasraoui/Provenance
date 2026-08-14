import { rule } from "@quality-sh/provenance";
import { startWorkflow } from "./runtime.js";

export const start = rule("start")
  .statement("Accepted workflows start")
  .implementedBy(startWorkflow);
