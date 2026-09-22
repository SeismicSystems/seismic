//! `verify`: deploy-verify a node's TDX attestation before relying on it.
//!
//! ```text
//! seismic-tee node verify --node n2.json --manifest m.json
//! ```
//!
//! The enclave's verify-quote library owns the whole relying-party flow: it
//! challenges the node's attestation service with a fresh nonce
//! (`getDeployVerificationEvidence` on `:7878`), recomputes the deploy
//! verification binding from the operator's own `--manifest` copy and that
//! nonce, and DCAP-verifies the returned quote against the measurement policy.
//! A pass proves a measured node holding this manifest answered this exact
//! request.
//!
//! The check protects the operator's own decisions — publishing the node's
//! address, handing it to later nodes as a bootnode, pointing tooling at it.
//! Membership in the network is granted by the network's own gates (the
//! attested root-key handshake and its admission policy), never by this check.
//!
//! Appraisal, not delivery: the node serves the evidence RPC for as long as it
//! runs and every run mints a fresh nonce, so this is re-runnable at any time —
//! after a reboot, after an image upgrade, on suspicion, or as a retry when
//! DCAP collateral was briefly unreachable. `configure` runs the same step
//! inline once the node it configured reaches a ready state; delivery itself
//! is once per boot (tdx-init accepts one config POST), which is why the check
//! has its own command.
//!
//! The policy the quote is appraised against is a network artifact: the
//! `measurement-policy-bootstrap.json` `assemble` wrote beside the manifest,
//! the document the manifest's `measurements.bootstrap_policy_hash` commits to
//! and the registry's genesis-seeded admission IDs were compiled from. So the
//! node is checked against the measurements *this* network founded on, not
//! against whatever measurements happen to be on the operator's disk.
//!
//! `--measurements` is the override for an operator who will not take the
//! founder's artifact at face value: they supply the image's expected
//! measurements (published by seismic-images CI) and this promotes them into a
//! policy of their own, with the same compiler the founding used. Either way
//! nothing here computes a security-critical measurement itself.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, bail};
use clap::Args;
use seismic_measurement_admission::promote_measurements;
use seismic_tee_common::network_dir::POLICY_FILENAME;
use seismic_tee_common::{Manifest, NetworkDir, NodeDescriptor};
use seismic_verify_quote::{SeismicMeasurementPolicy, verify_deploy};

use crate::args::NodeArgs;

/// The platform a policy promoted from `--measurements` pins, unless told
/// otherwise.
pub const DEFAULT_ATTESTATION_TYPE: &str = "azure-tdx";

/// Which measurement policy a node is appraised against.
///
/// Shared with `configure`, so an operator who tunes one command tunes both
/// the same way. The two sources are exclusive: `--policy` (or its default,
/// the network's own artifact) consumes a published policy document,
/// `--measurements` promotes a raw measurements file into one, and only that
/// path is steered by `--attestation-type`.
#[derive(Debug, Clone, Args)]
pub struct PolicySourceArgs {
    /// Measurement policy JSON the node's quote is verified against. Must be
    /// the document the manifest's measurements.bootstrap_policy_hash commits
    /// to, which is checked before use. Default:
    /// measurement-policy-bootstrap.json beside --manifest (the artifact-set
    /// layout `assemble` produces).
    #[arg(long, value_name = "FILE", conflicts_with = "measurements")]
    pub policy: Option<PathBuf>,

    /// Expected image measurements JSON (published by seismic-images CI for
    /// the node's image), promoted into a policy instead of using the
    /// network's published one. For an operator appraising the node against
    /// measurements they trust themselves.
    #[arg(long, value_name = "FILE")]
    pub measurements: Option<PathBuf>,

    /// Platform the policy promoted from --measurements pins.
    #[arg(long, value_name = "TYPE", default_value = DEFAULT_ATTESTATION_TYPE)]
    pub attestation_type: String,
}

