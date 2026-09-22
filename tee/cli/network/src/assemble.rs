//! `assemble`: derive the artifact set from a network directory's inputs.
//!
//! ```text
//! seismic-tee network assemble tee/networks/devnet-3
//! ```
//!
//! The step that pins the founding validator set into `network_id`. It reads
//! the authored inputs and the harvest under `inputs/`, re-verifies every
//! archived founding quote offline from its own archive, compiles the
//! measurement policy and injects the registry storage
//! into its copy of the reth genesis, completes the summit genesis with the
//! derived `eth_genesis_hash` and the founding validator set through summit's
//! own emitter — both by the image's own `seismic-reth` and `summit`, fetched
//! from the release `inputs/image.json` names and verified against its
//! `SHA256SUMS` ([`crate::shell_outs`]) — renders the manifest through the
//! enclave's renderer, runs every gate over the result, and writes the four
//! artifacts at the directory's top level. Everything top-level is hash-pinned by the manifest;
//! everything under `inputs/` is provenance.
//!
//! Each artifact is its input with derived fields filled in — never authored
//! twice. The summit genesis's `eth_genesis_hash` is derived from the reth
//! genesis, so whatever the input declares is replaced; the registry account's
//! storage is derived from the policy, so whatever the input holds is
//! replaced; the manifest itself is pure output. Edits go to the inputs, then
//! re-assemble. A manifest is immutable for the network's lifetime: an
//! existing one is never overwritten without `--force`.
//!
//! `--check` runs the same derivation and every gate over it, then compares
//! with what is on disk instead of writing — the shape of `cargo fmt
//! --check`. An artifact that differs from what the inputs and the harvest
//! derive to fails by name; nothing is written. It is the check to run after
//! a merge or whenever an artifact set is suspected of having drifted from
//! its inputs, and it rests on `assemble` being deterministic, which the
//! drift suite already relies on when it re-renders the committed manifests.
//! The one input it does not take from the cohort is the validators' IPs:
//! those are topology outside the pinned digest and rotate without
//! re-founding, so `--check` re-derives with the IPs the genesis on disk
//! seats ([`ValidatorIps::Seated`]) — a rotated box is not a difference, and
//! no node table is needed, so a committed directory checks as it audits.

use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::ExitCode;

use alloy_primitives::Address;
use anyhow::{Context as _, bail};
use clap::Args;
use seismic_manifest::{
    ContractsManifest, EthManifest, MeasurementsManifest, NetworkManifestV1, SummitManifest, render,
};
use seismic_measurement_admission::promote_measurements;
use seismic_tee_common::network_dir::INPUTS_DIRNAME;
use seismic_tee_common::{Artifact, Manifest, NetworkDir, next_step};
use seismic_tee_context::load_nodes;
use seismic_verify_quote::{SeismicMeasurementPolicy, archive, verify_archived_harvest};
use sha2::{Digest as _, Sha256};

use crate::args::DirArgs;
use crate::configure;
use crate::founding::{
    FoundingRecords, Validator, ValidatorIps, load_founding_set, seated_validator_ips,
};
use crate::gates::{
    ArtifactSet, compile, hex_0x, inject_registry_genesis_storage, run_validation_gates,
};
use crate::init::{absolute, network_name};
use crate::shell_outs::{DerivationArgs, Derivations};

/// Genesis-alloc addresses of the admission-policy contracts, named by role as
/// in the manifest schema: registry = the measurement allowlist
/// (MeasurementRegistry.sol), authority = its mutation authority (today
/// MeasurementAuthorityDev.sol).
pub const DEFAULT_REGISTRY: &str = "0x1000000000000000000000000000000000000001";
pub const DEFAULT_AUTHORITY: &str = "0x1000000000000000000000000000000000000002";

/// The platform a policy promoted from the measurements input pins, unless
/// told otherwise.
pub const DEFAULT_ATTESTATION_TYPE: &str = "azure-tdx";

/// The artifact set as `assemble` derived it, before it is written.
#[derive(Debug, Clone)]
pub struct Assembled {
    /// The rendered manifest, strictly parsed back, with its network id.
    pub manifest: Manifest,
    /// The promoted policy, byte-verbatim: the manifest commits to its hash.
    pub policy: Vec<u8>,
    /// The input reth genesis with the compiled registry storage injected —
    /// what `eth.genesis_hash` was computed from.
    pub reth_genesis: Vec<u8>,
    /// The completed summit genesis, as summit emitted it — what
    /// `summit.genesis_config_digest` was computed from.
    pub summit_genesis: Vec<u8>,
    pub warnings: Vec<String>,
}

/// Everything `assemble` derives from.
#[derive(Debug, Clone)]
pub struct AssembleInputs<'a> {
    pub name: &'a str,
    /// The authored, policy-free reth genesis.
    pub reth_genesis: &'a Artifact,
    /// The authored summit parameters.
    pub summit_genesis: &'a Artifact,
    /// The promoted policy document.
    pub policy: &'a [u8],
    /// The founding set, paired and in node-name order.
    pub validators: &'a [Validator],
    pub registry: Address,
    pub authority: Address,
}

