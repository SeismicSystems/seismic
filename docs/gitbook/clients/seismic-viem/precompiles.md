---
description: Seismic precompiled contracts for cryptographic operations
icon: microchip
---

# Precompiles

Seismic extends the EVM with precompiled contracts for common cryptographic operations. These are available at fixed addresses and can be called from any client with a `.call()` method -- either a `ShieldedPublicClient` or a `ShieldedWalletClient`.

## Import

```typescript
import {
  rng,
  ecdh,
  aesGcmEncrypt,
  aesGcmDecrypt,
  hdfk,
  secp256k1Sig,
} from "seismic-viem";
```

## Overview

| Precompile      | Address     | Description              | Input                           | Output                |
| --------------- | ----------- | ------------------------ | ------------------------------- | --------------------- |
| `rng`           | `0x...0064` | Random number generation | `numBytes`, `pers?`             | `bigint`              |
| `ecdh`          | `0x...0065` | ECDH key exchange        | `sk`, `pk`                      | `Hex` (shared secret) |
| `aesGcmEncrypt` | `0x...0066` | AES-GCM encryption       | `aesKey`, `nonce`, `plaintext`  | `Hex` (ciphertext)    |
| `aesGcmDecrypt` | `0x...0067` | AES-GCM decryption       | `aesKey`, `nonce`, `ciphertext` | `string` (plaintext)  |
| `hdfk`          | varies      | HKDF key derivation      | `ikm`                           | `Hex` (derived key)   |
| `secp256k1Sig`  | varies      | secp256k1 signing        | `sk`, `message`                 | `Signature`           |

{% hint style="info" %}
Precompile calls execute within the TEE, ensuring cryptographic operations are performed in a secure environment. The inputs and outputs are transmitted over the encrypted channel established during client construction.
{% endhint %}

## RNG -- Random Number Generation

Generates cryptographically secure random numbers using the TEE's CSPRNG.

### Standalone Function

```typescript
import { rng } from "seismic-viem";

const randomValue = await rng(client, {
  numBytes: 32,
});

console.log("Random value:", randomValue);
```

### Parameters

| Parameter  | Type               | Required | Description                                |
| ---------- | ------------------ | -------- | ------------------------------------------ |
| `numBytes` | `bigint \| number` | Yes      | Number of random bytes to generate (1--32) |
| `pers`     | `Hex \| ByteArray` | No       | Personalization string to seed the CSPRNG  |

### Returns

`bigint` -- the generated random value.

### Example

```typescript
import { createShieldedPublicClient } from "seismic-viem";
import { rng } from "seismic-viem";
import { seismicTestnet } from "seismic-viem/chains";
import { http } from "viem";

const client = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
});

// Generate 32 random bytes
const randomValue = await rng(client, { numBytes: 32 });
console.log("Random 256-bit value:", randomValue);

// Generate 16 random bytes with a personalization string
const seededValue = await rng(client, {
  numBytes: 16,
  pers: "0x6d79617070",
});
console.log("Seeded random value:", seededValue);
```

---

## ECDH -- Elliptic Curve Diffie-Hellman

Performs an ECDH key exchange inside the TEE and returns the shared secret.

### Standalone Function

```typescript
import { ecdh } from "seismic-viem";

const sharedSecret = await ecdh(client, {
  sk: "0x...", // 32-byte secret key
  pk: "0x...", // 33-byte compressed public key
});

console.log("Shared secret:", sharedSecret);
```

### Parameters

| Parameter | Type  | Required | Description                             |
| --------- | ----- | -------- | --------------------------------------- |
| `sk`      | `Hex` | Yes      | 32-byte secp256k1 secret key            |
| `pk`      | `Hex` | Yes      | 33-byte compressed secp256k1 public key |

### Returns

`Hex` -- 32-byte shared secret.

### Example

```typescript
import { ecdh } from "seismic-viem";

const sharedSecret = await ecdh(client, {
  sk: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
  pk: "0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
});

console.log("Shared secret:", sharedSecret);
```

