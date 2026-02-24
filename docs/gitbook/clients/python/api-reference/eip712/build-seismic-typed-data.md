---
description: Build eth_signTypedData_v4 payload for TxSeismic
icon: shield
---

# build_seismic_typed_data

Build a JSON-serializable [EIP-712](https://eips.ethereum.org/EIPS/eip-712) typed-data payload for a Seismic transaction.

This function exists primarily for parity with the TypeScript client, where `eth_signTypedData_v4` is used by browser wallets like MetaMask. In Python you'll typically sign transactions directly with [`sign_seismic_tx_eip712`](sign-seismic-tx-eip712.md), but this function is available when you need the raw typed-data structure (e.g. sending it to a frontend for wallet signing).

## Signature

```python
def build_seismic_typed_data(tx: UnsignedSeismicTx) -> dict[str, Any]
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tx` | [`UnsignedSeismicTx`](../transaction-types/unsigned-seismic-tx.md) | Yes | The unsigned Seismic transaction |

## Returns

| Type | Description |
|------|-------------|
| `dict[str, Any]` | Dict with keys `types`, `primaryType`, `domain`, `message` |

## Output structure

The returned dictionary follows the `eth_signTypedData_v4` format:

```json
{
  "types": {
    "EIP712Domain": [
      { "name": "name",              "type": "string"  },
      { "name": "version",           "type": "string"  },
      { "name": "chainId",           "type": "uint256" },
      { "name": "verifyingContract", "type": "address" }
    ],
    "TxSeismic": [
      { "name": "chainId",          "type": "uint64"   },
      { "name": "nonce",            "type": "uint64"   },
      { "name": "gasPrice",         "type": "uint128"  },
      { "name": "gasLimit",         "type": "uint64"   },
      { "name": "to",               "type": "address"  },
      { "name": "value",            "type": "uint256"  },
      { "name": "input",            "type": "bytes"    },
      { "name": "encryptionPubkey", "type": "bytes"    },
      { "name": "encryptionNonce",  "type": "uint96"   },
      { "name": "messageVersion",   "type": "uint8"    },
      { "name": "recentBlockHash",  "type": "bytes32"  },
      { "name": "expiresAtBlock",   "type": "uint64"   },
      { "name": "signedRead",       "type": "bool"     }
    ]
  },
  "primaryType": "TxSeismic",
  "domain": {
    "name": "Seismic Transaction",
    "version": "2",
    "chainId": 1946,
    "verifyingContract": "0x0000000000000000000000000000000000000000"
  },
  "message": {
    "chainId": 1946,
    "nonce": 42,
    "gasPrice": 20000000000,
    "gasLimit": 100000,
    "to": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    "value": 1000000000000000000,
    "input": "0xabcd...",
    "encryptionPubkey": "0x02...",
    "encryptionNonce": 12345,
    "messageVersion": 2,
    "recentBlockHash": "0x1234...",
    "expiresAtBlock": 12345678,
    "signedRead": false
  }
}
```

## Field encoding

### `message` values

- **Numeric fields** — integers (not hex strings)
- **Address fields** — checksummed `0x`-prefixed hex strings
- **Bytes fields** (`input`, `encryptionPubkey`, `recentBlockHash`) — `0x`-prefixed hex strings
- **`encryptionNonce`** — the 12-byte nonce converted to an integer via `int.from_bytes(…, "big")`
- **`to`** — becomes the zero address when `tx.to is None`

### `domain`

- **name** — `"Seismic Transaction"`
- **version** — `"2"` (matches `message_version`)
- **chainId** — numeric chain identifier
- **verifyingContract** — zero address (signing happens off-chain)

### `primaryType`

Always `"TxSeismic"`.

## Example

```python
from seismic_web3 import build_seismic_typed_data
import json

typed_data = build_seismic_typed_data(unsigned_tx)
print(json.dumps(typed_data, indent=2))
```

## Notes

- The hash computed from this data matches [`eip712_signing_hash(tx)`](eip712-signing-hash.md)
- The returned dict is JSON-serializable (no Python-specific types)

## Warnings

- If `tx.to` is `None`, it's encoded as the zero address
- All bytes fields must be `0x`-prefixed hex strings
- Ensure numeric values fit in their declared EIP-712 types (e.g. `uint64`)

## See Also

- [sign_seismic_tx_eip712](sign-seismic-tx-eip712.md) — signs using this data internally
- [eip712_signing_hash](eip712-signing-hash.md) — computes hash from typed data
- [domain_separator](domain-separator.md) — domain component
- [struct_hash](struct-hash.md) — message component
- [UnsignedSeismicTx](../transaction-types/unsigned-seismic-tx.md) — input transaction type
