//! `status`: watch a node's first-boot LUKS-provisioning progress.
//!
//! The attestation service serves `getLuksProvisioningStatus` on `:7878`
//! (JSON-RPC) for the duration of the first-boot disk wipe — the one long
//! (1h+), otherwise opaque phase. This module polls it and renders a progress
//! bar, and is the shared poller behind both `seismic-tee node status` and
//! `configure`'s default post-POST wait (and the founding cohort's dashboard,
//! which is why the state machine renders nothing itself).
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

use std::io::{IsTerminal as _, Write as _};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use clap::Args;
use jsonrpsee::rpc_params;
use seismic_tee_common::http::ATTESTATION_RPC_PORT;
use seismic_tee_common::{Error, rpc};
use serde::Deserialize;
use serde_json::Value;

use crate::args::NodeArgs;

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
/// `setup-persistent-luks` runs under `Restart=on-failure` and retries
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
pub struct StatusArgs {
    #[command(flatten)]
    pub node: NodeArgs,

    /// Print the current status as JSON and exit (no polling).
    #[arg(long)]
    pub once: bool,

    /// Poll interval (default 5s).
    #[arg(long, value_name = "SECONDS", default_value_t = POLL_INTERVAL.as_secs())]
    pub interval: u64,
}

pub async fn run(args: StatusArgs) -> anyhow::Result<ExitCode> {
    let (_, descriptor) = args.node.load()?;
    let client = rpc::Client::new(&descriptor.attestation_rpc_url())?;

    if args.once {
        let status = match fetch_status(&client).await {
            Ok(status) => status,
            // The service answered: its error already names the endpoint and
            // the reason, and "not reachable" would be untrue.
            Err(error @ Error::Rpc { .. }) => return Err(error.into()),
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("attestation service :{ATTESTATION_RPC_PORT} not reachable")
                });
            }
        };
        println!("{status}");
        return Ok(ExitCode::SUCCESS);
    }

    tokio::select! {
        ok = watch_luks_provisioning(&client, Duration::from_secs(args.interval)) => {
            Ok(if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE })
        }
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
    use crate::test_support::{FakeServer, refused_url, rpc_result};

    const GIB: u64 = 1 << 30;

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
