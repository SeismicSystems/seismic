---
description: ECDSA signature dataclass used for serialized TxSeismic
icon: file-code
---

# Signature

`Signature` stores ECDSA signature components.

## Definition

```python
@dataclass(frozen=True)
class Signature:
    v: int
    r: int
    s: int
```

## Fields

- `v`: recovery id (y-parity)
- `r`: first signature integer
- `s`: second signature integer

## Usage

Used by transaction serializers when building signed `TxSeismic` payloads.
