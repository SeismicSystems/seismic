---
description: The `w3.seismic` namespace classes
icon: brackets-curly
---

# Namespaces

The SDK attaches a `seismic` object to client instances.

## Public namespaces

- [SeismicPublicNamespace](seismic-public-namespace.md): sync read-only methods
- [AsyncSeismicPublicNamespace](async-seismic-public-namespace.md): async read-only methods

Available methods:

- `get_tee_public_key()`
- `get_deposit_root()`
- `get_deposit_count()`
- `contract(... )` returning public contract wrappers (`.tread` only)

## Wallet namespaces

- [SeismicNamespace](seismic-namespace.md): sync full wallet methods
- [AsyncSeismicNamespace](async-seismic-namespace.md): async full wallet methods

Wallet namespaces add:

- `send_shielded_transaction()`
- `debug_send_shielded_transaction()`
- `signed_call()`
- `deposit()`
- `contract(..., eip712=False)` returning shielded contract wrappers

## Method reference

See [Namespace Methods](methods/README.md) for per-method details.
