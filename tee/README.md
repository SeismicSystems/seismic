# tee — the `seismic-tee` CLI

`seismic-tee` founds, joins, governs and audits a Seismic TEE network. It is
one binary over the Cargo workspace in [`cli/`](cli/), with the founding
archives its tests replay in [`networks/`](networks/) beside it; the installer
is [`cli/install.sh`](cli/install.sh). The CLI is cloud-agnostic: it starts
at a **node descriptor** (a JSON map of node name → `{public_ip, fqdn}`) and
never provisions a machine. Provisioning — the Pulumi programs and the devnet
runbook — lives in the private
[deploy](https://github.com/SeismicSystems/deploy) repo, which produces those
descriptors as its `nodes` stack output.

For what the network is and how a founding is checked, read the
[TEE docs](../docs/tee/README.md): the trust model, the network manifest
reference, and the network-founding design.

## Install

```bash
# The newest release, into ~/.local/bin. `sh -s -- --version main` for the
# tip of main, `--help` for every option; afterwards `seismic-tee --upgrade`.
curl -fsSL https://raw.githubusercontent.com/SeismicSystems/seismic/main/tee/cli/install.sh | sh
seismic-tee --version
seismic-tee --help
seismic-tee ctx --help                     # anyone: pick a network and node
seismic-tee network --help                 # network founder
seismic-tee node --help                    # node operator
seismic-tee admission --help               # governance
seismic-tee verify-founding --help         # auditor
```

The installer checks the tarball against the release's `SHA256SUMS` and,
when `gh` is installed and logged in, verifies the binary's build provenance
attestation (`gh attestation verify seismic-tee --repo SeismicSystems/seismic`).
Prebuilt for linux amd64, linux arm64 and macOS arm64; the Linux binaries need
glibc 2.35 or newer (Ubuntu 22.04, Debian 12, and later) and nothing else from
the host. Anywhere else, build from source:

```bash
cargo install --git https://github.com/SeismicSystems/seismic seismic-tee
```

To upgrade, `seismic-tee --upgrade` — the newest release, or a version in the
same spellings (`--upgrade main` for the tip of main). It fetches and runs
this installer, so the download is checked the same way, and the new binary
lands where the running one is rather than at the installer's default.

To uninstall, delete the two paths the CLI writes — the binary, and the
context file if you want its network and node pointers gone with it:

```bash
rm ~/.local/bin/seismic-tee
rm -rf ~/.config/seismic          # or $XDG_CONFIG_HOME/seismic
```

Tab completion — of commands and flags, and of the context, network and node
names in the context file — is one line in your shell's rc file (drop it
again when you uninstall, since it runs the binary on every new shell):

```bash
source <(seismic-tee --completions bash)   # zsh: --completions zsh; fish: --completions fish | source
```

## Releases

Releases are GitHub releases of this repo, tagged by component because the
repo hosts others: `seismic-tee/vX.Y.Z` is a version, and every merge to
`main` that touches `tee/cli` or the release workflow publishes a `seismic-tee/main-<sha>` prerelease, so the tip of main is always
installable and every build has a permanent download URL. Each release ships
one tarball per platform, one `SHA256SUMS` over all of them, and a build
provenance attestation per binary. `seismic-tee --version` names the crate
version and the commit the binary was built from.

Cutting a version: bump `[workspace.package] version` in
[`cli/Cargo.toml`](cli/Cargo.toml) on `main`, then push the matching tag.
The workflow ([`seismic-tee-release.yml`](../.github/workflows/seismic-tee-release.yml))
refuses a tag that does not match the workspace version or is not higher than
every version already released; its header comment says what to do when a
run fails.

## How it's organized: one CLI, command groups by party

