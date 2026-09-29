---
icon: gas-pump
---

# Paying for Gas

## Overview

Two assets participate in Seismic fees:

- A **gas stablecoin** — the asset users hold and pay fees with. It is an ERC-20-style
  predeploy (today a USDC stub; the production stablecoin contract replaces it on mainnet).
- The **native token** — the protocol's settlement and gas-accounting unit. It is **not
  generally distributed**: it is held by the protocol and a small set of validators / special
  accounts and used only in special cases. Gas is *metered* in the native token internally, so
  the EVM's Ethereum-compatible fee machinery (EIP-1559 basefee, gas pricing, tooling) is
  unchanged — but most users never hold it.

Transaction gas can be paid in **either** token. The `value` field — a native-token transfer
carried in the transaction — can only be paid from the native balance. Because native is not
generally distributed, **a typical user holds only the stablecoin, sends `value = 0`
transactions, and pays gas in the stablecoin**; funds move between users as stablecoin token
transfers (ERC-20 `transfer` in calldata), not via the native `value` field. Native `value`
transfers are a special-case operation available to native holders.

Nothing in the transaction selects the fee token — the node decides per transaction from the
sender's balances. Construction, signing, and serialization are identical to Ethereum.

## The 1:1 numéraire

The native token is defined to be worth exactly **one unit of the gas stablecoin** — a
*definitional* 1:1, not a market rate. There is no price oracle and nothing to drift: the native
token is not a freely-traded asset, so "1 native = 1 stablecoin" is a protocol invariant rather
than a peg that must be maintained.

