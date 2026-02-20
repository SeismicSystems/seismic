---
description: Send a shielded transaction with encrypted calldata to the Seismic network
icon: shield-halved
---

# send\_shielded\_transaction

Send a shielded transaction with end-to-end encrypted calldata using Seismic's `TxSeismic` (type `0x4a`) transaction format. The calldata is encrypted client-side and decrypted only inside the node's TEE.

***

## Overview

This is the low-level method for sending encrypted transactions. It:

1. Encrypts the provided plaintext calldata using AES-GCM
2. Constructs a `TxSeismic` transaction with encryption metadata
3. Signs the transaction with your private key
4. Broadcasts it to the network via `eth_sendRawTransaction`
5. Returns the transaction hash

For contract interactions, use `contract.write.functionName(...)` instead — it handles ABI encoding and encryption automatically.

***

## Signatures

<table><thead><tr><th width="400">Sync</th><th>Async</th></tr></thead><tbody><tr><td><pre class="language-python"><code class="lang-python">def send_shielded_transaction(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> HexBytes
</code></pre></td><td><pre class="language-python"><code class="lang-python">async def send_shielded_transaction(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> HexBytes
</code></pre></td></tr></tbody></table>

***

## Parameters

All parameters are **keyword-only** to prevent accidental misuse.

| Parameter   | Type                                                                                          | Default  | Description                                            |
| ----------- | --------------------------------------------------------------------------------------------- | -------- | ------------------------------------------------------ |
| `to`        | `ChecksumAddress`                                                                             | Required | Recipient contract address                             |
| `data`      | `HexBytes`                                                                                    | Required | Plaintext calldata (will be encrypted)                 |
| `value`     | `int`                                                                                         | `0`      | Wei to transfer with the transaction                   |
| `gas`       | `int \| None`                                                                                 | `None`   | Gas limit (auto-estimated if `None`)                   |
| `gas_price` | `int \| None`                                                                                 | `None`   | Gas price in wei (uses network default if `None`)      |
| `security`  | [`SeismicSecurityParams`](../../api-reference/signature/seismic-security-params.md) \| `None` | `None`   | Custom security parameters (block hash, nonce, expiry) |
| `eip712`    | `bool`                                                                                        | `False`  | Use EIP-712 typed data signing instead of raw signing  |

***

## Returns

**Type:** `HexBytes`

The transaction hash returned by `eth_sendRawTransaction`.

```python
# Sync
tx_hash = w3.seismic.send_shielded_transaction(...)
assert isinstance(tx_hash, HexBytes)

# Async
tx_hash = await w3.seismic.send_shielded_transaction(...)
assert isinstance(tx_hash, HexBytes)
```

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_wallet_client, PrivateKey
from hexbytes import HexBytes

# Create wallet client
w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Send shielded transaction
tx_hash = w3.seismic.send_shielded_transaction(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=HexBytes("0xa9059cbb000000000000000000000000..."),
    value=0,
    gas=100_000,
    gas_price=20 * 10**9,  # 20 gwei
)

print(f"Transaction sent: {tx_hash.hex()}")

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Status: {'success' if receipt['status'] == 1 else 'failed'}")
```

### Async Usage

```python
from seismic_web3 import create_async_wallet_client, PrivateKey
from hexbytes import HexBytes

# Create async wallet client
w3 = await create_async_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Send shielded transaction
tx_hash = await w3.seismic.send_shielded_transaction(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=HexBytes("0xa9059cbb000000000000000000000000..."),
    value=0,
)

print(f"Transaction sent: {tx_hash.hex()}")

# Wait for confirmation
receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Status: {'success' if receipt['status'] == 1 else 'failed'}")
```

### Sending ETH

```python
# Send 1 ETH to contract with encrypted calldata
tx_hash = w3.seismic.send_shielded_transaction(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=HexBytes("0xd0e30db0"),  # deposit()
    value=10**18,  # 1 ETH in wei
)
```

### Custom Security Parameters

```python
from seismic_web3 import SeismicSecurityParams

# Use longer expiry window (200 blocks instead of 100)
security = SeismicSecurityParams(
    blocks_window=200,
)

tx_hash = w3.seismic.send_shielded_transaction(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=HexBytes("0x..."),
    security=security,
)
```

### With ABI Encoding

```python
from seismic_web3.contract.abi import encode_shielded_calldata

# Manually encode function call
abi = [
    {
        "name": "transfer",
        "type": "function",
        "inputs": [
            {"name": "to", "type": "address"},
            {"name": "amount", "type": "uint256"},
        ],
    },
]

calldata = encode_shielded_calldata(
    abi,
    "transfer",
    ["0x1234567890123456789012345678901234567890", 1000],
)

# Send transaction
tx_hash = w3.seismic.send_shielded_transaction(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=calldata,
)
```

### Using EIP-712 Signing

```python
# Use EIP-712 typed data signing (more secure, supported by some wallets)
tx_hash = w3.seismic.send_shielded_transaction(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=HexBytes("0x..."),
    eip712=True,
)
```

***

## Implementation Details

### Encryption Process

The SDK performs the following steps:

1. **Fetch security parameters** (if not provided):
   * Get latest block hash for freshness proof
   * Generate random 12-byte AES-GCM nonce
   * Calculate expiry block (current + 100 blocks)
2. **Construct metadata**:
   * Sender address
   * Chain ID, nonce, to, value
   * Block hash, expiry, encryption nonce
3. **Encrypt calldata**:
   * Use ECDH-derived AES-GCM key
   * Bind ciphertext to metadata via AAD (Additional Authenticated Data)
4. **Build transaction**:
   * Construct `TxSeismic` (type `0x4a`)
   * Include Seismic elements (encryption metadata)
5. **Sign and broadcast**:
   * Sign transaction with private key (raw or EIP-712)
   * Broadcast via `eth_sendRawTransaction`

### Gas Estimation

If `gas=None`, the SDK:

* Calls `eth_estimateGas` with the **encrypted** transaction
* Adds a buffer for potential variations
* Uses the estimated value

If the estimation fails, consider providing an explicit gas limit.

### Gas Price

If `gas_price=None`, the SDK uses `eth_gasPrice` to fetch the current network gas price.

***

## Privacy Guarantees

### What Gets Encrypted

* Function selector (4 bytes)
* All function arguments
* Encoding metadata

### What Remains Visible

These fields are **not** encrypted:

* `from` — Your wallet address
* `to` — Contract address
* `value` — ETH amount sent
* `gas` and `gas_price` — Gas parameters
* `nonce` — Transaction nonce
* Encryption metadata (block hash, expiry, nonce)

An observer can see you sent a transaction to a contract, but cannot see which function you called or what arguments you passed.

***

## Security Considerations

### Block Hash Freshness

Every shielded transaction includes a recent block hash as a freshness proof. The node validates:

1. The block hash corresponds to a real block
2. The block is recent (within the chain's freshness window)

This prevents replay attacks and stale submissions.

### Transaction Expiry

Transactions include an expiry block number (default: current + 100 blocks). After this block, the node rejects the transaction.

If your transaction doesn't get mined in time, you must create a new one with updated parameters.

### Nonce Uniqueness

Each transaction uses a cryptographically random 12-byte encryption nonce. **Never reuse nonces** — this breaks AES-GCM security.

The SDK generates random nonces automatically. Only override if you know what you're doing (e.g., testing).

***

## Error Handling

```python
from web3.exceptions import TransactionNotFound, TimeExhausted

try:
    tx_hash = w3.seismic.send_shielded_transaction(
        to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
        data=HexBytes("0x..."),
        gas=100_000,
    )

    receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=120)

    if receipt['status'] == 0:
        print("Transaction reverted")
    else:
        print("Transaction succeeded")

except ValueError as e:
    print(f"Transaction failed: {e}")
    # Possible causes: insufficient gas, invalid nonce, expired block hash

except TimeExhausted:
    print("Transaction not mined within timeout")
```

***

## When to Use

### Use `send_shielded_transaction()` When

* You need low-level control over transaction construction
* You have pre-encoded calldata
* You're implementing custom encryption logic
* You need to handle contract deployment (use `to=None`)

### Use `contract.write` When

* You're calling a contract function with known ABI
* You want automatic ABI encoding
* You prefer high-level, ergonomic API

***

## Notes

### Requires Wallet Client

This method is only available on wallet clients created with `create_wallet_client()`. It requires:

* A private key for signing
* Encryption state (derived from TEE public key)

Public clients (`create_public_client`) do not have this method.

### Transaction Type

This method creates `TxSeismic` (type `0x4a`) transactions. These are Seismic-specific and not compatible with standard Ethereum nodes.

### Nonce Management

The SDK automatically fetches and increments the nonce using `eth_getTransactionCount`. You don't need to manage nonces manually.

***

## See Also

* [contract.write](../../../../gitbook/client-libraries/seismic-python/namespaces/contract/namespaces/write.md) — High-level contract write API
* [debug\_send\_shielded\_transaction()](debug-send-shielded-transaction.md) — Send with debug info
* [signed\_call()](signed-call.md) — Execute signed reads instead of writes
* [SeismicSecurityParams](../../api-reference/signature/seismic-security-params.md) — Security parameter reference
* [Shielded Write Guide](../../guides/shielded-write.md) — Full encryption workflow
