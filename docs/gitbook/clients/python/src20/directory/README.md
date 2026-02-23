---
description: Directory contract helpers for SRC20 viewing keys
icon: key
---

# Directory

Directory helpers manage AES viewing keys stored in the genesis Directory contract (`0x1000...0004`).

## Available helpers

- [register_viewing_key](register-viewing-key.md)
- [get_viewing_key](get-viewing-key.md)
- [check_has_key / get_key_hash](check-has-key.md)

Async variants are also available for each operation.

## Behavior summary

- Register key: shielded write (`setKey`)
- Fetch own key: signed read (`getKey`)
- Query key presence/hash: plain public `eth_call`
