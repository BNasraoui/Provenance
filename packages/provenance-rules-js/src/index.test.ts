import { rule, verifies } from "./index.js";

const implementation = (hours: number): boolean => hours > 38;
const ruleId = "rule_overtime";
const bound = rule(ruleId, implementation);

if (bound !== implementation || !bound(39)) {
  throw new Error("rule must preserve the implementation");
}

verifies(ruleId, "examples");