/// What steers the verifier itself. Shared with `configure`.
#[derive(Debug, Clone, Args)]
pub struct VerifierArgs {
    /// PCCS URL for DCAP collateral, instead of the verifier's default
    /// provider.
    #[arg(long, value_name = "URL")]
    pub pccs_url: Option<String>,
}

/// Reject a contradictory or unreadable policy source up front, like every
/// other file flag. `--policy` itself is checked by [`resolve_policy`], which
/// also owns the default-location hint. `no_verify` is the caller's opt-out,
/// for the commands that offer one.
pub fn check_policy_source_files(source: &PolicySourceArgs, no_verify: bool) -> anyhow::Result<()> {
    if no_verify && (source.policy.is_some() || source.measurements.is_some()) {
        bail!(
            "--no-verify skips verification, so it contradicts --policy / --measurements \
             (which choose what to verify against). Drop one."
        );
    }
    if let Some(measurements) = &source.measurements
        && !measurements.is_file()
    {
        bail!("--measurements file not found: {}", measurements.display());
    }
    Ok(())
}

/// Resolve `--policy`, defaulting to the artifact-set convention:
/// `measurement-policy-bootstrap.json` beside the manifest, exactly where
/// `assemble` writes it — so the policy appraising the node is the one the
/// manifest's `measurements.bootstrap_policy_hash` was computed from.
///
/// `offer_no_verify` names the caller's opt-out in the hint, for a command
/// that verifies unless told not to.
pub fn resolve_policy_path(
    policy: Option<&Path>,
    manifest_path: &Path,
    offer_no_verify: bool,
) -> anyhow::Result<PathBuf> {
    let path = policy.map_or_else(
        || NetworkDir::of_manifest(manifest_path).policy(),
        Path::to_path_buf,
    );
    if path.is_file() {
        return Ok(path);
    }
    if policy.is_some() {
        bail!("--policy file not found: {}", path.display());
    }
    let mut escapes = vec![
        "  --policy FILE        the network's policy document, if it lives elsewhere",
        "  --measurements FILE  promote your own measurements into a policy",
    ];
    if offer_no_verify {
        escapes.push("  --no-verify          configure without appraising");
    }
    bail!(
        "measurement policy not found: {}\nThe default is {POLICY_FILENAME} beside --manifest \
         (the artifact set `assemble` writes). Instead:\n{}",
        path.display(),
        escapes.join("\n"),
    )
}

/// The policy a node will be appraised against, with where it came from —
/// for the line a command prints, or shows in its preview, before acting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPolicy {
    pub bytes: Vec<u8>,
    /// Provenance, for a human: the artifact path and what pins it, or the
    /// measurements file it was promoted from.
    pub source: String,
}

/// Resolve the measurement policy the node will be appraised against, before
/// the node is touched: a policy the manifest doesn't commit to, or a
/// measurements file the compiler rejects, fails here — where the fix costs
/// nothing, not after a config POST already landed.
///
/// Default: the network's own artifact, checked against the manifest's
/// `bootstrap_policy_hash` before use. A mismatch is fatal — appraising a node
/// against a policy this network never committed to proves nothing about
/// joining it.
///
/// With `--measurements`, promote the operator's file instead. No hash check
/// there: the whole point of that path is a policy the operator derived
/// themselves, which need not be the founder's.
pub fn resolve_policy(
    source: &PolicySourceArgs,
    manifest_path: &Path,
    manifest: &Manifest,
    offer_no_verify: bool,
) -> anyhow::Result<ResolvedPolicy> {
    if let Some(measurements) = &source.measurements {
        let raw = std::fs::read(measurements)
            .with_context(|| format!("reading --measurements {}", measurements.display()))?;
        let bytes = promote_measurements(&raw, None, Some(&source.attestation_type))
            .with_context(|| format!("--measurements {}", measurements.display()))?;
        return Ok(ResolvedPolicy {
            bytes,
            source: format!(
                "promoted from --measurements {} ({})",
                measurements.display(),
                source.attestation_type
            ),
        });
    }

    let policy_path =
        resolve_policy_path(source.policy.as_deref(), manifest_path, offer_no_verify)?;
    let policy = std::fs::read(&policy_path)
        .with_context(|| format!("reading measurement policy {}", policy_path.display()))?;
    manifest.check_policy(&policy).with_context(|| {
        format!(
            "{} is not the policy --manifest commits to.\nBoth files come from the same \
             `assemble` run — take them from one artifact set, or pass --measurements to \
             appraise the node against measurements of your own",
            policy_path.display(),
        )
    })?;
    Ok(ResolvedPolicy {
        bytes: policy,
        source: format!(
            "{} (pinned by --manifest {})",
            policy_path.display(),
            manifest_path.display()
        ),
    })
}