This is what lets the two tokens interoperate for gas with no oracle. Gas is priced and metered in
native wei (18 decimals), exactly as on Ethereum; charging a stablecoin-paying user is then a pure
decimal rescale (see [Conversion and rounding](#conversion-and-rounding)).

## Which token pays

Let `gas_max = max_fee_per_gas × gas_limit (+ blob fee)` be the most gas could cost, and `value`
the native transfer amount. The seismic-revm handler selects:

| Condition | Gas paid in | `value` paid from | Typical sender |
| --- | --- | --- | --- |
| `native ≥ gas_max + value` | native | native | native holder (validator / special) |
| `native < gas_max + value` and `native ≥ value` and `stablecoin ≥ gas` | stablecoin | native | — |
| `native < value` | — (rejected) | — | — |

In practice the **native-first** rule separates the two populations on its own: a normal user has
`native = 0`, so the first row never matches and they always pay in the stablecoin (with
`value = 0`, since `native ≥ value` forces it). Native holders take the first row and pay in
native.

The last row is the hard constraint: the stablecoin can never fund a native `value` transfer. A
stablecoin-only account attempting `value > 0` is rejected.

## The gas stablecoin contract

Today the stablecoin is a predeploy at:

```
0x790701048922E265105fd6a4467a2901c2201C43
```

It is a bytecode-less account whose `_balances` mapping lives at storage slot `3` (standard
Solidity `mapping(address => uint256)` layout). The handler reads and writes balances directly
against that slot using the confidential `CLOAD`/`CSTORE` opcodes — using `SSTORE` would flip the
slot's privacy flag to public and break later `CSTORE`s — so balance changes preserve the
`is_private` flag. RPC endpoints likewise read the slot directly rather than calling `balanceOf`
(there is no contract code to execute).

{% hint style="info" %}
This address currently points at a USDC stub. The production stablecoin contract that replaces it
on mainnet may use a different address; the decimal-scaling constant below assumes a **6-decimal**
token.
{% endhint %}

## Conversion and rounding

The stablecoin uses **6 decimals**; the native token uses **18 decimals** (wei). The handler
converts between them with a fixed divisor of `10^12` (= `10^(18-6)`) — the decimal alignment of
the 1:1 numéraire above. There is no oracle, only a rescale.

Gas is charged and refunded in whole stablecoin base units (millionths of a token), with
deliberately asymmetric rounding:

- **Charge** (when the transaction is admitted): `ceil(gas_wei / 10^12)`. Ceiling division
  guarantees that even a gas cost below `10^12` wei still costs at least one base unit —
  otherwise sub-divisor transactions would be free.
- **Refund** (unused gas, after execution): `floor(unused_gas_wei / 10^12)`. Flooring the refund
  means the block beneficiary keeps at most one base unit of rounding dust per transaction, and
  the refund can never exceed what was charged.

Mechanically, in the stablecoin path the **full gas reservation is transferred from the sender to
the block beneficiary up front**, and the unused remainder is returned after execution — both as
stablecoin transfers via `CLOAD`/`CSTORE`. The beneficiary keeps `effective_gas_price × gas_used`
(in stablecoin), with **nothing burned** and no treasury middleman. (Seismic does not burn the
basefee in the native path either — unlike mainnet EIP-1559, the full effective gas price goes to
the beneficiary.)

### Minimum charge and low gas prices

Because the stablecoin has only 6 decimals, the smallest chargeable amount is one base unit,
`0.000001` of a token (= `10^12 wei` at the 1:1 rate). With the ceiling rounding above, this means
**a stablecoin-paid transaction is never free** — but it also rounds gas *up* to that granularity.

For example, at a gas price of 100 wei and 21,000 gas, the true gas cost is `2,100,000 wei`
(`2.1 × 10^6`). Converting: `ceil(2.1×10^6 / 10^12) = 1` base unit, so the sender is charged
`0.000001` of a token — far above the true cost. This overcharge is only significant when
`gas_limit × effective_gas_price` is far below `10^12 wei` (sub-gwei prices); at ~1 gwei the
rounding is negligible (21,000 gas → 21 base units, essentially exact). The native path has no such
rounding — gas is deducted exactly in wei. Mainnet fee parameters should keep per-transaction fees
comfortably above this `0.000001`-token floor.

## Checking balances and estimating gas

- [`eth_getBalance`](rpc-methods/README.md) returns the **spendable** balance,
  `max(native_balance, stablecoin_balance × 10^12)`, so a stablecoin-only wallet reports a non-zero
  balance reflecting what it can actually spend on gas.
- `eth_estimateGas` accounts for the stablecoin the same way, so a stablecoin-only wallet receives
  a non-zero gas allowance instead of `gas required exceeds allowance (0)`.

Both report an *effective, spendable* view rather than the raw committed account record.

{% hint style="warning" %}
**Wallet integrators:** `eth_getBalance` reports *gas-spendability*, not the amount an account can
send as a native `value` transfer. A stablecoin-only account shows a non-zero balance but holds no
native token, so a native "send max" built from this number will fail — it requires `native ≥ value`,
so `eth_estimateGas` returns a 0 allowance and the transaction is rejected. Move funds with
stablecoin token transfers (ERC-20 `transfer`, `value = 0`) sized from the token's `balanceOf`, and
treat the reported balance as a gas budget rather than a sendable amount.
{% endhint %}

## Examples

The scenarios below assume an effective gas price of 1 gwei and 21,000 gas used, so the gas fee is
`21,000 × 10^9` wei, which converts to 21 stablecoin base units (`0.000021` of a token) at the 1:1
rate. Balances are written `native, stablecoin`; the sender is
`0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`.

| native, stablecoin | `value` | Gas paid in | Outcome |
| --- | --- | --- | --- |
| 0, 100 | 0 | stablecoin | **The common user case.** Gas paid in stablecoin to the beneficiary; native stays 0. |
| 1, 0 | 0 | native | Native holder (validator / special). Gas paid in native. |
| 1, 0 | 0.5 | native | Native holder transferring native; `value` + gas both from native. |
| 0.5, 100 | 0.5 | stablecoin | Native holder with limited native: `value` (0.5) from native, gas from stablecoin. |
| 0.3, 1000 | 0.5 | — | **Rejected** (`LackOfFundForMaxFee`): native cannot cover `value`, and the stablecoin cannot substitute. Enforced during execution, so the transaction is never included in a block. |

The first row is the path almost every user takes. The last row is the boundary that follows from
native not being generally distributed: native `value` transfers require a native balance, and the
stablecoin can never stand in for them.
