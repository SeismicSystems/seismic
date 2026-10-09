//! `status`: the live state of every node in the node table, and what it
//! says about the network; and the first-boot disk-wipe watch `configure`
//! waits on after its POST.
//!
//! The command reads each node through [`crate::probe`] — reachable, awaiting
//! or past its config, its candidate against the manifest's pin, key holder,
//! disk — and prints a row per node under a network line: live, unfounded, or
//! no reachable key holder. `--watch` re-reads and repaints until ctrl-C.
//!
//! The watch: the attestation service serves `getLuksProvisioningStatus` on
//! `:7878` (JSON-RPC) through the first-boot disk wipe — the one long (1h+),
//! otherwise opaque phase. [`poll_provisioning`] polls it to completion, and
//! is the shared poller behind `configure`'s post-POST wait and the founding
//! cohort's dashboard, which is why the state machine renders nothing itself.
//!
//! States, as the attestation service's `LuksProvisioningStatus` serializes
//! them:
//!
//! ```text
//! provisioning {bytes_done, bytes_total, eta_seconds?} | idle | error {error} | unknown
//! ```
//!
//! Watch-completion is deliberately conservative about `idle`: right after a
//! POST the server may be down (connection refused) or up-but-idle *before*
//! the wipe starts, which looks identical to idle-because-finished. So idle
//! counts as "done" only after provisioning has been seen; otherwise the watch
//! waits a short grace for the wipe to begin and, if it never does, concludes
//! there's no wipe (already finished, or a fast-unlock restart). This watches
//! only the wipe — it is NOT a node-readiness gate (summit/reth/genesis come
//! later).

use std::collections::BTreeMap;
use std::io::{IsTerminal as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::Args;
use jsonrpsee::rpc_params;
use seismic_tee_common::founding::load_harvest_records;
use seismic_tee_common::http::{ATTESTATION_RPC_PORT, HARVEST_PORT};
use seismic_tee_common::{
    Descriptors, Error, NetworkDir, http, next_step, note, rpc, select_descriptor,
};
use seismic_tee_context::{Context, load_nodes};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::args::NodeArgs;
use crate::dashboard::Painter;
use crate::load_manifest;
use crate::probe::{
    self, Asked, ConfigState, Disk, Founding, NetworkState, NodeState, assess, probe_all,
};

/// The attestation service's status method.
pub const RPC_METHOD: &str = "getLuksProvisioningStatus";

pub const POLL_INTERVAL: Duration = Duration::from_secs(5);
/// Max wait for `:7878` to first respond — covers attestation-service startup
/// (and, on a joiner, the root_key fetch that precedes the listener coming
/// up).
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(180);
/// Once reachable and idle, how long to wait for the wipe to begin before
/// concluding none is in progress. The service-up→first-wipe-tick gap (udev
/// settle, disk discovery, luksFormat warm-up) is seconds; a fast-unlock
/// restart stays idle forever, so this bounds the wait instead of hanging.
pub const IDLE_GRACE: Duration = Duration::from_secs(60);
/// How long a *continuous* error status must persist before it is terminal.
/// `persistent-luks-setup` runs under `Restart=on-failure` and retries
/// transient failures (data disk not yet attached, vTPM not ready, keyfile not
/// written yet), writing `error` to the status file on each failed attempt and
/// flipping back to `provisioning` on the next. So a lone `error` reading is
/// NOT terminal — only an error that sticks, with no recovering attempt within
/// the grace, is. Any non-error reading resets the clock.
pub const ERROR_GRACE: Duration = Duration::from_secs(120);

/// One-time context printed when the wipe is first seen: the bar alone
/// doesn't say what's happening, and this phase is long enough to look like a
/// hang.
const FIRST_BOOT_NOTE: &str = "First-boot: initializing the encrypted /persistent disk — a one-time \
                               full-disk wipe (LUKS + dm-integrity) that can take 1h+ on large \
                               disks. The node must finish this before it can boot further.";

/// `getLuksProvisioningStatus`'s result, as this build reads it.
///
/// Read tolerantly rather than as a closed enum: a state this build does not
/// know is reported and polled through, never a crash of a watch that may be
/// an hour in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuksProvisioningStatus {
    /// No wipe in flight — finished, or never started on this boot.
    Idle,
    /// The wipe is running. A `bytes_total` of 0 is the "just started, no
    /// measurement yet" marker.
    Provisioning {
        bytes_done: u64,
        bytes_total: u64,
        eta_seconds: Option<u64>,
    },
    /// The wipe failed; systemd is retrying it.
    Error { error: String },
    /// The status file exists but couldn't be read — not evidence of
    /// completion.
    Unknown,
    /// A state this build does not know.
    Other(String),
}

impl LuksProvisioningStatus {
    /// Read the RPC result. Anything that isn't a recognisable status object
    /// is [`Self::Other`], carrying what was there.
    pub fn from_result(result: &Value) -> Self {
        #[derive(Deserialize)]
        struct Raw {
            state: String,
            #[serde(default)]
            bytes_done: u64,
            #[serde(default)]
            bytes_total: u64,
            #[serde(default)]
            eta_seconds: Option<u64>,
            #[serde(default)]
            error: Option<String>,
        }
        let Ok(raw) = serde_json::from_value::<Raw>(result.clone()) else {
            return Self::Other(result.to_string());
        };
        match raw.state.as_str() {
            "idle" => Self::Idle,
            "provisioning" => Self::Provisioning {
                bytes_done: raw.bytes_done,
                bytes_total: raw.bytes_total,
                eta_seconds: raw.eta_seconds,
            },
            "error" => Self::Error {
                error: raw.error.unwrap_or_else(|| "?".to_string()),
            },
            "unknown" => Self::Unknown,
            other => Self::Other(other.to_string()),
        }
    }
}

