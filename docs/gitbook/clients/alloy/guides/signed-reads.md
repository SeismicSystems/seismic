---
description: Encrypted eth_call that proves your identity to the contract
icon: magnifying-glass
---

# Signed Reads

A signed read uses `.seismic().call()` (for functions without shielded params) or `.call()` directly (for functions with shielded params) to build a full `TxSeismic` just like a [shielded write](shielded-write.md), but targets the `eth_call` endpoint instead of broadcasting a transaction. The node decrypts the calldata inside the TEE, executes the call, encrypts the result, and returns it. The provider then decrypts the response automatically.

---

### Why signed reads exist

Any contract function that depends on `msg.sender` needs a signed read. A plain `eth_call` (transparent read) does not attach sender identity, so the contract sees the zero address as the caller.

```rust
// Signed read -- proves your identity to the contract
// msg.sender = your wallet address
let is_odd = contract.isOdd().seismic().call().await?;

// Transparent read -- msg.sender is 0x0
// The contract does not know who is calling
let is_odd = contract.isOdd().call().await?;
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

Signed reads require a `SeismicSignedProvider` because the provider needs a provider-scoped keypair for ECDH key derivation and response decryption.

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer);
let url: reqwest::Url = "https://testnet-1.seismictest.net/rpc".parse()?;

let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http(url)
    .await?;
```

#### 2. Define the contract interface and call

Use the `sol!` macro with `#[sol(rpc)]` to define the interface, then use `.seismic().call()`:

```rust
sol! {
    #[sol(rpc)]
    contract SeismicCounter {
        function isOdd() public view returns (bool);
    }
}

let contract = SeismicCounter::new(contract_address, &provider);

// Encrypted call + automatic response decryption
let is_odd = contract.isOdd().seismic().call().await?;
println!("Is odd: {is_odd}");
```

The return value is already decoded -- no manual ABI decoding needed.

{% hint style="info" %}
Use `.seismic().call()` for signed reads on functions without shielded params, and `.seismic().send()` or direct `.send()` for shielded writes. Both encrypt calldata, but `.call()` also decrypts the response and does not modify on-chain state. Functions with shielded parameters auto-encrypt, so `.seismic()` is not needed for those.
{% endhint %}

{% hint style="warning" %}
Calling `.seismic()` on a function that already has shielded parameters is `#[deprecated]` and produces a compiler warning -- it's redundant because the call builder is already a `ShieldedCallBuilder`. Call `.call()` or `.send()` directly on those functions.
{% endhint %}

---

### Signed read vs. transparent read

| Aspect            | Signed Read (`.seismic().call()`)     | Transparent Read (`.call()`) |
| ----------------- | ------------------------------------- | ---------------------------- |
| `msg.sender`      | Your wallet address                   | Zero address                 |
| Calldata          | Encrypted                             | Plaintext                    |
| Return data       | Encrypted, then decrypted by provider | Plaintext                    |
| Provider required | `SeismicSignedProvider`               | Any provider                 |
| Use case          | Private state, access-controlled data | Public state                 |

---

### Per-call security parameter overrides

You can customize encryption parameters on individual calls:

```rust
let is_odd = contract.isOdd()
    .seismic()
    .expires_at(current_block + 50)        // Custom expiration
    .recent_block_hash(specific_hash)       // Pin to specific chain state
    .call()
    .await?;
```

---

### How the provider handles encryption

Under the hood, a shielded `.call()` (via `.seismic().call()` or auto-encryption) performs the following sequence:

```
1. Fill the transaction (nonce, chain ID, seismic elements, gas)
2. Encrypt calldata with AES-GCM:
   - Key = ECDH(provider secret key, TEE public key)
   - AAD = RLP-encoded TxSeismicMetadata
3. Sign the call request with the wallet
4. Send to node as eth_call
5. Node decrypts calldata inside TEE
6. Node executes the call
7. Node encrypts the response with the same ECDH shared secret
8. Provider receives encrypted response
9. Provider decrypts response with provider secret key
10. Return decoded result to caller
```

---

### Low-level alternative

If you need direct control without the `#[sol(rpc)]` call builder, use `SignedProviderExt` methods:

```rust
// SignedProviderExt is included in the prelude
let is_odd: bool = provider.seismic_call(contract_address, SeismicCounter::isOddCall {}).await?;

// With SecurityParams overrides
let is_odd: bool = provider.seismic_call_with(contract_address, SeismicCounter::isOddCall {},
    SecurityParams::default().expires_at(current_block + 10)
).await?;
```

---

### Complete example

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc)]
    contract SeismicCounter {
        function isOdd() public view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Set up signed provider
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    // 2. Execute signed read
    let contract_address = std::env::var("CONTRACT_ADDRESS")?.parse()?;
    let contract = SeismicCounter::new(contract_address, &provider);

    let is_odd = contract.isOdd().seismic().call().await?;
    println!("Is odd: {is_odd}");

    Ok(())
}
```

---

### When to use transparent reads instead

Not every read needs encryption. Use a transparent read (`.call()`) when:

- The function does **not** depend on `msg.sender`
- The data is already public on-chain
- You want to use an unsigned provider (no private key)

```rust
// Transparent read -- works with any provider, including unsigned
let is_odd = contract.isOdd().call().await?;
```

## See Also

- [Shielded Write](shielded-write.md) - Encrypted state-changing transactions
- [Shielded Calls](../contract-interaction/shielded-calls.md) - API reference for both shielded operations
- [Transparent Calls](../contract-interaction/transparent-calls.md) - Non-encrypted operations
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider required for signed reads
- [Encryption](../provider/encryption.md) - Detailed encryption pipeline
