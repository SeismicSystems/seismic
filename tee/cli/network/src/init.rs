//! `init`: scaffold a network directory's authored inputs.
//!
//! The only command that takes loose files — each a local path or an
//! `https://` URL — and the first step of a founding:
//!
//! ```text
//! seismic-tee network init tee/networks/devnet-3 \
//!     --reth-genesis https://raw.githubusercontent.com/.../dev.json \
//!     --summit-genesis tee/networks/summit-genesis-starter.toml \
//!     --measurements https://github.com/SeismicSystems/seismic-images/releases/download/<image>/measurements.azure-tdx.json \
//!     --founders 4
//! ```
//!
//! The three inputs are copied into `inputs/` verbatim, each gated on parsing
//! as its format (the measurements additionally on carrying the
//! `measurement_id` of the image they measure) — except that a summit genesis
//! with an empty or missing `namespace` gets `namespace = <name>` filled in.
//! `--founders N` writes N placeholder withdrawal credentials, obviously fake
//! (`0x00…0<i>`) so a set that survives into a network anyone cares about
//! shows on sight. The founder edits all four in place, then provisions and
//! harvests the cohort before `assemble` derives the artifact set into the
//! directory's top level — inputs and the committed artifacts live together,
//! so the directory is the whole network.
//!
//! Creating a network selects it: once the scaffold is written, `init`
//! registers `[networks.<name>] dir` in the context file and sets `current`
//! to it, the way `kind create cluster` and `gcloud container clusters
//! create` both set the current context on creation. Every founding command
//! after this one can drop its `DIR` — the context supplies it — and
//! `--name <node>` works against the cohort the moment `ctx set-nodes`
//! imports it.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context as _, bail};
use clap::Args;
use seismic_tee_common::network_dir::{
    FOUNDERS_FILENAME, MEASUREMENTS_FILENAME, RETH_GENESIS_FILENAME, SUMMIT_GENESIS_FILENAME,
};
use seismic_tee_common::{NetworkDir, next_step};
use seismic_tee_context::config::Network;
use seismic_tee_context::{Context, ContextArgs, Selection, write};

/// How long one input fetch may take.
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(30);

/// The runbook's provisioning section, by URL: the CLI may be installed where
/// the repo is not.
pub const RUNBOOK_PROVISION_URL: &str = "https://github.com/SeismicSystems/deploy/blob/main/tee/docs/runbook-devnet.md#2-provision-the-cohort";

/// Suffix for parse-gate errors on fetched content: the classic mistake is
/// pasting a GitHub HTML page URL, which fetches fine but isn't the file.
fn raw_url_hint(source: &str) -> &'static str {
    if source.starts_with("https://") {
        " — for GitHub files pass the raw content URL (raw.githubusercontent.com), not the HTML \
         page"
    } else {
        ""
    }
}

/// The client input URLs are fetched with: https on the request *and on every
/// redirect hop* (a chain that starts https can still be downgraded
/// mid-redirect). Beyond that any host is accepted — founders running their
/// own forks fetch from their own mirrors, and the founder reviews every input
/// before assemble's gates run.
pub fn fetch_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(seismic_tee_common::http::USER_AGENT)
        .timeout(FETCH_TIMEOUT)
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.url().scheme() == "https" {
                attempt.follow()
            } else {
                let url = attempt.url().clone();
                attempt.error(format!("redirected through non-https URL {url}"))
            }
        }))
        .build()?)
}

/// Read one authored input from a local path or an `https://` URL.
///
/// URL inputs let `init` run without sibling checkouts: all SeismicSystems
/// repos are public, so raw.githubusercontent.com URLs work anonymously, and
/// since `init` copies inputs into `inputs/` the fetch is one-time — the
/// network directory stays self-contained.
pub async fn read_input_source(client: &reqwest::Client, source: &str) -> anyhow::Result<Vec<u8>> {
    if source.starts_with("http://") {
        bail!("insecure URL rejected (use https://): {source}");
    }
    if source.starts_with("https://") {
        let response = client
            .get(source)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|e| anyhow::anyhow!("failed to fetch {source}: {e}"))?;
        return Ok(response
            .bytes()
            .await
            .map_err(|e| anyhow::anyhow!("failed to fetch {source}: {e}"))?
            .to_vec());
    }
    let path = Path::new(source);
    if !path.is_file() {
        bail!("input file not found: {source}");
    }
    std::fs::read(path).with_context(|| format!("reading {source}"))
}