---

## AES-GCM Encrypt / Decrypt

Performs AES-GCM encryption and decryption inside the TEE.

### Encrypt

```typescript
import { aesGcmEncrypt } from "seismic-viem";

const ciphertext = await aesGcmEncrypt(client, {
  aesKey: "0x...", // 32-byte AES key
  nonce: 1, // numeric nonce
  plaintext: "hello world",
});

console.log("Ciphertext:", ciphertext);
```

#### Encrypt Parameters

| Parameter   | Type     | Required | Description                 |
| ----------- | -------- | -------- | --------------------------- |
| `aesKey`    | `Hex`    | Yes      | 32-byte AES-256 key         |
| `nonce`     | `number` | Yes      | Numeric nonce for AES-GCM   |
| `plaintext` | `string` | Yes      | Plaintext string to encrypt |

#### Encrypt Returns

`Hex` -- the AES-GCM ciphertext.

### Decrypt

```typescript
import { aesGcmDecrypt } from "seismic-viem";

const plaintext = await aesGcmDecrypt(client, {
  aesKey: "0x...", // same 32-byte AES key
  nonce: 1, // same nonce used for encryption
  ciphertext: "0x...",
});

console.log("Plaintext:", plaintext);
```

#### Decrypt Parameters

| Parameter    | Type     | Required | Description                  |
| ------------ | -------- | -------- | ---------------------------- |
| `aesKey`     | `Hex`    | Yes      | 32-byte AES-256 key          |
| `nonce`      | `number` | Yes      | Nonce used during encryption |
| `ciphertext` | `Hex`    | Yes      | Ciphertext to decrypt        |

#### Decrypt Returns

`string` -- the decrypted plaintext.

### Full Example

```typescript
import { aesGcmEncrypt, aesGcmDecrypt, rng } from "seismic-viem";

// Generate a random AES key using the RNG precompile
const aesKeyRaw = await rng(client, { numBytes: 32 });
const aesKey = `0x${aesKeyRaw.toString(16).padStart(64, "0")}` as const;

// Encrypt
const ciphertext = await aesGcmEncrypt(client, {
  aesKey,
  nonce: 1,
  plaintext: "secret message",
});

// Decrypt
const plaintext = await aesGcmDecrypt(client, {
  aesKey,
  nonce: 1,
  ciphertext,
});

console.log("Decrypted:", plaintext); // "secret message"
```

---

## HKDF -- Key Derivation

Derives a key from input key material using HKDF inside the TEE.

### Standalone Function

```typescript
import { hdfk } from "seismic-viem";

const derivedKey = await hdfk(client, "0x..."); // input key material

console.log("Derived key:", derivedKey);
```

### Parameters

| Parameter | Type  | Required | Description        |
| --------- | ----- | -------- | ------------------ |
| `ikm`     | `Hex` | Yes      | Input key material |

### Returns

`Hex` -- the derived key.

### Example

```typescript
import { hdfk } from "seismic-viem";

const inputKeyMaterial = "0xdeadbeef";
const derivedKey = await hdfk(client, inputKeyMaterial);
console.log("Derived key:", derivedKey);
```

---

## secp256k1 Signature

Generates a secp256k1 signature inside the TEE.

### Standalone Function

```typescript
import { secp256k1Sig } from "seismic-viem";

const signature = await secp256k1Sig(client, {
  sk: "0x...", // 32-byte secret key
  message: "hello", // message to sign
});

console.log("Signature:", signature);
```

### Parameters

| Parameter | Type     | Required | Description                  |
| --------- | -------- | -------- | ---------------------------- |
| `sk`      | `Hex`    | Yes      | 32-byte secp256k1 secret key |
| `message` | `string` | Yes      | Message to sign              |

### Returns

`Signature` -- a viem `Signature` object with `r`, `s`, and `v` components.

### Example

