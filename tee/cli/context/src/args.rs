//! `--context` and `--config`: how every command picks the context it acts on.
//!
//! [`ContextArgs`] is flattened into every command that resolves a target, so
//! an operator who learns one command's way of overriding the selection has
//! learned all of them. [`ConfigArgs`] is flattened once, into the root
//! command: which context file to use is a question about the invocation,
//! not about any one command, so the binary passes the path down to each.

use std::path::PathBuf;

use clap::Args;
use clap_complete::ArgValueCandidates;

use crate::complete;

#[derive(Debug, Clone, Default, Args)]
pub struct ContextArgs {
    /// Target this context instead of the current one. Beats the file's
    /// `current`; explicit target flags (--node, DIR) beat both.
    #[arg(
        long,
        value_name = "CTX",
        env = "SEISMIC_CONTEXT",
        add = ArgValueCandidates::new(complete::selections)
    )]
    pub context: Option<String>,
}

#[derive(Debug, Clone, Default, Args)]
pub struct ConfigArgs {
    /// Context file to read and write. Default:
    /// $XDG_CONFIG_HOME/seismic/config.toml, else ~/.config/seismic/config.toml.
    #[arg(
        long,
        global = true,
        value_name = "FILE",
        env = "SEISMIC_CONFIG",
        help_heading = "Global options"
    )]
    pub config: Option<PathBuf>,
}
