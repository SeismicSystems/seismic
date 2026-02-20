---
description: Encryption elements generation and calldata encryption filler
icon: shield-halved
---

# SeismicElementsFiller

`SeismicElementsFiller` is the core filler responsible for preparing Seismic shielded transactions. It generates encryption elements, fetches the TEE public key, encrypts calldata, and attaches all necessary metadata to the transaction request.

## Overview

When a Seismic transaction (type `0x4A`) passes through the filler pipeline, `SeismicElementsFiller` performs the following:

1. **Fetches the TEE public key** from the Seismic node (cached after first call)
2. **Fetches the latest block number** to calculate the transaction expiration
3. **Generates an encryption nonce** using ECDH between the ephemeral secret key and the TEE public key
4. **Encrypts the calldata** using AES-GCM
5. **Attaches `TxSeismicElements`** (nonce, ephemeral public key, expiration block) to the transaction request

For non-Seismic transactions, the filler is a no-op.

## Definition

```rust
pub struct SeismicElementsFiller {
    tee_pubkey: Option<PublicKey>,
    blocks_window: Option<u64>,
    ephemeral_secret_key: SecretKey,
    signed_read: bool,
}
```

| Field                  | Type                | Description                                       |
| ---------------------- | ------------------- | ------------------------------------------------- |
| `tee_pubkey`           | `Option<PublicKey>` | Cached TEE public key (fetched lazily if `None`)  |
| `blocks_window`        | `Option<u64>`       | Custom expiration window in blocks (default: 100) |
| `ephemeral_secret_key` | `SecretKey`         | Randomly generated ephemeral key for ECDH         |
| `signed_read`          | `bool`              | Whether to enable signed read mode                |

## Constructors

### `new`

Create a new filler with default settings. The TEE public key will be fetched lazily on the first transaction.

```rust
pub fn new() -> Self
```

**Returns:** A new `SeismicElementsFiller` with:

- No cached TEE public key (will be fetched from the node)
- Default blocks window (100)
- A freshly generated ephemeral secret key
- Signed read disabled

### `with_tee_pubkey_and_url`

Create a filler with a pre-cached TEE public key. This avoids the initial RPC call to fetch the key.

```rust
pub fn with_tee_pubkey_and_url(tee_pubkey: PublicKey) -> Self
```

| Parameter    | Type        | Required | Description                 |
| ------------ | ----------- | -------- | --------------------------- |
| `tee_pubkey` | `PublicKey` | Yes      | The TEE public key to cache |

**Returns:** A `SeismicElementsFiller` with the given TEE public key pre-cached.

## Configuration Methods

### `with_signed_read`

Set the signed read flag. When enabled, the filler includes additional data for signed read operations.

```rust
pub fn with_signed_read(self, signed_read: bool) -> Self
```

| Parameter     | Type   | Required | Description                        |
| ------------- | ------ | -------- | ---------------------------------- |
| `signed_read` | `bool` | Yes      | Whether to enable signed read mode |

**Returns:** `Self` (builder pattern).

### `with_blocks_window`

Set a custom expiration window. The transaction will expire after `current_block + blocks_window`.

```rust
pub fn with_blocks_window(self, blocks_window: u64) -> Self
```

| Parameter       | Type  | Required | Description                                |
| --------------- | ----- | -------- | ------------------------------------------ |
| `blocks_window` | `u64` | Yes      | Number of blocks until transaction expires |

**Returns:** `Self` (builder pattern).

{% hint style="info" %}
The default `BLOCKS_WINDOW` is **100 blocks**. In most cases, you do not need to change this. Increase it if transactions are expiring before confirmation on a congested network.
{% endhint %}

### `ephemeral_secret_key`

Access the ephemeral secret key generated at construction time.

```rust
pub fn ephemeral_secret_key(&self) -> &SecretKey
```

**Returns:** `&SecretKey` -- Reference to the ephemeral secret key.

## Two-Phase Design

`SeismicElementsFiller` implements Alloy's `TxFiller<N>` trait, which defines two phases:

### Prepare Phase

The prepare phase is async. It fetches external data needed for encryption:

1. **TEE public key** -- Fetched from the node via `seismic_getTeePublicKey` RPC call (or uses cached value)
2. **Latest block number** -- Fetched to calculate the expiration block

These values are returned as a "fillable" that is passed to the fill phase.

### Fill Phase

The fill phase is synchronous. It uses the prepared data to:

1. Calculate the expiration block: `latest_block + blocks_window`
2. Generate the encryption nonce via ECDH between the ephemeral secret key and the TEE public key
3. Encrypt the calldata using AES-GCM with the derived shared secret
4. Create `TxSeismicElements` containing the nonce, ephemeral public key, and expiration
5. Call `N::set_seismic_elements()` to attach elements to the request
6. Call `N::set_request_input()` to replace plaintext calldata with encrypted calldata

## Encryption Flow

```
ephemeral_secret_key + tee_public_key
  -> ECDH shared secret
  -> HKDF key derivation
  -> AES-GCM encryption of calldata
  -> encrypted calldata + nonce stored in transaction
```

The node's TEE can reverse this process because it holds the private key corresponding to `tee_public_key`. It uses the ephemeral public key (included in `TxSeismicElements`) to derive the same shared secret.

## Constants

| Constant        | Value | Description                                         |
| --------------- | ----- | --------------------------------------------------- |
| `BLOCKS_WINDOW` | `100` | Default number of blocks before transaction expires |

## Usage

### Default Configuration

In most cases, you do not interact with `SeismicElementsFiller` directly. It is automatically included in the provider's filler pipeline:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node.seismicdev.net/rpc".parse()?;

// SeismicElementsFiller is configured automatically
let provider = sreth_signed_provider(wallet, url).await?;
```

### With Pre-Cached TEE Key

If you already know the TEE public key (e.g., from a previous session):

```rust
use seismic_alloy::prelude::*;

let filler = SeismicElementsFiller::with_tee_pubkey_and_url(known_tee_pubkey);
```

### With Custom Expiration Window

```rust
use seismic_alloy::prelude::*;

let filler = SeismicElementsFiller::new()
    .with_blocks_window(200);  // Expire after 200 blocks instead of 100
```

### With Signed Reads

```rust
use seismic_alloy::prelude::*;

let filler = SeismicElementsFiller::new()
    .with_signed_read(true);
```

## Notes

- The ephemeral secret key is generated once at construction and reused for all transactions through this filler instance
- The TEE public key is fetched lazily and cached -- only one RPC call is made per filler instance
- For non-Seismic transactions, the filler does nothing (no encryption, no elements)
- The encryption uses `seismic-enclave` under the hood for ECDH and AES-GCM operations
- If the TEE public key changes (e.g., node rotation), you need a new filler or provider instance

## See Also

- [Fillers Overview](./) - How fillers compose in the pipeline
- [SeismicGasFiller](seismic-gas-filler.md) - Gas estimation filler (runs after this one)
- [Encryption](../provider/encryption.md) - Detailed encryption documentation
- [SeismicNetwork Trait](../network/seismic-network-trait.md) - Methods called by this filler
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider that uses this filler
