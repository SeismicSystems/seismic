# Seismic Contracts

This repository contains solidity smart contracts and libraries designed for the [Seismic](https://seismic.systems) blockchain, a privacy-preserving EVM-compatible network. These contracts demonstrate how to leverage Seismic's unique features—**shielded storage** and **cryptographic precompiles**—to build applications that are impossible on standard Ethereum.

### Build, Test, Lint

This project uses [Seismic-Foundry](https://github.com/SeismicSystems/seismic-foundry), which is needed to compile any contracts with shielded types or seismic precompiles.

We recommend using our [mise tasks](./mise.toml), as these are also used in CI:
```bash
mise run test
mise run fmt
mise run artifacts::sync
...
```

## Project Structure

```
contracts/
├── src/
│   ├── predeploys/                # Installed at fixed addresses in genesis (script/genesis-contracts.txt)
│   │   ├── DepositContract.sol    # Eth2 staking deposits
│   │   ├── Directory.sol          # Key management contract
│   │   ├── GasTokenRegistry.sol   # Ordered gas-token configuration read by the execution client
│   │   ├── Intelligence.sol       # Multi-provider encryption
│   │   ├── KeyRotationRegistry.sol
│   │   ├── MeasurementAuthorityDev.sol
│   │   ├── MeasurementRegistry.sol
│   │   ├── ProtocolParams.sol     # Protocol configuration
│   │   └── ShieldedDelegationAccount.sol
│   ├── seismic-std-lib/           # Published library for contracts building on Seismic
│   │   ├── SRC20.sol              # SRC20 token standard
│   │   ├── SRC20Factory.sol
│   │   ├── SRC20Multicall.sol
│   │   ├── SRC20Token.sol
│   │   ├── interfaces/            # Including the predeploys that dApps call
│   │   └── utils/
│   │       ├── TxUtils.sol
│   │       └── precompiles/
│   │           └── CryptoUtils.sol
│   └── examples/
├── test/                          # Foundry tests
└── artifacts/                     # Compiled contracts
```

### Artifacts

The `artifacts/` directory contains compiled contract artifacts, including ABIs and bytecode. These are used for deployment and interaction with the contracts.

They are built with the ssolc pinned in the root [`mise.toml`](../mise.toml) and trimmed to the ABI and bytecode. After changing a contract, run `mise run artifacts::sync` and commit the result; CI's `artifacts::check` fails otherwise.

`MeasurementRegistry.json` is frozen: its runtime code hash is pinned outside this repo, so the sync leaves it alone (see `script/sync-artifacts.sh`).

TODO: we need to figure out a way to version these and make it more explicit which of these are deployed on each network, and at which block (or genesis).

## Contracts

### GasTokenRegistry (`src/predeploys/GasTokenRegistry.sol`)

An owner-managed, append-only registry of tokens accepted for gas payment. It is
installed in genesis at `0x0000000000000000000000476173546f6b656e73` (the ASCII
suffix `GasTokens`), and the execution client reads it directly from state when
selecting and settling the fee asset for Seismic transactions. **Adding an active
entry here enables gas payment in that token; the owner must verify the metadata
below before registering, because the client trusts it without validation.**

Each entry contains a token address, an active flag, its balance mapping's storage
slot, immutable owner-supplied `uint8 decimals` in **0–18 inclusive**, and a
`BalanceStorageMode`: **Shielded = 0** for `mapping(address => suint256)` or
**Public = 1** for `mapping(address => uint256)`. The mode describes only the balance
mapping, not the token's other fields. Register the proxy address for upgradeable
tokens. The owner must verify the slot, mode, precision, and full-width balance
accounting, including after proxy upgrades. A precision/layout change requires
deactivation; immutable metadata cannot be edited or the address re-registered.
The contract does not execute token code to discover decimals or validate layout.

The conversion policy is **one whole registered token per whole native unit**;
decimals change base-unit scaling, not exchange rates. For precision `d`, the
conversion divisor is `10^(18-d)`: maximum requirements and upfront debits round
up, refunds round down, and the beneficiary receives the reserve remainder.
Six-decimal entries retain the existing conversion. Zero-decimal entries are valid
and have coarser rounding; they do not default to six decimals. The owner must
approve this economic policy and direct balance accounting that bypasses transfer
hooks, pause/blacklist checks, and transfer events.

The execution client uses confidential balance operations for Shielded entries and
public balance operations for Public entries, retaining the selected mode through
reserves, deductions, and refunds. A holder balance whose privacy flag contradicts
the registered mode is never converted: automatic selection skips that entry, and
explicit selection of it fails transaction validation.

#### Administration and reads

- `addToken(address token, uint256 balanceSlot, BalanceStorageMode balanceStorageMode, uint8 decimals)`
  appends an **active** entry and returns its permanent index. The enum accepts only
  Shielded (`0`) or Public (`1`); unsupported values revert during ABI decoding.
  Decimals above `MAX_DECIMALS = 18` revert with `UnsupportedDecimals(uint8)`.
  An explicit decimals argument is required; there is no three-argument overload.
  Zero addresses, addresses without deployed code, and duplicate addresses are
  rejected, including inactive duplicates. Proxy addresses with deployed code are
  accepted; implementations are not validated. Mapping slot zero is valid.
- `activateToken(address token)` and `deactivateToken(address token)` look up the
  registered token by address and toggle its active flag. Repeating the same
  operation is allowed. Unregistered addresses revert with `TokenNotRegistered`.
  Neither operation checks token code or changes the entry's address, balance slot,
  storage mode, decimals, or priority, so a registered token can still be disabled
  if its code becomes unusable.
- `tokenCount()` returns the total entry count, including inactive tokens;
  `tokens(uint256 index)` returns `(address token, bool active, BalanceStorageMode
  balanceStorageMode, uint8 decimals, uint256 balanceSlot)`.
- `TokenAdded` includes the index, token address, balance slot, storage mode, and decimals;
  `TokenActivationChanged` reports the index, token address, and active flag.

Entries cannot be removed, reordered, or have their address, balance slot,
storage mode, or decimals updated. `MAX_TOKENS` is **32**, including inactive entries:
deactivation does not free capacity. This bounds execution-client scans. Automatic
payment is native first, then the first active, compatible entry in insertion order
that can cover the entire maximum gas cost. Seismic transactions carry a signed
`gasPayment` selector: `auto` (the ordered fallback above), strict `native`, or an
explicit registered token, the latter two without fallback. No gas cost is split
across assets; native currency alone funds transaction value.

Authorization matches `ProtocolParams`: a public `owner`, `OnlyOwner` checks,
`transferOwnership(address)`, and `renounceOwnership()`. Renouncing permanently
freezes configuration without clearing entries. For ordinary deployments the
constructor sets `owner = msg.sender`. Genesis installation bypasses the
constructor and must seed owner storage with the ProtocolParams initial owner
(`0xd412c5Ecd343e264381fF15aFC0aD78a67B79F35` in the current dev/testnet manifest).
Ownership is independent: transferring ProtocolParams ownership does not transfer
registry ownership.

#### Storage layout for client integration

The public layout is fixed for direct reads by execution clients:

| Location | Contents |
| --- | --- |
| Slot `0` | Owner address |
| Slot `1` | Token array length |
| `keccak256(abi.encode(uint256(1))) + 2 * index` | Token address in bits 0–159; active byte at offset 20; mode byte at offset 21; decimals byte at offset 22 |
| Preceding slot `+ 1` | Balance mapping slot (`uint256`) |

Execution clients read this layout directly from node state rather than call
`tokens(index)`. For an entry's first 256-bit storage word, decode the fields as:

```text
token    = word & ((1 << 160) - 1)
active   = ((word >> 160) & 0xff) != 0
mode     = (word >> 168) & 0xff
decimals = (word >> 176) & 0xff
```

Only bits 160–167 determine activation. Any nonzero value in that byte is true,
matching Solidity's storage reads of `bool`; do not require the byte to equal `1`.
Bits 168–175 contain mode, which must be exactly `0` (Shielded) or `1` (Public).
Bits 176–183 contain decimals, which must be in `0..=18`; zero means zero precision,
not missing metadata. Only bits 184–255 are ignored padding. Inactive entries are
skipped before validating mode/decimals. Automatic selection skips unsupported
entries without root/balance reads; explicit selection fails without fallback.
`word >> 160 != 0` is not a valid active check: it includes mode, decimals, and padding.

Canonical contract-written words are
`token | ((active ? 1 : 0) << 160) | (mode << 168) | (decimals << 176)` with zero
upper 72-bit padding. The two-slot entry stride and full-width mapping-root position
are unchanged. Activation normalizes the active byte but preserves all immutable
metadata and padding. Tests cover both modes at every supported precision, all
256 active-byte values, full-width roots, and fuzzed padding/activation changes.

The fresh-chain genesis installs the runtime with a nonzero seeded owner and
**zero token entries** (`tokens.length = 0`). No old-entry compatibility decoder
or legacy migration is provided. After genesis, native-funded owner transactions
register tokens with explicitly verified decimals. Initialize Shielded balances
through contract execution, not nonzero public genesis slots. Genesis validation
must check the runtime/address, owner, empty length, canonical words, and public
registry storage; runtime decoding does not reject configuration privacy flags.

For each token, a sender's balance key is
`keccak256(abi.encode(sender, balanceSlot))`. Token balances are stored at the
registered token address, not in this registry. Raw-layout and simulated-genesis
tests are in `test/GasTokenRegistry.t.sol`.

### Directory (`src/predeploys/Directory.sol`)

A key management and encryption service that allows users to register encryption keys and enables others to encrypt messages to them.

#### How It Works

1. Users call `setKey(suint256 _key)` to register their 256-bit encryption key
2. Anyone can call `encrypt(address to, bytes plaintext)` to encrypt a message to a registered user
3. The recipient calls `decrypt(bytes encryptedData)` to decrypt messages sent to them

#### Seismic Features Used

- **Shielded Storage**: Keys are stored as `mapping(address => suint256)`, making them invisible to external observers while remaining usable within the contract
- **AES Precompiles**: Encryption (`0x66`) and decryption (`0x67`) are performed via precompiled contracts

#### Comparison to Ethereum

On standard Ethereum, this contract would be **impossible to implement securely**:
- All storage is publicly readable, so encryption keys stored on-chain would be exposed
- Users would need to manage keys off-chain, defeating the purpose of a decentralized directory
- Any encryption scheme using on-chain keys would be trivially breakable

---

### Intelligence (`src/predeploys/Intelligence.sol`)

A multi-provider encryption orchestration contract that encrypts data to multiple registered providers simultaneously.

#### How It Works

1. Owner adds providers via `addProvider(address)`
2. Providers register their encryption keys in the Directory contract
3. Anyone calls `encryptToProviders(bytes plaintext)` to encrypt data to all providers at once
4. Returns an array of key hashes and corresponding encrypted data for each provider

#### Seismic Features Used

- **Directory Integration**: Leverages the Directory contract (at genesis address `0x1000000000000000000000000000000000000004`) for key management and encryption
- **Inherits Shielded Storage**: Provider keys remain confidential through the Directory's `suint256` storage

#### Comparison to Ethereum

On Ethereum, broadcasting encrypted data to multiple parties would require:
- Off-chain key exchange with each provider
- Client-side encryption before submission
- No guarantee that providers have registered valid keys

With Seismic, the entire workflow happens trustlessly on-chain with cryptographic guarantees.

---

### ShieldedDelegationAccount (`src/predeploys/ShieldedDelegationAccount.sol`)

An experimental [EIP-7702](https://eips.ethereum.org/EIPS/eip-7702) delegation contract that supports session keys with spend limits and encrypted transaction execution.

> **WARNING**: This contract is experimental and has not been audited.

#### How It Works

1. **Initialization**: Owner calls `setAESKey()` to generate a random AES key (stored as `suint256`)
2. **Key Authorization**: Owner authorizes session keys via `authorizeKey(keyType, publicKey, expiry, limitWei)`
   - Supports P256, WebAuthnP256, and Secp256k1 key types
   - Each key has an expiry timestamp and optional spend limit
3. **Encrypted Execution**:
   - Caller encrypts transaction data using `encrypt(plaintext)` (view function)
   - Caller signs the encrypted payload using EIP-712 typed data
   - Caller submits via `execute(nonce, encryptedCalls, signature, keyIndex)`
   - Contract verifies signature, checks limits, decrypts, and executes

#### Seismic Features Used

- **Shielded Storage**: The AES key is stored as `suint256 aesKey`, keeping it confidential
- **RNG Precompile** (`0x64`): Generates random AES key and nonces
- **AES Precompiles** (`0x66`, `0x67`): Encrypts/decrypts transaction calldata

#### Comparison to Ethereum

On Ethereum, session keys exist but with significant limitations:
- Transaction contents are always visible in the mempool and on-chain
- Spend limits can only restrict ETH value, not hide transaction details
- Attackers can front-run or analyze transaction patterns

With Seismic's ShieldedDelegationAccount:
- Transaction calldata is encrypted—observers cannot see what actions are being taken
- The AES key never leaves the enclave, so only the account can decrypt
- Combined with Seismic's encrypted transaction type (`0x4a`), the entire flow is private
