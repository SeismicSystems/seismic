# Seismic viem actions

This is a package to extend viem clients for use with the Seismic blockchain

## Docs

The docs are hosted [here](https://docs.seismic.systems/clients/typescript/viem)

## Contributor docs

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the package layout, the transaction flow diagram, and the `actions/` ↔ viem-decorator relationship.


## Event Decryption Guide

Seismic features a shielded virtual machine where event data can be encrypted to maintain privacy. To read and decrypt these events on the client side using `seismic-viem`, you must use our specialized cryptographic utilities.

### Prerequisites

Ensure you have `@seismic-systems/seismic-viem` and `viem` installed in your project.

```bash
npm install @seismic-systems/seismic-viem viem
```

### Step-by-Step Decryption

To decrypt an event payload, you need the encrypted data from the logs and your user's ephemeral private key or the system's viewing key depending on the encryption context.

#### 1. Initialize the Seismic Client

```typescript
import { createClient, http } from 'viem';
import { seismicActions } from '@seismic-systems/seismic-viem';

const client = createClient({
  chain: seismic, // Your configured Seismic chain network
  transport: http()
}).extend(seismicActions());
```

#### 2. Fetch and Decrypt the Encrypted Log

Use the `decryptLog` utility provided by the Seismic extensions to parse and reveal the plaintext event fields.

```typescript
import { decryptLog } from '@seismic-systems/seismic-viem';

async function processEncryptedEvent(txHash: `0x${string}`) {
  // 1. Fetch transaction receipt to get logs
  const receipt = await client.getTransactionReceipt({ hash: txHash });
  
  // 2. Map through logs and decrypt target encrypted events
  for (const log of receipt.logs) {
    try {
      const decryptedLog = await client.decryptLog({
        log,
        // The user private key authorized to view this data (passed via TEE context)
        privateKey: process.env.USER_PRIVATE_KEY as `0x${string}` 
      });
      
      console.log("Decrypted Event Data:", decryptedLog);
    } catch (error) {
      console.error("Failed to decrypt log or log is not encrypted:", error);
    }
  }
}
```

### Common Pitfalls

* **Unauthorized Keys:** Decryption will fail with a `Mac Mismatch` or `Decryption Failed` error if the provided private key does not own or have viewing permissions for the targeted shielded state.
* **Non-Shielded Events:** Standard EVM events emitted by public smart contracts do not require decryption. Calling `decryptLog` on them will safely return the original log unmodified or throw an invalid format error.
