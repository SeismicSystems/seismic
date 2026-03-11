---
icon: flask-vial
---

# Testnet

|              |                                                                                  |
| ------------ | -------------------------------------------------------------------------------- |
| Chain ID     | `5124`                                                                           |
| RPC (HTTPS)  | `https://gcp-1.seismictest.net/rpc`                                              |
| RPC (WSS)    | `wss://gcp-1.seismictest.net/ws`                                                 |
| Faucet       | [https://faucet.seismictest.net/](https://faucet.seismictest.net/)               |
| Explorer     | [https://seismic-testnet.socialscan.io/](https://seismic-testnet.socialscan.io/) |

## Quick Test

```bash
curl -X POST https://gcp-1.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

## Client Setup

```typescript
import { seismicTestnet } from "seismic-viem";

console.log(seismicTestnet.rpcUrls.default.http[0]);
// "https://gcp-1.seismictest.net/rpc"
```

```python
from seismic_web3 import SEISMIC_TESTNET

print(SEISMIC_TESTNET.rpc_url)
# "https://gcp-1.seismictest.net/rpc"
```