/// Set `eth_genesis_hash` in an authored summit genesis to the computed value.
///
/// The hash is derived from the reth genesis — never authored — but summit's
/// genesis parser requires the field to be present in the TOML it reads, so
/// the template fed to `summit genesis set-validators` must carry it. Any
/// declared value is dropped (it can only be stale copy-paste) and the
/// computed one is prepended — always valid TOML for a top-level key;
/// set-validators re-renders the completed file, which is what
/// `genesis_config_digest` commits to and the artifact set ships.
pub fn fill_eth_genesis_hash(
    authored: &[u8],
    eth_genesis_hash: [u8; 32],
) -> anyhow::Result<Vec<u8>> {
    let text = std::str::from_utf8(authored).context("summit genesis is not valid TOML")?;
    let mut kept = String::with_capacity(text.len() + 96);
    let mut in_table = false;
    for line in text.split_inclusive('\n') {
        let stripped = line.trim_start();
        // Top-level keys can only appear before the first table header; a
        // same-named key inside a table (none exists today) is left alone.
        if stripped.starts_with('[') {
            in_table = true;
        }
        let is_hash_line = !in_table
            && stripped
                .strip_prefix("eth_genesis_hash")
                .is_some_and(|rest| rest.trim_start().starts_with('='));
        if !is_hash_line {
            kept.push_str(line);
        }
    }
    let parsed: toml::Table = toml::from_str(&kept).context("summit genesis is not valid TOML")?;
    if parsed.contains_key("eth_genesis_hash") {
        bail!(
            "could not replace the authored file's declared eth_genesis_hash (unusual TOML \
             layout); delete the line by hand — the value is derived from the reth genesis"
        );
    }
    let mut out = format!("eth_genesis_hash = \"{}\"\n", hex_0x(&eth_genesis_hash));
    out.push_str(&kept);
    Ok(out.into_bytes())
}

/// Assemble, render, and gate-check a v1 network manifest and the artifacts
/// it pins.
pub async fn assemble(
    inputs: &AssembleInputs<'_>,
    derive: &impl Derivations,
) -> anyhow::Result<Assembled> {
    let genesis: serde_json::Value = serde_json::from_slice(inputs.reth_genesis.bytes())
        .with_context(|| format!("{} is not valid JSON", inputs.reth_genesis.path().display()))?;
    let chain_id = genesis.get("config").and_then(|c| c.get("chainId"));
    let Some(chain_id) = chain_id.and_then(serde_json::Value::as_u64) else {
        bail!(
            "reth genesis config.chainId is {}, not an int",
            chain_id.map_or_else(|| "absent".to_string(), ToString::to_string)
        );
    };

    let authored_text = std::str::from_utf8(inputs.summit_genesis.bytes()).with_context(|| {
        format!(
            "{} is not valid TOML",
            inputs.summit_genesis.path().display()
        )
    })?;
    let authored: toml::Table = toml::from_str(authored_text).with_context(|| {
        format!(
            "{} is not valid TOML",
            inputs.summit_genesis.path().display()
        )
    })?;
    let Some(namespace) = authored.get("namespace").and_then(toml::Value::as_str) else {
        bail!(
            "summit genesis has no namespace string (got {})",
            authored
                .get("namespace")
                .map_or_else(|| "nothing".to_string(), ToString::to_string)
        );
    };

    // The registry account's genesis storage is derived, not authored: the
    // policy document is compiled and its registry_genesis_storage injected
    // into the shipped genesis copy, so eth.genesis_hash commits to the
    // reviewed policy. The gates then re-validate the injected copy against
    // an independent compile of the same document.
    let report = compile(inputs.policy)?;
    let reth_genesis_bytes =
        inject_registry_genesis_storage(inputs.reth_genesis.bytes(), inputs.registry, &report)?;
    let eth_hash = derive.reth_genesis_hash(&reth_genesis_bytes).await?;
    if let Some(declared) = authored
        .get("eth_genesis_hash")
        .and_then(toml::Value::as_str)
        && declared.to_lowercase() != hex_0x(&eth_hash)
    {
        eprintln!(
            "replacing the authored eth_genesis_hash {declared} with the computed {} (the value \
             is derived from the reth genesis)",
            hex_0x(&eth_hash)
        );
    }

    if inputs.validators.is_empty() {
        bail!(
            "no founding validators — the validator set is pinned from the harvest, and a \
             founding with an empty set is not a network"
        );
    }
    // summit requires the field to *parse* a genesis (its Genesis type has no
    // serde default), and set-validators loads the template before replacing
    // whatever set it declares — so an input authored without one gets an
    // empty placeholder purely to make the template loadable. The shipped set
    // always comes from `validators`.
    let mut authored_bytes = inputs.summit_genesis.bytes().to_vec();
    if !authored.contains_key("validators") {
        let mut with_placeholder = b"validators = []\n".to_vec();
        with_placeholder.append(&mut authored_bytes);
        authored_bytes = with_placeholder;
    }
    let template = fill_eth_genesis_hash(&authored_bytes, eth_hash)?;
    let summit_genesis_bytes = derive
        .summit_set_validators(&template, inputs.validators)
        .await?;
    let config_digest = derive.summit_config_digest(&summit_genesis_bytes).await?;

    // Rendering belongs to the enclave's renderer: the values assembled here
    // go to it as the typed manifest and the canonical bytes come back, then
    // are strictly parsed back — so a bad manifest never leaves this command,
    // and the gates run over the same bytes that ship.
    let rendered = render(&NetworkManifestV1 {
        manifest_version: NetworkManifestV1::VERSION,
        name: inputs.name.to_string(),
        eth: EthManifest {
            chain_id,
            genesis_hash: eth_hash,
        },
        summit: SummitManifest {
            genesis_config_digest: config_digest,
            namespace: namespace.to_string(),
        },
        measurements: MeasurementsManifest {
            bootstrap_policy_hash: Sha256::digest(inputs.policy).into(),
            contracts: ContractsManifest {
                registry: inputs.registry.into_array(),
                authority: inputs.authority.into_array(),
            },
        },
    });
    let manifest =
        Manifest::from_json_bytes(rendered).context("the rendered manifest does not parse")?;

    let set = ArtifactSet {
        manifest,
        reth_genesis: Artifact::new(inputs.reth_genesis.path(), reth_genesis_bytes),
        summit_genesis: Artifact::new(inputs.summit_genesis.path(), summit_genesis_bytes),
        policy: Artifact::new("measurement-policy-bootstrap.json", inputs.policy),
    };
    let warnings = run_validation_gates(&set, derive).await?;
    let ArtifactSet {
        manifest,
        reth_genesis,
        summit_genesis,
        policy,
    } = set;
    Ok(Assembled {
        manifest,
        policy: policy.bytes().to_vec(),
        reth_genesis: reth_genesis.bytes().to_vec(),
        summit_genesis: summit_genesis.bytes().to_vec(),
        warnings,
    })
}