The command groups follow
[the trust model's parties](../docs/tee/trust-model.md#the-trust-anchor-per-action):
its table names who takes every trust-sensitive action in a network's life,
and each party with CLI work today gets a group. `network` is the **genesis
deployer**'s. `node` is standing up and appraising a node — named for its
subject rather than for the **validator**, because the validator's actions in
that table are the enclave's, not a human's, and a non-staking full-node
operator runs the same commands. `admission` is **governance**'s: the pipeline
from an image's measurements to the policy record a network accepts. The
**auditor** takes no trust-sensitive action — they verify the others' after the
fact — which is why `verify-founding` sits at the top level rather than in any
group. `ctx`, the context file, is everyone's: which network and node the
other commands act on, kubeconfig-style, holding pointers and never a
credential.

| Command | Run by | Purpose |
|---|---|---|
| `network init` | founder, once per network | Scaffold a network directory's authored inputs. |
| `network harvest` | founder, once per network | Collect and DCAP-verify the founding cohort's summit keys into `inputs/harvest/` — the provenance `assemble` pins the validator set from. |
| `network assemble` | founder, once per network | Derive the artifact set from a network directory's inputs: pins the harvested founding set and mints `network_id`. `--check` re-derives and compares with what is on disk. |
| `network configure` | founder | Configure a cohort in parallel (one genesis node + N joiners), then run the launch assertions against the manifest's pins. `--check` re-runs those assertions alone on a live cohort. |
| `node configure` | any operator, on first boot | POST the node TOML to tdx-init, recording the exact body under `nodes/`; runs `verify` once the node is up. |
| `node verify` | any operator, whenever they rely on a node | Deploy-verify one node's TDX attestation against the network manifest and the measurement policy it pins. Read-only and re-runnable. |
| `node status` | any operator | Watch the node's first-boot disk wipe to completion. |
| `admission promote` | whoever proposes an image | Promote raw `make measure` output into the policy document. |
| `admission compile` | anyone reviewing a policy | Report the admission IDs a measurement-policy document admits and the registry genesis storage seeding them. |
| `verify-founding` | anyone holding a committed network directory | Re-verify a founding offline: every archived quote against the bundle and policy archived with it, and the archive against the validator set the summit genesis seats. |

Each command ends by naming the next step, and `--help` on any of them is
the reference; nothing here repeats it.

## The workspace

[`cli/`](cli/) is its own Cargo workspace, scoped to that directory rather
than the repo root, with its own toolchain pin and lockfile. Five library
crates and the binary, split by who may depend on whom — a rule the compiler
enforces, so each party's crate stays free of the others' dependencies:

| Crate | Owns | Depends on |
|---|---|---|
| `common` | the node descriptor, the network-directory layout, HTTP and error types | nothing of ours |
| `context` | the `ctx` group and the context file | `common` |
| `node` | the `node` group | `common`, `context` |
| `network` | the `network` group and `verify-founding` | all of the above |
| `admission` | the `admission` group | nothing of ours |
| `seismic-tee` | the binary; mounts the four command-bearing crates | all of the above |

Every rule the network's identity rests on — the manifest schema and
`network_id`, admission-ID derivation, quote verification — has exactly one
implementation, in the [enclave](https://github.com/SeismicSystems/enclave)
repo, and the CLI links those crates (git dependencies pinned by rev in
[`cli/Cargo.toml`](cli/Cargo.toml)) rather than reimplementing them, so the
CLI's answer and a node's answer are the same answer. Helpers whose only
implementation is a foreign repo's binary (`summit genesis`, `seismic-reth
genesis-hash`) stay shell-outs.

### `assemble` on macOS

`assemble` runs the image's own `seismic-reth` and `summit`, fetched from the
seismic-images release `inputs/image.json` names and verified against its
`SHA256SUMS`, so the digests the manifest pins come from the bytes the nodes
boot. Those binaries are x86-64 Linux, so a Mac cannot run them — nor can an
arm64 Linux box, nor a Linux VM on Apple silicon where the kernel executes
x86-64 through Rosetta, since the image's `summit` crashes there. `assemble`
refuses on all three and names the way out. Either run it in an amd64 Linux
container, where the default path works:

```bash
docker run --platform linux/amd64 -v "$PWD:/w" -w /w seismic-tee \
    network assemble tee/networks/<name>
```

or point it at binaries this host can run:

```bash
seismic-tee network assemble tee/networks/<name> \
    --reth-bin <path> --summit-bin <path>
```

Neither seismic-reth nor summit publishes a macOS build today, so the second
spelling means building both from source at the revs `inputs/image.json`
pins. A rebuild at those revs computes the same digests — it just no longer
proves they came from the image.

## Development

```bash
make -C tee/cli help        # the targets
make -C tee/cli locked      # Cargo.lock agrees with Cargo.toml (CI runs this first)
make -C tee/cli check       # fmt-check + clippy + test: what CI requires
make -C tee/cli drift       # cross-repo drift guards; needs seismic-reth on PATH
```

`check` is hermetic. Its suite replays the committed founding archive in
[`networks/fixture-devnet/`](networks/fixture-devnet/) through the pinned
enclave verifier, so a pin bump that judges those quotes differently fails
the PR that bumps it; [`networks/README.md`](networks/README.md) says what a
network directory holds and how the fixture is refreshed. The drift guards
reach outside the repo and run in their own, non-required CI job
([`seismic-tee.yml`](../.github/workflows/seismic-tee.yml)).

The rules a new command follows: every group stays cloud-agnostic and none
wraps a provisioner, the dependency direction above holds, and the command
goes in the group of the party whose action it is — or in `node` if it acts
on one node, whoever runs it.
