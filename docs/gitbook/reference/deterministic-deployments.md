---
icon: layer-group
---

# Deterministic deployments

Seismic provides common infrastructure contracts at addresses shared with
other EVM networks. These contracts are deployed after the chain starts using
ordinary transactions whose senders, nonces, and creation code produce the
same addresses on every compatible network.

They are not [genesis predeploys](predeploys.md). A deterministic contract is
unavailable until its deployment transaction has been included, and its
deployment produces normal transaction history and a receipt.

{% hint style="info" %}
Availability can differ between Seismic networks. Use `eth_getCode` to confirm
that a contract is installed before depending on it.
{% endhint %}

## Addresses

| Contract | Address |
| --- | --- |
| Create2 — Nick's method | `0x4e59b44847b379578588920cA78FbF26c0B4956C` |
| ERC-1820 Registry | `0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24` |
| SingletonFactory (ERC-2470) | `0xce0042B868300000d44A59004Da54A005ffdcf9f` |
| Multicall3 | `0xcA11bde05977b3631167028862bE2a173976CA11` |
| Deterministic Deployment Proxy | `0x7A0D94F55792C434d74a40883C6ed8545E406D12` |
| Inefficient ImmutableCreate2Factory | `0xcfA3A7637547094fF06246817a35B8333C315196` |
| ImmutableCreate2Factory | `0x0000000000FFe8B47B3e2130213B802212439497` |
| Permit2 | `0x000000000022D473030F116dDEE9F6B43aC78BA3` |
| CreateX | `0xba5Ed099633D3B313e4D5F7bdc1305d3c28ba5Ed` |

These contracts provide shared deployment factories, interface discovery,
batched reads and calls, and token approval infrastructure. Keeping their
canonical addresses makes applications and deployment tooling portable across
EVM networks.

{% hint style="warning" %}
The contracts with “Proxy” in their names are deterministic **deployment**
utilities. They are not upgradeability proxies for Seismic's genesis
contracts.
{% endhint %}

## Verify a deployment

Check an address before using it:

```bash
cast code \
  0xcA11bde05977b3631167028862bE2a173976CA11 \
  --rpc-url "$SEISMIC_RPC_URL"
```

An installed contract returns non-empty bytecode. `0x` means no contract is
currently deployed at that address on the selected network.
