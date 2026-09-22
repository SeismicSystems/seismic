//! `harvest`: collect and DCAP-verify each cohort box's summit keys.
//!
//! ```text
//! seismic-tee network harvest tee/networks/devnet-3
//! ```
//!
//! A founding cohort boots identity-free: each box's `summit-key-holder`
//! generates its summit keypairs in RAM at boot and serves
//! `GET /v1/quote?nonce=…` → `{pubkeys, evidence}` on `:7879` until the box
//! accepts its config POST. Harvest is the step between provisioning and
//! `assemble`: it polls every box's holder, fetches its pubkeys plus a TDX
//! quote over a fresh per-box nonce (`report_data` binds the nonce and both
//! pubkeys, so a quote replayed from an earlier harvest can't satisfy it),
//! DCAP-verifies each quote against the network's intended image
//! measurements, and archives the verified facts under `inputs/harvest/` — the
//! provenance `assemble` pins the founding validator set from.
//!
//! Verification here is load-bearing, not hygiene: consensus membership is
//! gated by whose pubkeys enter the genesis validator set, and founding keys
//! bypass the deposit contract's admission path — so the harvest is the one
//! moment TEE residency can be checked before the set is pinned. The check is
//! the enclave's verify-quote library, against the policy promoted from
//! `inputs/measurements.json` by the same admission compiler `assemble` uses.
//! It is purely preventive: future users and joiners re-run the same
//! verification against the archive rather than trust this run's verdict.
//! Each archived record is a complete input to that check, which
//! `seismic-tee verify-founding` replays over the whole committed
//! directory.
//!
//! The archived file is the verifier's own document, written verbatim: the
//! record beside everything the verdict rested on — the DCAP collateral the
//! verifier consumed, the instant it judged at, digests of the trust anchors
//! it judged with, and its report. Intel's TCB Info, QE Identity and both
//! CRLs carry `nextUpdate` on a roughly 30-day cadence, so without that
//! bundle a founding quote stops being re-verifiable about a month after the
//! founding. The document shape is the verifier's and frozen: it is what the
//! verifier reads back.
//!
//! Any anomaly burns the whole harvest: a quote window already closed (HTTP
//! 410 — the box accepted a config POST), a failed verification, or a cohort
//! whose size doesn't match the authored withdrawal credentials all abort the
//! run and write nothing. A harvested key is trustworthy only if the same box
//! later accepts the real configure cleanly — never retry around a burned
//! harvest; re-found instead (`pulumi destroy` + fresh `up`).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use clap::Args;
use seismic_measurement_admission::promote_measurements;
use seismic_tee_common::network_dir::INPUTS_DIRNAME;
use seismic_tee_common::{Descriptors, NetworkDir, NodeDescriptor, http, next_step};
use seismic_tee_context::load_nodes;
use seismic_verify_quote::{HarvestRecord, SeismicMeasurementPolicy, verify_harvest};
use serde_json::{Value, json};

use crate::args::DirArgs;
use crate::assemble::DEFAULT_ATTESTATION_TYPE;
use crate::founding::{is_bare_hex, load_founder_credentials};
use crate::init::absolute;

/// Holder-readiness polling. The holder starts at network-online — well
/// before the config POST — so an unreachable box is normally just still
/// booting; same cadence as the other cohort gathers (bootnodes, genesis).
pub const POLL_INTERVAL: Duration = Duration::from_secs(5);
pub const HARVEST_TIMEOUT: Duration = Duration::from_secs(15 * 60);
pub const WAIT_LOG_INTERVAL: Duration = Duration::from_secs(30);

/// One cohort box: its node name (its key in the descriptor map, and its
/// filename in `inputs/harvest/`), where its holder is, and the fresh 32-byte
/// nonce minted for this run's quote request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarvestTarget {
    pub name: String,
    /// `http://<ip>:7879`, or wherever the holder answers.
    pub holder_url: String,
    pub nonce: [u8; 32],
}

impl HarvestTarget {
    /// A target for one descriptor, with a nonce minted here and never handed
    /// to anyone but the box, so its quote can only answer this request.
    pub fn new(name: &str, descriptor: &NodeDescriptor) -> Self {
        Self {
            name: name.to_string(),
            holder_url: descriptor.key_holder_url(),
            nonce: rand::random(),
        }
    }

