---
icon: file-code
---

# The contract

`BlindBid` is a single contract, ~170 lines of Solidity. The version below is the **shielded** build: it
compiles with Seismic's own compiler ([`ssolc`](https://github.com/SeismicSystems/seismic-solidity)) and
uses nothing but shielded types.

Full source: <https://github.com/Juniorj87/blind-bid/blob/main/contracts/BlindBidShielded.sol>

## Confidential bid amounts

```solidity
mapping(uint256 => mapping(uint32 => suint256)) private _bidAmount;
mapping(uint256 => mapping(uint32 => address)) public bidOwner;
mapping(uint256 => suint256) private _total;
```

A bid amount is a `suint256`, so the compiler emits `CSTORE`/`CLOAD` and marks the slot private on its
own. The supplier's identity is public (the event carries it); the price is not. A raw
`eth_getStorageAt` on that slot returns zero for every observer.

`_total` keeps the sum of all bids as a single confidential value. The buyer can prove the total
committed without any individual bid being revealed — the property a regulator asks for and a bidder
would not hand over.

## Filing a bid

```solidity
function submitBid(uint256 tenderId, uint256 amount) external returns (uint32 idx) {
    Tender storage t = tenders[tenderId];
    if (t.status != Status.Open) revert TenderClosed();
    if (block.timestamp >= t.deadline) revert TooLate();

    idx = t.bidCount;
    _bidAmount[tenderId][idx] = suint256(amount);
    bidOwner[tenderId][idx] = msg.sender;
    t.bidCount = idx + 1;
    _total[tenderId] = suint256(uint256(_total[tenderId]) + amount);

    emit BidSubmitted(tenderId, idx, msg.sender);
}
```

The `amount` never touches public storage. The transaction itself is sent as type `0x4A`, so the amount
is not in the calldata either — it is encrypted to the node's public key before it is signed.

## The award

```solidity
uint32 best = 0;
for (uint32 i = 1; i < n; i++) {
    if (uint256(_bidAmount[tenderId][i]) < uint256(_bidAmount[tenderId][best])) best = i;
}

t.winner = bidOwner[tenderId][best];
if (!t.sealedPrice) {
    t.winningAmount = uint256(_bidAmount[tenderId][best]);
    t.priceRevealed = true;
}
t.status = Status.Awarded;
```

The comparison runs over confidential values inside the enclave. To the outside world the lot goes from
"3 envelopes" to "awarded, winner `0x…`" — there is no moment at which the ordering of the bids was
observable, and with `sealedPrice` there is no moment at which the price is observable either.

## Buyer-only views

```solidity
function tenderBids(uint256 tenderId) external view returns (address[] memory bidders, uint256[] memory amounts) {
    Tender storage t = tenders[tenderId];
    if (msg.sender != t.buyer) revert NotBuyer();
    if (block.timestamp < t.deadline) revert TooEarly();
    ...
}
```

`msg.sender` is only meaningful here if the request carries a signature — on Seismic, `eth_call` does not
apply `from`, so `msg.sender` is recovered from the signature instead. That is why the front end uses a
signed read for the buyer sheet, and why the same call without a signature simply reverts.

## The portable build

The same contract can also be compiled with stock `solc` by moving the private-slot primitives into a
small Yul library that injects `CSTORE` (`0xB1`) through `verbatim`:

```yul
function store(slot, value) {
    verbatim_2i_0o(hex"b1", slot, value)
}
```

It exists because Seismic's compiler has no Windows build in every setup. The library is called with an
ordinary `call`/`staticcall` and **not** `delegatecall` — `CLOAD` inside a delegatecall does not work in
the `eth_call`/static context the node uses to simulate.

Both builds are deployed and pass the same checks.

## Footguns worth knowing

- **Shielded types consume a whole storage slot.** A `suint64` still occupies 32 bytes, and shielded
  values are never packed — a slot is entirely private or entirely public.
- **`SLOAD` on a private slot reverts.** Read shielded values with the shielded type, never with
  `vm.load`/`eth_getStorageAt` in application code.
- **A sealed bid is expensive.** Type `0x4A` transactions cost roughly twenty times a plain one
  (≈ 0.07 sUSDC against ≈ 0.003 sUSDC on the testnet).
