import { defineSpec, requirement, rule } from "@quality-sh/provenance";
import * as runtime from "./runtime.js";
import { WorkflowRunner } from "./runtime.js";

const direct = rule("direct").implementedBy(WorkflowRunner);
const namespaced = rule("namespaced").implementedBy(runtime.NamespacedRunner);
const workflows = requirement("workflows")
  .statement("Accepted workflows execute")
  .rules(direct, namespaced);

export const spec = defineSpec("workflow-runtime").requirements(workflows).build();

const bound = defineSpec("bound-workflow-runtime");
const boundRunner = bound.rule("runner").implementedBy(WorkflowRunner);
export const boundSpec = bound.build(
  bound.requirement("workflows").statement("Workflows run").rules(boundRunner),
);
