//! `admission`: measurement admission from the human side — the policies
//! that decide which images a network accepts.
//!
//! The governance party's group. The trust model's governance action is
//! "change the accepted measurement set", and this is its tooling: the
//! pipeline from an image's measurements to the policy record a network
//! accepts, in the order it is run. The steps that exist today are the
//! authoring and review half; the mutation half — proposing a record to the
//! manifest-pinned authority contract, deprecating one — is open and belongs
//! here when it lands. Policy semantics (which registers form guest identity,
//! which value forms are canonical, how admission IDs key registry storage)
//! have one implementation, the enclave's `seismic-measurement-admission`
//! crate, and every command here is that crate at the command line:
//!
//! - `promote` turns raw `make measure` output into the policy document: the
//!   record a network's `measurement-policy-bootstrap.json` carries at
//!   founding, and the record a later image is proposed to the accepted set
//!   as. An already-promoted document passes through byte-verbatim, since
//!   the manifest commits to its bytes by hash.
//! - `compile` reports what a policy admits — the admission IDs and the
//!   registry genesis storage seeding them: what `network assemble` writes
//!   into the reth genesis, what the genesis gate's error text points a
//!   reviewer at, and what the enclave's golden fixtures are regenerated
//!   with. The review step before a record is pinned or proposed.
//!
//! The group is a crate of its own, depending on neither the operator nor the
//! founder side, so that what governance needs — today the admission
//! compiler, later a transaction client and a signer — never becomes a
//! dependency of theirs.
//!
//! Both commands print JSON to stdout, so a report can be read, diffed, or
//! filed beside the policy it describes.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context as _;
use clap::Subcommand;
use seismic_measurement_admission::{CompileReport, compile_policy, promote_measurements};

/// The `admission` command group, declared in pipeline order — measurements
/// to policy record to review — which is the order `--help` lists them in.
#[derive(Debug, Subcommand)]
pub enum AdmissionCommand {
    /// Promote raw `make measure` output into a measurement-policy document
    /// (JSON on stdout): one record binding exactly the schema registers,
    /// compiled before it is emitted. An input that already is a record
    /// list is compiled and passed through byte-verbatim.
    Promote {
        /// The make-measure measurements file.
        #[arg(value_name = "MEASUREMENTS")]
        measurements: PathBuf,
        /// Policy record id, conventionally the registered image artifact
        /// filename; overrides one stamped into the measurements file.
        #[arg(long)]
        measurement_id: Option<String>,
        /// Default attestation type when the measurements file carries none.
        #[arg(long)]
        attestation_type: Option<String>,
    },
    /// Compile a measurement-policy document into the admission IDs it
    /// admits and the registry genesis storage seeding them (JSON on stdout).
    Compile {
        /// The measurement-policy document, e.g. a network directory's
        /// measurement-policy-bootstrap.json.
        #[arg(value_name = "POLICY")]
        policy: PathBuf,
    },
}

/// Run one `admission` command, writing its report to stdout.
pub fn run(command: AdmissionCommand) -> anyhow::Result<ExitCode> {
    let report = report(command)?;
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&report)?;
    stdout.flush()?;
    Ok(ExitCode::SUCCESS)
}

/// The bytes a command puts on stdout.
///
/// Bytes rather than a string: a passed-through policy is hash-committed, so
/// it must reach stdout exactly as the library produced it.
fn report(command: AdmissionCommand) -> anyhow::Result<Vec<u8>> {
    match command {
        AdmissionCommand::Promote {
            measurements,
            measurement_id,
            attestation_type,
        } => {
            let bytes = read("the measurements", &measurements)?;
            Ok(promote_measurements(
                &bytes,
                measurement_id.as_deref(),
                attestation_type.as_deref(),
            )?)
        }
        AdmissionCommand::Compile { policy } => {
            let bytes = read("the policy", &policy)?;
            let compiled = compile_policy(&bytes)?;
            Ok(CompileReport::new(&compiled).to_json().into_bytes())
        }
    }
}

