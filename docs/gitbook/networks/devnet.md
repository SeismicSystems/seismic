---
description: Try out the developer testnet
icon: down-right
metaLinks:
  alternates:
    - https://app.gitbook.com/s/hkB2uNxma1rxIgBfHgAT/appendix/devnet
---

# Deploy an Encrypted Contract on our Devnet

<figure><img src="../../.gitbook/assets/Screenshot 2025-03-25 at 3.27.58 PM.png" alt=""><figcaption></figcaption></figure>

Welcome! This walkthrough is quick. It only requires a minute of actual attention, while the rest is waiting. **If you run into any issues, please check if it's one of the 10 common errors resolved in the** [**FAQ**](devnet.md#faq) **section.** You can also hop in [our discord](https://discord.com/invite/seismic) and ask questions in the `#devnet` channel.

If you end up deploying your own custom contract, please send the github link to [@lyronc](https://t.me/lyronc) on TG! Also note, this **is not an incentivized testnet**.

Works on Mac, Linux, and Windows via WSL (see [FAQ](devnet.md#faq)).

## Deploy an encrypted contract

#### 1. Install Rust

```bash
curl https://sh.rustup.rs -sSf | sh  # choose default, just press enter
. "$HOME/.cargo/env"
```

#### 2. Install jq

For Mac. See instructions for your machine [here](https://jqlang.org/download/). Only step that isn't OS agnostic.

```bash
brew install jq
```

#### 3. Install sfoundryup

```bash
curl -L \
     -H "Accept: application/vnd.github.v3.raw" \
     "https://api.github.com/repos/SeismicSystems/seismic-foundry/contents/sfoundryup/install?ref=seismic" | bash
source ~/.bashrc
```

#### 4. Run sfoundryup

```bash
sfoundryup  # takes between 5m to 60m, and stalling for a while at 98% normal
```

#### 5. Clone repository

```bash
git clone --recurse-submodules https://github.com/SeismicSystems/try-devnet.git
cd try-devnet/packages/contract/
```

#### 6. Deploy contract

```bash
bash script/deploy.sh
```

## Interact with an encrypted contract

#### 1. Install Bun

```bash
curl -fsSL https://bun.sh/install | bash
```

#### 2. Install node dependencies

```bash
cd try-devnet/packages/cli/
bun install
```

#### 3. Send transactions

```bash
bash script/transact.sh
```

## FAQ

<details>

<summary>What if I'm on Windows?</summary>

We recommend using [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) to run commands as if you were on a Linux machine. Run

```bash
wsl --install
```

Now restart your computer. After booting back up, you should be able to run the below command and follow the rest of the steps like normal

```bash
wsl
```

</details>

<details>

<summary>I'm stuck at <code>1108/1112</code> when running <code>sfoundryup</code> .</summary>

Some machines take up to an hour to do this step. If it takes longer, ask a question in [our discord's](https://discord.com/invite/seismic) `#devnet` channel.

</details>

<details>

<summary>I'm getting <code>Command failed: cargo build --bins --release</code>.</summary>

Means your machine doesn't have cargo. If you're on Linux, run

```bash
sudo apt update && sudo apt install -y build-essential
sudo apt install cargo -y
```

</details>

<details>

<summary>I'm getting <code>jq (command not found)</code>.</summary>

Means [step #2](devnet.md#id-2.-install-jq) didn't work. If you're on Linux, run

```bash
sudo apt-get install jq
```

</details>

<details>

<summary>I'm getting <code>Address not funded. Please check if your faucet transaction went...</code></summary>

Means your wallet has no testnet ETH. Please go to the [faucet](https://faucet-2.seismicdev.net/), enter the address the script gave you, and wait for the green confirmation.

![](<../../.gitbook/assets/Screenshot 2025-03-25 at 4.01.46 PM (1).png>)

</details>

<details>

<summary>I'm getting <code>Command 'brew' not found</code>.</summary>

Means your machine doesn't have the [Homebrew](https://brew.sh/) package manager. Run

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

</details>

<details>

<summary>I'm getting <code>linker 'cc' not found</code>.</summary>

You can resolve by running

```bash
sudo apt update && sudo apt install -y build-essential
sudo apt install cargo -y
```

</details>

<details>

<summary>I'm getting <code>command not found: sfoundryup</code> .</summary>

If this comes up even after you complete [step #3](devnet.md#id-3.-install-sfoundryup) successfully, restart your terminal. Should be able to run it after.

</details>

<details>

<summary>I'm getting <code>info: aborting installation</code> .</summary>

Means you aren't selecting an option for your Rust installation. Run the `curl` command again, and press Enter.

![](<../../.gitbook/assets/Screenshot 2025-03-25 at 4.55.36 PM.png>)

</details>

<details>

<summary>I'm getting <code>Command: 'bun' not found</code>.</summary>

You need to add bun to your PATH. You can either do this temporarily in your current terminal via the below command (you'll have to do it for every new window):

```bash
export PATH="/home/$(whoami)/.bun/bin:$PATH"
```

Or set it properly, by opening up your `~/.bashrc` and adding

```bash
PATH="/home/$(whoami)/.bun/bin:$PATH"
```

</details>

## View official links

<table><thead><tr><th width="158.1484375">Item</th><th width="589.20703125">Value</th></tr></thead><tbody><tr><td>Network Name</td><td>Seismic devnet</td></tr><tr><td>Currency Symbol</td><td>ETH</td></tr><tr><td>Chain ID</td><td>5124</td></tr><tr><td>RPC URL (HTTP)</td><td><a href="https://node-2.seismicdev.net/rpc">https://node-2.seismicdev.net/rpc</a></td></tr><tr><td>RPC URL (WS)</td><td><a href="wss://node-2.seismicdev.net/ws">wss://node-2.seismicdev.net/ws</a></td></tr><tr><td>Explorer</td><td><a href="https://explorer-2.seismicdev.net/">https://explorer-2.seismicdev.net/</a></td></tr><tr><td>Faucet</td><td><a href="https://faucet-2.seismicdev.net/">https://faucet-2.seismicdev.net/</a></td></tr><tr><td>Starter Repo</td><td><a href="https://github.com/SeismicSystems/seismic-starter/tree/main">https://github.com/SeismicSystems/seismic-starter</a></td></tr></tbody></table>

NOTE: This is a testnet with known decryption keys. Please don't put real information on it!
