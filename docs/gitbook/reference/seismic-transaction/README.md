# The Seismic Transaction

<figure><img src="../../.gitbook/assets/image.png" alt=""><figcaption></figcaption></figure>

## Tx Lifecycle

### Key management

- The network manages its keys through the [`seismic-enclave-server`](https://github.com/SeismicSystems/enclave/tree/seismic/crates/enclave-server) crate. Reth (and Summit) can make RPC calls to this server. One such call is to get the network keys
- Enclave can boot in either `--genesis-node` or `--peers <ip>` mode. The former generates its own root key and shares it with peers after they pass validation. The latter loops through its peer IPs until it receives the root key from one of them
- Enclave then derives a few different keys from that root key. Most importantly, it derives the encryption secret key. This is the key used to decrypt Seismic transaction calldata
- When [`seismic-reth`](https://github.com/SeismicSystems/seismic-reth/tree/seismic/bin/seismic-reth) boots, it requests the derived keys from the enclave and keeps them in memory
- We exposed a new RPC method, [`seismic_getTeePublicKey`](../rpc-methods/seismic-get-tee-public-key.md). Its response is the public key of the aforementioned secret key that decrypts Seismic transactions. Calling this endpoint is the first step that a client takes to build a Seismic transaction
- Clients also generate their own secret key, which can be either ephemeral (default) or long-lived, if they prefer to manage this themselves
- Their secret key and the network public key combine to create an AES key, which is used to encrypt calldata

### Encryption & Metadata

- The client will include their encryption public key in the transaction, along with four other parameters to tighten the security of encryption. These are:
  - a 12-byte `encryptionNonce`
  - `recentBlockHash`, which must be within 100 blocks of the latest block
  - an `expiresAtBlock` number, after which this transaction is invalid
  - optionally, a boolean `signedRead`. If set, it only allows the transaction to be used for signed reads. Otherwise, it will be rejected by transaction pool validation
- There is one other field in Seismic transaction metadata: an integer called `messageVersion`. This field is a hack to allow browser extension wallets (e.g. MetaMask) to support the Seismic transaction type. Message version 0, the default, means a standard Seismic transaction as described above. Message version 2 corresponds to a transaction sent as EIP-712 typed data, which browser extension wallets can sign. There is no message version 1, but we reserved this for implementing Seismic transactions via `eth_personalSign`
- These six Seismic-specific fields are called `SeismicElements`: the client encryption public key, the four security fields above, and the message version. They are combined with the EOA address (`sender`), `chainId`, `nonce`, `to` address, and `value`. The full set of 11 fields is referred to as the Seismic transaction metadata (`TxSeismicMetadata`) in [`seismic-alloy-consensus`](https://github.com/SeismicSystems/seismic-alloy/tree/seismic/crates/consensus)
- To encrypt calldata, we use AES with AEAD. The AEAD is an RLP encoding of the `TxSeismicMetadata`. The nonce for this encryption is the same 12-byte encryption nonce as we included in the metadata
- The encrypted calldata is then passed to a Seismic transaction in the data/input field. Seismic transactions work just like Legacy Ethereum transactions, except:
  - they have a tx type of `74` or `0x4a`
  - they have these `SeismicElements` attached as well

### Decryption

- Transactions sent with type `0x4a` but incomplete Seismic elements are rejected from the tx pool, as are transactions not marked as `0x4a` but do contain Seismic elements
- Nodes will decrypt Seismic transactions by:
  - decoding the transaction
  - recovering the signer
  - assembling the metadata
  - encoding it as AEAD via RLP
  - generating the AES key by combining the network's secret key with the transaction's encryption public key
- If transaction decryption fails, the transaction is removed from the transaction pool
- If it succeeds, it passes to the revm transaction environment _as plaintext_
- Calls to get transaction information after they have landed on chain should return input as the encrypted calldata, and not the plaintext
- A note on calldata encoding: s-types are encoded the same way as their public analogues. The consequence is an interesting artifact of the Seismic protocol (and maybe a security concern): Solidity has no clue whether functions are called with Seismic transactions or not. As a result, it's totally allowed to alter shielded state with vanilla Ethereum transactions. It's also allowed to do the reverse, by altering public state with Seismic transactions

### Shielded storage

- Inside Solidity, we've defined two new opcodes: `CLOAD` and `CSTORE`. These are the shielded analogues to `SLOAD` and `SSTORE`. We use `CLOAD`/`CSTORE` when the underlying type is an s-type.
- Importantly, we maintain only one state tree, as you can see in [`seismic-trie`](https://github.com/SeismicSystems/seismic-trie/) (a fork of alloy-trie). We modified the state tree by adding an extra boolean flag, `is_private`, to leaves. This is wrapped inside the struct called `FlaggedStorage`, which replaces `StorageValue` (a plain `U256`). We use this flag to validate `CLOAD`/`CSTORE` calls, and to know whether we can expose this piece of storage, via e.g. `eth_getStorageAt`
- When we see a `CLOAD` opcode inside [`seismic-revm`](https://github.com/SeismicSystems/seismic-revm), we load the storage slot and validate that it either has `is_private` set to `true` or is an uninitialized slot. For `CSTORE`, we write the storage value with `is_private = true`. We throw errors when trying to `CLOAD`/`CSTORE` a `FlaggedStorage` that has `is_private = false` ("Invalid public storage access"). Similarly, we throw errors when trying to `SLOAD`/`SSTORE` on `FlaggedStorage` that has `is_private = true` ("Invalid private storage access"). These errors are thrown inside `seismic-revm`.
- Unlike SSTORE/SLOAD, the gas cost of CLOAD/CSTORE does not change based on the length of the word stored. This is to prevent attackers from deducing information about shielded storage values

### Notes

- Outside of `SeismicElements` and the encrypted calldata, Seismic transactions have the same fields as legacy transactions. In particular, it uses the same gas parameters: `gasPrice` and `gasLimit`
- Seismic transactions cannot be used to deploy bytecode via `Create` calls. Instead, we only allow Seismic transactions to be sent to a specific address. We did this to make metadata validation easier, and we can't see a good reason to deploy encrypted bytecode

## Signed Reads

- The Seismic transaction has a counterpart, which we call "signed reads"
- The motivation for a signed read is: in the EVM, users can make an `eth_call` and set the "from" field to any address they'd like to spoof
- We want to give contract developers the ability to validate contract reads against msg.sender. For example, a user could implement an ERC20 token where only the owner can view their balance
- To solve this, we do two things: (1) we zero out the "from" field when users make a vanilla `eth_call` and (2) we created the "signed read" to allow users to make a call with a specific `from` address
- Signed reads are sent to Seismic's `eth_call` endpoint in the same way we would send a signed Seismic transaction to `eth_sendRawTransaction`. This can be either with a normal raw transaction, or an EIP-712 transaction
- Signed reads must be a valid Seismic transaction (type `0x4a`). They cannot be made with any other transaction type
- The response to a signed read will be encrypted to the encryption public key included in the transaction's Seismic elements. Therefore anyone who manages to intercept the response still cannot decrypt the response to the signed read.
- Users can set the `signed_read` field in SeismicElements to `true`. We encourage this, and it is the default behaviour in our client implementations. The purpose of this is to prevent anyone who has managed to intercept this `eth_call` from sending that same payload to `eth_sendRawTransaction`, which would otherwise trigger a write transaction. When validating a transaction before it hits the pool, we check if the `signed_read` field is set to true; if it is, the transaction is rejected
  - This does not apply the other way around. Meaning, if `signed_read` is `false` (and the user wants to create a transaction), it can still be passed to `eth_call`. We think this validation is unnecessary because:
    - For an attacked to decrypt the response to a signed read, they'd need the user's secret encryption key
    - The account's transaction nonce will increment right after their Seismic transaction is included in a block, after which the read will fail
    - This also allows `eth_estimateGas` to function properly; otherwise it would not pass validation on the `signed_read` field
