---
icon: scale-balanced
---

# Sealed-Bid Tender

A procurement flow where **the bid amount never appears in public state and never appears in the
transaction**.

This is the [sealed-bid auction use case](../overview/use-cases.md) taken one step further: the award is
not just encrypted, it is **computed inside the TEE** — the ordering of the bids never existed publicly
either, and the winning price can stay sealed after the award.

A live instance is deployed on the Seismic testnet (chain 5124):

| Contract | Address |
|---|---|
| `BlindBid` | [`0x9a0e48764f7e761cc8dfd99e82e239031ec13a10`](https://seismic-testnet.socialscan.io/address/0x9a0e48764f7e761cc8dfd99e82e239031ec13a10) |
| `BlindBid` (portable build, stock `solc`) | [`0xd9191d5f065a44778890de4e80c57fc2b1237ec4`](https://seismic-testnet.socialscan.io/address/0xd9191d5f065a44778890de4e80c57fc2b1237ec4) |

Source, scripts, a front end and the generated test report:
**<https://github.com/Juniorj87/blind-bid>**

## The three layers

| Layer | What an observer sees |
|---|---|
| **Transaction** | "someone called a function on the contract" — not which function, not the arguments (type `0x4A`) |
| **State** | `eth_getStorageAt` → `0x0000…0000` for every bid slot |
| **Comparison** | the winner only — the ordering was computed inside the enclave and never left it |

```solidity
mapping(uint256 => mapping(uint32 => suint256)) private _bidAmount; // confidential
mapping(uint256 => suint256) private _total;                        // confidential aggregate
```

Because the amount is a `suint256`, the compiler emits `CSTORE`/`CLOAD` for it and flags the storage slot
private by itself — no `verbatim`, no assembly. See [the contract](the-contract.md) for the details.

## What it does that the auction snippet does not

The use-case page sketches a *highest-bid* auction that reveals the winning amount in `getWinner()`. A
procurement tender needs three things more:

1. **Lowest bid wins.** A tender is not an auction — the buyer wants the cheapest compliant offer, and
   the comparison happens over the confidential amounts inside the enclave.
2. **The winning price can stay sealed** (`sealedPrice`). In a normal auction the price must be disclosed
   to everyone, otherwise it is not an auction; in compliance-grade procurement it does not have to be.
   The winner is published, the price is not.
3. **An audit aggregate** (`totalCommitted`): the sum of every bid, confidential as a whole, so a
   regulator can verify the total committed without any single bid being revealed.

## Reading the result

Two views of the same lot, one after the award:

- **public**: envelope count, deadline, winner, and — if the price is not sealed — the winning price.
- **buyer**: a signed read returns every amount and the aggregate. On Seismic, `eth_call` does not apply
  `from`, so `msg.sender` is recovered from a signature; that is why this is a signed read and not a
  normal call.

The generated test report of the live deployment
([TEST-REPORT.md](https://github.com/Juniorj87/blind-bid/blob/main/TEST-REPORT.md)) contains the raw
private-slot reads — every one of them zero — next to the buyer-only table.

## Try it

```bash
git clone https://github.com/Juniorj87/blind-bid.git
cd blind-bid && npm install
echo "MAIN_PK=0x..." > .env        # your own funded testnet key
npm run build:shielded            # compile with ssolc (Seismic's compiler)
npm run seed:shielded             # deploy + demo lots with sealed bids
npm test                          # 17 read-only checks against your deployment
npm run writer & npm run web      # the front end on http://localhost:5173
```

A sealed bid is a type `0x4A` transaction and costs ≈ 0.07 sUSDC; a plain transaction ≈ 0.003 sUSDC.
