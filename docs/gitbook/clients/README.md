---
icon: bring-forward
---

# Clients

Seismic maintains client libraries for three languages.

## TypeScript

### seismic-viem

[seismic-viem](https://www.npmjs.com/package/seismic-viem) composes with [viem](https://viem.sh/) to add Seismic transaction support, encrypted calldata, and signed reads. See the [full documentation](typescript/viem/README.md).

### seismic-react

[seismic-react](https://www.npmjs.com/package/seismic-react) composes with [wagmi](https://wagmi.sh/) to provide React hooks for shielded reads, writes, and wallet management. See the [full documentation](typescript/react/README.md).

## Python — seismic-web3

[seismic-web3](https://pypi.org/project/seismic-web3/) composes with [web3.py](https://github.com/ethereum/web3.py) to interact with Seismic nodes from Python. See the [full documentation](python/README.md).

## Rust — seismic-alloy

[seismic-alloy](https://github.com/SeismicSystems/seismic-alloy) composes with [alloy](https://github.com/alloy-rs/alloy) to provide Seismic transaction types and encryption-aware providers. See the [full documentation](alloy/README.md).

{% hint style="warning" %}
The docs for all three of these libraries are pure AI slop. The Python docs have been manually reviewed and cleaned up; we're auditing the TypeScript and Rust docs shortly. Refer to the source code if anything seems off.
{% endhint %}