/// One `getLuksProvisioningStatus` call → its result, verbatim.
///
/// Fails with [`Error::RpcTransport`] while the attestation service is coming
/// up, which is the normal state right after a POST.
pub async fn fetch_status(client: &rpc::Client) -> seismic_tee_common::Result<Value> {
    client.call(RPC_METHOD, rpc_params![]).await
}

fn gib(n: u64) -> String {
    format!("{:.1}", n as f64 / f64::from(1u32 << 30))
}

fn duration(seconds: u64) -> String {
    let (hours, rem) = (seconds / 3600, seconds % 3600);
    let (minutes, secs) = (rem / 60, rem % 60);
    if hours > 0 {
        format!("{hours}h{minutes:02}m")
    } else if minutes > 0 {
        format!("{minutes}m{secs:02}s")
    } else {
        format!("{secs}s")
    }
}

fn bar(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0 * width as f64) as i64).clamp(0, width as i64) as usize;
    format!("[{}{}]", "#".repeat(filled), "-".repeat(width - filled))
}

/// Render a `provisioning` status as a one-line progress string.
pub fn format_provisioning(bytes_done: u64, bytes_total: u64, eta_seconds: Option<u64>) -> String {
    if bytes_total == 0 {
        return "encrypting disk: starting (no measurement yet)".to_string();
    }
    let pct = 100.0 * bytes_done as f64 / bytes_total as f64;
    let mut line = format!(
        "encrypting disk {} {pct:5.1}%  {}/{} GiB",
        bar(pct, 30),
        gib(bytes_done),
        gib(bytes_total),
    );
    if let Some(eta) = eta_seconds.filter(|&eta| eta > 0) {
        line.push_str(&format!("  eta {}", duration(eta)));
    }
    line
}

/// Which stage of the watch an [`Update`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Connecting,
    Provisioning,
    Waiting,
    Done,
    Error,
}

/// One observation of the watch. `line` is a compact, render-agnostic status
/// string; `transient` hints single-line renderers whether it's an in-place
/// progress update or a permanent line. `done` marks the terminal observation
/// — `ok` = the wipe finished (or none was needed), not-`ok` = it errored or
/// the node never became reachable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub phase: Phase,
    pub line: String,
    pub transient: bool,
    pub done: bool,
    pub ok: bool,
}

impl Update {
    fn transient(phase: Phase, line: impl Into<String>) -> Self {
        Self {
            phase,
            line: line.into(),
            transient: true,
            done: false,
            ok: false,
        }
    }

    fn permanent(phase: Phase, line: impl Into<String>) -> Self {
        Self {
            transient: false,
            ..Self::transient(phase, line)
        }
    }

    fn terminal(ok: bool, line: impl Into<String>) -> Self {
        Self {
            done: true,
            ok,
            ..Self::permanent(if ok { Phase::Done } else { Phase::Error }, line)
        }
    }
}

/// What one poll of the status endpoint came back with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Observation {
    /// No answer: connection refused, timeout, or an HTTP failure. Normal
    /// while the attestation service is coming up.
    Unreachable,
    /// The service answered with a status.
    Status(LuksProvisioningStatus),
    /// The service answered, but not with a status: a JSON-RPC error object.
    /// Polling on cannot change that, so it ends the watch.
    Refused(String),
}

/// The watch's state machine: feed it observations, get [`Update`]s.
///
/// Time is a parameter so the grace periods can be exercised without waiting
/// them out. Renders nothing — [`poll_provisioning`] drives it against a live
/// node and hands each update to a renderer.
#[derive(Debug, Clone)]
pub struct ProvisioningWatch {
    connect_timeout: Duration,
    idle_grace: Duration,
    error_grace: Duration,
    start: Option<Instant>,
    first_reachable: Option<Instant>,
    seen_provisioning: bool,
    /// When the current run of `error` began.
    error_since: Option<Instant>,
}

impl Default for ProvisioningWatch {
    fn default() -> Self {
        Self::with_grace(CONNECT_TIMEOUT, IDLE_GRACE, ERROR_GRACE)
    }
}

impl ProvisioningWatch {
    /// A watch with its own patience, in the order the module constants are
    /// declared: how long to wait for `:7878` to answer at all, for a wipe to
    /// begin once it does, and for an `error` to recover.
    pub fn with_grace(
        connect_timeout: Duration,
        idle_grace: Duration,
        error_grace: Duration,
    ) -> Self {
        Self {
            connect_timeout,
            idle_grace,
            error_grace,
            start: None,
            first_reachable: None,
            seen_provisioning: false,
            error_since: None,
        }
    }

