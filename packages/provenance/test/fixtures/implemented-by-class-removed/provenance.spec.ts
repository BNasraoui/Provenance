import { rule } from "@quality-sh/provenance";
import { WorkflowRunner } from "./runtime.js";

export const workflow = rule("workflow").implementedBy(WorkflowRunner);