/// Read an input document, so a failure names the path rather than the
/// artifact it was supposed to hold.
fn read(what: &str, path: &Path) -> anyhow::Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("reading {what} {}", path.display()))
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    fn write_file(dir: &tempfile::TempDir, name: &str, contents: &[u8]) -> PathBuf {
        let path = dir.path().join(name);
        std::fs::write(&path, contents).expect("writing test input");
        path
    }

    #[derive(Parser)]
    struct Probe {
        #[command(subcommand)]
        command: AdmissionCommand,
    }

    fn parse(argv: &[&str]) -> AdmissionCommand {
        Probe::try_parse_from(std::iter::once(&"admission").chain(argv))
            .expect("well-formed argv")
            .command
    }

    /// `make measure` output as its wrapper: every populated register, the
    /// stamped artifact id, extra fields the policy must not carry.
    fn raw_measurements() -> String {
        serde_json::json!({
            "measurement_id": "img.vhd",
            "attestation_type": "azure-tdx",
            "measurements": {
                "4": {"expected": "ab".repeat(32)},
                "9": {"expected": "cd".repeat(32)},
                "11": {"expected": "ef".repeat(32)},
                "12": {"expected": "00".repeat(32)},
            },
            "event_log": [],
        })
        .to_string()
    }

    fn promote(path: &Path, extra: &[&str]) -> AdmissionCommand {
        let mut argv = vec!["promote", path.to_str().unwrap()];
        argv.extend_from_slice(extra);
        parse(&argv)
    }

    fn compile(path: &Path) -> AdmissionCommand {
        parse(&["compile", path.to_str().unwrap()])
    }

    #[test]
    fn the_command_tree_is_well_formed() {
        use clap::CommandFactory as _;
        Probe::command().debug_assert();
    }

    #[test]
    fn promote_selects_the_schema_registers_and_compile_reports_them() {
        let dir = tempfile::tempdir().unwrap();
        let raw = write_file(&dir, "measurements.json", raw_measurements().as_bytes());

        let policy = report(promote(&raw, &["--attestation-type", "azure-tdx"])).unwrap();
        let records: serde_json::Value = serde_json::from_slice(&policy).unwrap();
        let record = &records[0];
        assert_eq!(record["measurement_id"], "img.vhd");
        assert_eq!(record["attestation_type"], "azure-tdx");
        let registers: Vec<_> = record["measurements"].as_object().unwrap().keys().collect();
        assert_eq!(registers, ["pcr4", "pcr9", "pcr11"]);

        let policy_path = write_file(&dir, "policy.json", &policy);
        let compiled: serde_json::Value =
            serde_json::from_slice(&report(compile(&policy_path)).unwrap()).unwrap();
        assert_eq!(compiled["accepted_count"], 1);
        assert!(compiled["registry_genesis_storage"].is_object());
    }

    /// The manifest commits to a policy's bytes, so an already-promoted
    /// document leaves exactly as it arrived — odd formatting included.
    #[test]
    fn an_already_promoted_policy_passes_through_verbatim() {
        let dir = tempfile::tempdir().unwrap();
        let odd = format!(
            "[{{\"measurement_id\": \"x\", \"attestation_type\": \"azure-tdx\",   \
             \"measurements\": {{\"4\": {{\"expected\": \"{}\"}}, \"9\": {{\"expected\": \
             \"{}\"}}, \"11\": {{\"expected\": \"{}\"}}}}}}]",
            "ab".repeat(32),
            "cd".repeat(32),
            "ef".repeat(32)
        );
        let path = write_file(&dir, "policy.json", odd.as_bytes());
        assert_eq!(report(promote(&path, &[])).unwrap(), odd.as_bytes());
    }

    /// The compiler's diagnostics are the failure: a wrapper missing a schema
    /// register is refused by naming it.
    #[test]
    fn promote_surfaces_compiler_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        let partial = serde_json::json!({
            "measurement_id": "img.vhd",
            "measurements": {"4": {"expected": "ab".repeat(32)}},
        })
        .to_string();
        let path = write_file(&dir, "partial.json", partial.as_bytes());
        let error = format!("{:?}", report(promote(&path, &[])).unwrap_err());
        assert!(error.contains("pcr9"), "{error}");

        let empty = write_file(&dir, "empty.json", b"[]");
        assert!(report(compile(&empty)).is_err());
    }

    /// A mistyped path is reported as the path, never as a bad policy.
    #[test]
    fn a_missing_file_is_named() {
        let error = format!(
            "{:?}",
            report(compile(Path::new("/absent/policy.json"))).unwrap_err()
        );
        assert!(error.contains("/absent/policy.json"), "{error}");
    }
}
