# py-docs-contract branch status

## What this branch does

Replaces made-up / vague function names across all Python SDK contract docs with traceable names from either:

1. **IExampleVault** — a Solidity interface defined in `docs/gitbook/clients/python/contract/README.md` that gives every code snippet a concrete reference contract
2. **Real specs** — SRC20 (`balanceOf`, `allowance`) and ERC20 (`transfer`, `approve`, `totalSupply`, `name`, `symbol`, `decimals`)

## Completed

- Added `IExampleVault` interface to `contract/README.md` (public reads, shielded reads, writes)
- Added CLAUDE.md rule: "No made-up function names in examples"
- Replaced `getMyBalance()` and `getBalance()` with real SRC20 `balanceOf()`
- Replaced all remaining made-up names across 11 doc files:
  - `namespaces/read.md` — getTokenBalance, getAllowance, complexCalculation, simulateDeposit, getArray, getStatus
  - `namespaces/tread.md` — getTopHolders, getReserves, getArray, maxSupply
  - `namespaces/write.md` — complexOperation
  - `namespaces/twrite.md` — complexOperation, complexMethod, register, vote, bid, castPrivateVote
  - `namespaces/dwrite.md` — complexMethod, problematicMethod, highValueTransfer
  - `namespaces/README.md` — setPublicData (was already cleaned in earlier commit)
  - `shielded-contract.md` — setPublicData, getPublicData, sensitiveOperation, performAction, myFunction, deposit(amount=...)
  - `async-shielded-contract.md` — same as above plus riskyOperation, doSomething
  - `public-contract.md` — getTotalSupply, getInfo, getAllHolders, getTotalCount, getRiskyData, getData, myFunction
  - `async-public-contract.md` — same as above plus slowFunction
- Removed "Development Workflow" section from namespaces/README.md (generic advice, not specific to any namespace)

## Not yet done

- Final review pass: read each file end-to-end to verify all examples are coherent with the IExampleVault interface
- `hasRole(bytes32, address)` in public-contract.md — currently kept as-is (it's a real OpenZeppelin function, arguably fine)
- `register(username)` / `vote(proposalId, bool)` patterns in twrite.md were simplified; could restore with inline Solidity if more context is wanted

## Codex audit update (this branch)

- Confirmed in SDK source (`clients/py/src/seismic_web3/contract/shielded.py`) that both `.read` and `.tread` now call `decode_abi_output(...)` and return ABI-decoded Python values.
- Fixed stale docs that still described raw bytes/manual decoding:
  - `contract/public-contract.md`
  - `contract/async-public-contract.md`
- Updated namespace docs to match runtime semantics for no-output functions (`None`, not empty bytes):
  - `contract/namespaces/read.md`
  - `contract/namespaces/tread.md`
- Corrected stale client usage in edited examples:
  - `create_public_client(...)` no longer shows `chain=...`
  - async public examples now `await create_async_public_client(...)`
  - direct `PublicContract`/`AsyncPublicContract` examples now pass `w3` (not `w3.eth`)

## Remaining drift spotted (not patched in this pass)

- `contract/shielded-contract.md` and `contract/async-shielded-contract.md` still contain old constructor/factory examples (`chain=...`, `account=...`, and `w3=w3.eth`) that do not match current client signatures.