/// Apply init's namespace rule to an authored summit genesis.
///
/// `namespace` is the signature domain separator and must be unique per
/// network, so a shared starter (like the committed
/// `summit-genesis-starter.toml`) cannot choose one: it carries the visible
/// fill-me slot `namespace = ""`. init fills `namespace = <network name>` when
/// the authored genesis leaves it empty or omits the key — an empty string is
/// never authorable intent (assemble would accept a namespace no one chose) —
/// and copies a non-empty namespace untouched. The authored bytes are
/// otherwise verbatim.
pub fn fill_summit_namespace(raw: &[u8], name: &str, source: &str) -> anyhow::Result<Vec<u8>> {
    let text = std::str::from_utf8(raw)
        .map_err(|e| anyhow::anyhow!("{source} is not valid TOML: {e}{}", raw_url_hint(source)))?;
    let parsed: toml::Table = toml::from_str(text)
        .map_err(|e| anyhow::anyhow!("{source} is not valid TOML: {e}{}", raw_url_hint(source)))?;
    if parsed
        .get("namespace")
        .and_then(toml::Value::as_str)
        .is_some_and(|ns| !ns.is_empty())
    {
        return Ok(raw.to_vec());
    }
    // A JSON string is a valid TOML basic string for a simple name.
    let filled = format!(
        "namespace = {}",
        serde_json::to_string(name).expect("a string serializes")
    );
    if parsed.contains_key("namespace") {
        let mut rewritten = String::with_capacity(text.len() + filled.len());
        let mut count = 0;
        for line in text.split_inclusive('\n') {
            let body = line.strip_suffix('\n').unwrap_or(line);
            let is_namespace_line = count == 0 && {
                let rest = body.strip_prefix("namespace").unwrap_or("");
                rest != body && rest.trim_start().starts_with('=')
            };
            if is_namespace_line {
                count += 1;
                rewritten.push_str(&filled);
                if line.ends_with('\n') {
                    rewritten.push('\n');
                }
            } else {
                rewritten.push_str(line);
            }
        }
        if count != 1 {
            bail!(
                "{source} has an empty namespace that init cannot rewrite (no top-level \
                 `namespace = ...` line) — set it to the network's unique namespace"
            );
        }
        return Ok(rewritten.into_bytes());
    }
    let mut out = text.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("\n# Unique per network: namespaces summit signatures to this chain.\n");
    out.push_str(&filled);
    out.push('\n');
    Ok(out.into_bytes())
}

/// Gate a measurements input on carrying the id of the image it measures.
///
/// The measurements file is the only binding between a network's PCR
/// allowlist and an image: seismic-images' `make measure` stamps the versioned
/// VHD filename into it, and promotion reads the id from there. `init` gates
/// on the stamp so a missing one surfaces while the operator still holds loose
/// files, not after the cohort has been provisioned and harvested. A promoted
/// policy is a record list, each record carrying its own id.
pub fn require_measurement_id(raw: &[u8], source: &str) -> anyhow::Result<()> {
    let value: serde_json::Value = serde_json::from_slice(raw)
        .map_err(|e| anyhow::anyhow!("{source} is not valid JSON: {e}{}", raw_url_hint(source)))?;
    if value.is_object() && value.get("measurement_id").is_none() {
        bail!(
            "{source} carries no measurement_id — re-export the measurements with seismic-images' \
             `make measure`, which stamps the versioned image filename these PCRs measure into \
             the file"
        );
    }
    Ok(())
}

/// The loose files `init` scaffolds a network directory from.
#[derive(Debug, Clone)]
pub struct InitInputs<'a> {
    pub name: &'a str,
    pub reth_genesis: &'a str,
    pub measurements: &'a str,
    pub summit_genesis: &'a str,
    pub founders: usize,
}

