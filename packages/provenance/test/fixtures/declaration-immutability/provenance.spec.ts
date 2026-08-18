import { defineSpec } from "../../../dist/index.js";
import type { RequirementDeclaration } from "../../../dist/index.js";

const provenance = defineSpec("immutable");
const requirement: RequirementDeclaration<"immutable", "sharing"> = provenance
  .requirement("sharing")
  .statement("Shares expire");

requirement.key = "sessions";