    fn nonce_hex(&self) -> String {
        hex::encode(self.nonce)
    }
}

/// What a holder served: the pubkeys in summit's keystore wire spelling and
/// the evidence, verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quote {
    pub node_public_key: String,
    pub consensus_public_key: String,
    pub evidence: Value,
}

/// Why one quote fetch did not produce a quote.
#[derive(Debug)]
pub enum FetchError {
    /// HTTP 410: the box already took a config POST. Burns the harvest.
    WindowClosed(String),
    /// Any other 4xx: not a boot-tail condition. Burns the harvest.
    Rejected(u16, String),
    /// A malformed response body: retrying can't fix a holder serving the
    /// wrong shape. Burns the harvest.
    Malformed(String),
    /// Transport failure or 5xx: the normal boot tail, retried.
    Retry(String),
}

/// Fetch one box's `{pubkeys, evidence}` from its summit-key-holder.
pub async fn fetch_quote(
    client: &reqwest::Client,
    target: &HarvestTarget,
) -> Result<Quote, FetchError> {
    let url = format!("{}/v1/quote", target.holder_url);
    let response = client
        .get(&url)
        .query(&[("nonce", target.nonce_hex())])
        .send()
        .await
        .map_err(|e| FetchError::Retry(e.to_string()))?;
    let status = response.status();
    if status.as_u16() == 410 {
        return Err(FetchError::WindowClosed(url));
    }
    if status.is_client_error() {
        return Err(FetchError::Rejected(status.as_u16(), url));
    }
    if !status.is_success() {
        return Err(FetchError::Retry(format!(
            "HTTP {} from {url}",
            status.as_u16()
        )));
    }
    let body: Value = response
        .json()
        .await
        .map_err(|e| FetchError::Malformed(format!("{url}: response is not JSON: {e}")))?;
    let Some(object) = body.as_object() else {
        return Err(FetchError::Malformed(format!(
            "{url}: expected a JSON object, got {body}"
        )));
    };
    let key = |name: &str, nbytes: usize| -> Result<String, FetchError> {
        match object.get(name).and_then(Value::as_str) {
            Some(value) if is_bare_hex(value, nbytes) => Ok(value.to_string()),
            other => Err(FetchError::Malformed(format!(
                "{url}: {name} is not {} lowercase hex chars: {}",
                2 * nbytes,
                other.map_or_else(|| "absent".to_string(), |v| format!("{v:?}")),
            ))),
        }
    };
    let node_public_key = key("node_public_key", 32)?;
    let consensus_public_key = key("consensus_public_key", 48)?;
    let Some(evidence) = object.get("evidence").filter(|e| e.is_object()) else {
        return Err(FetchError::Malformed(format!(
            "{url}: response carries no evidence object"
        )));
    };
    Ok(Quote {
        node_public_key,
        consensus_public_key,
        evidence: evidence.clone(),
    })
}

