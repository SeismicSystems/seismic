//! `exec`: run a command with the selected node's `ETH_RPC_URL` set.
//!
//! The child also gets `SEISMIC_CONTEXT`, the same pair `env` exports, so a
//! `seismic-tee` run inside the command acts on the same selection.
//!
//! `exec(2)` rather than spawn-and-wait: there is no remote hop to manage and
//! no exit code to translate. Signals and the TTY pass through natively and
//! the child's exit code is the process's own, which is what `env(1)` and
//! `aws-vault exec` do.

use std::process::ExitCode;

use clap::Args;

use crate::{Context, ContextArgs, Selection, echo};

#[derive(Debug, Args)]
pub struct ExecArgs {
    #[command(flatten)]
    pub context: ContextArgs,
    /// Which node, when the context selects a network only.
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,
    /// Print the command and the environment it would get, and run nothing.
    #[arg(long)]
    pub dry_run: bool,
    #[arg(last = true, required = true, num_args = 1.., value_name = "COMMAND")]
    pub command: Vec<String>,
}

/// Run `exec`.
pub fn run(args: ExecArgs) -> anyhow::Result<ExitCode> {
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

    let rpc_url = descriptor.eth_rpc_url();
    let (program, rest) = args.command.split_first().expect("clap requires COMMAND");

    let selection = selection.to_string();

    if args.dry_run {
        println!(
            "ETH_RPC_URL={rpc_url} SEISMIC_CONTEXT={selection} {program} {}",
            rest.join(" ")
        );
        return Ok(ExitCode::SUCCESS);
    }

    let mut command = std::process::Command::new(program);
    command
        .args(rest)
        .env("ETH_RPC_URL", rpc_url)
        .env("SEISMIC_CONTEXT", selection);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        // Returns only on failure to launch the child at all — a successful
        // exec never returns here.
        let error = command.exec();
        Err(anyhow::Error::new(error).context(format!("exec {program}")))
    }
    #[cfg(not(unix))]
    {
        anyhow::bail!(
            "`exec` needs a Unix process model; use `eval \"$(seismic-tee ctx env)\"` instead"
        )
    }
}
