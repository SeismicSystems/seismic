//! `env`: print export lines for the selected node.
//!
//! `eval "$(seismic-tee ctx env)"` is the equivalent of `kubectx -s`: a shell
//! pinned to its selection no matter what any other shell runs, with no
//! subshell and no temp file. `SEISMIC_CONTEXT` is exported alongside
//! `ETH_RPC_URL` on purpose — it is what makes the pin stick, since
//! [`crate::args::ContextArgs`]'s `--context` reads it back on the next
//! invocation. Everything but the export (or `unset`) lines goes to stderr,
//! so an `eval` of stdout never runs anything but a shell assignment.

use std::process::ExitCode;

use clap::Args;

use crate::{Context, ContextArgs, Selection, echo};

#[derive(Debug, Args)]
pub struct EnvArgs {
    #[command(flatten)]
    pub context: ContextArgs,
    /// Which node, when the context selects a network only.
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,
    /// Emit `unset` lines instead, so an `eval` is reversible.
    #[arg(long)]
    pub unset: bool,
}

/// Run `env`.
pub fn run(args: EnvArgs) -> anyhow::Result<ExitCode> {
    if args.unset {
        println!("unset ETH_RPC_URL");
        println!("unset SEISMIC_CONTEXT");
        return Ok(ExitCode::SUCCESS);
    }

    let context = Context::load(args.context.config.as_deref())?;
    let selected = context.select(args.context.context.as_deref())?;
    let (node, descriptor) = selected.node(args.name.as_deref())?;
    // Pinned to the node it resolved to, so a network-only context plus
    // --name yields a `<network>/<node>` the next command needs no flag for.
    let selection = Selection {
        network: selected.selection.network.clone(),
        node: Some(node.to_string()),
    };
    echo(&selection, &descriptor.eth_rpc_url());

    println!("export ETH_RPC_URL='{}'", descriptor.eth_rpc_url());
    println!("export SEISMIC_CONTEXT='{selection}'");
    Ok(ExitCode::SUCCESS)
}
