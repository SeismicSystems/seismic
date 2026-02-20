---
icon: dice-d6
---

# Overview

Seismic maintains client libraries for three languages.

### TypeScript — seismic-viem

[seismic-viem](https://www.npmjs.com/package/seismic-viem) composes with [viem](https://viem.sh/) to make calls to an RPC provider.

The documentation for seismic-viem can be found [here](https://client.seismic.systems/).

### Python — seismic-web3

[seismic-web3](https://pypi.org/project/seismic-web3/) composes with [web3.py](https://github.com/ethereum/web3.py) to interact with Seismic nodes from Python. See the [full documentation](/broken/pages/KO8qvfyLVN4qVGHa6mqq).

### Rust — seismic-alloy

[seismic-alloy](https://github.com/SeismicSystems/seismic-alloy) contains a crate called `seismic-alloy-provider`.

* Use `SeismicSignedProvider` to instantiate a client that can sign transactions (e.g. wallet client)
* Use `SeismicUnsignedProvider` for a read-only client (e.g. public)
