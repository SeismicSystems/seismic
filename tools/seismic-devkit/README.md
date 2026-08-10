# 🛡️ Seismic DevKit & Web Studio

An interactive development studio and utility suite for **Seismic** — the privacy-enabled EVM blockchain supporting shielded state types (`suint`, `sbool`, `saddress`) and hardware-level TEE execution.

---

## 🌟 Features

- 🛡️ **Shielded State Simulator**: Encrypt and decrypt private state variables and test transaction payload generation for Seismic TEE enclaves.
- 🚰 **Testnet Faucet & Wallet Manager**: Generate new Seismic devnet keypairs and testnet drip simulations on Chain ID `5124`.
- 🌐 **Interactive Web Studio**: Real-time block tracker, wallet generator, and shielded payload inspector (`http://localhost:3405`).
- ⌨️ **Universal CLI (`seismic-cli`)**: Terminal utility for network status, wallet generation, faucet requests, and encryption.

---

## 🚀 Quickstart

```bash
# Launch Developer Web Studio
npm start
# Visit http://localhost:3405

# Or run via CLI
node bin/seismic-cli.js status
node bin/seismic-cli.js wallet
node bin/seismic-cli.js encrypt suint256 1000000
```
