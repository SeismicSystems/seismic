# seismic-encrypt

Encrypt a standard viem transaction for the Seismic blockchain. One function, no custom clients — just encrypt, sign, and send via `eth_sendRawTransaction`.

## Install

```bash
npm install seismic-encrypt viem
```

## Quick start

```ts
import { encryptSeismicTx, serializeSeismicTx } from 'seismic-encrypt'
import type { SeismicTxSerializer } from 'seismic-encrypt'
import { createPublicClient, encodeFunctionData, http } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

const RPC_URL = 'https://testnet-1.seismictest.net/rpc'

const account = privateKeyToAccount('0xYourPrivateKey')
const client = createPublicClient({ transport: http(RPC_URL) })

// 1. Build a normal transaction — same as any viem tx
const tx = {
  to: '0xYourContractAddress' as `0x${string}`,
  data: encodeFunctionData({
    abi: myContractAbi,
    functionName: 'transfer',
    args: ['0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266', 100n],
  }),
  nonce: await client.getTransactionCount({ address: account.address }),
  gasPrice: await client.getGasPrice(),
  gas: 100_000n,
  chainId: 5124,
}

// 2. Encrypt it for Seismic
const { seismicTx } = await encryptSeismicTx({
  tx,
  sender: account.address,
  rpcUrl: RPC_URL,
})

// 3. Sign and send — standard viem, nothing special
const signed = await account.signTransaction<SeismicTxSerializer>(seismicTx, {
  serializer: serializeSeismicTx,
})

const hash = await client.sendRawTransaction({
  serializedTransaction: signed,
})

console.log('tx hash:', hash)
```

That's it. The calldata is AES-256-GCM encrypted before it hits the network. The Seismic node decrypts it inside its TEE (Trusted Execution Environment).

## How it works

`encryptSeismicTx` does the following under the hood:

1. Fetches the node's **TEE public key** via `seismic_getTeePublicKey` and the **latest block** (for replay protection) — both in a single parallel RPC round-trip
2. Generates an **ephemeral secp256k1 keypair** (or uses one you provide)
3. Performs **ECDH** between your ephemeral key and the TEE key, then derives an AES-256 key via SHA-256 + HKDF
4. Encrypts your calldata with **AES-256-GCM**, using RLP-encoded transaction metadata as additional authenticated data (AAD) — this binds the ciphertext to this specific transaction so it can't be replayed or tampered with
5. Resolves the public `gasPayment` selector (Auto if omitted), then returns the encrypted transaction fields and a `serialize` function that produces the final `0x4a`-prefixed bytes. The selector is signed metadata, not encrypted calldata or AAD.

## API

### `encryptSeismicTx(params)`

| Parameter              | Type                       | Description                                                        |
| ---------------------- | -------------------------- | ------------------------------------------------------------------ |
| `tx.to`                | `Address`                  | Destination address                                                |
| `tx.data`              | `Hex`                      | Plaintext calldata to encrypt                                      |
| `tx.nonce`             | `number`                   | Sender's transaction nonce                                         |
| `tx.gasPrice`          | `bigint`                   | Gas price                                                          |
| `tx.gas`               | `bigint`                   | Explicit gas limit; preserved without automatic estimation         |
| `tx.gasPayment`        | `GasPayment?`              | Public fee selector; defaults to `{ type: 'auto' }` before signing |
| `tx.chainId`           | `number`                   | Chain ID (`5124` for Seismic testnet, `31337` for local sanvil)    |
| `tx.value`             | `bigint?`                  | ETH value in wei (default `0`)                                     |
| `tx.authorizationList` | `SignedAuthorizationList?` | Optional EIP-7702 authorization list                               |
| `sender`               | `Address`                  | Sender address (must match the signer)                             |
| `rpcUrl`               | `string`                   | Seismic node RPC URL                                               |
| `encryptionPrivateKey` | `Hex?`                     | Your own ephemeral key. One is generated per call if omitted.      |
| `blocksWindow`         | `bigint?`                  | Blocks until the tx expires (default `100`)                        |

Returns a `Promise<EncryptSeismicTxResult>`:

| Field                  | Type                       | Description                                                     |
| ---------------------- | -------------------------- | --------------------------------------------------------------- |
| `seismicTx`            | `object`                   | All transaction fields with encrypted `data`, ready to sign     |
| `serialize`            | `(sig?: Signature) => Hex` | Omit for unsigned bytes; pass a Viem signature for signed bytes |
| `unsignedSerializedTx` | `Hex`                      | The unsigned serialized bytes (for inspection/debugging)        |

### Gas payment

The exported `GasPayment` type accepts:

