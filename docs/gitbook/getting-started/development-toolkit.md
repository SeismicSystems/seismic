---
description: Use sfoundry to write and test smart contract code locally before deployment
icon: keyboard-brightness
---

# Development Toolkit

---

### Mappings to foundry

Seismic's development toolkit closely mirrors [Foundry](https://getfoundry.sh/) (it's a [fork](https://github.com/SeismicSystems/seismic-foundry)!). The mapping is as follows:

```
// foundry tool -> seismic version of foundry tool
forge -> sforge
anvil -> sanvil
cast -> scast
```

You should use the righthand version of all tools when developing for Seismic to get expected behavior. Our documentation assumes familiarity with foundry.

---

### Quick actions

Substitute `sforge` for `forge` to execute against Seismic's superset of the EVM. More on this in the next section.

{% tabs %}
{% tab title="Initialize sforge project" %}

```bash
# Initializes a project called `Counter`
sforge init Counter
```

{% endtab %}

{% tab title="Run tests" %}

```bash
# Run tests for the Counter contract
sforge test
```

{% endtab %}

{% tab title="Deploy contract" %}

```bash
# Use sforge scripts to deploy the Counter contract
# Running `sanvil` @ http://localhost:8545
# Set the private key in the env
export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" # Address - 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
# Run the script and broadcast the deploy transaction
sforge script script/Counter.s.sol --rpc-url http://127.0.0.1:8545 --broadcast --private-key $PRIVATE_KEY
```

{% endtab %}
{% endtabs %}

---

### Gas payment with scast

Encrypted `scast send --seismic` and `scast call --seismic` support
`--gas-payment auto|native|token:ADDRESS`:

* Omitted or `auto`: delegate fee-asset selection to the network.
* `native`: require native payment, without fallback.
* `token:ADDRESS`: require that nonzero token address, without fallback. The token
  must be eligible under the target network's gas-token registry.

```bash
scast send "$CONTRACT" 'setNumber(uint256)' 42 --seismic \
  --gas-payment "token:$GAS_TOKEN" --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY"
scast call "$CONTRACT" 'number()(uint256)' --seismic \
  --gas-payment native --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY"
```

The preference is authenticated by the transaction signature. It does not change
calldata encryption or AAD. Signed gas estimation preserves the same preference;
explicit gas limits are preserved. Native/Token is rejected on ordinary Ethereum
routes and local `call --trace` execution rather than silently ignored.

These tools use the mandatory Seismic transaction format, with the selector after
gas limit. They do not upgrade old signed bytes. `scast decode-transaction` can
inspect new-format writes offline, but rejects signed-read bytes and unknown types;
it does not decrypt calldata. `scast tx HASH gasPayment` displays the signed
preference. An Auto response does **not** identify the asset ultimately charged.

Use a compatible Seismic Reth network for registry-backed token gas. Full `sanvil`
token-gas execution parity is not part of this tooling migration.

---

### Local node

Use `sanvil` to run a local Seismic node for development and testing:

```bash
sanvil
```

This starts a local node at `http://localhost:8545` with pre-funded accounts, similar to Foundry's `anvil`.