/// Challenge one node's attestation service and return the verifier's report.
///
/// The one place a policy becomes a challenge, so every caller — the
/// standalone command, the per-node `configure`, the founding cohort —
/// appraises a node the same way. `endpoint` is the node's attestation RPC
/// ([`NodeDescriptor::attestation_rpc_url`]).
pub async fn challenge_node(
    endpoint: &str,
    manifest: &Manifest,
    policy: &[u8],
    pccs_url: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    // Collateral fetches go over TLS, and the dependency graph enables more
    // than one rustls crypto provider (the attestation backend's aws-lc-rs,
    // the DCAP verifier's ring), which leaves rustls no process default to
    // pick. Choose the backend's, as the attestation service does. Idempotent:
    // a second install is a no-op error.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Fail closed: there is no accept-any path, so an unparseable policy must
    // stop the run rather than widen it.
    let policy =
        SeismicMeasurementPolicy::from_json_bytes(policy).context("loading measurement policy")?;
    let verified = verify_deploy(
        endpoint,
        manifest.bytes(),
        policy,
        pccs_url.map(str::to_string),
    )
    .await?;
    Ok(verified.to_json())
}

/// Challenge one node and print the verification report.
///
/// Failure is the error: an operator who asked for verification must not see
/// a zero exit from a node that didn't pass it. `endpoint` is normally the
/// descriptor's attestation RPC, passed apart so a test can point elsewhere.
pub async fn verify_deployment(
    descriptor: &NodeDescriptor,
    endpoint: &str,
    manifest: &Manifest,
    policy: &[u8],
    pccs_url: Option<&str>,
) -> anyhow::Result<()> {
    let NodeDescriptor { fqdn, public_ip } = descriptor;
    eprintln!("Deploy-verifying {fqdn} ({public_ip})...");
    match challenge_node(endpoint, manifest, policy, pccs_url).await {
        Ok(report) => {
            println!("✓ {fqdn} deploy-verified: {report}");
            Ok(())
        }
        Err(error) => bail!(
            "{fqdn} ({public_ip}): deploy verification FAILED:\n{error:?}\nDo not rely on this \
             node — publish its address, hand it to later nodes as a bootnode — until \
             `seismic-tee node verify` passes against it."
        ),
    }
}

/// The flags to repeat in a suggested `verify` command, so the retry runs the
/// same appraisal this run chose: the policy source, plus any non-default
/// verifier tooling (a retry pointed at the default PCCS fails for reasons
/// unrelated to the node). Empty when everything was default; otherwise
/// leading-space-prefixed, so it splices into a command line.
pub fn retry_flags(source: &PolicySourceArgs, verifier: &VerifierArgs) -> String {
    let mut flags = String::new();
    if let Some(measurements) = &source.measurements {
        flags.push_str(&format!(" --measurements {}", measurements.display()));
        if source.attestation_type != DEFAULT_ATTESTATION_TYPE {
            flags.push_str(&format!(" --attestation-type {}", source.attestation_type));
        }
    } else if let Some(policy) = &source.policy {
        flags.push_str(&format!(" --policy {}", policy.display()));
    }
    if let Some(pccs_url) = &verifier.pccs_url {
        flags.push_str(&format!(" --pccs-url {pccs_url}"));
    }
    flags
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    #[command(flatten)]
    pub node: NodeArgs,

    /// Network manifest JSON the node is expected to have booted with (the one
    /// delivered by `configure`). Its exact bytes are the network identity
    /// the quote's binding commits to, and it pins the measurement policy the
    /// quote is checked against. Omit it to use the current context's
    /// network.
    #[arg(long, value_name = "FILE")]
    pub manifest: Option<PathBuf>,

    #[command(flatten)]
    pub policy_source: PolicySourceArgs,

    #[command(flatten)]
    pub verifier: VerifierArgs,
}