| Choice                              | Meaning                                                                     |
| ----------------------------------- | --------------------------------------------------------------------------- |
| Omitted or `{ type: 'auto' }`       | Native funds first, then eligible active registry tokens in insertion order |
| `{ type: 'native' }`                | Native funds only; no token fallback                                        |
| `{ type: 'token', token: address }` | Exactly this registered token; no fallback                                  |

Pass the selector inside `tx`, alongside `gas` and `gasPrice`:

```ts
import type { GasPayment } from 'seismic-encrypt'
import type { Address } from 'viem'

const gasTokenAddress: Address = '0xYourGasTokenAddress'
const payment: GasPayment = { type: 'token', token: gasTokenAddress }
const { seismicTx, serialize } = await encryptSeismicTx({
  tx: { ...tx, gasPayment: payment },
  sender: account.address,
  rpcUrl: RPC_URL,
})
// seismicTx.gasPayment is always resolved, including when the option is omitted.
```

Replace the placeholder with a valid nonzero 20-byte registered token address.
The token need not be the called contract; use the registered proxy address for
upgradeable tokens. Unknown tags, zero/malformed token addresses, extra properties,
and token properties on Auto/Native are rejected before RPC requests.

`value` and `gasPrice` remain denominated in native wei. Selection does not mint
or fund tokens. This standalone helper preserves your explicit `gas` limit; it
does not estimate gas or apply seismic-viem's automatic final-ciphertext minimum.
Choose a limit sufficient for encrypted-input admission and execution.

### `serializeSeismicTx(tx, signature?)`

Lower-level serializer if you want to build the transaction yourself. Takes a full seismic transaction object and an optional signature, returns `0x4a`-prefixed RLP-encoded bytes. Its optional `gasPayment` also resolves to Auto **before signing**; the wire field is never omitted.

The exported `SeismicSerializableTransaction` type describes complete standalone fields; low-level callers may omit `type`. `SeismicTxSerializer` is the Viem-compatible serializer type used in the quick start. Ordinary Ethereum or incomplete transaction inputs are rejected rather than encoded as Seismic transactions.

Signatures accept Viem's `Signature`: `r`/`s` with either `v` (`0`, `1`, `27`, or `28`) or `yParity` (`0` or `1`). When both are present, `yParity` takes precedence. `serialize()` without a signature returns the same bytes as `unsignedSerializedTx`, supporting Viem's unsigned signing-hash pass.

The unsigned field order is:

```text
[chainId, nonce, gasPrice, gas, gasPayment, to, value,
 encryptionPubkey, encryptionNonce, messageVersion, recentBlockHash,
 expiresAtBlock, signedRead, data, authorizationList]
```

`gasPayment` is a nested `[kind, token]` RLP list: Auto is `[0, empty]`, Native is
`[1, empty]`, and Token is `[2, address bytes]`. Signature fields follow the unsigned
fields. Integers, including the encryption nonce and authorization signature
scalars, use canonical RLP encoding; encryption AAD retains its existing encoding.

This format requires a node supporting the mandatory gas-payment selector. Old
already-signed Seismic transactions are not made compatible by adding a default;
there is no legacy wire mode or new public raw decoder.

### `SEISMIC_TX_TYPE`

The Seismic transaction type constant: `0x4a` (74).

## Sending ETH (no calldata)

For simple ETH transfers, pass `0x` as `data` — it will be left unencrypted since there's nothing to encrypt:

```ts
const { seismicTx, serialize } = await encryptSeismicTx({
  tx: {
    to: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
    data: '0x',
    value: 1_000_000_000_000_000_000n, // 1 ETH
    nonce: 0,
    gasPrice: 1_000_000_000n,
    gas: 21_000n,
    chainId: 5124,
  },
  sender: account.address,
  rpcUrl: 'https://testnet-1.seismictest.net/rpc',
})
```

## Using with local sanvil

```ts
const { seismicTx, serialize } = await encryptSeismicTx({
  tx: { ...tx, chainId: 31337 },
  sender: account.address,
  rpcUrl: 'http://127.0.0.1:8545',
})
```

## Package consumers and verification

ESM imports and CommonJS `require('seismic-encrypt')` expose the same helpers. The CommonJS build uses a `.cjs` extension so Node does not interpret it as ESM under this package's `type: module`.

From `clients/ts`, run:

```bash
bun run encrypt:test
bun run encrypt:consumer:test
```

The consumer check builds the package, verifies both public module entry points against the shared Rust vectors, exercises encryption/signing with mocked RPC, and compiles public declaration examples without workspace source aliases. It does not connect to a node or broadcast transactions.

## Relationship to seismic-viem

`seismic-viem` provides a full-featured `ShieldedWalletClient` with contract helpers, signed reads, SRC20 support, and more. This package extracts just the encryption and serialization into a single function with no custom client setup. Use this when you want to integrate Seismic encryption into an existing viem workflow without pulling in the full SDK.
