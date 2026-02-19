---
icon: bring-forward
---

# Clients

### Typescript

Seismic maintains [seismic-viem](https://www.npmjs.com/package/seismic-viem), which composes with [viem](https://viem.sh/) to make calls to an RPC provider

The documentation for seismic-viem can be found [here](https://seismic-docs.netlify.app/seismic-react/globals)

### Rust

Seismic maintains [seismic-alloy](https://github.com/SeismicSystems/seismic-alloy), which contains a crate called `seismic-alloy-provider`

* Use `SeismicSignedProvider` to instantiate a client that can sign transactions (e.g. wallet client)
* Use `SeismicUnsignedProvider` for a read-only client (e.g. public)