    /// Record one observation made at `now` and say what it means so far.
    pub fn observe(&mut self, now: Instant, observation: Observation) -> Update {
        let start = *self.start.get_or_insert(now);
        let status = match observation {
            Observation::Unreachable => {
                if now.duration_since(start) > self.connect_timeout {
                    return Update::terminal(
                        false,
                        format!(
                            "attestation service :{ATTESTATION_RPC_PORT} never became reachable \
                             after {}s — is the node up?",
                            self.connect_timeout.as_secs(),
                        ),
                    );
                }
                return Update::transient(
                    Phase::Connecting,
                    format!("waiting for attestation service :{ATTESTATION_RPC_PORT} ..."),
                );
            }
            Observation::Refused(message) => return Update::terminal(false, message),
            Observation::Status(status) => status,
        };

        let first_reachable = *self.first_reachable.get_or_insert(now);
        if !matches!(status, LuksProvisioningStatus::Error { .. }) {
            // Any non-error reading clears the error clock.
            self.error_since = None;
        }

        match status {
            LuksProvisioningStatus::Provisioning {
                bytes_done,
                bytes_total,
                eta_seconds,
            } => {
                self.seen_provisioning = true;
                Update::transient(
                    Phase::Provisioning,
                    format_provisioning(bytes_done, bytes_total, eta_seconds),
                )
            }
            LuksProvisioningStatus::Error { error } => {
                // `error` is written per failed attempt, but the setup service
                // is restarted on failure and retries transient failures, so a
                // lone `error` is not terminal. Only one that persists past the
                // grace (no recovery in sight) is.
                let error_since = *self.error_since.get_or_insert(now);
                if now.duration_since(error_since) > self.error_grace {
                    return Update::terminal(
                        false,
                        format!(
                            "LUKS provisioning stuck in error for >{}s (not recovering): {error}",
                            self.error_grace.as_secs(),
                        ),
                    );
                }
                Update::transient(
                    Phase::Error,
                    format!("LUKS attempt failed, auto-retrying: {error}"),
                )
            }
            LuksProvisioningStatus::Idle => {
                if self.seen_provisioning {
                    return Update::terminal(true, "disk provisioning complete.");
                }
                if now.duration_since(first_reachable) > self.idle_grace {
                    return Update::terminal(
                        true,
                        "no first-boot wipe in progress (already finished, or a fast-unlock \
                         restart).",
                    );
                }
                Update::transient(Phase::Waiting, "waiting for provisioning to start ...")
            }
            LuksProvisioningStatus::Unknown => Update::permanent(
                Phase::Waiting,
                "status pipeline returned 'unknown' — still polling",
            ),
            LuksProvisioningStatus::Other(state) => Update::permanent(
                Phase::Waiting,
                format!("unexpected status {state:?} — still polling"),
            ),
        }
    }
}

/// Poll `getLuksProvisioningStatus` on `client` every `interval`, handing each
/// [`Update`] to `on_update`, until a terminal one. Returns its `ok`.
///
/// Renders nothing: this is the shared driver behind the single-node
/// [`watch_luks_provisioning`] and a cohort dashboard. Cancellation is the
/// caller's: dropping the future (a `select!` against ctrl-C) ends the watch
/// within one request.
pub async fn poll_provisioning(
    client: &rpc::Client,
    mut watch: ProvisioningWatch,
    interval: Duration,
    mut on_update: impl FnMut(&Update),
) -> bool {
    loop {
        let observation = match fetch_status(client).await {
            Ok(result) => Observation::Status(LuksProvisioningStatus::from_result(&result)),
            Err(error @ Error::Rpc { .. }) => Observation::Refused(error.to_string()),
            Err(_) => Observation::Unreachable,
        };
        let update = watch.observe(Instant::now(), observation);
        on_update(&update);
        if update.done {
            return update.ok;
        }
        tokio::time::sleep(interval).await;
    }
}

/// Poll and render progress until the wipe finishes, errors, or the watch
/// concludes none is in progress. Returns whether it ended well (wipe done, or
/// no wipe) — the `status` command's exit status.
///
/// Renders an in-place bar on a TTY; plain lines otherwise (so CI logs stay
/// readable).
pub async fn watch_luks_provisioning(client: &rpc::Client, interval: Duration) -> bool {
    let isatty = std::io::stdout().is_terminal();
    let mut seen_provisioning = false;
    // A TTY bar (no trailing newline) is currently shown.
    let mut on_bar_line = false;

    let mut emit = |line: &str, transient: bool| {
        let mut stdout = std::io::stdout().lock();
        if isatty && transient {
            let _ = write!(stdout, "\r\x1b[K{line}");
            on_bar_line = true;
        } else {
            if on_bar_line {
                // Close the in-place bar before a permanent line.
                let _ = writeln!(stdout);
                on_bar_line = false;
            }
            let _ = writeln!(stdout, "{line}");
        }
        let _ = stdout.flush();
    };

    poll_provisioning(client, ProvisioningWatch::default(), interval, |update| {
        if update.phase == Phase::Provisioning && !seen_provisioning {
            seen_provisioning = true;
            emit(FIRST_BOOT_NOTE, false);
        }
        emit(&update.line, update.transient);
    })
    .await
}

#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    seismic-tee node status                    every node in the selected network, and the network line\n  \
    seismic-tee node status --name alpha       one node\n  \
    seismic-tee node status --watch            repaint every 5s until ctrl-C\n  \
    seismic-tee node status --json             the same reading, for a script\n  \
    seismic-tee node status --node nodes.json --manifest network-manifest.json")]
pub struct StatusArgs {
    /// The node table: every node in it is read. --name narrows it to one.
    #[command(flatten)]
    pub node: NodeArgs,

    /// Network manifest JSON, for its pinned candidate (founding_tx_io_pk)
    /// and the harvest records beside it, which name the pinned box. Omit it
    /// to use the current context's network; without either, a node's
    /// candidate is shown but not judged, and the network never reads as
    /// unfounded.
    #[arg(long, value_name = "FILE")]
    pub manifest: Option<PathBuf>,

    /// Re-read every node and repaint, until ctrl-C.
    #[arg(long)]
    pub watch: bool,