/// Poll every target's holder until each serves its quote, or `timeout`.
///
/// Round-robin like the other cohort gathers, so a slow box doesn't serialize
/// behind the others. Transport errors and 5xx are the normal boot tail —
/// retried until the deadline, then aborted with a per-box report. A closed
/// quote window (410) or any other 4xx burns the harvest immediately: waiting
/// can't fix a box that already took its config POST, or a holder that
/// rejects well-formed requests.
pub async fn collect_quotes(
    client: &reqwest::Client,
    targets: &[HarvestTarget],
    timeout: Duration,
    interval: Duration,
) -> anyhow::Result<BTreeMap<String, Quote>> {
    let mut quotes = BTreeMap::new();
    let mut last_error: BTreeMap<String, String> = BTreeMap::new();
    let started = Instant::now();
    let deadline = started + timeout;
    let mut next_log = started;
    loop {
        for target in targets {
            if quotes.contains_key(&target.name) {
                continue;
            }
            match fetch_quote(client, target).await {
                Ok(quote) => {
                    println!("  ✓ {}: pubkeys + quote harvested", target.name);
                    quotes.insert(target.name.clone(), quote);
                }
                Err(FetchError::WindowClosed(_)) => bail!(
                    "{}: quote window closed (HTTP 410) — the box already accepted a config POST, \
                     so its founding keys are not harvestable. The harvest is burned: re-found \
                     (`pulumi destroy` + fresh `up`) rather than retrying around it.",
                    target.name
                ),
                Err(FetchError::Rejected(status, url)) => bail!(
                    "{}: holder rejected the quote request (HTTP {status} from {url}) — not a \
                     boot-tail condition; check that the image and this CLI agree on the holder \
                     API.",
                    target.name
                ),
                Err(FetchError::Malformed(message)) => bail!("{}: {message}", target.name),
                Err(FetchError::Retry(message)) => {
                    last_error.insert(target.name.clone(), message);
                }
            }
        }
        let pending: Vec<&str> = targets
            .iter()
            .filter(|t| !quotes.contains_key(&t.name))
            .map(|t| t.name.as_str())
            .collect();
        if pending.is_empty() {
            return Ok(quotes);
        }
        let now = Instant::now();
        if now >= deadline {
            let listing: Vec<String> = pending
                .iter()
                .map(|name| {
                    format!(
                        "  ✗ {name}: {}",
                        last_error
                            .get(*name)
                            .map_or("never answered", String::as_str)
                    )
                })
                .collect();
            bail!(
                "{} box(es) never served a founding quote after {}s (holder not up?):\n{}",
                pending.len(),
                timeout.as_secs(),
                listing.join("\n")
            );
        }
        if now >= next_log {
            println!(
                "waiting for founding quotes ({}s elapsed, {}s until timeout): {}",
                now.duration_since(started).as_secs(),
                deadline.saturating_duration_since(now).as_secs(),
                pending.join(", ")
            );
            next_log = now + WAIT_LOG_INTERVAL;
        }
        tokio::time::sleep(interval).await;
    }
}

/// Abort if two boxes served the same pubkey.
///
/// Summit's genesis keys validator accounts by node pubkey, so a repeated key
/// silently collapses the set — and two boxes holding the same consensus key
/// is accidental-equivocation material. Either way the cohort is not the N
/// distinct founders being pinned: burn.
pub fn assert_unique_keys(quotes: &BTreeMap<String, Quote>) -> anyhow::Result<()> {
    for (field, key_of) in [
        (
            "node_public_key",
            (|q: &Quote| q.node_public_key.as_str()) as fn(&Quote) -> &str,
        ),
        ("consensus_public_key", |q: &Quote| {
            q.consensus_public_key.as_str()
        }),
    ] {
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for (name, quote) in quotes {
            let key = key_of(quote);
            if let Some(first) = seen.get(key) {
                bail!(
                    "{first} and {name} served the same {field} ({key}); the cohort is not the \
                     distinct founder set being pinned. The harvest is burned: re-found."
                );
            }
            seen.insert(key, name);
        }
    }
    Ok(())
}

/// One box's harvest record: the nonce this run minted, the pubkeys its holder
/// served, and the evidence whose `report_data` binds all three.
///
/// The verifier's input. The archive it renders carries these same four
/// fields beside the verdict's provenance, so the document a later reader
/// re-verifies is the one the verifier passed.
pub fn build_record(target: &HarvestTarget, quote: &Quote) -> Value {
    json!({
        "harvest_nonce": target.nonce_hex(),
        "node_public_key": quote.node_public_key,
        "consensus_public_key": quote.consensus_public_key,
        "evidence": quote.evidence,
    })
}

/// DCAP-verify one harvest record, live, against `policy`. Returns the
/// founding archive document the verifier rendered for it: the record, the
/// bundle the verdict depended on, and the verifier's report. A failure burns
/// the harvest: a founding key whose quote doesn't verify must never reach
/// `assemble`.
pub async fn verify_record(
    name: &str,
    record: &Value,
    policy: &SeismicMeasurementPolicy,
    pccs_url: Option<&str>,
) -> anyhow::Result<String> {
    let verdict = async {
        let record: HarvestRecord =
            serde_json::from_value(record.clone()).context("the harvest record does not parse")?;
        let verified = verify_harvest(record, policy.clone(), pccs_url.map(str::to_string)).await?;
        // The verifier is the only component that knows which bundle it used;
        // it renders the archive document and reads it back before handing it
        // over, so a document that cannot be replayed fails the harvest here.
        verified.archive_document()
    }
    .await;
    verdict.map_err(|error| {
        anyhow::anyhow!(
            "{name}: quote verification failed:\n{error:?}\nThe harvest is burned: re-found rather \
             than retrying around it."
        )
    })
}

