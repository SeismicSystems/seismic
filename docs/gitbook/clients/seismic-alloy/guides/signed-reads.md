---
description: Encrypted eth_call that proves your identity to the contract
icon: magnifying-glass
---

# Signed Reads

A signed read uses `seismic_call()` to build a full `TxSeismic` just like a [shielded write](shielded-write.md), but targets the `eth_call` endpoint instead of broadcasting a transaction. The node decrypts the calldata inside the TEE, executes the call, encrypts the result, and returns it. The provider then decrypts the response automatically.

---

### Why signed reads exist

Any contract function that depends on `msg.sender` needs a signed read. A plain `eth_call` (transparent read) does not attach sender identity, so the contract sees the zero address as the caller.

```rust
// Signed read -- proves your identity to the contract
// msg.sender = your wallet address
let result = provider.seismic_call(SendableTx::Builder(tx.into())).await?;

// Transparent read -- msg.sender is 0x0
// The contract does not know who is calling
let result = provider.call(tx).await?;
```

A common example: a contract with a `balanceOf()` function that takes no arguments and uses `msg.sender` internally to look up the caller's balance. If you call it with a transparent read, the contract sees the zero address and returns its balance -- which is almost certainly zero.

---

### What gets encrypted

Both the calldata you send **and** the result you get back are encrypted. An observer watching the network can see that you made a call to a particular contract address, but not what function you called or what was returned.

| Direction              | Encrypted | Description                              |
| ---------------------- | --------- | ---------------------------------------- |
| Request (calldata)     | Yes       | AES-GCM with ECDH shared secret          |
| Response (return data) | Yes       | TEE encrypts with the same shared secret |

---

### Step-by-step

#### 1. Set up a signed provider

Signed reads require a `SeismicSignedProvider` because the provider needs an ephemeral keypair for ECDH key derivation and response decryption.

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url: reqwest::Url = "https://node.seismicdev.net/rpc".parse()?;

let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;
```

#### 2. Encode the call input

Use the `sol!` macro to define the interface and encode the view function:

```rust
use alloy::sol;
use alloy::sol_types::SolCall;

sol! {
    interface ISeismicCounter {
        function isOdd() public view returns (bool);
    }
}

let call_input = ISeismicCounter::isOddCall {}.abi_encode();
```

#### 3. Build a seismic transaction

Mark the transaction as seismic with `.seismic()` so the provider knows to encrypt:

```rust
use alloy_primitives::{Bytes, TxKind};

let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
    .with_input(Bytes::from(call_input))
    .with_kind(TxKind::Call(contract_address))
    .into()
    .seismic();
```

#### 4. Use `seismic_call()` instead of `call()`

`seismic_call()` encrypts the request, sends a signed `eth_call`, and decrypts the response:

```rust
use alloy_provider::SendableTx;

let result = provider.seismic_call(SendableTx::Builder(tx.into())).await?;
```

{% hint style="info" %}
Use `seismic_call()` for signed reads and `send_transaction()` for shielded writes. Both encrypt calldata, but `seismic_call()` also decrypts the response and does not modify on-chain state.
{% endhint %}

#### 5. Decode the decrypted response

The `result` is a `Bytes` value containing the ABI-encoded return data. Decode it using the generated types:

```rust
use alloy::sol_types::SolCall;

let decoded = ISeismicCounter::isOddReturn::abi_decode(&result, true)?;
println!("Is odd: {}", decoded._0);
```

---

### Signed read vs. transparent read

| Aspect            | Signed Read (`seismic_call`)          | Transparent Read (`call`) |
| ----------------- | ------------------------------------- | ------------------------- |
| `msg.sender`      | Your wallet address                   | Zero address              |
| Calldata          | Encrypted                             | Plaintext                 |
| Return data       | Encrypted, then decrypted by provider | Plaintext                 |
| Provider required | `SeismicSignedProvider`               | Any provider              |
| Use case          | Private state, access-controlled data | Public state              |

---

### How the provider handles encryption

Under the hood, `seismic_call()` performs the following sequence:

```
1. Fill the transaction (nonce, chain ID, seismic elements, gas)
2. Encrypt calldata with AES-GCM:
   - Key = ECDH(provider ephemeral secret, TEE public key)
   - AAD = RLP-encoded TxSeismicMetadata
3. Sign the call request with the wallet
4. Send to node as eth_call
5. Node decrypts calldata inside TEE
6. Node executes the call
7. Node encrypts the response with the same ECDH shared secret
8. Provider receives encrypted response
9. Provider decrypts response with ephemeral secret key
10. Return decrypted bytes to caller
```

---

### Complete example

```rust
use seismic_alloy::prelude::*;
use alloy::sol;
use alloy::sol_types::SolCall;
use alloy_primitives::{Bytes, TxKind};
use alloy_provider::SendableTx;
use alloy_signer_local::PrivateKeySigner;

sol! {
    interface ISeismicCounter {
        function isOdd() public view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Set up signed provider
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    // 2. Encode the view call
    let contract_address = std::env::var("CONTRACT_ADDRESS")?.parse()?;
    let call_input = ISeismicCounter::isOddCall {}.abi_encode();

    // 3. Build seismic transaction
    let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(call_input))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    // 4. Execute signed read
    let result = provider.seismic_call(SendableTx::Builder(tx.into())).await?;

    // 5. Decode the decrypted response
    let decoded = ISeismicCounter::isOddReturn::abi_decode(&result, true)?;
    println!("Is odd: {}", decoded._0);

    Ok(())
}
```

---

### When to use transparent reads instead

Not every read needs encryption. Use a transparent read (`provider.call()`) when:

- The function does **not** depend on `msg.sender`
- The data is already public on-chain
- You want to use an unsigned provider (no private key)

```rust
// Transparent read -- works with any provider, including unsigned
let calldata = IMyContract::getPublicValueCall {}.abi_encode();
let tx = seismic_foundry_tx_builder()
    .with_input(Bytes::from(calldata))
    .with_kind(TxKind::Call(contract_address))
    .into();

let result = provider.call(tx).await?;
```

## See Also

- [Shielded Write](shielded-write.md) - Encrypted state-changing transactions
- [Shielded Calls](../contract-interaction/shielded-calls.md) - API reference for both shielded operations
- [Transparent Calls](../contract-interaction/transparent-calls.md) - Non-encrypted operations
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider required for signed reads
- [Encryption](../provider/encryption.md) - Detailed encryption pipeline