    /// With --watch: seconds between readings.
    #[arg(
        long,
        value_name = "SECONDS",
        default_value_t = POLL_INTERVAL.as_secs(),
        requires = "watch"
    )]
    pub interval: u64,

    /// Print the reading as JSON on stdout.
    #[arg(long, conflicts_with = "watch")]
    pub json: bool,
}

impl StatusArgs {
    /// The flags that name this node table and manifest, for a suggested
    /// command acting on the same nodes. `--name` is left to the caller.
    fn table_flags(&self) -> String {
        let mut flags = String::new();
        if let Some(node) = &self.node.node {
            flags.push_str(&format!(" --node {}", node.display()));
        } else if let Some(context) = &self.node.context.context {
            flags.push_str(&format!(" --context {context}"));
        }
        flags
    }

    fn manifest_flag(&self) -> String {
        self.manifest
            .as_ref()
            .map(|path| format!(" --manifest {}", path.display()))
            .unwrap_or_default()
    }
}

/// The node table, narrowed by `--name`: `--node FILE` when given, else the
/// context's network.
fn load_table(args: &StatusArgs, config: Option<&Path>) -> anyhow::Result<Descriptors> {
    let nodes = load_nodes(
        args.node.node.as_deref(),
        &args.node.context,
        config,
        "--node",
    )?;
    let Some(name) = args.node.name.as_deref() else {
        return Ok(nodes);
    };
    let holder = match &args.node.node {
        Some(path) => path.display().to_string(),
        None => "the node table".to_string(),
    };
    let (name, descriptor) = select_descriptor(&nodes, Some(name), &holder)?;
    Ok(Descriptors::from([(name.to_string(), descriptor.clone())]))
}

/// The founding the manifest and its harvest pin, when a manifest is at
/// hand: `--manifest`, else the context network's, when it has a directory
/// and that directory has been assembled. An explicit `--node` keeps the
/// context unread, as everywhere.
fn load_founding(args: &StatusArgs, config: Option<&Path>) -> anyhow::Result<Option<Founding>> {
    let path = match &args.manifest {
        Some(path) => path.clone(),
        None if args.node.node.is_some() => return Ok(None),
        None => {
            let context = Context::load(config)?;
            let selected = context.select(args.node.context.context.as_deref())?;
            if selected.network.dir.is_none() {
                return Ok(None);
            }
            let path = selected.manifest()?;
            if !path.is_file() {
                return Ok(None);
            }
            path
        }
    };
    let manifest = load_manifest(&path)?;
    let dir = NetworkDir::of_manifest(&path);
    let records = if dir.harvest().is_dir() {
        match load_harvest_records(&dir) {
            Ok(records) => Some(records),
            Err(error) => {
                note(&format_args!(
                    "harvest records unreadable, so the pinned box is unknown: {error:#}"
                ));
                None
            }
        }
    } else {
        None
    };
    Ok(Some(Founding::new(&manifest, records.as_ref())))
}

/// A `:7879` reading, as `--json` spells it.
fn config_value(config: &ConfigState) -> &'static str {
    match config {
        ConfigState::Awaiting { .. } => "awaiting",
        ConfigState::Configured => "configured",
        ConfigState::NotReady(_) => "not_ready",
        ConfigState::Refused(_) => "refused",
        ConfigState::NoAnswer(_) => "no_answer",
    }
}

/// A `:7879` reading, as the table shows it: no answer is `-`, as in every
/// other column.
fn config_cell(config: &ConfigState) -> &'static str {
    match config {
        ConfigState::NotReady(_) => "not ready",
        ConfigState::NoAnswer(_) => "-",
        other => config_value(other),
    }
}

/// An awaiting node's candidate, judged against the pin when one is known.
fn candidate_cell(config: &ConfigState, founding: Option<&Founding>) -> String {
    let ConfigState::Awaiting {
        candidate_tx_io_public_key,
        ..
    } = config
    else {
        return "-".to_string();
    };
    match founding {
        Some(founding) if founding.pins(candidate_tx_io_public_key) => "pinned".to_string(),
        Some(_) => "not pinned".to_string(),
        None => format!("0x{}…", &candidate_tx_io_public_key[..8]),
    }
}

fn disk_cell(key_holder: Option<&Disk>) -> String {
    match key_holder {
        None => "-".to_string(),
        Some(Disk::Error(_)) => "unread".to_string(),
        Some(Disk::Status(status)) => match LuksProvisioningStatus::from_result(status) {
            LuksProvisioningStatus::Idle => "idle".to_string(),
            LuksProvisioningStatus::Provisioning {
                bytes_done,
                bytes_total,
                eta_seconds,
            } => format_provisioning(bytes_done, bytes_total, eta_seconds),
            LuksProvisioningStatus::Error { error } => format!("error, retrying: {error}"),
            LuksProvisioningStatus::Unknown => "unknown".to_string(),
            LuksProvisioningStatus::Other(state) => state,
        },
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

/// The network line, without its next step.
fn network_line(network: &NetworkState) -> String {
    let reading = match network {
        NetworkState::Live { key_holders } => {
            format!("live — key holder(s): {}", key_holders.join(", "))
        }
        NetworkState::Unfounded { pinned_box } => format!(
            "unfounded — the pinned box {pinned_box} awaits its config, still holding the pinned \
             candidate"
        ),
        NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
            pinned_box,
            pinned_box_answered: true,
        }) => format!(
            "no reachable key holder — every founding node is in this node table, and the pinned \
             box {pinned_box} no longer holds the pinned candidate: the founding root_key is gone"
        ),
        NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
            pinned_box,
            pinned_box_answered: false,
        }) => format!(
            "no reachable key holder — the pinned box {pinned_box} does not answer: still \
             booting, or gone"
        ),
        NetworkState::NoReachableKeyHolder(Asked::Some) => "no reachable key holder among these \
             nodes — a node table without every founding node says nothing about the rest of the \
             network"
            .to_string(),
    };
    format!("network: {reading}")
}

