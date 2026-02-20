---
description: >-
  Rust client library for Seismic, built on top of alloy with the
  seismic-alloy-provider crate.
icon: gear
---

# Seismic Alloy (Rust)

{% hint style="info" %}
This page is being ported from [client.seismic.systems](https://client.seismic.systems/). Content coming soon.
{% endhint %}

Seismic maintains [seismic-alloy](https://github.com/SeismicSystems/seismic-alloy), which contains a crate called `seismic-alloy-provider`. This crate provides two provider types for interacting with the Seismic network from Rust:

- **`SeismicSignedProvider`** -- A client that can sign transactions. Use this for write operations and any interaction that requires a wallet (similar to a wallet client in viem).
- **`SeismicUnsignedProvider`** -- A read-only client. Use this for public queries and read operations that do not require signing (similar to a public client in viem).

Both providers handle Seismic transaction construction (type `0x4A`), calldata encryption, and TEE public key fetching automatically.

### Links

- GitHub: [github.com/SeismicSystems/seismic-alloy](https://github.com/SeismicSystems/seismic-alloy)