/// Refuse to clobber an existing harvest unless `--force`.
///
/// The archive is founding provenance — the nonces it holds are what make the
/// archived quotes re-verifiable — so replacing it is a deliberate re-harvest,
/// not a default.
pub fn check_overwrite(dir: &NetworkDir, names: &[String], force: bool) -> anyhow::Result<()> {
    let existing: Vec<&str> = names
        .iter()
        .filter(|name| dir.harvest_record(name).exists())
        .map(String::as_str)
        .collect();
    if !existing.is_empty() && !force {
        bail!(
            "refusing to overwrite existing harvest file(s) in {}: {} — pass --force for a fresh \
             harvest (new nonces; the archived provenance is replaced)",
            dir.harvest().display(),
            existing.join(", ")
        );
    }
    Ok(())
}

/// Write the archive, one document per box, verbatim as the verifier rendered
/// it: what is archived is what was verified. Returns the paths written.
///
/// Written only after every box passed, so a burned harvest leaves nothing
/// behind.
pub fn save_harvest(
    dir: &NetworkDir,
    archives: &BTreeMap<String, String>,
) -> anyhow::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dir.harvest())
        .with_context(|| format!("creating {}", dir.harvest().display()))?;
    let mut written = Vec::with_capacity(archives.len());
    for (name, document) in archives {
        let path = dir.harvest_record(name);
        std::fs::write(&path, document).with_context(|| format!("writing {}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}

/// The cohort as harvest targets, one per descriptor, in node-name order.
pub fn targets(descriptors: &Descriptors) -> Vec<HarvestTarget> {
    descriptors
        .iter()
        .map(|(name, descriptor)| HarvestTarget::new(name, descriptor))
        .collect()
}

/// Count the authored credentials against the live cohort, before any quote
/// is fetched, so the fix costs nothing: `assemble` pairs the i-th address
/// with the i-th box in node-name order, so a mismatch would leave a box
/// unpinnable or pin a set other than the one the founders authored for.
pub fn check_founders(dir: &NetworkDir, cohort: &[String]) -> anyhow::Result<()> {
    let founders = load_founder_credentials(&dir.founders())?;
    if founders.len() != cohort.len() {
        bail!(
            "{} carries {} withdrawal credential(s) but the cohort has {} box(es) ({}) — author \
             one address per founding node",
            dir.founders().display(),
            founders.len(),
            cohort.len(),
            cohort.join(", ")
        );
    }
    Ok(())
}

#[derive(Debug, Args)]
pub struct HarvestArgs {
    /// Network directory (from `init`): reads the authored
    /// inputs/founder-withdrawal-credentials.json and
    /// inputs/measurements.json, and writes the harvested facts to
    /// inputs/harvest/. Omit it to use the current context's network.
    #[command(flatten)]
    pub dir: DirArgs,

    /// Cohort's node table: `pulumi stack output nodes --json`, i.e.
    /// {<name>: {public_ip, fqdn}, …} — every node in it is harvested. Omit
    /// it to use the current context's network.
    #[arg(long, value_name = "FILE")]
    pub nodes: Option<PathBuf>,

    /// Platform the policy promoted from inputs/measurements.json pins.
    #[arg(long, value_name = "TYPE", default_value = DEFAULT_ATTESTATION_TYPE)]
    pub attestation_type: String,

    /// PCCS URL for DCAP collateral, instead of the verifier's default
    /// provider.
    #[arg(long, value_name = "URL")]
    pub pccs_url: Option<String>,

    /// Overwrite existing harvest file(s) — a fresh harvest with new nonces,
    /// replacing the archived provenance.
    #[arg(long)]
    pub force: bool,
}

pub async fn run(args: HarvestArgs) -> anyhow::Result<ExitCode> {
    let root = args.dir.load()?;
    if !root.is_dir() {
        bail!("network directory not found: {}", root.display());
    }
    let dir = NetworkDir::new(absolute(&root)?);
    let measurements = dir.input_measurements();
    if !measurements.is_file() {
        bail!(
            "{} not found — authored inputs live under {INPUTS_DIRNAME}/; scaffold them with `init`",
            measurements.display()
        );
    }
    if !dir.founders().is_file() {
        bail!(
            "{} not found — author it as a JSON array of the founders' withdrawal credentials \
             (0x-prefixed addresses), one per founding node",
            dir.founders().display()
        );
    }
    // The cohort is the node table, whole: its keys are the harvest's node
    // names (the inputs/harvest/ filenames, and the order the authored
    // withdrawal credentials pair against), unique by construction.
    let descriptors = load_nodes(args.nodes.as_deref(), &args.dir.context, "--nodes")?;
    let targets = targets(&descriptors);
    let names: Vec<String> = targets.iter().map(|t| t.name.clone()).collect();
    check_founders(&dir, &names)?;
    check_overwrite(&dir, &names, args.force)?;

    // Promote the policy before touching the cohort: a rejected measurements
    // file must fail while the fix still costs nothing.
    let raw = std::fs::read(&measurements)
        .with_context(|| format!("reading {}", measurements.display()))?;
    let policy_bytes = promote_measurements(&raw, None, Some(&args.attestation_type))
        .with_context(|| format!("{}", measurements.display()))?;
    let policy = SeismicMeasurementPolicy::from_json_bytes(&policy_bytes)
        .context("loading the promoted measurement policy")?;

    println!("Harvesting founding keys from {} box(es)...", targets.len());
    let client = http::client()?;
    let quotes = collect_quotes(&client, &targets, HARVEST_TIMEOUT, POLL_INTERVAL).await?;
    assert_unique_keys(&quotes)?;

    // Collateral fetches go over TLS; see `node verify` for why the provider
    // is chosen here.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let mut archives = BTreeMap::new();
    for target in &targets {
        let record = build_record(target, &quotes[&target.name]);
        let document =
            verify_record(&target.name, &record, &policy, args.pccs_url.as_deref()).await?;
        println!(
            "  ✓ {}: quote DCAP-verified against the policy",
            target.name
        );
        archives.insert(target.name.clone(), document);
    }

    for path in save_harvest(&dir, &archives)? {
        println!("wrote {}", path.display());
    }
    println!(
        "Harvest complete: {} founding box(es) verified and archived under {}",
        archives.len(),
        dir.harvest().display()
    );
    next_step::print(
        "",
        &[format!(
            "seismic-tee network assemble{}",
            args.dir.as_args()
        )],
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use seismic_tee_common::test_support::{FakeServer, refused_url};

    use super::*;
    use crate::founding::tests::{azure_evidence, no_attestation_evidence};

    const NODE_KEY: &str = "abababababababababababababababababababababababababababababababab";

    fn consensus_key() -> String {
        "cd".repeat(48)
    }

    fn quote_body(node_key: &str, consensus_key: &str) -> String {
        json!({
            "node_public_key": node_key,
            "consensus_public_key": consensus_key,
            "evidence": azure_evidence(),
        })
        .to_string()
    }

    fn target(name: &str, holder_url: &str) -> HarvestTarget {
        HarvestTarget {
            name: name.to_string(),
            holder_url: holder_url.to_string(),
            nonce: [0x11; 32],
        }
    }

    fn quote() -> Quote {
        Quote {
            node_public_key: NODE_KEY.to_string(),
            consensus_public_key: consensus_key(),
            evidence: azure_evidence(),
        }
    }

    /// The request carries this run's nonce; the answer is the three fields.
    #[tokio::test]
    async fn fetch_quote_passes_the_nonce_and_reads_the_quote() {
        let server = FakeServer::serve(vec![(200, quote_body(NODE_KEY, &consensus_key()))]);
        let client = http::client().unwrap();
        let fetched = fetch_quote(&client, &target("node-1", &server.url))
            .await
            .unwrap();
        assert_eq!(fetched, quote());

        let requests = server.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(
            requests[0].path,
            format!("/v1/quote?nonce={}", "11".repeat(32))
        );
    }

    /// 410 is a closed window and burns; another 4xx is a rejection and burns;
    /// a 5xx and no answer are retried; a malformed body burns.
    #[tokio::test]
    async fn fetch_quote_sorts_failures_by_what_waiting_could_fix() {
        let client = http::client().unwrap();

        let server = FakeServer::serve(vec![(410, "gone".to_string())]);
        assert!(matches!(
            fetch_quote(&client, &target("n", &server.url)).await,
            Err(FetchError::WindowClosed(_))
        ));

        let server = FakeServer::serve(vec![(400, "bad nonce".to_string())]);
        assert!(matches!(
            fetch_quote(&client, &target("n", &server.url)).await,
            Err(FetchError::Rejected(400, _))
        ));

        let server = FakeServer::serve(vec![(500, "boom".to_string())]);
        assert!(matches!(
            fetch_quote(&client, &target("n", &server.url)).await,
            Err(FetchError::Retry(_))
        ));
        assert!(matches!(
            fetch_quote(&client, &target("n", &refused_url())).await,
            Err(FetchError::Retry(_))
        ));

        for body in [
            quote_body(&NODE_KEY.to_uppercase(), &consensus_key()),
            quote_body(NODE_KEY, "cd"),
            json!({"node_public_key": NODE_KEY, "consensus_public_key": consensus_key()})
                .to_string(),
            "[]".to_string(),
        ] {
            let server = FakeServer::serve(vec![(200, body.clone())]);
            assert!(
                matches!(
                    fetch_quote(&client, &target("n", &server.url)).await,
                    Err(FetchError::Malformed(_))
                ),
                "{body}"
            );
        }
    }

    /// A still-booting box is retried until it answers; a box that never does
    /// is listed at the deadline, and only it.
    #[tokio::test]
    async fn collect_quotes_retries_the_boot_tail_and_lists_the_stuck() {
        let client = http::client().unwrap();
        let late = FakeServer::serve_after(
            Duration::from_millis(200),
            vec![(200, quote_body(NODE_KEY, &consensus_key()))],
        );
        let quotes = collect_quotes(
            &client,
            &[target("node-1", &late.url)],
            Duration::from_secs(10),
            Duration::from_millis(20),
        )
        .await
        .unwrap();
        assert_eq!(quotes["node-1"], quote());

        let up = FakeServer::serve(vec![(200, quote_body(NODE_KEY, &consensus_key()))]);
        let err = collect_quotes(
            &client,
            &[target("node-1", &up.url), target("node-2", &refused_url())],
            Duration::from_millis(100),
            Duration::from_millis(20),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("1 box(es) never served a founding quote"),
            "{err}"
        );
        assert!(err.contains("✗ node-2"), "{err}");
        assert!(!err.contains("✗ node-1"), "{err}");
    }

    #[tokio::test]
    async fn a_closed_window_or_a_rejection_burns_the_harvest_at_once() {
        let client = http::client().unwrap();
        let server = FakeServer::serve(vec![(410, "gone".to_string())]);
        let err = collect_quotes(
            &client,
            &[target("node-1", &server.url)],
            Duration::from_secs(10),
            Duration::from_millis(20),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.contains("node-1: quote window closed"), "{err}");
        assert!(err.contains("burned"), "{err}");

        let server = FakeServer::serve(vec![(404, "no".to_string())]);
        let err = collect_quotes(
            &client,
            &[target("node-1", &server.url)],
            Duration::from_secs(10),
            Duration::from_millis(20),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.contains("holder rejected the quote request"), "{err}");
        assert!(err.contains("HTTP 404"), "{err}");
    }

    #[test]
    fn repeated_keys_burn() {
        let mut quotes = BTreeMap::new();
        quotes.insert("node-1".to_string(), quote());
        quotes.insert(
            "node-2".to_string(),
            Quote {
                node_public_key: "ef".repeat(32),
                consensus_public_key: "12".repeat(48),
                ..quote()
            },
        );
        assert_unique_keys(&quotes).unwrap();

        quotes.insert("node-3".to_string(), quote());
        let err = assert_unique_keys(&quotes).unwrap_err().to_string();
        assert!(err.contains("node-1 and node-3"), "{err}");
        assert!(err.contains("node_public_key"), "{err}");

        quotes.insert(
            "node-3".to_string(),
            Quote {
                node_public_key: "34".repeat(32),
                ..quote()
            },
        );
        let err = assert_unique_keys(&quotes).unwrap_err().to_string();
        assert!(err.contains("consensus_public_key"), "{err}");
    }

    /// The record is the verifier's input, in the frozen shape: the four
    /// fields, the nonce as hex, the evidence verbatim.
    #[test]
    fn the_record_is_the_verifiers_input() {
        let record = build_record(&target("node-1", "http://h:7879"), &quote());
        assert_eq!(
            record,
            json!({
                "harvest_nonce": "11".repeat(32),
                "node_public_key": NODE_KEY,
                "consensus_public_key": consensus_key(),
                "evidence": azure_evidence(),
            })
        );
        let parsed: HarvestRecord = serde_json::from_value(record).unwrap();
        assert_eq!(parsed.harvest_nonce, "11".repeat(32));
    }

    /// A record whose evidence carries no attestation fails verification
    /// before any collateral is fetched, and the failure burns by name.
    #[tokio::test]
    async fn a_failed_verification_burns_by_name() {
        let policy =
            SeismicMeasurementPolicy::from_json_bytes(crate::gates::tests::FIXTURE_POLICY).unwrap();
        let record = build_record(
            &target("node-1", "http://h:7879"),
            &Quote {
                evidence: no_attestation_evidence(),
                ..quote()
            },
        );
        let err = format!(
            "{:?}",
            verify_record("node-1", &record, &policy, None)
                .await
                .unwrap_err()
        );
        assert!(
            err.starts_with("node-1: quote verification failed"),
            "{err}"
        );
        assert!(err.contains("burned"), "{err}");
    }

    /// One document per box, in the frozen layout, verbatim.
    #[test]
    fn save_writes_one_document_per_box() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        let mut archives = BTreeMap::new();
        archives.insert("node-1".to_string(), "{\n  \"version\": 1\n}\n".to_string());

        let written = save_harvest(&dir, &archives).unwrap();
        assert_eq!(written, [dir.harvest_record("node-1")]);
        assert_eq!(
            std::fs::read_to_string(dir.harvest_record("node-1")).unwrap(),
            archives["node-1"]
        );
    }

    /// An existing archive refuses without --force; fresh dirs and --force
    /// pass.
    #[test]
    fn check_overwrite_guards_the_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        let names = vec!["node-1".to_string(), "node-2".to_string()];
        check_overwrite(&dir, &names, false).unwrap();

        std::fs::create_dir_all(dir.harvest()).unwrap();
        std::fs::write(dir.harvest_record("node-2"), "{}").unwrap();
        let err = check_overwrite(&dir, &names, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("refusing to overwrite"), "{err}");
        assert!(err.contains("node-2"), "{err}");
        assert!(!err.contains("node-1"), "{err}");
        check_overwrite(&dir, &names, true).unwrap();

        std::fs::write(dir.harvest_record("node-1"), "{}").unwrap();
        let err = check_overwrite(&dir, &names, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("node-1, node-2"), "{err}");
    }

    #[test]
    fn the_cohort_is_the_descriptor_map_in_name_order_with_fresh_nonces() {
        let descriptors: Descriptors = serde_json::from_str::<serde_json::Map<String, Value>>(
            r#"{"node-2": {"public_ip": "203.0.113.8", "fqdn": "n2"},
                "node-1": {"public_ip": "203.0.113.7", "fqdn": "n1"}}"#,
        )
        .unwrap()
        .into_iter()
        .map(|(name, entry)| {
            (
                name,
                NodeDescriptor {
                    public_ip: entry["public_ip"].as_str().unwrap().to_string(),
                    fqdn: entry["fqdn"].as_str().unwrap().to_string(),
                },
            )
        })
        .collect();
        let targets = targets(&descriptors);
        assert_eq!(
            targets.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
            ["node-1", "node-2"]
        );
        assert_eq!(targets[0].holder_url, "http://203.0.113.7:7879");
        assert_ne!(targets[0].nonce, targets[1].nonce);
        assert_ne!(targets[0].nonce, [0; 32]);
    }

    #[test]
    fn check_founders_counts_the_cohort() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        std::fs::create_dir_all(dir.inputs()).unwrap();
        std::fs::write(
            dir.founders(),
            format!(r#"["0x{}", "0x{}"]"#, "01".repeat(20), "02".repeat(20)),
        )
        .unwrap();
        check_founders(&dir, &["a".to_string(), "b".to_string()]).unwrap();
        let err = check_founders(&dir, &["a".to_string()])
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("2 withdrawal credential(s) but the cohort has 1 box(es) (a)"),
            "{err}"
        );
    }
}