pub async fn run(args: VerifyArgs) -> anyhow::Result<ExitCode> {
    let (_, descriptor) = args.node.load()?;
    check_policy_source_files(&args.policy_source, false)?;
    let manifest_path = crate::resolve_manifest(args.manifest.as_deref(), &args.node.context)?;
    let manifest = crate::load_manifest(&manifest_path)?;

    let policy = resolve_policy(&args.policy_source, &manifest_path, &manifest, false)?;
    eprintln!("Appraising against {}", policy.source);
    verify_deployment(
        &descriptor,
        &descriptor.attestation_rpc_url(),
        &manifest,
        &policy.bytes,
        args.verifier.pccs_url.as_deref(),
    )
    .await?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use seismic_tee_common::network_dir::MANIFEST_FILENAME;

    use super::*;
    use crate::test_support::{manifest_pinning, refused_url, write_file};

    /// Stands in for a promoted policy document: only its bytes matter here,
    /// since the manifest commits to it by hash and the verifier is the one
    /// that parses it.
    const POLICY_BYTES: &[u8] = b"[{\"measurement_id\": \"seismic-node.vhd\"}]\n";

    /// A raw `make measure` wrapper as seismic-images emits it: numeric keys,
    /// `expected` (not `expected_any`), a zero register the schema drops, and
    /// the stamped artifact id.
    fn raw_measurements() -> String {
        serde_json::json!({
            "measurement_id": "my-build.vhd",
            "measurements": {
                "4": {"expected": "ab".repeat(32)},
                "8": {"expected": "00".repeat(32)},
                "9": {"expected": "cd".repeat(32)},
                "11": {"expected": "ef".repeat(32)},
            },
        })
        .to_string()
    }

    #[derive(Parser)]
    struct Probe {
        #[command(flatten)]
        source: PolicySourceArgs,
        #[command(flatten)]
        verifier: VerifierArgs,
    }

    fn parse(argv: &[&str]) -> Result<Probe, clap::Error> {
        Probe::try_parse_from(std::iter::once(&"probe").chain(argv))
    }

    fn source(argv: &[&str]) -> PolicySourceArgs {
        parse(argv).expect("well-formed argv").source
    }

    /// An artifact set as `assemble` writes it: a manifest pinning the policy
    /// beside it.
    struct ArtifactSet {
        _dir: tempfile::TempDir,
        manifest_path: PathBuf,
        manifest: Manifest,
        policy_path: PathBuf,
    }

    fn artifact_set() -> ArtifactSet {
        let dir = tempfile::tempdir().unwrap();
        let bytes = manifest_pinning(POLICY_BYTES);
        let manifest_path = write_file(&dir, MANIFEST_FILENAME, &bytes);
        let policy_path = write_file(&dir, POLICY_FILENAME, POLICY_BYTES);
        ArtifactSet {
            manifest: Manifest::from_json_bytes(bytes).unwrap(),
            _dir: dir,
            manifest_path,
            policy_path,
        }
    }

    fn resolve(
        set: &ArtifactSet,
        argv: &[&str],
        offer_no_verify: bool,
    ) -> anyhow::Result<ResolvedPolicy> {
        resolve_policy(
            &source(argv),
            &set.manifest_path,
            &set.manifest,
            offer_no_verify,
        )
    }

    fn failure(result: anyhow::Result<ResolvedPolicy>) -> String {
        format!("{:?}", result.unwrap_err())
    }

    #[test]
    fn defaults_to_the_policy_beside_the_manifest() {
        let set = artifact_set();
        let policy = resolve(&set, &[], false).unwrap();
        assert_eq!(policy.bytes, POLICY_BYTES);
        assert!(policy.source.contains(POLICY_FILENAME), "{}", policy.source);
        assert!(
            policy.source.contains("pinned by --manifest"),
            "{}",
            policy.source
        );
    }

    #[test]
    fn a_policy_the_manifest_does_not_pin_is_rejected() {
        let set = artifact_set();
        std::fs::write(
            &set.policy_path,
            b"[{\"measurement_id\": \"some-other-image.vhd\"}]\n",
        )
        .unwrap();

        let error = failure(resolve(&set, &[], false));
        assert!(error.contains("bootstrap_policy_hash mismatch"), "{error}");
        assert!(error.contains(POLICY_FILENAME), "{error}");
        assert!(error.contains("--measurements"), "{error}");
    }

    #[test]
    fn an_explicit_policy_elsewhere_is_pinned_just_the_same() {
        let set = artifact_set();
        std::fs::remove_file(&set.policy_path).unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let path = write_file(&elsewhere, "policy.json", POLICY_BYTES);
        let argv = ["--policy", path.to_str().unwrap()];

        assert_eq!(resolve(&set, &argv, false).unwrap().bytes, POLICY_BYTES);

        std::fs::write(&path, b"[]\n").unwrap();
        let error = failure(resolve(&set, &argv, false));
        assert!(error.contains("bootstrap_policy_hash mismatch"), "{error}");
    }

    #[test]
    fn a_missing_explicit_policy_is_rejected_by_flag_name() {
        let set = artifact_set();
        let error = failure(resolve(&set, &["--policy", "/absent/policy.json"], false));
        assert!(error.contains("--policy file not found"), "{error}");
    }

    /// The default's absence names the alternatives — and `verify` has nothing
    /// to skip (the appraisal is the whole command), so `--no-verify` appears
    /// only for a caller that offers it.
    #[test]
    fn a_missing_default_policy_names_the_alternatives() {
        let set = artifact_set();
        std::fs::remove_file(&set.policy_path).unwrap();

        let error = failure(resolve(&set, &[], false));
        assert!(error.contains(POLICY_FILENAME), "{error}");
        assert!(error.contains("--policy"), "{error}");
        assert!(error.contains("--measurements"), "{error}");
        assert!(!error.contains("--no-verify"), "{error}");

        let error = failure(resolve(&set, &[], true));
        assert!(error.contains("--no-verify"), "{error}");
    }

    /// An operator's own measurements need not promote to the founder's
    /// policy — that is the point of the override — so nothing is hash-checked
    /// against the manifest here, and the promotion is the admission crate's.
    #[test]
    fn measurements_promote_instead_of_reading_the_artifact() {
        let set = artifact_set();
        std::fs::write(&set.policy_path, b"not the manifest's policy\n").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let measurements = write_file(&dir, "measurements.json", raw_measurements().as_bytes());

        let promoted = resolve(
            &set,
            &["--measurements", measurements.to_str().unwrap()],
            false,
        )
        .unwrap();

        assert!(
            promoted.source.contains("--measurements"),
            "{}",
            promoted.source
        );
        assert!(
            promoted.source.contains(DEFAULT_ATTESTATION_TYPE),
            "{}",
            promoted.source
        );
        let records: serde_json::Value = serde_json::from_slice(&promoted.bytes).unwrap();
        let record = &records[0];
        assert_eq!(record["measurement_id"], "my-build.vhd");
        assert_eq!(record["attestation_type"], DEFAULT_ATTESTATION_TYPE);
        let registers: Vec<_> = record["measurements"].as_object().unwrap().keys().collect();
        assert_eq!(registers, ["pcr4", "pcr9", "pcr11"]);
    }

    /// The compiler's verdict on a measurements file is the failure, by flag.
    #[test]
    fn rejected_measurements_fail_fast() {
        let set = artifact_set();
        let dir = tempfile::tempdir().unwrap();
        let measurements = write_file(&dir, "measurements.json", b"{}");

        let error = failure(resolve(
            &set,
            &["--measurements", measurements.to_str().unwrap()],
            false,
        ));
        assert!(error.contains("--measurements"), "{error}");
        assert!(error.contains("measurement_id"), "{error}");
    }

    #[test]
    fn two_policy_sources_are_rejected() {
        assert!(parse(&["--policy", "p.json", "--measurements", "m.json"]).is_err());
    }

    #[test]
    fn no_verify_contradicts_a_policy_source_and_measurements_must_exist() {
        let error = check_policy_source_files(&source(&["--policy", "p.json"]), true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("--no-verify"), "{error}");

        check_policy_source_files(&source(&["--policy", "p.json"]), false).unwrap();

        let error =
            check_policy_source_files(&source(&["--measurements", "/absent/m.json"]), false)
                .unwrap_err()
                .to_string();
        assert!(error.contains("--measurements file not found"), "{error}");
    }

    /// A suggested retry repeats the appraisal this run chose, and nothing
    /// that was default.
    #[test]
    fn retry_flags_repeat_the_non_default_choices() {
        let probe = parse(&[]).unwrap();
        assert_eq!(retry_flags(&probe.source, &probe.verifier), "");

        let probe = parse(&["--policy", "p.json", "--pccs-url", "http://pccs"]).unwrap();
        assert_eq!(
            retry_flags(&probe.source, &probe.verifier),
            " --policy p.json --pccs-url http://pccs"
        );

        let probe = parse(&["--measurements", "m.json"]).unwrap();
        assert_eq!(
            retry_flags(&probe.source, &probe.verifier),
            " --measurements m.json"
        );

        let probe = parse(&["--measurements", "m.json", "--attestation-type", "gcp-tdx"]).unwrap();
        assert_eq!(
            retry_flags(&probe.source, &probe.verifier),
            " --measurements m.json --attestation-type gcp-tdx"
        );
    }

    /// A failed challenge is a nonzero exit naming the node and the warning,
    /// and the verifier's reason travels with it.
    #[tokio::test]
    async fn a_failed_verification_names_the_node_and_the_reason() {
        let set = artifact_set();
        let descriptor = NodeDescriptor {
            public_ip: "203.0.113.7".into(),
            fqdn: "node1.example.com".into(),
        };
        // A policy the verifier parses, so the failure is the challenge's: the
        // endpoint refuses the connection.
        let policy = br#"[{"attestation_type": "azure-tdx", "measurement_id": "img.vhd",
            "measurements": {"pcr4": {"expected_any":
            ["d57063c0669599b885c43a0683436a3463ad49513ddb3996e6fc96040508fd8e"]}}}]"#;
        let endpoint = refused_url();

        let error = format!(
            "{:?}",
            verify_deployment(&descriptor, &endpoint, &set.manifest, policy, None)
                .await
                .unwrap_err()
        );
        assert!(
            error.contains("node1.example.com (203.0.113.7): deploy verification FAILED"),
            "{error}"
        );
        assert!(error.contains("Do not rely"), "{error}");
        assert!(error.contains(&endpoint), "{error}");
    }

    /// Fail-closed on the policy: an unparseable one stops the run before the
    /// node is contacted.
    #[tokio::test]
    async fn a_malformed_policy_is_rejected_before_the_challenge() {
        let set = artifact_set();
        let error = format!(
            "{:?}",
            challenge_node("http://127.0.0.1:1", &set.manifest, b"{ not a policy", None)
                .await
                .unwrap_err()
        );
        assert!(error.contains("measurement policy"), "{error}");
    }
}
