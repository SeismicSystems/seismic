# Gas-payment wire/signing vectors

`gas-payment.json` is a frozen, Rust-generated fixture shared by the TS and Python unit suites. It contains 36 cases: Auto/Native/Token × raw/EIP-712 × zero/leading-zero/max U96 encryption nonce × call/signed-read or create/authorization-list. Input bytes are opaque codec fixtures, not real encrypted application calls. Authorization signatures are synthetic; this is encoding/signing coverage, not execution or broadcast eligibility.

The expected unsigned/signed bytes, signing hashes, signatures, and transaction hashes come from current `seismic-alloy-consensus::TxSeismic` and secp256k1, not from either SDK. Unsigned fixture bytes include `0x4a`; Python's `serialize_unsigned` returns bare RLP and its signing hash adds the prefix. Signed fixture bytes and both SDKs' signed serialization include `0x4a`.

## Provenance

Source checkout HEAD was `19bade73ce734817bac4cdd129487ad9363c3c50`, with uncommitted mandatory-selector changes. The source hashes below, rather than HEAD alone, identify the implementation used:

| File | SHA-256 |
| --- | --- |
| `seismic-alloy/crates/consensus/src/transaction/seismic.rs` | `24158555400d2a5c5bb502256baf8d996cda5dbf605cb188c42ff5b9d2852af9` |
| `seismic-alloy/crates/consensus/src/transaction/gas_payment.rs` | `6c196736ffbd734ec87ead6645c9cf0e43259617dc2928e9dccf5c5243126bee` |
| `gas-payment-generator.rs` | `3366c78e4d5085f24b3651a7d282eec5e4d1c5ba9567a0fa7b9cbaf2fc498940` |
| `gas-payment.json` | `18e369680d0fe65733605200519237d02f9dcd6a9d42ca6a59ca992a9916ae77` |

The generator also verifies Rust signed-RLP round trips, rejects missing raw/JSON/typed selectors and malformed typed selectors, and checks typed-data decoding. Typed data carries only the authorization-list **hash**, not its tuples: decoding alone does not reconstruct a nonempty authorization list. The fixture checks the hash commitment and raw tuples, not live typed-envelope authorization transport.

## Regeneration

Do not regenerate automatically during SDK tests. Use an isolated Cargo package, with `gas-payment-generator.rs` copied to `src/main.rs` and `generator.Cargo.toml` copied to `Cargo.toml`. Replace `@SEISMIC_ALLOY_ROOT@` with the absolute path of the reviewed seismic-alloy checkout. The manifest reproduces the dependency versions and core/enclave patches used here; it is a local test tool, not a release dependency pin.

```sh
CARGO_BUILD_JOBS=1 cargo run --manifest-path /tmp/gas-sdk-vectors/Cargo.toml --offline --quiet > /tmp/gas-payment.json
```

Review source hashes, compare all expected outputs, then deliberately replace the frozen JSON and update provenance. Never bless SDK output as the Rust authority.

## Verification

From `clients/ts`: `bun run viem:unit:test`. From `clients/py`: `uv run pytest tests/test_gas_payment.py`.

Tests compare canonical bytes, hashes, and signatures; tampered selectors recover a different signer. Signature JSON hex is quantity-form in Rust and fixed-width in viem, so compare numeric values/padded signatures rather than requiring their original hex spelling to match. Encryption/AAD retain the fixed-width nonce; only transaction RLP encodes its minimal integer.
