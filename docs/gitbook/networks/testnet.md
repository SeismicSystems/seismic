---
icon: flask-vial
---

# Testnet

|              |                                                                                  |
| ------------ | -------------------------------------------------------------------------------- |
| Chain ID     | `5124`                                                                           |
| RPC (HTTPS)  | `https://testnet-1.seismictest.net/rpc`                                              |
| RPC (WSS)    | `wss://testnet-1.seismictest.net/ws`                                                 |
| Faucet       | [https://faucet.seismictest.net/](https://faucet.seismictest.net/)               |
| Explorer     | [https://seismic-testnet.socialscan.io/](https://seismic-testnet.socialscan.io/) |

## Quick Test

```bash
curl -X POST https://testnet-1.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

## Chain Definition

```typescript
import { seismicTestnet } from "seismic-viem";

console.log(seismicTestnet.rpcUrls.default.http[0]);
// "https://testnet-1.seismictest.net/rpc"
```

```python
from seismic_web3 import SEISMIC_TESTNET

print(SEISMIC_TESTNET.rpc_url)
# "https://testnet-1.seismictest.net/rpc"
```
