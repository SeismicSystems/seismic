//! `--context`/`--config`: the flattened pair every context-resolving command
//! carries.
//!
//! Flattened into every command that resolves a target, so an operator who
//! learns one command's way of overriding the selection has learned all of
//! them.

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

    /// Context file to read. Default: $XDG_CONFIG_HOME/seismic/config.toml,
    /// else ~/.config/seismic/config.toml.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}