/// One reading of the node table, rendered.
struct Report<'a> {
    states: &'a BTreeMap<String, NodeState>,
    founding: Option<&'a Founding>,
    network: NetworkState,
}

impl<'a> Report<'a> {
    fn new(states: &'a BTreeMap<String, NodeState>, founding: Option<&'a Founding>) -> Self {
        Self {
            states,
            founding,
            network: assess(states, founding),
        }
    }

    /// The table, what a cell could not hold, and the network line.
    fn lines(&self) -> Vec<String> {
        let header = [
            "NODE",
            "REACHABLE",
            "CONFIG",
            "CANDIDATE",
            "KEY HOLDER",
            "DISK",
        ]
        .map(String::from);
        let mut rows = vec![header];
        let mut details = Vec::new();
        for (name, state) in self.states {
            rows.push([
                name.clone(),
                yes_no(state.reachable()).to_string(),
                config_cell(&state.config).to_string(),
                candidate_cell(&state.config, self.founding),
                yes_no(state.key_holder.is_some()).to_string(),
                disk_cell(state.key_holder.as_ref()),
            ]);
            if let ConfigState::NotReady(message) | ConfigState::Refused(message) = &state.config {
                details.push(format!("{name}: :{HARVEST_PORT}: {message}"));
            }
            if let Some(Disk::Error(message)) = &state.key_holder {
                details.push(format!("{name}: :{ATTESTATION_RPC_PORT}: {message}"));
            }
        }
        let mut widths = [0; 6];
        for row in &rows {
            for (width, cell) in widths.iter_mut().zip(row) {
                *width = (*width).max(cell.chars().count());
            }
        }
        let mut lines: Vec<String> = rows
            .iter()
            .map(|row| {
                let mut line = String::new();
                for (i, (cell, width)) in row.iter().zip(widths).enumerate() {
                    if i + 1 == row.len() {
                        line.push_str(cell);
                    } else {
                        line.push_str(&format!("{cell:<width$}  "));
                    }
                }
                line.trim_end().to_string()
            })
            .collect();
        lines.extend(details);
        lines.push(String::new());
        lines.push(network_line(&self.network));
        lines
    }

    /// The nodes awaiting their config.
    fn awaiting(&self) -> Vec<&str> {
        self.states
            .iter()
            .filter(|(_, state)| matches!(state.config, ConfigState::Awaiting { .. }))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// The command to run next, with its lead.
    fn next_step(&self, args: &StatusArgs) -> Option<(String, Vec<String>)> {
        let table = args.table_flags();
        let manifest = args.manifest_flag();
        let join = |lead: &str| {
            let awaiting = self.awaiting();
            (!awaiting.is_empty()).then(|| {
                (
                    lead.to_string(),
                    awaiting
                        .iter()
                        .map(|name| {
                            format!(
                                "seismic-tee node configure{table} --name {name} --bootnode \
                                 <ENODE>{manifest}"
                            )
                        })
                        .collect(),
                )
            })
        };
        match &self.network {
            NetworkState::Unfounded { pinned_box } => Some((
                "found it:".to_string(),
                vec![format!(
                    "seismic-tee node configure{table} --genesis-node {pinned_box}{manifest}"
                )],
            )),
            NetworkState::Live { key_holders } => join(&format!(
                "join through a key holder's enode (`seismic_nodeInfo` on {}):",
                key_holders[0]
            )),
            NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
                pinned_box_answered: true,
                ..
            }) => {
                let dir = args
                    .manifest
                    .as_deref()
                    .map(NetworkDir::of_manifest)
                    .filter(|dir| !dir.root().as_os_str().is_empty())
                    .map(|dir| format!(" {}", dir.root().display()))
                    .unwrap_or_default();
                // The fresh boxes keep their names, so their harvest records
                // are already there to replace.
                Some((
                    "re-found: `pulumi destroy`, a fresh `pulumi up`, then".to_string(),
                    vec![format!("seismic-tee node harvest{dir}{table} --force")],
                ))
            }
            NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
                pinned_box_answered: false,
                ..
            }) => Some((
                "watch for it to come up:".to_string(),
                vec![format!("seismic-tee node status --watch{table}{manifest}")],
            )),
            NetworkState::NoReachableKeyHolder(Asked::Some) => {
                join("join through a live peer's enode (its `seismic_nodeInfo`):")
            }
        }
    }

    fn json(&self) -> Value {
        let nodes: serde_json::Map<String, Value> = self
            .states
            .iter()
            .map(|(name, state)| {
                let (candidate, pinned) = match &state.config {
                    ConfigState::Awaiting {
                        candidate_tx_io_public_key,
                        ..
                    } => (
                        Some(candidate_tx_io_public_key.as_str()),
                        self.founding.map(|f| f.pins(candidate_tx_io_public_key)),
                    ),
                    _ => (None, None),
                };
                let disk = match &state.key_holder {
                    Some(Disk::Status(status)) => status.clone(),
                    Some(Disk::Error(message)) => json!({"error": message}),
                    None => Value::Null,
                };
                (
                    name.clone(),
                    json!({
                        "reachable": state.reachable(),
                        "config": config_value(&state.config),
                        "candidate_tx_io_public_key": candidate,
                        "candidate_pinned": pinned,
                        "key_holder": state.key_holder.is_some(),
                        "disk": disk,
                    }),
                )
            })
            .collect();
        let network = match &self.network {
            NetworkState::Live { key_holders } => {
                json!({"state": "live", "key_holders": key_holders})
            }
            NetworkState::Unfounded { pinned_box } => {
                json!({"state": "unfounded", "pinned_box": pinned_box})
            }
            NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
                pinned_box,
                pinned_box_answered,
            }) => json!({
                "state": "no_reachable_key_holder",
                "every_founder_asked": true,
                "pinned_box": pinned_box,
                "pinned_box_answered": pinned_box_answered,
            }),
            NetworkState::NoReachableKeyHolder(Asked::Some) => json!({
                "state": "no_reachable_key_holder",
                "every_founder_asked": false,
            }),
        };
        json!({"nodes": nodes, "network": network})
    }
}

