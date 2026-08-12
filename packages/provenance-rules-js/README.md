# `@provenance/rules`

Tiny identity helpers for binding Provenance Rule records to JavaScript and
TypeScript code. A Rule is an independent behavioural obligation; `rule` marks
its primary production implementation without defining the Rule. The
declaration shape follows the same idea as Vercel Flags: wrap the function
where it is declared, keep its inferred callable type, and use the returned
callable normally. `rule` returns the exact function object. `verifies` binds
evidence from the containing test and returns nothing. Neither helper registers
global state or changes application behavior.

```ts
import { rule, verifies } from "@provenance/rules";

export const paysOvertime = rule("rule_overtime", (hours: number) =>
  hours > 38,
);

test("hours above 38 attract overtime", function overtimeExamples() {
  verifies("rule_overtime", "examples");
  expect(paysOvertime(39)).toBe(true);
});
```

Provenance's scanner is deliberately line-oriented. Keep `rule("id",` on one
line (the assignment may be on that line or immediately above it), and put
`verifies("id", "method")` inside a named function or function-valued `const`.
The supported methods are `exhaustion`, `property`, `examples`, `conformance`,
`construction`, and `proof`.

The package is TypeScript-first and uses only `tsc`:

```sh
npm run build
npm test
```
