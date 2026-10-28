# Test report

Generated 2026-10-21T19:37:54.384Z · chain 5124 (Seismic testnet) · block 65169405

## Deployment

| Contract | Address |
| --- | --- |
| BlindBid | `0x9a0e48764f7e761cc8dfd99e82e239031ec13a10` |
| SealStore (private storage) | `0x427b3bc3a4fae1862ca380ffba35b349e7f91e85` |

RPC: `https://testnet-3.seismictest.net/rpc` · explorer: https://seismic-testnet.socialscan.io

## Self-test

```
BLINDBID · self-test
rpc https://testnet-3.seismictest.net/rpc · contract 0x9a0e48764f7e761cc8dfd99e82e239031ec13a10

network
  PASS  chainId is 5124  — got 5124
  PASS  chain is producing blocks  — 65169128 -> 65169128

contracts
  PASS  BlindBid has code  — 0x9a0e48764f7e761cc8dfd99e82e239031ec13a10 (7061 bytes)
  PASS  SealStore has code  — 0x427b3bc3a4fae1862ca380ffba35b349e7f91e85 (222 bytes)

tenders
  PASS  at least one tender exists  — 3 tenders

  lot #0  "Supply of server hardware"  bids=2  status=1
  PASS  private slots read as zero for an outside observer  — 0x0000 0x0000
  PASS  winner is set  — 0x9D3E6620ea0f85C73eF118504159558F41A0ffe7
  PASS  winning amount is published  — 12 sUSDC
  PASS  winner holds the lowest bid  — min=12
  PASS  aggregate matches the sum of bids  — 62 sUSDC
  PASS  winner appears in the bid list  — index 1

  lot #1  "Confidential audit of a treasury"  bids=0  status=0

  lot #2  "Supply of network switches"  bids=2  status=1
  PASS  private slots read as zero for an outside observer  — 0x0000 0x0000
  PASS  winner is set  — 0x06eb92C5D278176a22BAb02528dD1c3b40F79F4b
  PASS  winning amount is published  — 63 sUSDC
  PASS  winner holds the lowest bid  — min=63
  PASS  aggregate matches the sum of bids  — 141 sUSDC
  PASS  winner appears in the bid list  — index 1

17 passed, 0 failed
```

## On-chain state

### Lot #0

- subject: Supply of server hardware
- price limit: 100 sUSDC
- envelopes filed: 2
- status: AWARDED
- winner: `0x9D3E6620ea0f85C73eF118504159558F41A0ffe7`
- winning price: 12 sUSDC

**Private slots, read raw from the chain** (what any observer sees):

| Slot | Raw value |
| --- | --- |
| `0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5` | `0x0000000000000000000000000000000000000000000000000000000000000000` |
| `0xa6eef7e35abe7026729641147f7915573c7e97b47efa546f5f6e3230263bcb49` | `0x0000000000000000000000000000000000000000000000000000000000000000` |

**Buyer view** (signed read, decrypted for the buyer only):

| Bidder | Amount |
| --- | --- |
| `0x604238455d9fE35f199cD528b431c04698A45F49` | 50 sUSDC |
| `0x9D3E6620ea0f85C73eF118504159558F41A0ffe7` | 12 sUSDC |

### Lot #1

- subject: Confidential audit of a treasury
- price limit: 250 sUSDC
- envelopes filed: 0
- status: ACCEPTING BIDS


**Private slots, read raw from the chain** (what any observer sees):

| Slot | Raw value |
| --- | --- |
| — | — |


### Lot #2

- subject: Supply of network switches
- price limit: 200 sUSDC
- envelopes filed: 2
- status: AWARDED
- winner: `0x06eb92C5D278176a22BAb02528dD1c3b40F79F4b`
- winning price: 63 sUSDC

**Private slots, read raw from the chain** (what any observer sees):

| Slot | Raw value |
| --- | --- |
| `0xabbb5caa7dda850e60932de0934eb1f9d0f59695050f761dc64e443e5030a569` | `0x0000000000000000000000000000000000000000000000000000000000000000` |
| `0xd9d16d34ffb15ba3a3d852f0d403e2ce1d691fb54de27ac87cd2f993f3ec330f` | `0x0000000000000000000000000000000000000000000000000000000000000000` |



## What the tests prove

1. The bid amounts never appear in public state — the slots that hold them read as zero.
2. A bid never exposes its arguments — it is a type `0x4A` transaction with encrypted calldata.
3. The winner is decided inside the TEE, so the ranking of the bids was never public either.
4. The buyer can still verify everything, and can prove an aggregate (sum of bids) without
   revealing any single bid.