pub async fn run(args: StatusArgs, config: Option<&Path>) -> anyhow::Result<ExitCode> {
    let table = load_table(&args, config)?;
    let founding = load_founding(&args, config)?;
    let targets = probe::targets(&table);
    let client = http::client()?;

    if !args.watch {
        let states = probe_all(&client, &targets).await;
        let report = Report::new(&states, founding.as_ref());
        if args.json {
            println!("{}", report.json());
            return Ok(ExitCode::SUCCESS);
        }
        for line in report.lines() {
            println!("{line}");
        }
        if let Some((lead, commands)) = report.next_step(&args) {
            next_step::print(&lead, &commands);
        }
        return Ok(ExitCode::SUCCESS);
    }

    let interval = Duration::from_secs(args.interval);
    let mut painter = Painter::new();
    let watch = async {
        loop {
            let states = probe_all(&client, &targets).await;
            painter.paint(&Report::new(&states, founding.as_ref()).lines());
            tokio::time::sleep(interval).await;
        }
    };
    tokio::select! {
        () = watch => unreachable!("the watch runs until ctrl-C"),
        _ = tokio::signal::ctrl_c() => {
            println!("\nStopped watching.");
            Ok(ExitCode::from(130))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::test_support::{
        FIXTURE_FOUNDING_TX_IO_PK, FakeServer, candidate_tx_io_public_key, refused_url, rpc_result,
    };

    const GIB: u64 = 1 << 30;

    fn status_args(argv: &[&str]) -> StatusArgs {
        #[derive(clap::Parser)]
        struct Cli {
            #[command(flatten)]
            status: StatusArgs,
        }
        <Cli as clap::Parser>::parse_from(std::iter::once("status").chain(argv.iter().copied()))
            .status
    }

    /// The pinned box re-minted (its candidate is no longer the pin) and the
    /// other founders are configured: the founding root_key is gone.
    fn lost_founding() -> (BTreeMap<String, NodeState>, Founding) {
        let awaiting = |candidate: String| NodeState {
            config: ConfigState::Awaiting {
                node_public_key: "ab".repeat(32),
                candidate_tx_io_public_key: candidate,
            },
            key_holder: None,
        };
        let configured = || NodeState {
            config: ConfigState::Configured,
            key_holder: None,
        };
        let states = BTreeMap::from([
            (
                "alpha".to_string(),
                awaiting(candidate_tx_io_public_key("aa")),
            ),
            ("beta".to_string(), configured()),
        ]);
        let founding = Founding {
            pin: FIXTURE_FOUNDING_TX_IO_PK.to_string(),
            pinned_box: Some("alpha".to_string()),
            founders: ["alpha", "beta"].map(String::from).into(),
        };
        (states, founding)
    }

    /// The fresh boxes keep their names, so the harvest the re-found step
    /// names has records to replace: without --force it would refuse.
    #[test]
    fn the_re_found_step_replaces_the_harvest_in_the_manifests_directory() {
        let (states, founding) = lost_founding();
        let report = Report::new(&states, Some(&founding));
        let args = status_args(&[
            "--node",
            "nodes.json",
            "--manifest",
            "net/network-manifest.json",
        ]);
        let (_, commands) = report.next_step(&args).unwrap();
        assert_eq!(
            commands,
            ["seismic-tee node harvest net --node nodes.json --force"]
        );

        // A manifest in the working directory leaves DIR out, not blank.
        let args = status_args(&[
            "--node",
            "nodes.json",
            "--manifest",
            "network-manifest.json",
        ]);
        let (_, commands) = report.next_step(&args).unwrap();
        assert_eq!(
            commands,
            ["seismic-tee node harvest --node nodes.json --force"]
        );
    }

    #[test]
    fn json_spells_every_config_reading_as_a_word() {
        for (config, value) in [
            (ConfigState::Configured, "configured"),
            (ConfigState::NotReady("HTTP 503".to_string()), "not_ready"),
            (ConfigState::Refused("HTTP 404".to_string()), "refused"),
            (ConfigState::NoAnswer("refused".to_string()), "no_answer"),
        ] {
            let states = BTreeMap::from([(
                "alpha".to_string(),
                NodeState {
                    config,
                    key_holder: None,
                },
            )]);
            let json = Report::new(&states, None).json();
            assert_eq!(json["nodes"]["alpha"]["config"], value, "{json}");
        }
    }

    fn now() -> Instant {
        Instant::now()
    }

    fn status(json: &str) -> Observation {
        Observation::Status(LuksProvisioningStatus::from_result(
            &serde_json::from_str(json).unwrap(),
        ))
    }

    /// Feed a fixed sequence of observations at one instant and collect every
    /// update, stopping at the first terminal one.
    fn run_watch(mut watch: ProvisioningWatch, observations: &[Observation]) -> Vec<Update> {
        let at = now();
        let mut updates = Vec::new();
        for observation in observations {
            let update = watch.observe(at, observation.clone());
            let done = update.done;
            updates.push(update);
            if done {
                break;
            }
        }
        updates
    }

    #[test]
    fn provisioning_renders_a_bar_with_percent_size_and_eta() {
        let line = format_provisioning(GIB, 4 * GIB, Some(291));
        assert!(line.contains("25.0%"), "{line}");
        assert!(line.contains("1.0/4.0 GiB"), "{line}");
        assert!(line.contains("eta 4m51s"), "{line}");

        let line = format_provisioning(0, GIB, None);
        assert!(line.contains("0.0%"), "{line}");
        assert!(!line.contains("eta"), "{line}");
    }

    /// bytes_total 0 is the "just started, no measurement yet" marker — must
    /// not divide by zero.
    #[test]
    fn zero_total_is_indeterminate() {
        assert!(format_provisioning(0, 0, None).contains("starting"));
    }

    #[test]
    fn duration_formats() {
        assert_eq!(duration(45), "45s");
        assert_eq!(duration(291), "4m51s");
        assert_eq!(duration(3725), "1h02m");
    }

    /// Out-of-range percentages must not overflow the bar width.
    #[test]
    fn the_bar_is_clamped() {
        assert_eq!(bar(0.0, 10), "[----------]");
        assert_eq!(bar(100.0, 10), "[##########]");
        assert_eq!(bar(150.0, 10).len(), 12);
        assert_eq!(bar(-5.0, 10).len(), 12);
    }

    /// The wire shape is the attestation service's tagged enum; unknown
    /// states are carried, not crashed on.
    #[test]
    fn reads_every_state_the_service_serializes() {
        let read =
            |json: &str| LuksProvisioningStatus::from_result(&serde_json::from_str(json).unwrap());

        assert_eq!(read(r#"{"state": "idle"}"#), LuksProvisioningStatus::Idle);
        assert_eq!(
            read(r#"{"state": "provisioning", "bytes_done": 5, "bytes_total": 10}"#),
            LuksProvisioningStatus::Provisioning {
                bytes_done: 5,
                bytes_total: 10,
                eta_seconds: None
            }
        );
        assert_eq!(
            read(
                r#"{"state": "provisioning", "bytes_done": 5, "bytes_total": 10, "eta_seconds": 7}"#
            ),
            LuksProvisioningStatus::Provisioning {
                bytes_done: 5,
                bytes_total: 10,
                eta_seconds: Some(7)
            }
        );
        assert_eq!(
            read(r#"{"state": "error", "error": "boom"}"#),
            LuksProvisioningStatus::Error {
                error: "boom".into()
            }
        );
        assert_eq!(
            read(r#"{"state": "unknown"}"#),
            LuksProvisioningStatus::Unknown
        );
        assert_eq!(
            read(r#"{"state": "rebooting"}"#),
            LuksProvisioningStatus::Other("rebooting".into())
        );
        assert_eq!(read("42"), LuksProvisioningStatus::Other("42".into()));
    }

    /// The part that bit us: a transient `error` (the setup service failing an
    /// attempt but restarting under Restart=on-failure) must NOT be terminal.
    #[test]
    fn transient_error_recovers_not_terminal() {
        let updates = run_watch(
            ProvisioningWatch::default(),
            &[
                status(r#"{"state": "error", "error": "data disk not yet attached"}"#),
                status(r#"{"state": "error", "error": "data disk not yet attached"}"#),
                status(r#"{"state": "provisioning", "bytes_done": 5, "bytes_total": 10}"#),
                status(r#"{"state": "idle"}"#),
            ],
        );

        // Only the final idle (after provisioning) is terminal — not the errors.
        assert_eq!(updates.len(), 4);
        assert!(updates[..3].iter().all(|u| !u.done));
        assert!(updates[3].done && updates[3].ok);
        // The transient errors were surfaced as retrying, not fatal.
        assert!(updates.iter().any(|u| u.phase == Phase::Error && !u.done));
    }

    /// Grace collapsed to zero → a stuck error is terminal (restarts exhausted).
    #[test]
    fn persistent_error_is_terminal() {
        let mut watch = ProvisioningWatch::with_grace(CONNECT_TIMEOUT, IDLE_GRACE, Duration::ZERO);
        let at = now();
        let first = watch.observe(at, status(r#"{"state": "error", "error": "boom"}"#));
        assert!(!first.done, "the first reading starts the clock");

        let stuck = watch.observe(
            at + Duration::from_millis(1),
            status(r#"{"state": "error", "error": "boom"}"#),
        );
        assert!(stuck.done && !stuck.ok);
        assert!(stuck.line.contains("boom"), "{}", stuck.line);
        assert!(stuck.line.contains("not recovering"), "{}", stuck.line);
    }

    #[test]
    fn provisioning_then_idle_completes_ok() {
        let updates = run_watch(
            ProvisioningWatch::default(),
            &[
                status(r#"{"state": "provisioning", "bytes_done": 5, "bytes_total": 10}"#),
                status(r#"{"state": "idle"}"#),
            ],
        );
        let last = updates.last().unwrap();
        assert!(last.done && last.ok);
        assert_eq!(last.line, "disk provisioning complete.");
    }

    /// Idle before any provisioning was seen is ambiguous: wait out the grace,
    /// then conclude there is no wipe rather than hang on a fast-unlock boot.
    #[test]
    fn idle_before_provisioning_waits_then_concludes_no_wipe() {
        let mut watch =
            ProvisioningWatch::with_grace(CONNECT_TIMEOUT, Duration::from_secs(60), ERROR_GRACE);
        let at = now();

        let waiting = watch.observe(at, status(r#"{"state": "idle"}"#));
        assert_eq!(waiting.phase, Phase::Waiting);
        assert!(!waiting.done);

        let concluded = watch.observe(at + Duration::from_secs(61), status(r#"{"state": "idle"}"#));
        assert!(concluded.done && concluded.ok);
        assert!(
            concluded.line.contains("no first-boot wipe"),
            "{}",
            concluded.line
        );
    }

    /// Connection refused is the normal state after a POST — until it isn't.
    #[test]
    fn unreachable_is_connecting_until_the_timeout_makes_it_terminal() {
        let mut watch =
            ProvisioningWatch::with_grace(Duration::from_secs(180), IDLE_GRACE, ERROR_GRACE);
        let at = now();

        let connecting = watch.observe(at, Observation::Unreachable);
        assert_eq!(connecting.phase, Phase::Connecting);
        assert!(connecting.transient && !connecting.done);
        assert!(connecting.line.contains(":7878"), "{}", connecting.line);

        let gave_up = watch.observe(at + Duration::from_secs(181), Observation::Unreachable);
        assert!(gave_up.done && !gave_up.ok);
        assert!(
            gave_up.line.contains("never became reachable after 180s"),
            "{}",
            gave_up.line
        );
    }

    /// `unknown` and states this build doesn't know are surfaced as permanent
    /// lines and polled through — never treated as done.
    #[test]
    fn unknown_states_are_reported_and_polled_through() {
        let updates = run_watch(
            ProvisioningWatch::default(),
            &[
                status(r#"{"state": "unknown"}"#),
                status(r#"{"state": "rebooting"}"#),
            ],
        );
        assert_eq!(updates.len(), 2);
        for update in &updates {
            assert_eq!(update.phase, Phase::Waiting);
            assert!(!update.transient && !update.done, "{update:?}");
        }
        assert!(
            updates[1].line.contains("\"rebooting\""),
            "{}",
            updates[1].line
        );
    }

    /// A JSON-RPC error object is an answer, not an outage: polling on cannot
    /// change it, so the watch ends with the message.
    #[test]
    fn an_rpc_error_ends_the_watch() {
        let update = ProvisioningWatch::default()
            .observe(now(), Observation::Refused("RPC error from n: boom".into()));
        assert!(update.done && !update.ok);
        assert!(update.line.contains("boom"));
    }

    #[tokio::test]
    async fn fetch_status_builds_the_request_and_extracts_the_result() {
        let server = FakeServer::serve(vec![(200, rpc_result(json!({"state": "idle"})))]);
        let client = rpc::Client::new(&server.url).unwrap();

        let result = fetch_status(&client).await.unwrap();
        assert_eq!(result, json!({"state": "idle"}));

        let requests = server.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "POST");
        let body: Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body["method"], RPC_METHOD);
        assert_eq!(body["jsonrpc"], "2.0");
    }

    #[tokio::test]
    async fn fetch_status_surfaces_an_rpc_error() {
        let server = FakeServer::serve(vec![(
            200,
            r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32000, "message": "boom"}}"#
                .to_string(),
        )]);
        let client = rpc::Client::new(&server.url).unwrap();

        let err = fetch_status(&client).await.unwrap_err();
        assert!(matches!(err, Error::Rpc { .. }), "{err:?}");
        assert!(err.to_string().contains("boom"), "{err}");
    }

    /// The driver end to end against a node that reports a wipe, then idle.
    #[tokio::test]
    async fn poll_provisioning_runs_the_watch_to_completion() {
        let server = FakeServer::serve(vec![
            (
                200,
                rpc_result(json!({"state": "provisioning", "bytes_done": 5, "bytes_total": 10})),
            ),
            (200, rpc_result(json!({"state": "idle"}))),
        ]);
        let client = rpc::Client::new(&server.url).unwrap();
        let mut phases = Vec::new();

        let ok = poll_provisioning(
            &client,
            ProvisioningWatch::default(),
            Duration::ZERO,
            |update| phases.push(update.phase),
        )
        .await;

        assert!(ok);
        assert_eq!(phases, [Phase::Provisioning, Phase::Done]);
    }

    /// A node that never answers ends the watch as not-ok once the connect
    /// patience is spent, and an RPC error ends it at once. The error object
    /// carries a `code`, as the spec requires and every server sends: without
    /// one the reply is not JSON-RPC, and the client rightly reads it as no
    /// answer.
    #[tokio::test]
    async fn poll_provisioning_ends_on_unreachable_and_on_refused() {
        let mut lines = Vec::new();
        let ok = poll_provisioning(
            &rpc::Client::new(&refused_url()).unwrap(),
            ProvisioningWatch::with_grace(Duration::ZERO, IDLE_GRACE, ERROR_GRACE),
            Duration::ZERO,
            |update| lines.push(update.line.clone()),
        )
        .await;
        assert!(!ok);
        assert!(
            lines.last().unwrap().contains("never became reachable"),
            "{lines:?}"
        );

        let server = FakeServer::serve(vec![(
            200,
            r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32601, "message": "no such method"}}"#
                .to_string(),
        )]);
        let mut lines = Vec::new();
        let ok = poll_provisioning(
            &rpc::Client::new(&server.url).unwrap(),
            ProvisioningWatch::default(),
            Duration::ZERO,
            |update| lines.push(update.line.clone()),
        )
        .await;
        assert!(!ok);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("no such method"), "{lines:?}");
    }
}