/// Everything the layout owns under `dir` that exists: the authored inputs
/// and the harvest (`inputs/`), the derived artifact set, and the infra
/// state (`nodes/`). This is what `--force` starts over — and nothing else
/// in the directory, so a `--force` aimed at the wrong directory removes no
/// file that is not a network directory's.
fn network_state(dir: &NetworkDir) -> Vec<PathBuf> {
    [
        dir.inputs(),
        dir.manifest(),
        dir.policy(),
        dir.reth_genesis(),
        dir.summit_genesis(),
        dir.nodes(),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .collect()
}

/// Scaffold a network directory's four authored inputs under `inputs/`.
/// Returns the paths written. With `force`, the directory is a network
/// started over: whatever [`network_state`] finds is removed first.
pub async fn init_network_dir(
    client: &reqwest::Client,
    dir: &NetworkDir,
    inputs: &InitInputs<'_>,
    force: bool,
) -> anyhow::Result<Vec<PathBuf>> {
    let measurements = read_input_source(client, inputs.measurements).await?;
    require_measurement_id(&measurements, inputs.measurements)?;
    let reth_genesis = read_input_source(client, inputs.reth_genesis).await?;
    serde_json::from_slice::<serde_json::Value>(&reth_genesis).map_err(|e| {
        anyhow::anyhow!(
            "{} is not valid JSON: {e}{}",
            inputs.reth_genesis,
            raw_url_hint(inputs.reth_genesis)
        )
    })?;
    let summit_genesis = fill_summit_namespace(
        &read_input_source(client, inputs.summit_genesis).await?,
        inputs.name,
        inputs.summit_genesis,
    )?;
    let credentials: Vec<String> = (1..=inputs.founders)
        .map(|i| format!("0x{i:040x}"))
        .collect();
    let mut founders = serde_json::to_vec_pretty(&credentials).expect("strings serialize");
    founders.push(b'\n');

    let contents = [
        (RETH_GENESIS_FILENAME, reth_genesis),
        (MEASUREMENTS_FILENAME, measurements),
        (SUMMIT_GENESIS_FILENAME, summit_genesis),
        (FOUNDERS_FILENAME, founders),
    ];
    let existing = network_state(dir);
    if !existing.is_empty() {
        if !force {
            bail!(
                "refusing to overwrite the network directory {}: it holds {} — pass --force to \
                 start it over (the authored inputs and harvest, the artifact set and nodes/ are \
                 removed first; re-authoring the inputs and re-assembling is a new network \
                 identity)",
                dir.root().display(),
                existing
                    .iter()
                    .map(|p| p
                        .strip_prefix(dir.root())
                        .unwrap_or(p)
                        .display()
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        // Whole, not just the four inputs: a harvest or artifact set left
        // from an earlier cohort would describe boxes these inputs never
        // met, and `harvest` would refuse the stale records by name later.
        for path in &existing {
            if path.is_dir() {
                std::fs::remove_dir_all(path)
            } else {
                std::fs::remove_file(path)
            }
            .with_context(|| format!("removing {}", path.display()))?;
            eprintln!("removed {}", path.display());
        }
    }
    let inputs_dir = dir.inputs();
    std::fs::create_dir_all(&inputs_dir)
        .with_context(|| format!("creating {}", inputs_dir.display()))?;
    let mut written = Vec::with_capacity(contents.len());
    for (name, bytes) in contents {
        let path = inputs_dir.join(name);
        std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Network directory to create.
    #[arg(value_name = "DIR")]
    pub dir: PathBuf,

    /// Network name, filled in as the summit genesis's namespace when the
    /// authored genesis leaves it empty. Default: the directory's basename
    /// (which is also what assemble uses as the manifest name).
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,

    /// reth genesis (local path or https:// URL), copied in as
    /// inputs/reth-genesis.json. Required: an external fact (chain state +
    /// contract alloc) init cannot invent.
    #[arg(long, value_name = "PATH_OR_URL")]
    pub reth_genesis: String,

    /// The image's measurements (or a promoted policy; local path or https://
    /// URL) — for a CI-published image, the measurements.azure-tdx.json asset
    /// of the seismic-images release named after it. Copied in as
    /// inputs/measurements.json. Required: the PCRs of a real published
    /// image, never generated.
    #[arg(long, value_name = "PATH_OR_URL")]
    pub measurements: String,

    /// Authored summit genesis (local path or https:// URL), copied in
    /// verbatim except that an empty namespace is filled with <name>.
    /// Required: every value in it is a per-network choice; start from
    /// tee/networks/summit-genesis-starter.toml and review each parameter.
    #[arg(long, value_name = "PATH_OR_URL")]
    pub summit_genesis: String,

    /// How many placeholder withdrawal credentials to scaffold into
    /// inputs/founder-withdrawal-credentials.json (one per founding node,
    /// paired in node-name order at assemble time). The placeholders are all
    /// a throwaway needs; a real founding replaces them with the founders'
    /// addresses. Default: an empty list to fill in.
    #[arg(long, value_name = "N", default_value_t = 0)]
    pub founders: usize,

    /// Start the network over: remove what the directory holds for it — the
    /// authored inputs and harvest, the artifact set, nodes/ — then scaffold.
    /// Re-authoring the inputs and re-assembling is a new network identity.
    #[arg(long)]
    pub force: bool,

    #[command(flatten)]
    pub context: ContextArgs,
}

/// The directory as an absolute path, so every path a command prints is
/// clickable and names one directory unambiguously. Resolved without requiring
/// the directory to exist yet, which `init` does not.
pub fn absolute(dir: &Path) -> anyhow::Result<PathBuf> {
    std::path::absolute(dir).with_context(|| format!("resolving {}", dir.display()))
}

/// The manifest name a network directory implies: its basename.
pub fn network_name(dir: &Path) -> anyhow::Result<String> {
    dir.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty())
        .with_context(|| format!("{} has no basename to name the network by", dir.display()))
}

pub async fn run(args: InitArgs) -> anyhow::Result<ExitCode> {
    let root = absolute(&args.dir)?;
    let name = match &args.name {
        Some(name) => name.clone(),
        None => network_name(&root)?,
    };
    let dir = NetworkDir::new(&root);
    let client = fetch_client()?;
    let written = init_network_dir(
        &client,
        &dir,
        &InitInputs {
            name: &name,
            reth_genesis: &args.reth_genesis,
            measurements: &args.measurements,
            summit_genesis: &args.summit_genesis,
            founders: args.founders,
        },
        args.force,
    )
    .await?;
    for path in &written {
        eprintln!("wrote {}", path.display());
    }

    // Creating a network selects it: the commands after this one can drop
    // their DIR, and `node --name <x>` works against the cohort the moment
    // its nodes are imported. A failed write is an error, not a warning —
    // "wrote" and "registered" already printed above it would be false, and a
    // half-done registration is worse than a loud one.
    let context = Context::load(args.context.config.as_deref())?;
    let config_path = context.path().to_path_buf();
    write::set_network(&config_path, &name, &Network::of_dir(&root))?;
    write::set_current(
        &config_path,
        &Selection {
            network: name.clone(),
            node: None,
        },
    )?;

    let founders_hint = if args.founders > 0 {
        "update the placeholder addresses in"
    } else {
        "fill in one address per founding node in"
    };
    // The authored credentials size the cohort: harvest and assemble both
    // refuse a cohort whose node count disagrees with them. Provisioning is
    // the runbook's (it spans Pulumi), linked by URL because the CLI may be
    // installed where the repo is not; the step after it is spelled here to
    // bring the founder back to this CLI.
    next_step::print_steps(&[
        format!(
            "review every file under {} — {founders_hint} {}",
            dir.inputs().display(),
            dir.founders()
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
        ),
        format!("provision the cohort: {RUNBOOK_PROVISION_URL}"),
        "harvest its founding keys from the node table the provisioner prints (pulumi stack \
         output nodes --json > nodes.json):\nseismic-tee network harvest --nodes nodes.json"
            .to_string(),
    ]);
    // What the registration above bought, and what importing the cohort adds
    // to it: both are the context's, so they are told together.
    next_step::print_ctx(
        &format!(
            "{name} is registered and selected in {},\nso harvest and every command after it find \
             the network directory without DIR.\nImport the node table too, and they find the \
             cohort without --nodes:",
            config_path.display()
        ),
        &[format!(
            "pulumi stack output nodes --json | seismic-tee ctx set-nodes {name}"
        )],
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use seismic_tee_common::test_support::{refused_url, write_file};

    use super::*;

    const STARTER: &str = "# starter params\nnamespace = \"\"\nleader_timeout_ms = 2000\n";
    const MEASUREMENTS: &str =
        r#"{"measurement_id": "img.vhd", "measurements": {"4": {"expected": "ab"}}}"#;
    const RETH_GENESIS: &str = r#"{"config": {"chainId": 5124}}"#;

    struct Loose {
        _dir: tempfile::TempDir,
        reth_genesis: PathBuf,
        measurements: PathBuf,
        starter: PathBuf,
        out: NetworkDir,
    }

    fn loose() -> Loose {
        let dir = tempfile::tempdir().unwrap();
        let out = NetworkDir::new(dir.path().join("networks").join("testnet-1"));
        Loose {
            reth_genesis: write_file(&dir, "dev.json", RETH_GENESIS.as_bytes()),
            measurements: write_file(&dir, "measurements.json", MEASUREMENTS.as_bytes()),
            starter: write_file(&dir, "summit-genesis-starter.toml", STARTER.as_bytes()),
            out,
            _dir: dir,
        }
    }

    fn s(path: &Path) -> &str {
        path.to_str().unwrap()
    }

    async fn init(loose: &Loose, founders: usize, force: bool) -> anyhow::Result<Vec<PathBuf>> {
        init_network_dir(
            &fetch_client().unwrap(),
            &loose.out,
            &InitInputs {
                name: "testnet-1",
                reth_genesis: s(&loose.reth_genesis),
                measurements: s(&loose.measurements),
                summit_genesis: s(&loose.starter),
                founders,
            },
            force,
        )
        .await
    }

    #[tokio::test]
    async fn scaffolds_the_four_inputs_and_fills_the_namespace() {
        let loose = loose();
        let written = init(&loose, 0, false).await.unwrap();
        let inputs = loose.out.inputs();
        assert!(written.iter().all(|p| p.parent() == Some(inputs.as_path())));
        let mut names: Vec<_> = written
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(
            names,
            [
                "founder-withdrawal-credentials.json",
                "measurements.json",
                "reth-genesis.json",
                "summit-genesis.toml",
            ]
        );
        // No --founders: an empty list to fill in, not a guessed cohort size.
        assert_eq!(
            std::fs::read_to_string(loose.out.founders()).unwrap(),
            "[]\n"
        );
        assert_eq!(
            std::fs::read(loose.out.input_reth_genesis()).unwrap(),
            RETH_GENESIS.as_bytes()
        );
        assert_eq!(
            std::fs::read(loose.out.input_measurements()).unwrap(),
            MEASUREMENTS.as_bytes()
        );
        // The starter's namespace slot is empty, so init fills the network
        // name into it; every other authored line (comments included) is
        // untouched.
        assert_eq!(
            std::fs::read_to_string(loose.out.input_summit_genesis()).unwrap(),
            "# starter params\nnamespace = \"testnet-1\"\nleader_timeout_ms = 2000\n"
        );
    }

    #[tokio::test]
    async fn founders_scaffolds_placeholder_credentials() {
        let loose = loose();
        init(&loose, 3, false).await.unwrap();
        let credentials: Vec<String> =
            serde_json::from_slice(&std::fs::read(loose.out.founders()).unwrap()).unwrap();
        assert_eq!(
            credentials,
            [
                format!("0x{}1", "0".repeat(39)),
                format!("0x{}2", "0".repeat(39)),
                format!("0x{}3", "0".repeat(39)),
            ]
        );
        assert!(credentials.iter().all(|c| crate::founding::is_address(c)));
    }

    #[test]
    fn the_namespace_rule_fills_empty_or_missing_and_keeps_a_chosen_one() {
        // A genesis with no namespace line at all gets one appended.
        let filled =
            fill_summit_namespace(b"leader_timeout_ms = 2000\n", "testnet-1", "s").unwrap();
        assert!(filled.starts_with(b"leader_timeout_ms = 2000\n"));
        let parsed: toml::Table = toml::from_str(std::str::from_utf8(&filled).unwrap()).unwrap();
        assert_eq!(parsed["namespace"].as_str(), Some("testnet-1"));

        // A chosen namespace is copied verbatim.
        let chosen = b"namespace = \"mine\"\nleader_timeout_ms = 2000\n";
        assert_eq!(
            fill_summit_namespace(chosen, "testnet-1", "s").unwrap(),
            chosen
        );

        // An empty namespace spelled in a form the line rewrite can't find
        // (quoted key) fails loudly instead of shipping an empty namespace.
        let err = fill_summit_namespace(b"\"namespace\" = \"\"\n", "testnet-1", "s")
            .unwrap_err()
            .to_string();
        assert!(err.contains("empty namespace"), "{err}");

        let err = fill_summit_namespace(b"<html>not a genesis</html>", "t", "https://x/y")
            .unwrap_err()
            .to_string();
        assert!(err.contains("not valid TOML"), "{err}");
        assert!(err.contains("raw.githubusercontent.com"), "{err}");
    }

    #[test]
    fn measurements_must_be_stamped_unless_already_a_policy() {
        require_measurement_id(MEASUREMENTS.as_bytes(), "m.json").unwrap();
        // A promoted policy is a record list, each record carrying its own id.
        require_measurement_id(br#"[{"measurement_id": "img.vhd"}]"#, "p.json").unwrap();

        let err = require_measurement_id(br#"{"measurements": {}}"#, "m.json")
            .unwrap_err()
            .to_string();
        assert!(err.contains("carries no measurement_id"), "{err}");
        assert!(err.contains("make measure"), "{err}");

        let err = require_measurement_id(b"{not json", "https://x/m.json")
            .unwrap_err()
            .to_string();
        assert!(err.contains("not valid JSON"), "{err}");
        assert!(err.contains("raw content URL"), "{err}");
    }

    /// A second `init` refuses whatever the layout already holds, naming it;
    /// `--force` starts the network over — inputs, harvest, artifact set and
    /// nodes/ all gone, so no record of an earlier cohort survives to be
    /// refused by `harvest` later — and touches nothing else in the
    /// directory.
    #[tokio::test]
    async fn refuses_to_overwrite_unless_forced_and_then_starts_over() {
        let loose = loose();
        init(&loose, 0, false).await.unwrap();
        let out = &loose.out;
        let plant = |relative: &str| {
            let path = out.root().join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, b"{}").unwrap();
            path
        };
        let stale_record = plant("inputs/harvest/old-1.json");
        plant("network-manifest.json");
        plant("nodes/bootnodes.json");
        let unrelated = plant("NOTES.md");

        let err = init(&loose, 0, false).await.unwrap_err().to_string();
        assert!(err.contains("refusing to overwrite"), "{err}");
        assert!(err.contains("--force"), "{err}");
        for held in ["inputs", "network-manifest.json", "nodes"] {
            assert!(err.contains(held), "{err}");
        }
        assert!(stale_record.exists());

        init(&loose, 2, true).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<Vec<String>>(&std::fs::read(out.founders()).unwrap())
                .unwrap()
                .len(),
            2
        );
        assert!(!stale_record.exists());
        assert!(!out.harvest().exists());
        assert!(!out.manifest().exists());
        assert!(!out.nodes().exists());
        assert!(unrelated.exists());
    }

    #[tokio::test]
    async fn a_missing_or_malformed_input_is_named() {
        let loose = loose();
        let client = fetch_client().unwrap();
        let err = read_input_source(&client, "/absent/dev.json")
            .await
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("input file not found: /absent/dev.json"),
            "{err}"
        );

        std::fs::write(&loose.reth_genesis, "{not json").unwrap();
        let err = init(&loose, 0, false).await.unwrap_err().to_string();
        assert!(err.contains("is not valid JSON"), "{err}");
        assert!(err.contains("dev.json"), "{err}");
    }

    /// URL inputs: http is refused before any request; an https fetch that
    /// fails names the URL.
    #[tokio::test]
    async fn url_inputs_are_fetched_and_only_over_https() {
        let client = fetch_client().unwrap();
        let err = read_input_source(&client, "http://example.test/dev.json")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("insecure URL rejected"), "{err}");

        let url = format!("{}/gone.json", refused_url().replace("http://", "https://"));
        let err = read_input_source(&client, &url)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("failed to fetch"), "{err}");
        assert!(err.contains(&url), "{err}");
    }

    #[test]
    fn the_network_name_is_the_directory_basename() {
        assert_eq!(
            network_name(Path::new("/nets/devnet-3")).unwrap(),
            "devnet-3"
        );
        assert_eq!(network_name(Path::new("devnet-3")).unwrap(), "devnet-3");
        assert!(network_name(Path::new("/")).is_err());
    }

    fn init_args(loose: &Loose, name: Option<&str>, force: bool, config_path: PathBuf) -> InitArgs {
        InitArgs {
            dir: loose.out.root().to_path_buf(),
            name: name.map(String::from),
            reth_genesis: s(&loose.reth_genesis).to_string(),
            measurements: s(&loose.measurements).to_string(),
            summit_genesis: s(&loose.starter).to_string(),
            founders: 0,
            force,
            context: ContextArgs {
                context: None,
                config: Some(config_path),
            },
        }
    }

    fn read_config(path: &Path) -> seismic_tee_context::config::Config {
        toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[tokio::test]
    async fn a_run_registers_and_selects_the_network_and_a_forced_rerun_rewrites_it() {
        let loose = loose();
        let config_dir = tempfile::tempdir().unwrap();
        let config_path = config_dir.path().join("config.toml");
        let expected_dir = absolute(loose.out.root()).unwrap();

        run(init_args(&loose, None, false, config_path.clone()))
            .await
            .unwrap();
        let config = read_config(&config_path);
        assert_eq!(config.current.as_deref(), Some("testnet-1"));
        assert_eq!(
            config.networks["testnet-1"].dir.as_deref(),
            Some(expected_dir.as_path())
        );

        // A second run needs --force for the scaffold step, and rewrites the
        // same registration.
        run(init_args(&loose, None, true, config_path.clone()))
            .await
            .unwrap();
        let config = read_config(&config_path);
        assert_eq!(config.current.as_deref(), Some("testnet-1"));
        assert_eq!(config.networks.len(), 1);
    }

    #[tokio::test]
    async fn name_registers_under_the_given_name_not_the_basename() {
        let loose = loose();
        let config_dir = tempfile::tempdir().unwrap();
        let config_path = config_dir.path().join("config.toml");

        run(init_args(
            &loose,
            Some("custom-name"),
            false,
            config_path.clone(),
        ))
        .await
        .unwrap();
        let config = read_config(&config_path);
        assert_eq!(config.current.as_deref(), Some("custom-name"));
        assert!(config.networks.contains_key("custom-name"));
        assert!(!config.networks.contains_key("testnet-1"));
    }

    /// A registration failure is a hard error, not a warning: `init` already
    /// printed that it wrote the scaffold, and a half-done registration would
    /// be worse than a loud one.
    #[tokio::test]
    #[cfg(unix)]
    async fn a_read_only_config_directory_fails_the_run() {
        use std::os::unix::fs::PermissionsExt;

        let loose = loose();
        let config_dir = tempfile::tempdir().unwrap();
        let readonly = config_dir.path().join("ro");
        std::fs::create_dir_all(&readonly).unwrap();
        std::fs::set_permissions(&readonly, std::fs::Permissions::from_mode(0o500)).unwrap();
        let config_path = readonly.join("config.toml");

        let result = run(init_args(&loose, None, false, config_path)).await;

        // Restore write permission so the tempdir can clean itself up,
        // regardless of the assertion outcome.
        std::fs::set_permissions(&readonly, std::fs::Permissions::from_mode(0o700)).unwrap();

        assert!(result.is_err());
        assert!(
            loose.out.input_reth_genesis().is_file(),
            "the scaffold itself still wrote, even though registration failed"
        );
    }
}
