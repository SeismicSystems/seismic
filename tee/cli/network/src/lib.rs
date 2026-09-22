//! `network`: bring a whole network into existence — the genesis deployer's
//! command group.
//!
//! [The trust model](https://github.com/SeismicSystems/seismic/blob/main/docs/tee/trust-model.md#the-trust-anchor-per-action)
//! names the genesis deployer as the party whose trust anchor is its own
//! verification at assemble; these commands are that verification and the
//! steps around it. The group is run by whoever founds a network — one of
//! Seismic's, a fork, or a private devnet. It spans both sides of the
//! cohort's existence: deriving the artifact set that *is* the network's
//! identity from its authored inputs, which needs no machine at all, and then
//! harvesting each node's founding keys and configuring the cohort, which
//! needs all of them running. What it never does is provision — the
//! descriptors come from the Pulumi program, which provisions one node or N
//! from one stack.
//!
//! Four commands, in founding order: [`init`] scaffolds a network directory's
//! authored inputs; [`harvest`] collects and DCAP-verifies the cohort's
//! founding keys; [`assemble`] derives the artifact set and mints
//! `network_id` (and, with `--check`, re-derives it and holds the set on
//! disk to the result instead of writing); [`configure`] founds the cohort —
//! one genesis node plus its joiners — and asserts the launch against what
//! the manifest pins (and, with `--check`, re-asserts a live cohort's launch
//! without configuring anything). Between `init` and `harvest` the cohort is provisioned
//! with the Pulumi program; `pulumi destroy` tears it down.
//!
//! One more command lives in this crate without being a founding step.
//! [`verify_founding`] is the audit of one: it re-verifies a committed
//! network directory's founding offline — every archived quote against its
//! own collateral snapshot and the policy the manifest pins, the archive
//! against the validator set the summit genesis seats. Its audience is
//! anyone holding the directory, and the binary mounts it at the top level
//! rather than in this group: the auditor takes no trust-sensitive action of
//! their own, and their subject is the network's record as a whole. It lives
//! here because it runs the same function as `assemble`'s gate on the
//! archive, on purpose.
//!
//! Every rule the network's identity rests on — the manifest's canonical bytes
//! and strict schema, admission-ID derivation and registry genesis storage,
//! DCAP verification of a quote — has exactly one implementation, in the
//! enclave repo, and this crate links it. The two helpers whose only
//! implementation is a foreign repo's node binary (`summit genesis`,
//! `seismic-reth genesis-hash`) are shell-outs ([`shell_outs`]).
//!
//! It is the only crate allowed to depend on both sides: founding a network
//! includes doing to each node what [`seismic_tee_node`] does to one. The
//! rule cuts the other way too: anything that acts on one node belongs on the
//! other side of the seam — the node descriptor — not here. The whole
//! workspace is headed for a public repo, since an auditor who can read
//! `assemble` can see what `verify-founding` re-runs.

pub mod args;
pub mod assemble;
pub mod bootnodes;
pub mod configure;
pub mod dashboard;
pub mod founding;
pub mod gates;
pub mod harvest;
pub mod init;
pub mod launch;
pub mod shell_outs;
pub mod verify_founding;

use std::process::ExitCode;

use clap::Subcommand;

/// The `network` command group, declared in founding order — the order
/// `--help` lists them in.
#[derive(Debug, Subcommand)]
pub enum NetworkCommand {
    /// Scaffold a network directory's authored inputs.
    Init(init::InitArgs),
    /// Harvest + DCAP-verify a founding cohort's summit keys into inputs/.
    Harvest(harvest::HarvestArgs),
    /// Derive the artifact set from a network directory's inputs (--check:
    /// re-derive and compare with what is on disk instead of writing).
    Assemble(assemble::AssembleArgs),
    /// Configure a cohort in parallel: one genesis + N joiners, one command
    /// (--check: re-assert the launch against the manifest's pins instead of
    /// configuring).
    Configure(configure::ConfigureArgs),
}

/// Run one `network` command.
pub async fn run(command: NetworkCommand) -> anyhow::Result<ExitCode> {
    match command {
        NetworkCommand::Init(args) => init::run(args).await,
        NetworkCommand::Harvest(args) => harvest::run(args).await,
        NetworkCommand::Assemble(args) => assemble::run(args).await,
        NetworkCommand::Configure(args) => configure::run(args).await,
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::*;

    /// The group as the binary mounts it.
    #[derive(Parser)]
    struct Probe {
        #[command(subcommand)]
        command: NetworkCommand,
    }

    #[test]
    fn the_command_tree_is_well_formed() {
        Probe::command().debug_assert();
    }

    /// The four founding commands in founding order — and nothing else: the
    /// audit sits at the binary's top level, the policy review in its own
    /// group, and checking an assembled set or a launched cohort is a
    /// `--check` on the command that produced it, not a command.
    #[test]
    fn the_commands_are_listed_in_founding_order() {
        let names: Vec<_> = Probe::command()
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect();
        assert_eq!(names, ["init", "harvest", "assemble", "configure"]);
    }
}
