---
description: Setting up your local machine to develop with Seismic
icon: globe-pointer
---

# Installation

***

### System requirements

Before you begin, make sure your machine meets the following requirements:

* x84\_64 or arm64 architecture
* MacOS, Ubuntu, or Windows

***

### Install the local development suite

The local development suite uses `sforge` as the testing framework, `sanvil` as the local node, and `ssolc` as the compiler.&#x20;

1. Install [rust](https://www.rust-lang.org/tools/install) and [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) on your machine if you don't already have them. Default installation works well.

```bash
curl https://sh.rustup.rs -sSf | sh
```

2. Download and execute the sfoundryup installation script.

```bash
curl -L \
     -H "Accept: application/vnd.github.v3.raw" \
     "https://api.github.com/repos/SeismicSystems/seismic-foundry/contents/sfoundryup/install?ref=seismic" | bash
source ~/.zshenv  # or ~/.bashrc or ~/.zshrc
```

3. Install `sforge`, `sanvil`, `ssolc`. Expect this to take between 5-20 minutes depending on your machine.

```bash
sfoundryup
source ~/.zshenv  # or ~/.bashrc or ~/.zshrc
```

4. (Optional) Remove old build artifacts in existing projects. You can ignore this step if you aren't working with existing foundry projects.

```bash
sforge clean  # run in your project's contract directory
```

***

### Set up the VSCode extension

We recommend adding syntax highlighting via the [`seismic`](https://marketplace.visualstudio.com/items?itemName=SeismicSys.seismic) (or [`seismic`](https://open-vsx.org/extension/SeismicSys/seismic) for Open VSX) extension from the VSCode marketplace. If you already have the `solidity` extension, you'll have to disable it while writing Seismic code.