/// The four files an artifact set is, each at its place in the network
/// directory with the bytes `assemble` derived for it — what `write` writes
/// and `check` compares.
fn artifact_files<'a>(dir: &NetworkDir, assembled: &'a Assembled) -> [(PathBuf, &'a [u8]); 4] {
    [
        (dir.manifest(), assembled.manifest.bytes()),
        (dir.policy(), assembled.policy.as_slice()),
        (dir.reth_genesis(), assembled.reth_genesis.as_slice()),
        (dir.summit_genesis(), assembled.summit_genesis.as_slice()),
    ]
}

/// Write the network artifact set: manifest, policy, and assemble's copies of
/// the genesis artifacts the manifest commits to.
///
/// A manifest is immutable for the network's lifetime — refuse to overwrite
/// an existing one unless forced.
pub fn write_artifact_set(
    dir: &NetworkDir,
    assembled: &Assembled,
    force: bool,
) -> anyhow::Result<()> {
    let manifest_path = dir.manifest();
    if manifest_path.exists() && !force {
        let existing = std::fs::read(&manifest_path)
            .with_context(|| format!("reading {}", manifest_path.display()))?;
        bail!(
            "{} already exists (network_id {}); a manifest is immutable — pass --force only for a \
             new network",
            manifest_path.display(),
            hex_0x(&Sha256::digest(&existing)),
        );
    }
    std::fs::create_dir_all(dir.root())
        .with_context(|| format!("creating {}", dir.root().display()))?;
    for (path, bytes) in artifact_files(dir, assembled) {
        std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
        eprintln!("wrote {}", path.display());
    }
    Ok(())
}

/// Hold the artifact set on disk to the one just derived: every file must be
/// byte-for-byte what `write_artifact_set` would write. Nothing is written;
/// every file that differs or is missing is named in one failure.
///
/// Byte equality is the right bar, not gate-passing: a set that passes every
/// gate against itself can still be another network's — one assembled from
/// inputs since edited — and the manifest's bytes *are* `network_id`.
pub fn check_artifact_set(dir: &NetworkDir, assembled: &Assembled) -> anyhow::Result<()> {
    let mut wrong = Vec::new();
    for (path, derived) in artifact_files(dir, assembled) {
        match std::fs::read(&path) {
            Ok(on_disk) if on_disk == derived => eprintln!("{}: unchanged", path.display()),
            Ok(_) => wrong.push(format!("{}: differs", path.display())),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                wrong.push(format!("{}: missing", path.display()));
            }
            Err(error) => {
                return Err(error).with_context(|| format!("reading {}", path.display()));
            }
        }
    }
    if !wrong.is_empty() {
        bail!(
            "the artifact set on disk is not what its inputs derive to:\n  {}\nEither the inputs \
             moved after the set was assembled — restore them — or the set is stale: re-run \
             `assemble` without --check (with --force if the manifest is among them: a manifest \
             that differs is another network_id)",
            wrong.join("\n  ")
        );
    }
    Ok(())
}

/// Re-verify every archived founding record against `policy`, offline, at
/// the instant its own verification was held to.
///
/// The harvest verified these records when it collected them, but nothing
/// downstream trusts that run's verdict: assemble is the step that pins the
/// validator set into `network_id`, so it hands each archived record back to
/// the verifier before pinning anything, and `verify-founding` runs the same
/// function over the committed directory for as long as it exists (the
/// records are plain files that may have been copied, committed, and edited
/// since the harvest).
///
/// Each record is a whole founding archive — the quote, the DCAP bundle it
/// verified against, the instant, and the anchors, in one document — and is
/// replayed from that, so this gate behaves the same on the founding day and
/// four hundred days later. A record that is not a whole archive fails:
/// Intel's live collateral would answer for it today and stop answering in
/// about a month, which would make the verdict depend on when it ran. A
/// record that re-verifies under trust anchors other than the founding's
/// passes with a warning naming the difference. Fails on the first record
/// that does not verify, naming it; one line per verified record on stderr.
pub fn verify_harvest_records(
    dir: &NetworkDir,
    records: &FoundingRecords,
    policy: &[u8],
) -> anyhow::Result<()> {
    // Replaying an archive parses Intel's material, whose TLS-bearing types
    // want a rustls process default; see `node verify` for why one has to
    // be chosen. Idempotent: a second install is a no-op error.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Fail closed: there is no accept-any path, so an unparseable policy must
    // stop the run rather than widen it.
    let policy = SeismicMeasurementPolicy::from_json_bytes(policy)
        .context("loading the measurement policy")?;
    for (name, record) in records {
        let path = dir.harvest_record(name);
        let verdict = archive::parse(&record.document.to_string())
            .with_context(|| {
                format!(
                    "{} is not a founding archive — the founding quote can only be re-verified \
                     against the bundle its own harvest archived with it, so this cohort has to \
                     be re-harvested (or re-founded) rather than assembled around",
                    path.display()
                )
            })
            .and_then(|archive| verify_archived_harvest(archive, policy.clone()));
        let verified = match verdict {
            Ok(verified) => verified,
            Err(error) => bail!(
                "{name}: {error:?}\nA founding key whose archived quote does not verify must not \
                 be pinned — re-found (or re-harvest an unchanged cohort) rather than assembling \
                 around it"
            ),
        };
        if let Some(drift) = &verified.anchor_drift {
            eprintln!(
                "warning: {name}: re-verified under trust anchors other than the founding's \
                 ({drift}); the verdict is this build's, not the founding's own"
            );
        }
        eprintln!("{name}: founding quote re-verified from its archive");
    }
    Ok(())
}

