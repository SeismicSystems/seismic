---
icon: flask-vial
---

# Testnet

## RPC URL

| Protocol  | URL                                    |
| --------- | -------------------------------------- |
| HTTPS     | `https://gcp-1.seismictest.net/rpc`    |
| WebSocket | `wss://gcp-1.seismictest.net/ws`       |

## Faucet

Request testnet ETH at [https://faucet.seismictest.net/](https://faucet.seismictest.net/).

## Explorer

Browse transactions and contracts at [https://seismic-testnet.socialscan.io/](https://seismic-testnet.socialscan.io/).

## Chain ID

`5124`

## Client Setup

### seismic-viem (TypeScript)

Import the `seismicTestnet` chain and pass it to your client:

```typescript
import { seismicTestnet } from "seismic-viem";
import { createShieldedWalletClient } from "seismic-viem";
import { http } from "viem";

const client = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: myAccount,
});
```

### seismic-web3 (Python)

Import the `SEISMIC_TESTNET` network and create a client:

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey.from_hex("0x...")
w3 = SEISMIC_TESTNET.wallet_client(pk)
```
