---
description: Deploy a private SRC20 token on Seismic testnet with a single command — no compiler required
icon: rocket
---

# Deploy an SRC20 in 60 Seconds

The SRC20 Factory is a pre-deployed contract on Seismic testnet that lets anyone create a private token without installing `sforge` or writing a single line of Solidity. You give it a name, symbol, and supply; it hands back a deployed token address.

## Quickstart

```bash
bunx create-src20
```

That's it. The CLI walks you through the rest interactively, or you can pass everything up front:

```bash
bunx create-src20 \
  --name "My Private Token" \
  --symbol "MPT" \
  --supply 1000000 \
  --key 0xYourPrivateKey
```

Example output:

```
  Create SRC20 Token on Seismic

  Deploying to Seismic testnet...

  Token deployed!

  Address:  0xabc...
  Name:     My Private Token
  Symbol:   MPT
  Supply:   1,000,000
  Owner:    0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
  Tx:       0xdef...
  Explorer: https://seismic-testnet.socialscan.io/address/0xabc...
```

## CLI flags

| Flag       | Description                                                    | Default                             |
| ---------- | -------------------------------------------------------------- | ----------------------------------- |
| `--name`   | Token name                                                     | prompted                            |
| `--symbol` | Token symbol                                                   | prompted                            |
| `--supply` | Initial supply in whole tokens (multiplied by 10¹⁸ internally) | prompted                            |
| `--key`    | 0x-prefixed 64-character hex private key                       | prompted (hidden input)             |
| `--rpc`    | Custom RPC URL                                                 | `https://testnet-2.seismictest.net/rpc` |

{% hint style="info" %}
Supply is always treated as whole tokens. Passing `--supply 1000000` mints `1,000,000 × 10¹⁸` base units, using 18 decimals.
{% endhint %}

## What you get

Every deployed token is a full `SRC20Token` contract — Seismic's private variant of ERC20. Balances, transfer amounts, and allowances are encrypted on-chain using `suint256` shielded types. Outside observers see only encrypted ciphertext. The token deployer becomes the `owner` and has exclusive mint and burn rights.

## Interfaces

The factory exposes four interfaces depending on how you want to integrate:

| Interface                | When to use                               |
| ------------------------ | ----------------------------------------- |
| [CLI](README.md)         | Deploy a token manually from the terminal |
| [TypeScript SDK](sdk.md) | Embed token creation in a dApp or script  |
| [Web GUI](web.md)        | Deploy via browser without writing code   |
| [REST API](api.md)       | Query deployed tokens from any language   |

## Network

- **Chain ID:** 5124 (Seismic testnet)
- **Factory address:** `0x87F850cbC2cFfac086F20d0d7307E12d06fA2127`
- **Explorer:** [seismic-testnet.socialscan.io](https://seismic-testnet.socialscan.io/address/0x87F850cbC2cFfac086F20d0d7307E12d06fA2127)
