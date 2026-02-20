---
description: Execute a signed read (eth_call) with encrypted calldata and decrypted response
icon: book-open
---

# signed_call()

Execute a signed read operation (`eth_call`) with encrypted calldata and a decrypted response. This allows reading private contract state that requires authentication or privacy, without broadcasting a transaction to the network.

***

## Overview

A signed read is like a regular `eth_call`, but with Seismic's encryption:

1. Your calldata is encrypted using AES-GCM (like shielded writes)
2. The call is authenticated with your signature
3. The TEE executes the call inside its secure environment
4. The response is encrypted by the TEE and decrypted by your client

This is useful for:
- Reading private state (balances, allowances, account data)
- Querying view functions that require authentication
- Testing write operations before broadcasting

***

## Signatures

<table>
<thead>
<tr>
<th width="400">Sync</th>
<th>Async</th>
</tr>
</thead>
<tbody>
<tr>
<td>

```python
def signed_call(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int = 30_000_000,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> HexBytes | None
```

</td>
<td>

```python
async def signed_call(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int = 30_000_000,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> HexBytes | None
```

</td>
</tr>
</tbody>
</table>

***

## Parameters

All parameters are **keyword-only**.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `to` | `ChecksumAddress` | Required | Contract address to call |
| `data` | `HexBytes` | Required | Plaintext calldata (will be encrypted) |
| `value` | `int` | `0` | Wei to include in the call context |
| `gas` | `int` | `30_000_000` | Gas limit for the call |
| `security` | [`SeismicSecurityParams`](../../api-reference/transaction-types/seismic-security-params.md) \| `None` | `None` | Custom security parameters (block hash, nonce, expiry) |
| `eip712` | `bool` | `False` | Use EIP-712 typed data signing instead of raw signing |

***

## Returns

**Type:** `HexBytes | None`

- **`HexBytes`**: The decrypted response from the contract (if non-empty)
- **`None`**: If the contract returned empty data

```python
# Sync
result = w3.seismic.signed_call(...)
if result:
    print(f"Response: {result.hex()}")

# Async
result = await w3.seismic.signed_call(...)
if result:
    print(f"Response: {result.hex()}")
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

# Execute signed call
result = w3.seismic.signed_call(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0x70a08231000000000000000000000000..."),  # balanceOf(address)
)

if result:
    # Decode the result (e.g., uint256)
    balance = int.from_bytes(result, byteorder='big')
    print(f"Balance: {balance}")
else:
    print("No response")
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

# Execute signed call
result = await w3.seismic.signed_call(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0x70a08231000000000000000000000000..."),
)

if result:
    balance = int.from_bytes(result, byteorder='big')
    print(f"Balance: {balance}")
```

### With ABI Encoding

```python
from seismic_web3.contract.abi import encode_shielded_calldata

# Manually encode function call
abi = [
    {
        "name": "balanceOf",
        "type": "function",
        "inputs": [{"name": "account", "type": "address"}],
        "outputs": [{"name": "", "type": "uint256"}],
    },
]

calldata = encode_shielded_calldata(
    abi,
    "balanceOf",
    ["0x1234567890123456789012345678901234567890"],
)

# Execute signed call
result = w3.seismic.signed_call(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=calldata,
)

if result:
    balance = int.from_bytes(result, byteorder='big')
    print(f"Balance: {balance}")
```

### Using Contract Namespace

```python
# Recommended: use contract.read for automatic ABI handling
contract = w3.seismic.contract(
    address="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    abi=ABI,
)

# Execute signed read via contract namespace
balance = contract.read.balanceOf(
    "0x1234567890123456789012345678901234567890",
)
print(f"Balance: {balance}")
```

### Custom Security Parameters

```python
from seismic_web3 import SeismicSecurityParams

# Use longer expiry window
security = SeismicSecurityParams(blocks_window=200)

result = w3.seismic.signed_call(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0x..."),
    security=security,
)
```

### With Value

```python
# Include ETH value in the call context
# (useful for payable view functions)
result = w3.seismic.signed_call(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0x..."),
    value=10**18,  # 1 ETH
)
```

### Custom Gas Limit

```python
# Use custom gas limit (default is 30M)
result = w3.seismic.signed_call(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0x..."),
    gas=50_000_000,
)
```

### Using EIP-712 Signing

```python
# Use EIP-712 typed data signing
result = w3.seismic.signed_call(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0x..."),
    eip712=True,
)
```

***

## Implementation Details

### RPC Method

This method uses a Seismic-specific RPC call:

```
seismic_signedCall
```

The call includes:
- Encrypted calldata
- Signature proving your identity
- Security parameters (block hash, expiry, nonce)

### Encryption Process

The SDK performs the following steps:

1. **Fetch security parameters** (if not provided):
   - Get latest block hash for freshness proof
   - Generate random 12-byte AES-GCM nonce
   - Calculate expiry block (current + 100 blocks)

2. **Construct metadata**:
   - Sender address
   - Chain ID, nonce, to, value
   - Block hash, expiry, encryption nonce
   - Mark as signed read

3. **Encrypt calldata**:
   - Use ECDH-derived AES-GCM key
   - Bind ciphertext to metadata via AAD

4. **Sign and send**:
   - Sign the transaction (raw or EIP-712)
   - Send via `seismic_signedCall`

5. **Decrypt response**:
   - TEE encrypts the response
   - SDK decrypts using the shared key
   - Returns the plaintext result

### No State Changes

Signed calls do **not** modify blockchain state. They are read-only operations, similar to `eth_call`. The signature is used for:
- Authentication (proving who is calling)
- Encryption key derivation
- Access control (contracts can enforce read permissions)

### Gas Limit

Unlike `eth_call`, signed calls have a gas limit (default: 30,000,000). This prevents infinite loops and ensures the call completes in reasonable time.

If your call requires more gas, increase the `gas` parameter.

***

## Privacy Guarantees

### What Gets Encrypted

- Function selector (4 bytes)
- All function arguments
- Encoding metadata
- The response from the contract

### What Remains Visible

- Your IP address (to the RPC node)
- The contract address you're calling
- Timing information

An observer watching network traffic can see you made a call to a contract, but cannot see:
- Which function you called
- What arguments you passed
- What data was returned

***

## Use Cases

### Reading Private State

```python
# Read your private balance in a shielded token contract
balance = contract.read.balanceOf(w3.eth.default_account)
print(f"My balance: {balance}")
```

### Access-Controlled Queries

```python
# Query data that requires authentication
# (contract enforces msg.sender == authorized_user)
result = contract.read.getPrivateData()
```

### Testing Before Broadcasting

```python
# Test a transaction before sending it
try:
    result = w3.seismic.signed_call(
        to=contract_address,
        data=encoded_calldata,
        value=10**18,
        gas=200_000,
    )
    print("Call succeeded, safe to broadcast")
except Exception as e:
    print(f"Call would fail: {e}")
```

***

## Security Considerations

### Signed Read Flag

Every signed call includes a `signed_read: true` flag in its Seismic elements. This prevents the transaction from being replayed as a write transaction.

The node enforces:
- Signed reads cannot modify state
- They use `seismic_signedCall`, not `eth_sendRawTransaction`

### Block Hash Freshness

Like shielded writes, signed calls include a recent block hash to prevent replay attacks and ensure freshness.

### Transaction Expiry

Signed calls expire after a certain number of blocks (default: 100). If the call takes too long or is retried after expiry, it will be rejected.

### Nonce Uniqueness

Each signed call uses a cryptographically random encryption nonce. The SDK generates this automatically — never reuse nonces.

***

## Error Handling

```python
try:
    result = w3.seismic.signed_call(
        to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        data=HexBytes("0x..."),
    )

    if result:
        print(f"Response: {result.hex()}")
    else:
        print("Empty response")

except ValueError as e:
    print(f"Call failed: {e}")
    # Possible causes: contract reverted, insufficient gas, invalid calldata

except Exception as e:
    print(f"Unexpected error: {e}")
```

***

## When to Use

### Use `signed_call()` When

- You need low-level control over the call
- You have pre-encoded calldata
- You're implementing custom encryption logic

### Use `contract.read` When

- You're calling a contract function with known ABI
- You want automatic ABI encoding and decoding
- You prefer high-level, ergonomic API

### Use `contract.tread` When

- The data is public (no privacy needed)
- You want to save gas and reduce latency
- The function doesn't require authentication

***

## Notes

### Requires Wallet Client

This method is only available on wallet clients created with `create_wallet_client()`. It requires:
- A private key for signing
- Encryption state (derived from TEE public key)

### No Transaction Hash

Unlike `send_shielded_transaction()`, this method does not return a transaction hash. It returns the decrypted response directly.

### Nonce Not Incremented

Signed calls do not increment your account nonce. You can make multiple signed calls without affecting your ability to send transactions.

***

## See Also

- [contract.read](../contract/namespaces/read.md) — High-level signed read API
- [send_shielded_transaction()](send-shielded-transaction.md) — Send write transactions
- [contract.tread](../contract/namespaces/tread.md) — Transparent reads without encryption
- [SeismicSecurityParams](../../api-reference/transaction-types/seismic-security-params.md) — Security parameter reference
- [Signed Reads Guide](../../guides/signed-reads.md) — Full guide to authenticated reads
