---
description: Use sfoundry to write and test smart contract code locally before deployment
icon: keyboard-brightness
---

# Development Toolkit

---

### Mappings to foundry

Seismic's development toolkit closely mirrors [foundry](https://getfoundry.sh/) (it's a fork!). The mapping is as follows:

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

### Local node

Use `sanvil` to run a local Seismic node for development and testing:

```bash
sanvil
```

This starts a local node at `http://localhost:8545` with pre-funded accounts, similar to Foundry's `anvil`.
