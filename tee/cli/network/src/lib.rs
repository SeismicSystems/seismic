//! `network`: a network directory — its authored inputs, the artifact set
//! derived from them, and its removal.
//!
//! The group is named for the object its commands act on, and the rule that
//! holds it together is one line: **nothing here contacts a node**. Every
//! command reads and writes local files; `init` also fetches an image's
//! release assets over HTTP, but never from a machine of the network. The
//! commands that reach a running box — harvesting a cohort's founding keys
//! and configuring it — live in the `seismic-tee-node` crate, so a founding
//! alternates between the two: `network init` → provision → `node harvest`
//! → `network assemble` → configure. The alternation is the point: it makes
//! plain that `assemble` touches nothing but the directory.
//!
//! [The trust model](https://github.com/SeismicSystems/seismic/blob/main/docs/tee/trust-model.md#the-trust-anchor-per-action)
//! names the genesis deployer as the party whose trust anchor is its own
//! verification at assemble; [`assemble`] is that verification. The group is
//! run by whoever founds a network — one of Seismic's, a fork, or a private
//! devnet.
//!
//! Three commands: [`init`] scaffolds a network directory's authored inputs;
//! [`assemble`] derives the artifact set and mints `network_id` (and, with
//! `--check`, re-derives it and holds the set on disk to the result instead
//! of writing); [`rm`] — the counterpart of `init`, once `pulumi destroy`
//! has torn the cohort down — deletes the directory and its context entry.
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
//! This crate and `seismic-tee-node` depend on neither each other, only on
//! [`seismic_tee_common`] and [`seismic_tee_context`]: what both read from a
//! network directory — its layout, the founding inputs — lives there. The
//! whole workspace is headed for a public repo, since an auditor who can read
//! `assemble` can see what `verify-founding` re-runs.

pub mod assemble;
pub mod gates;
pub mod image;
pub mod init;
pub mod rm;
pub mod shell_outs;
pub mod verify_founding;

use std::process::ExitCode;

use clap::Subcommand;

/// The `network` command group, declared in founding order — the order
/// `--help` lists them in — and then the one that ends a network's life on
/// this machine.
#[derive(Debug, Subcommand)]
pub enum NetworkCommand {
    /// Scaffold a network directory's authored inputs.
    Init(init::InitArgs),
    /// Derive the artifact set from a network directory's inputs (--check:
    /// re-derive and compare with what is on disk instead of writing).
    Assemble(assemble::AssembleArgs),
    /// Delete a network directory and its context entry, by name — once its
    /// stack is destroyed (the counterpart of init). `ctx rm` forgets the
    /// entry and keeps the directory.
    #[command(visible_alias = "remove")]
    Rm(rm::RmArgs),
}

/// Run one `network` command.
pub async fn run(command: NetworkCommand) -> anyhow::Result<ExitCode> {
    match command {
        NetworkCommand::Init(args) => init::run(args).await,
        NetworkCommand::Assemble(args) => assemble::run(args).await,
        NetworkCommand::Rm(args) => rm::run(args).await,
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

    /// The two founding commands in founding order, then `rm` — and nothing
    /// else: the audit sits at the binary's top level, the policy review in
    /// its own group, the commands that reach a node in `node`, and checking
    /// an assembled set is a `--check` on the command that produced it, not a
    /// command.
    #[test]
    fn the_commands_are_listed_in_founding_order() {
        let names: Vec<_> = Probe::command()
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect();
        assert_eq!(names, ["init", "assemble", "rm"]);
    }
}
