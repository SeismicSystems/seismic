---
description: Step-by-step guides for common workflows
icon: book-open
---

# Guides

Comprehensive guides for working with the Seismic Alloy (Rust) SDK.

## Available Guides

### [Shielded Write](shielded-write.md)

Complete guide to sending shielded transactions with encrypted calldata. Covers:

- Setting up a signed provider
- Defining contract interfaces with the `sol!` macro
- Encoding calldata and building transactions
- Marking transactions as seismic with `.seismic()`
- Sending and confirming transactions
- Security parameters and expiration windows

### [Signed Reads](signed-reads.md)

Guide to executing signed reads (encrypted `eth_call`). Covers:

- Why signed reads exist and when you need them
- Using `seismic_call()` for encrypted requests and responses
- Decoding decrypted return data
- Comparison with transparent reads

## Quick Navigation

| Topic                     | Description                                      | Guide                               |
| ------------------------- | ------------------------------------------------ | ----------------------------------- |
| **Shielded Transactions** | Send encrypted writes to modify on-chain state   | [Shielded Write](shielded-write.md) |
| **Signed Reads**          | Execute encrypted `eth_call` with identity proof | [Signed Reads](signed-reads.md)     |

## Related Documentation

- [Contract Interaction](../contract-interaction/) - Shielded and transparent call patterns
- [Examples](../examples/) - Runnable code examples
- [Provider](../provider/) - Provider setup and configuration
- [Installation](../installation.md) - Cargo setup and dependencies

## Contributing

Have a guide you'd like to see? Open an issue or submit a PR!