```typescript
import { secp256k1Sig } from "seismic-viem";

const signature = await secp256k1Sig(client, {
  sk: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
  message: "sign this message",
});

console.log("r:", signature.r);
console.log("s:", signature.s);
console.log("v:", signature.v);
```

---

## Using Precompiles via Client

All precompiles are available as methods directly on `ShieldedPublicClient` and `ShieldedWalletClient`, so you can call them without importing the standalone functions.

| Client Method                       | Standalone Function             | Description              |
| ----------------------------------- | ------------------------------- | ------------------------ |
| `client.rng(params)`                | `rng(client, params)`           | Random number generation |
| `client.ecdh(params)`               | `ecdh(client, params)`          | ECDH key exchange        |
| `client.aesGcmEncryption(params)`   | `aesGcmEncrypt(client, params)` | AES-GCM encryption       |
| `client.aesGcmDecryption(params)`   | `aesGcmDecrypt(client, params)` | AES-GCM decryption       |
| `client.hdfk(ikm)`                  | `hdfk(client, ikm)`             | HKDF key derivation      |
| `client.secp256k1Signature(params)` | `secp256k1Sig(client, params)`  | secp256k1 signing        |

### Example

```typescript
import { createShieldedPublicClient } from "seismic-viem";
import { seismicTestnet } from "seismic-viem/chains";
import { http } from "viem";

const client = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
});

// Call precompiles as client methods
const randomValue = await client.rng({ numBytes: 32 });

const sharedSecret = await client.ecdh({
  sk: "0x...",
  pk: "0x...",
});

const ciphertext = await client.aesGcmEncryption({
  aesKey: "0x...",
  nonce: 1,
  plaintext: "hello",
});

const derivedKey = await client.hdfk("0x...");
```

## Custom Precompile Pattern

Each precompile is defined as a `Precompile<P, R>` object with a standard interface. You can use the `callPrecompile` utility to invoke any precompile directly.

### `Precompile<P, R>` Type

| Property               | Type                  | Description                                   |
| ---------------------- | --------------------- | --------------------------------------------- |
| `address`              | `Hex`                 | Fixed address of the precompile contract      |
| `gasCost(args)`        | `(args: P) => bigint` | Computes the gas cost for the given arguments |
| `encodeParams(args)`   | `(args: P) => Hex`    | ABI-encodes the arguments for the call        |
| `decodeResult(result)` | `(result: Hex) => R`  | Decodes the raw call result                   |

### Using `callPrecompile`

```typescript
import { callPrecompile, rngPrecompile } from "seismic-viem";

// Call the RNG precompile directly using the precompile object
const result = await callPrecompile({
  client,
  precompile: rngPrecompile,
  args: { numBytes: 32n },
});

console.log("Random value:", result);
```

### Available Precompile Objects

| Export                    | Type                                         |
| ------------------------- | -------------------------------------------- |
| `rngPrecompile`           | `Precompile<RngParams, bigint>`              |
| `ecdhPrecompile`          | `Precompile<EcdhParams, Hex>`                |
| `aesGcmEncryptPrecompile` | `Precompile<AesGcmEncryptionParams, Hex>`    |
| `aesGcmDecryptPrecompile` | `Precompile<AesGcmDecryptionParams, string>` |
| `hdfkPrecompile`          | `Precompile<Hex, Hex>`                       |
| `secp256k1SigPrecompile`  | `Precompile<Secp256K1SigParams, Signature>`  |

The `CallClient` type accepts any client with a `.call()` method, so both `ShieldedPublicClient` and `ShieldedWalletClient` work with `callPrecompile`.

## See Also

- [Shielded Public Client](shielded-public-client.md) -- Read-only client with precompile methods
- [Shielded Wallet Client](shielded-wallet-client.md) -- Full-featured client with precompile methods
- [Encryption](encryption.md) -- Client-side ECDH key exchange and AES-GCM encryption
- [Chains](chains.md) -- Chain configurations for Seismic networks
