# Rule bindings outside Rust

The scanner remains line-oriented rather than parsing each language's syntax
tree. These helpers are intentionally inert: they keep a rule id next to the
Rule's primary production implementation without introducing a registry or
changing the function. The binding does not define the Rule; a Rule may exist
before any implementation is bound to it.
JavaScript and TypeScript can use the source package in
`packages/provenance-rules-js/`. The snippets below are small enough to keep in
an application's own support module.

## Python decorator

```python
from collections.abc import Callable
from typing import ParamSpec, TypeVar

P = ParamSpec("P")
R = TypeVar("R")


def rule(_rule_id: str) -> Callable[[Callable[P, R]], Callable[P, R]]:
    def bind(function: Callable[P, R]) -> Callable[P, R]:
        return function

    return bind


@rule("rule_overtime")
def pays_overtime(hours: int) -> bool:
    return hours > 38
```

The scanner recognizes `@rule("id")` directly above a `def`. Python
verification decorators are not binding-grade yet; use the universal comment
channel shown below for tests.

## Go wrapper

```go
func rule[Function any](_ string, function Function) Function {
	return function
}

var paysOvertime = rule("rule_overtime", func(hours int) bool {
	return hours > 38
})
```

The scanner recognizes a same-line `rule("id",` call and takes the variable on
the left of `=` as the item name. Verification calls are not recognized in Go.

## Java static helper

```java
import java.util.function.IntPredicate;

final class ProvenanceRules {
    private ProvenanceRules() {}

    static <Function> Function rule(String ruleId, Function function) {
        return function;
    }
}

final class PayrollRules {
    private static final IntPredicate PAYS_OVERTIME =
        ProvenanceRules.rule("rule_overtime", hours -> hours > 38);
}
```

The scanner recognizes `rule("id",` after a plain or qualified helper name and
uses the assigned field as the item name. Java verification helpers are not
recognized.

## Universal comment floor

All six supported languages can bind rules and verification evidence through a
comment immediately above the relevant function. Use this channel whenever a
native helper pattern is not binding-grade:

```python
# @provenance verification: examples
# @provenance rule: rule_overtime
def test_overtime_examples() -> None:
    ...
```

Comments are portable but can drift away from the symbol. Prefer a recognized
native binding for the primary implementation, and use comments only for the
gaps named above.
