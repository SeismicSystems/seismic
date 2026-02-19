---
description: shielded boolean
metaLinks:
  alternates:
    - https://app.gitbook.com/s/hkB2uNxma1rxIgBfHgAT/core/basics/sbool
---

# sbool

All comparisons and operators for `sbool` function identically to `bool`. The universal casting rules and restrictions described in [Basics](./) apply.

We recommend reading the point on conditional execution in [Best Practices & Gotchas](../best-practices-and-gotchas.md) prior to using `sbool` since it's easy to accidentally leak information with this type.

```
sbool a = sbool(true)
sbool b = sbool(false)

// == EXAMPLES
a && b  // false
!b  // true
```