#[derive(Debug, Args)]
pub struct AssembleArgs {
    /// Network directory from `init`: reads its inputs/ (reth-genesis.json,
    /// summit-genesis.toml, measurements.json, the founder credentials and
    /// the harvest), takes the network name from its basename, and writes the
    /// artifact set at the top level. Omit it to use the current context's
    /// network.
    #[command(flatten)]
    pub dir: DirArgs,

    /// Cohort's node table: `pulumi stack output nodes --json`, i.e.
    /// {<name>: {public_ip, fqdn}, …} — supplies each founding validator's
    /// IP. Omit it to use the current context's network. Not read under
    /// --check, which takes the IPs from the summit genesis on disk.
    #[arg(long, value_name = "FILE", conflicts_with = "check")]
    pub nodes: Option<PathBuf>,

    /// Platform the policy promoted from inputs/measurements.json pins.
    #[arg(long, value_name = "TYPE", default_value = DEFAULT_ATTESTATION_TYPE)]
    pub attestation_type: String,

    /// Measurement-registry contract address in the genesis alloc.
    #[arg(long, value_name = "ADDRESS", default_value = DEFAULT_REGISTRY)]
    pub registry: Address,

    /// Registry mutation-authority contract address.
    #[arg(long, value_name = "ADDRESS", default_value = DEFAULT_AUTHORITY)]
    pub authority: Address,

    /// Overwrite an existing manifest (a new network identity).
    #[arg(long, conflicts_with = "check")]
    pub force: bool,

    /// Derive and gate-check as usual, then compare with the artifact set on
    /// disk instead of writing it: any artifact that is not what the inputs
    /// and the harvest derive to fails by name. Nothing is written. Needs no
    /// node table: the validators' IPs are topology outside the pinned digest
    /// and are taken from the summit genesis on disk, so a box whose IP moved
    /// since the founding is not a difference.
    #[arg(long)]
    pub check: bool,

    #[command(flatten)]
    pub derivations: DerivationArgs,
}

