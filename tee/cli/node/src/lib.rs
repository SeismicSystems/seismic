//! `node`: stand up and appraise a node — the operator's command group.
//!
//! The group is named for its subject rather than for a party of the trust
//! model. The validator's trust-sensitive actions there — release and fetch
//! `root_key`, stake, resync — are the enclave's, not a human's, and a
//! non-staking full-node operator runs exactly these commands, so what the
//! commands share is the node. They are cloud-agnostic and start at the node
//! descriptor: each consumes a descriptor for an already-running node and
//! talks to it over HTTP. **They never provision**: producing descriptors is
//! the Pulumi program's job. Founding a network is the `network` group's
//! ([`seismic_tee_network`]), and this crate must never depend on it: the
//! crates are split on that line so the compiler enforces it, and this half
//! plus [`seismic_tee_common`] stays free of founder-only dependencies
//! whichever binary mounts it.
//!
//! Three commands, in the order an operator meets them: [`configure`]
//! delivers a node's config on first boot and waits for it to come up,
//! [`verify`] appraises a running node's attestation, and [`status`] watches
//! the first-boot disk wipe on its own. The `network` group configures a
//! cohort by doing to each node what these do to one, so the flows behind the
//! commands — building the config, POSTing it, the status poller, the
//! appraisal — are this crate's library surface as well as its command
//! group's.

pub mod args;
pub mod configure;
pub mod status;
pub mod verify;

/// The fake server and fixtures the tests here share with the network crate's.
#[cfg(test)]
pub(crate) use seismic_tee_common::test_support;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, bail};
use clap::Subcommand;
use seismic_tee_common::Manifest;
use seismic_tee_context::{Context, ContextArgs};

/// The `node` command group, declared in workflow order — the order `--help`
/// lists them in.
#[derive(Debug, Subcommand)]
pub enum NodeCommand {
    /// Configure a node to join a network: assemble + POST config to tdx-init.
    Configure(configure::ConfigureArgs),
    /// Deploy-verify a node's TDX attestation against the intended image.
    Verify(verify::VerifyArgs),
    /// Watch a node's first-boot LUKS provisioning progress.
    Status(status::StatusArgs),
}

/// Run one `node` command.
pub async fn run(command: NodeCommand) -> anyhow::Result<ExitCode> {
    match command {
        NodeCommand::Configure(args) => configure::run(args).await,
        NodeCommand::Verify(args) => verify::run(args).await,
        NodeCommand::Status(args) => status::run(args).await,
    }
}

/// Read `--manifest`: present, and a manifest the strict v1 schema accepts.
///
/// Every command starts here, because every command's other inputs are
/// checked against it — the manifest is what makes a genesis or a policy
/// *this* network's rather than some network's.
pub fn load_manifest(path: &Path) -> anyhow::Result<Manifest> {
    if !path.is_file() {
        bail!("--manifest file not found: {}", path.display());
    }
    Manifest::load(path).with_context(|| format!("--manifest {}: invalid manifest", path.display()))
}

/// Resolve `--manifest`: the flag when given, else the selected context's
/// network — `<dir>/network-manifest.json` for one registered with `dir`, or
/// the `manifest` path for one registered loose. Shared by `configure` and
/// `verify`, so both agree on where the manifest comes from when neither
/// names one.
pub fn resolve_manifest(flag: Option<&Path>, context: &ContextArgs) -> anyhow::Result<PathBuf> {
    if let Some(path) = flag {
        return Ok(path.to_path_buf());
    }
    let loaded = Context::load(context.config.as_deref())?;
    let selected = loaded.select(context.context.as_deref())?;
    selected.manifest()
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::*;
    use crate::test_support::{FIXTURE_MANIFEST, write_file};

    /// The group as the binary mounts it.
    #[derive(Parser)]
    struct Probe {
        #[command(subcommand)]
        command: NodeCommand,
    }

    #[test]
    fn the_command_tree_is_well_formed() {
        Probe::command().debug_assert();
    }

    /// The three operator commands, listed in workflow order.
    #[test]
    fn the_commands_are_listed_in_workflow_order() {
        let names: Vec<_> = Probe::command()
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect();
        assert_eq!(names, ["configure", "verify", "status"]);
    }

    #[test]
    fn the_manifest_is_checked_before_anything_reads_it() {
        let dir = tempfile::tempdir().unwrap();

        let error = load_manifest(&dir.path().join("absent.json"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("--manifest file not found"), "{error}");

        let bad = write_file(&dir, "bad.json", br#"{"manifest_version": 1}"#);
        let error = format!("{:?}", load_manifest(&bad).unwrap_err());
        assert!(error.contains("invalid manifest"), "{error}");
        assert!(error.contains("bad.json"), "{error}");

        let good = write_file(&dir, "network-manifest.json", FIXTURE_MANIFEST);
        assert_eq!(load_manifest(&good).unwrap().eth.chain_id, 5124);
    }

    /// The flag wins outright; absent, the selected context's network
    /// supplies the manifest.
    #[test]
    fn resolve_manifest_falls_back_to_the_context() {
        let flag = Path::new("/explicit/network-manifest.json");
        assert_eq!(
            resolve_manifest(Some(flag), &ContextArgs::default()).unwrap(),
            flag
        );

        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
current = "devnet-1"

[networks.devnet-1]
dir = "/nets/devnet-1"
"#,
        )
        .unwrap();
        let context = ContextArgs {
            context: None,
            config: Some(config_path),
        };
        assert_eq!(
            resolve_manifest(None, &context).unwrap(),
            PathBuf::from("/nets/devnet-1/network-manifest.json")
        );
    }
}
