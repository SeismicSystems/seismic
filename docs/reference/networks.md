---
description: Seismic network configurations, RPC endpoints, and chain IDs
icon: globe
---

# Networks & RPC Endpoints

## Deploy Tools

You can find our deploy tools at [github.com/SeismicSystems/deploy](https://github.com/SeismicSystems/deploy).

<details>

<summary>Mainnet</summary>

| Property   | Value                         |
| ---------- | ----------------------------- |
| Name       | Seismic                       |
| Chain ID   | 5123 (0x1403)                 |
| Chain type | EVM L1                        |
| RPC (HTTP) | TBA                           |
| RPC (WS)   | TBA                           |
| Block time | 1 block per ~600ms            |
| Finality   | 1 block (may become 2 blocks) |

The mainnet genesis date will be announced publicly. Follow official channels for updates.

**Explorer** — Currently in development. The explorer will support contract verification for Seismic Solidity.

</details>

<details>

<summary>Testnet</summary>

| Property   | Value                                                                            |
| ---------- | -------------------------------------------------------------------------------- |
| Name       | Seismic Testnet                                                                  |
| Chain ID   | 5124 (0x1404)                                                                    |
| Chain type | EVM L1                                                                           |
| RPC (HTTP) | `https://gcp-2.seismictest.net/rpc`                                              |
| RPC (WS)   | `wss://gcp-2.seismictest.net/ws`                                                 |
| Block time | ~600ms                                                                           |
| Finality   | 1 block                                                                          |
| Explorer   | [https://seismic-testnet.socialscan.io/](https://seismic-testnet.socialscan.io/) |
| Faucet     | [https://faucet.seismictest.net/](https://faucet.seismictest.net/)               |

Please don't hesitate to reach out to `dev@seismic.systems` for direct support from the team.

</details>

---

## See Also

- [Deploy an Encrypted Contract on our Devnet](../gitbook/networks/devnet.md) — step-by-step tutorial
- [Migrating from Ethereum](../gitbook/networks/migrating-from-ethereum.md) — porting existing contracts to Seismic