pub async fn run(args: AssembleArgs) -> anyhow::Result<ExitCode> {
    let root = absolute(&args.dir.load()?)?;
    let name = network_name(&root)?;
    let dir = NetworkDir::new(&root);

    let inputs = [
        dir.input_reth_genesis(),
        dir.input_summit_genesis(),
        dir.input_measurements(),
    ];
    let missing: Vec<String> = inputs
        .iter()
        .filter(|p| !p.exists())
        .map(|p| p.display().to_string())
        .collect();
    if !missing.is_empty() {
        bail!(
            "missing authored input(s): {} — authored inputs live under {INPUTS_DIRNAME}/; \
             scaffold them with `init`",
            missing.join(", ")
        );
    }
    let [reth_genesis, summit_genesis, measurements] = inputs;

    // The founding-era IPs under --check, the cohort's current ones otherwise;
    // see `ValidatorIps`. Read before anything slower, so a directory with no
    // set to check against is named at once.
    let descriptors;
    let ips = if args.check {
        let path = dir.summit_genesis();
        if !path.is_file() {
            bail!(
                "{} not found — no artifact set to check; derive one with `assemble` without \
                 --check",
                path.display()
            );
        }
        let on_disk =
            std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        ValidatorIps::Seated(
            seated_validator_ips(&on_disk).with_context(|| path.display().to_string())?,
        )
    } else {
        descriptors = load_nodes(args.nodes.as_deref(), &args.dir.context, "--nodes")?;
        ValidatorIps::Cohort(&descriptors)
    };
    let founding = load_founding_set(&dir, &ips)?;
    eprintln!(
        "founding set: {} validator(s) from {}",
        founding.validators.len(),
        dir.harvest().display()
    );
    let raw = std::fs::read(&measurements)
        .with_context(|| format!("reading {}", measurements.display()))?;
    let policy = promote_measurements(&raw, None, Some(&args.attestation_type))
        .with_context(|| format!("{}", measurements.display()))?;

    // The offline replay gate — the same function `verify-founding` runs over
    // the committed directory afterwards.
    verify_harvest_records(&dir, &founding.records, &policy)?;

    // The image's own binaries, unless told otherwise — after every gate
    // that needs no network, so a directory with nothing to assemble is
    // refused before anything is fetched.
    let shell_outs = args.derivations.resolve(&dir).await?;

    let assembled = assemble(
        &AssembleInputs {
            name: &name,
            reth_genesis: &Artifact::read(&reth_genesis)?,
            summit_genesis: &Artifact::read(&summit_genesis)?,
            policy: &policy,
            validators: &founding.validators,
            registry: args.registry,
            authority: args.authority,
        },
        &shell_outs,
    )
    .await?;
    for warning in &assembled.warnings {
        eprintln!("warning: {warning}");
    }
    if args.check {
        check_artifact_set(&dir, &assembled)?;
    } else {
        write_artifact_set(&dir, &assembled, args.force)?;
    }
    println!("network_id: {}", assembled.manifest.network_id());
    // Straight to the founding: every gate and the replay `verify-founding`
    // runs both already ran above, over this very set. Genesis is named from
    // the founding set just pinned, so the line cannot name a node this
    // artifact set does not seat.
    let genesis = founding
        .records
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| "<genesis-node>".to_string());
    next_step::print("", &[configure::invocation(&genesis, &args.dir, &dir)]);
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::Path;

    use seismic_tee_common::network_dir::MANIFEST_FILENAME;
    use seismic_tee_common::test_support::write_file;

    use super::*;
    use crate::gates::tests::{FIXTURE_POLICY, FIXTURE_RETH_GENESIS, REGISTRY, other_policy};

    pub(crate) const AUTHORITY: Address =
        alloy_primitives::address!("0x1000000000000000000000000000000000000002");

    /// A founding validator entry as `load_founding_set` builds it.
    pub(crate) fn validator() -> Validator {
        Validator {
            node_public_key: "ab".repeat(32),
            consensus_public_key: "cd".repeat(48),
            ip_address: "203.0.113.7:18551".into(),
            withdrawal_credentials: format!("0x{}", "f3".repeat(20)),
        }
    }

    /// Stand-ins for the two binaries: a fixed genesis hash, a content-derived
    /// digest (a byte hash, not summit's SSZ digest, so tamper-detection gates
    /// still fire), and a line-level splice for set-validators (not a
    /// re-render, so byte-oriented assertions about the rest of the template
    /// stay meaningful).
    pub(crate) struct Fake {
        pub eth_hash: [u8; 32],
        /// A fixed digest instead of the content-derived one: lets a test
        /// reach the gates *behind* the digest gate with a tampered file.
        pub digest: Option<[u8; 32]>,
    }

    impl Default for Fake {
        fn default() -> Self {
            Self {
                eth_hash: [0x12; 32],
                digest: None,
            }
        }
    }

    impl Derivations for Fake {
        async fn reth_genesis_hash(&self, _genesis: &[u8]) -> anyhow::Result<[u8; 32]> {
            Ok(self.eth_hash)
        }

        async fn summit_config_digest(&self, genesis: &[u8]) -> anyhow::Result<[u8; 32]> {
            Ok(self
                .digest
                .unwrap_or_else(|| Sha256::digest(genesis).into()))
        }

        async fn summit_set_validators(
            &self,
            template: &[u8],
            validators: &[Validator],
        ) -> anyhow::Result<Vec<u8>> {
            let mut sorted = validators.to_vec();
            sorted.sort_by(|a, b| a.node_public_key.cmp(&b.node_public_key));
            let entries: Vec<String> = sorted
                .iter()
                .map(|v| {
                    format!(
                        "{{ consensus_public_key = {:?}, ip_address = {:?}, node_public_key = {:?}, \
                         withdrawal_credentials = {:?} }}",
                        v.consensus_public_key,
                        v.ip_address,
                        v.node_public_key,
                        v.withdrawal_credentials
                    )
                })
                .collect();
            let text = String::from_utf8(template.to_vec()).unwrap();
            assert!(
                text.contains("validators = []\n"),
                "the template carries the placeholder set"
            );
            Ok(text
                .replacen(
                    "validators = []\n",
                    &format!("validators = [{}]\n", entries.join(", ")),
                    1,
                )
                .into_bytes())
        }
    }

    /// An authored artifact set in a temp dir: the fixture network's reth
    /// genesis (its registry predeploy is the canonical runtime) and a
    /// minimal summit genesis with no eth_genesis_hash (assemble fills it).
    pub(crate) struct Authored {
        pub dir: tempfile::TempDir,
        pub reth_genesis: PathBuf,
        pub summit_genesis: PathBuf,
    }

    pub(crate) fn authored() -> Authored {
        let dir = tempfile::tempdir().unwrap();
        let reth_genesis = write_file(&dir, "reth-genesis.json", FIXTURE_RETH_GENESIS);
        let summit_genesis =
            write_file(&dir, "summit-genesis.toml", b"namespace = \"testnet-1\"\n");
        Authored {
            dir,
            reth_genesis,
            summit_genesis,
        }
    }

    pub(crate) async fn assemble_with(
        authored: &Authored,
        policy: &[u8],
        fake: &Fake,
    ) -> anyhow::Result<Assembled> {
        assemble(
            &AssembleInputs {
                name: "testnet-1",
                reth_genesis: &Artifact::read(&authored.reth_genesis).unwrap(),
                summit_genesis: &Artifact::read(&authored.summit_genesis).unwrap(),
                policy,
                validators: &[validator()],
                registry: REGISTRY,
                authority: AUTHORITY,
            },
            fake,
        )
        .await
    }

    #[tokio::test]
    async fn assemble_passes_its_own_gates_and_is_deterministic() {
        let authored = authored();
        let first = assemble_with(&authored, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap();
        let second = assemble_with(&authored, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap();
        assert_eq!(first.manifest.bytes(), second.manifest.bytes());
        assert_eq!(first.manifest.network_id(), second.manifest.network_id());
        assert!(first.warnings.is_empty(), "{:?}", first.warnings);

        // The manifest carries the derived values.
        assert_eq!(first.manifest.name, "testnet-1");
        assert_eq!(first.manifest.eth.chain_id, 5124);
        assert_eq!(first.manifest.eth.genesis_hash, [0x12; 32]);
        assert_eq!(first.manifest.summit.namespace, "testnet-1");
        assert_eq!(
            first.manifest.measurements.bootstrap_policy_hash,
            <[u8; 32]>::from(Sha256::digest(FIXTURE_POLICY))
        );
        assert_eq!(
            first.manifest.measurements.contracts.registry,
            REGISTRY.into_array()
        );
        assert_eq!(first.policy, FIXTURE_POLICY);
    }

    /// The registry storage is injected into assemble's genesis copy from the
    /// policy's compile, and eth.genesis_hash is computed over that copy —
    /// so the hash commits to the reviewed policy.
    #[tokio::test]
    async fn assemble_injects_the_registry_storage_it_pins() {
        let authored = authored();
        let policy = other_policy();
        let assembled = assemble_with(&authored, &policy, &Fake::default())
            .await
            .unwrap();
        let genesis: serde_json::Value = serde_json::from_slice(&assembled.reth_genesis).unwrap();
        let storage = genesis["alloc"][&hex_0x(REGISTRY.as_slice())]["storage"]
            .as_object()
            .unwrap();
        let report = compile(&policy).unwrap();
        assert_eq!(storage.len(), report.registry_genesis_storage.len());
        // The example's committed storage was compiled from another policy,
        // and was replaced wholesale.
        let example: serde_json::Value = serde_json::from_slice(FIXTURE_RETH_GENESIS).unwrap();
        assert_ne!(
            genesis["alloc"][&hex_0x(REGISTRY.as_slice())]["storage"],
            example["alloc"][&hex_0x(REGISTRY.as_slice())]["storage"]
        );
    }

    /// The summit genesis is completed: the derived hash first, then the
    /// founding set through summit's emitter; a declared hash is replaced.
    #[tokio::test]
    async fn assemble_completes_the_summit_genesis() {
        let authored = authored();
        std::fs::write(
            &authored.summit_genesis,
            format!(
                "eth_genesis_hash = \"0x{}\"\nnamespace = \"testnet-1\"\nleader_timeout_ms = 2000\n",
                "de".repeat(32)
            ),
        )
        .unwrap();
        let assembled = assemble_with(&authored, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap();
        let text = String::from_utf8(assembled.summit_genesis.clone()).unwrap();
        assert!(
            text.starts_with(&format!("eth_genesis_hash = \"0x{}\"\n", "12".repeat(32))),
            "{text}"
        );
        assert!(!text.contains(&"de".repeat(32)), "{text}");
        let parsed: toml::Table = toml::from_str(&text).unwrap();
        let validators = parsed["validators"].as_array().unwrap();
        assert_eq!(validators.len(), 1);
        assert_eq!(
            validators[0]["node_public_key"].as_str(),
            Some("ab".repeat(32).as_str())
        );
        assert_eq!(parsed["leader_timeout_ms"].as_integer(), Some(2000));
        // The digest commits to the emitted genesis.
        assert_eq!(
            assembled.manifest.summit.genesis_config_digest,
            <[u8; 32]>::from(Sha256::digest(&assembled.summit_genesis))
        );
    }

    #[tokio::test]
    async fn assemble_rejects_an_empty_validator_set() {
        let authored = authored();
        let err = assemble(
            &AssembleInputs {
                name: "testnet-1",
                reth_genesis: &Artifact::read(&authored.reth_genesis).unwrap(),
                summit_genesis: &Artifact::read(&authored.summit_genesis).unwrap(),
                policy: FIXTURE_POLICY,
                validators: &[],
                registry: REGISTRY,
                authority: AUTHORITY,
            },
            &Fake::default(),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.contains("no founding validators"), "{err}");
    }

    #[tokio::test]
    async fn assemble_warns_on_the_default_summit_namespace() {
        let authored = authored();
        std::fs::write(&authored.summit_genesis, "namespace = \"_SUMMIT\"\n").unwrap();
        let assembled = assemble_with(&authored, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap();
        assert!(
            assembled.warnings.iter().any(|w| w.contains("_SUMMIT")),
            "{:?}",
            assembled.warnings
        );
    }

    #[tokio::test]
    async fn assemble_needs_a_chain_id_and_a_namespace() {
        let string_chain_id = authored();
        std::fs::write(
            &string_chain_id.reth_genesis,
            r#"{"config": {"chainId": "5124"}, "alloc": {}}"#,
        )
        .unwrap();
        let err = assemble_with(&string_chain_id, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("chainId"), "{err}");

        let no_namespace = authored();
        std::fs::write(&no_namespace.summit_genesis, "leader_timeout_ms = 2000\n").unwrap();
        let err = assemble_with(&no_namespace, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("no namespace string"), "{err}");
    }

    /// The `eth_genesis_hash` line is replaced only at the top level; a
    /// same-named key inside a table is left alone.
    #[test]
    fn fill_eth_genesis_hash_replaces_the_top_level_key_only() {
        let filled = fill_eth_genesis_hash(
            b"eth_genesis_hash = \"0xdead\"\nnamespace = \"n\"\n[extra]\neth_genesis_hash = \"0xdead\"\n",
            [0x12; 32],
        )
        .unwrap();
        let text = String::from_utf8(filled).unwrap();
        assert_eq!(
            text,
            format!(
                "eth_genesis_hash = \"0x{}\"\nnamespace = \"n\"\n[extra]\neth_genesis_hash = \"0xdead\"\n",
                "12".repeat(32)
            )
        );

        // A layout the line scan cannot rewrite fails rather than shipping a
        // stale hash.
        let err = fill_eth_genesis_hash(b"\"eth_genesis_hash\" = \"0xdead\"\n", [0x12; 32])
            .unwrap_err()
            .to_string();
        assert!(err.contains("delete the line by hand"), "{err}");
    }

    /// The artifact set lands at the directory's top level; the manifest is
    /// immutable without --force; the written bytes are the assembled ones.
    #[tokio::test]
    async fn write_artifact_set_refuses_to_overwrite_a_manifest() {
        let authored = authored();
        let assembled = assemble_with(&authored, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap();
        let out = NetworkDir::new(authored.dir.path().join("out"));
        write_artifact_set(&out, &assembled, false).unwrap();
        for path in [
            out.manifest(),
            out.policy(),
            out.reth_genesis(),
            out.summit_genesis(),
        ] {
            assert!(path.is_file(), "{}", path.display());
        }
        assert_eq!(
            std::fs::read(out.manifest()).unwrap(),
            assembled.manifest.bytes()
        );
        assert_eq!(
            std::fs::read(out.reth_genesis()).unwrap(),
            assembled.reth_genesis
        );
        assert_eq!(
            std::fs::read(out.summit_genesis()).unwrap(),
            assembled.summit_genesis
        );
        // Round-trip: the written bytes hash back to the same network_id.
        assert_eq!(
            Manifest::load(&out.manifest()).unwrap().network_id(),
            assembled.manifest.network_id()
        );

        let err = write_artifact_set(&out, &assembled, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("immutable"), "{err}");
        assert!(err.contains(MANIFEST_FILENAME), "{err}");
        write_artifact_set(&out, &assembled, true).unwrap();
    }

    /// The `init` → `assemble` loop: authored inputs under inputs/, the
    /// derived artifact set at the top level, the inputs untouched.
    #[tokio::test]
    async fn init_then_assemble_share_a_directory() {
        let authored = authored();
        let net = NetworkDir::new(authored.dir.path().join("networks").join("testnet-1"));
        let raw = write_file(
            &authored.dir,
            "raw-measurements.json",
            br#"{"measurement_id": "img.vhd", "measurements": {"4": {"expected": "ab"}}}"#,
        );
        crate::init::init_network_dir(
            &crate::init::fetch_client().unwrap(),
            &net,
            &crate::init::InitInputs {
                name: "testnet-1",
                image: None,
                measurements: Some(raw.to_str().unwrap()),
                reth_genesis: Some(authored.reth_genesis.to_str().unwrap()),
                summit_genesis: Some(authored.summit_genesis.to_str().unwrap()),
                founders: 1,
            },
            false,
        )
        .await
        .unwrap();
        let authored_summit = std::fs::read(net.input_summit_genesis()).unwrap();

        let assembled = assemble(
            &AssembleInputs {
                name: "testnet-1",
                reth_genesis: &Artifact::read(&net.input_reth_genesis()).unwrap(),
                summit_genesis: &Artifact::read(&net.input_summit_genesis()).unwrap(),
                policy: FIXTURE_POLICY,
                validators: &[validator()],
                registry: REGISTRY,
                authority: AUTHORITY,
            },
            &Fake::default(),
        )
        .await
        .unwrap();
        write_artifact_set(&net, &assembled, false).unwrap();

        assert_eq!(
            std::fs::read(net.input_summit_genesis()).unwrap(),
            authored_summit
        );
        assert_eq!(
            std::fs::read(net.input_reth_genesis()).unwrap(),
            FIXTURE_RETH_GENESIS
        );
        assert_eq!(
            std::fs::read(net.summit_genesis()).unwrap(),
            assembled.summit_genesis
        );
        assert_eq!(
            std::fs::read(net.reth_genesis()).unwrap(),
            assembled.reth_genesis
        );
        assert_ne!(net.reth_genesis(), net.input_reth_genesis());
    }

    /// Re-verification is offline and per record: a record that is not a
    /// whole founding archive fails closed, by file, before any verifier
    /// runs.
    #[test]
    fn harvest_records_must_be_whole_archives() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        let mut records = FoundingRecords::new();
        records.insert(
            "node-1".to_string(),
            crate::founding::FoundingRecord {
                node_public_key: "ab".repeat(32),
                consensus_public_key: "cd".repeat(48),
                document: serde_json::json!({
                    "harvest_nonce": "11".repeat(32),
                    "node_public_key": "ab".repeat(32),
                    "consensus_public_key": "cd".repeat(48),
                    "evidence": crate::founding::tests::no_attestation_evidence(),
                }),
            },
        );
        let err = verify_harvest_records(&dir, &records, FIXTURE_POLICY)
            .unwrap_err()
            .to_string();
        assert!(err.contains("node-1:"), "{err}");
        assert!(
            err.contains("node-1.json is not a founding archive"),
            "{err}"
        );
        assert!(err.contains("re-harvested"), "{err}");
        assert!(err.contains("must not be pinned"), "{err}");

        // Fail closed on the policy, before any record is looked at.
        let err = verify_harvest_records(&dir, &records, b"{ not a policy")
            .unwrap_err()
            .to_string();
        assert!(err.contains("measurement policy"), "{err}");
    }

    /// `--check` holds every file on disk to the derivation, names each one
    /// that differs or is missing, and writes nothing.
    #[tokio::test]
    async fn check_holds_the_disk_to_the_derivation_and_writes_nothing() {
        let authored = authored();
        let assembled = assemble_with(&authored, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap();
        let out = NetworkDir::new(authored.dir.path().join("out"));

        // Nothing there yet: every artifact is missing, and stays so.
        let err = check_artifact_set(&out, &assembled)
            .unwrap_err()
            .to_string();
        for path in [
            out.manifest(),
            out.policy(),
            out.reth_genesis(),
            out.summit_genesis(),
        ] {
            assert!(
                err.contains(&format!("{}: missing", path.display())),
                "{err}"
            );
        }
        assert!(!out.root().exists());

        write_artifact_set(&out, &assembled, false).unwrap();
        check_artifact_set(&out, &assembled).unwrap();

        // The same inputs assembled again still match what was written.
        let again = assemble_with(&authored, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap();
        check_artifact_set(&out, &again).unwrap();

        // One edited file is named — and only that one; the edit survives,
        // because a check never writes.
        let tampered = b"namespace = \"other\"\n".to_vec();
        std::fs::write(out.summit_genesis(), &tampered).unwrap();
        std::fs::remove_file(out.policy()).unwrap();
        let err = check_artifact_set(&out, &assembled)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(&format!("{}: differs", out.summit_genesis().display())),
            "{err}"
        );
        assert!(
            err.contains(&format!("{}: missing", out.policy().display())),
            "{err}"
        );
        assert!(!err.contains(MANIFEST_FILENAME), "{err}");
        assert!(!err.contains("reth-genesis.json"), "{err}");
        assert!(err.contains("without --check"), "{err}");
        assert_eq!(std::fs::read(out.summit_genesis()).unwrap(), tampered);
        assert!(!out.policy().exists());

        // Inputs that moved after the set was written: the derivation is
        // another network's, and the manifest is among the differences.
        std::fs::write(out.policy(), &assembled.policy).unwrap();
        std::fs::write(out.summit_genesis(), &assembled.summit_genesis).unwrap();
        let drifted = assemble_with(&authored, &other_policy(), &Fake::default())
            .await
            .unwrap();
        let err = check_artifact_set(&out, &drifted).unwrap_err().to_string();
        assert!(err.contains(MANIFEST_FILENAME), "{err}");
        assert!(err.contains("--force"), "{err}");
        assert_eq!(
            std::fs::read(out.manifest()).unwrap(),
            assembled.manifest.bytes()
        );
    }

    /// A box whose IP rotated since the founding: re-deriving with the
    /// cohort's current table would call the on-disk summit genesis stale,
    /// which is why `--check` re-derives with the IPs that genesis seats —
    /// the founding-era ones — and finds nothing changed.
    #[tokio::test]
    async fn check_takes_the_ips_the_genesis_on_disk_seats_not_the_cohorts() {
        let authored = authored();
        let founded = assemble_with(&authored, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap();
        let out = NetworkDir::new(authored.dir.path().join("out"));
        write_artifact_set(&out, &founded, false).unwrap();

        async fn with(authored: &Authored, validators: &[Validator]) -> Assembled {
            assemble(
                &AssembleInputs {
                    name: "testnet-1",
                    reth_genesis: &Artifact::read(&authored.reth_genesis).unwrap(),
                    summit_genesis: &Artifact::read(&authored.summit_genesis).unwrap(),
                    policy: FIXTURE_POLICY,
                    validators,
                    registry: REGISTRY,
                    authority: AUTHORITY,
                },
                &Fake::default(),
            )
            .await
            .unwrap()
        }
        let rotated = Validator {
            ip_address: "198.51.100.9:18551".into(),
            ..validator()
        };
        let from_cohort = with(&authored, std::slice::from_ref(&rotated)).await;
        let err = check_artifact_set(&out, &from_cohort)
            .unwrap_err()
            .to_string();
        assert!(err.contains("summit-genesis.toml: differs"), "{err}");

        let seated = seated_validator_ips(&std::fs::read(out.summit_genesis()).unwrap()).unwrap();
        assert_eq!(seated[&rotated.node_public_key], validator().ip_address);
        let founding_era = Validator {
            ip_address: seated[&rotated.node_public_key].clone(),
            ..rotated
        };
        let from_seated = with(&authored, &[founding_era]).await;
        check_artifact_set(&out, &from_seated).unwrap();
    }

    #[test]
    fn the_addresses_parse_from_the_flags() {
        let args = AssembleArgs::try_parse_from_probe(&["assemble", "/nets/x"]);
        assert_eq!(args.registry, REGISTRY);
        assert_eq!(args.authority, AUTHORITY);
        assert_eq!(args.attestation_type, DEFAULT_ATTESTATION_TYPE);
        assert!(!args.force);
        assert!(!args.check);
        // No binary named: the image's own, from inputs/image.json.
        assert_eq!(args.derivations.reth_bin, None);
        assert_eq!(args.derivations.summit_bin, None);
        assert_eq!(args.dir.dir.as_deref(), Some(Path::new("/nets/x")));
    }

    /// `--check` writes nothing, so there is nothing for `--force` to
    /// permit, and reads no node table, so `--nodes` would be ignored: each
    /// pair is a usage error, not a silent no-op.
    #[test]
    fn check_excludes_force_and_nodes() {
        let args = AssembleArgs::try_parse_from_probe(&["assemble", "--check", "/nets/x"]);
        assert!(args.check);
        assert!(AssembleArgs::try_parse_probe(&["assemble", "--check", "--force", "n"]).is_err());
        assert!(
            AssembleArgs::try_parse_probe(&["assemble", "--check", "--nodes", "n.json", "n"])
                .is_err()
        );
    }

    impl AssembleArgs {
        fn try_parse_probe(argv: &[&str]) -> Result<Self, clap::Error> {
            use clap::Parser as _;
            #[derive(clap::Parser)]
            struct Probe {
                #[command(flatten)]
                args: AssembleArgs,
            }
            Probe::try_parse_from(argv).map(|probe| probe.args)
        }

        fn try_parse_from_probe(argv: &[&str]) -> Self {
            Self::try_parse_probe(argv).expect("well-formed argv")
        }
    }
}
