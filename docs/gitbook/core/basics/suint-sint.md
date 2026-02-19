---
description: shielded unsigned integer / shielded integer
---

# suint / sint

All comparisons and operators for `suint` / `sint`  are functionally identical to `uint` / `int`. The universal casting rules and restrictions described in [Basics](./) apply.

```
suint256 a = suint256(10)
suint256 b = suint256(3)

// == EXAMPLES
a > b  // true
a | b  // 11
a << 2  // 40
a % b  // 1
```
