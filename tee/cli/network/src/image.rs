//! The image a network is founded on, by its seismic-images release.
//!
//! seismic-images publishes each image it builds as a GitHub release tagged
//! with the image's name, `seismic_<date>.<commit>`, carrying everything a
//! founding takes from the image: `image.json` (which image, what it was
//! built from, and per cloud target where its bytes are), the measurements,
//! the two genesis templates as the image's own code has them, and the
//! `seismic-reth` and `summit` binaries lifted out of the image's initrd —
//! with one `SHA256SUMS` over all of it, `image.json` included. So the
//! directory an `image.json` sits in is the whole identity: `init
//! --image-json <PATH_OR_URL>` takes its inputs from beside the record and
//! copies the record into `inputs/`, and `assemble` reads the image's name
//! back from there to run the image's own binaries from its release,
//! verified against the same `SHA256SUMS`, instead of two binaries on PATH at
//! a rev derived by hand across two repos. A local build's `build/` has the
//! same layout, so an image built by hand founds the same way.
//!
//! `SHA256SUMS` comes from the same place as what it lists, so whoever could
//! swap an asset could swap it too. Before anything is checked against it,
//! its build provenance is: a Sigstore attestation that seismic-images'
//! publishing workflow, on its publishing branch, produced exactly these
//! bytes ([`RELEASE_SIGNER`]). Every other asset matches an attested
//! `SHA256SUMS` or is refused, so each is attested bytes by the same check.
//! The check is on the bytes, not on where they came from: a release
//! verifies from a mirror or a download, and a faithful local rebuild of one
//! verifies too. A local build of anything else does not, and founds only
//! with the check skipped by name (`init --allow-unattested`).

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context as _, bail};
use seismic_tee_common::NetworkDir;
use serde::Deserialize;
use sha2::{Digest as _, Sha256};

/// Where seismic-images' releases download from; `<tag>/<asset>` follows.
pub const RELEASES_URL: &str = "https://github.com/SeismicSystems/seismic-images/releases/download";

/// The repository whose attestations a release's provenance is looked up in.
pub const RELEASE_REPO: &str = "SeismicSystems/seismic-images";

/// The identity a release's `SHA256SUMS` must be attested under, matched
/// exactly against the signing certificate: the workflow that publishes
/// releases, on the branch it publishes from. The workflow and the ref, not
/// the repository alone, so bytes signed by another workflow or another ref
/// — a pull request's run included — are refused.
pub const RELEASE_SIGNER: &str = "https://github.com/SeismicSystems/seismic-images/.github/workflows/seismic.yml@refs/heads/seismic";

/// The release's assets, by the names seismic-images gives them.
pub const IMAGE_JSON_ASSET: &str = "image.json";
pub const RETH_GENESIS_ASSET: &str = "reth-genesis.json";
pub const SUMMIT_STARTER_ASSET: &str = "summit-genesis-starter.toml";
pub const RETH_BIN_ASSET: &str = "seismic-reth";
pub const SUMMIT_BIN_ASSET: &str = "summit";
pub const SHA256SUMS_ASSET: &str = "SHA256SUMS";

/// How long one asset download may take, binaries included (the larger is
/// about 65 MB). The connect timeout is the one that fires for a host that
/// is not there; this bounds a stalled transfer.
pub const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);

/// One image's release files: the directory its `image.json` sits in, every
/// other asset beside it under the name seismic-images gives it. A GitHub
/// release's download directory is one; so is a local build's `build/`
/// once seismic-images' `make founding-inputs` and `make image-json` have
/// run in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRelease {
    base: Base,
    /// The GitHub CLI that verifies `SHA256SUMS`'s build provenance; `None`
    /// when the founder opted out (`init --allow-unattested`) and for a
    /// test's local release, which no workflow built.
    gh: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Base {
    /// `<url>/<asset>`, the URL with no trailing slash.
    Url(String),
    Dir(PathBuf),
}

impl ImageRelease {
    /// seismic-images' GitHub release of image `tag`.
    pub fn new(tag: &str) -> Self {
        Self::at(&format!("{RELEASES_URL}/{tag}"))
    }

    /// The release files under the URL `base` — a test's local server, a
    /// mirror.
    pub fn at(base: &str) -> Self {
        Self {
            base: Base::Url(base.trim_end_matches('/').to_string()),
            gh: Some("gh".to_string()),
        }
    }

    /// The release whose `image.json` is `source`, a local path or an
    /// `https://` URL. It must be named `image.json`, the name `SHA256SUMS`
    /// lists it under.
    pub fn beside(source: &str) -> anyhow::Result<Self> {
        if source.starts_with("http://") {
            bail!("insecure URL rejected (use https://): {source}");
        }
        let named = |name: &str| {
            if name == IMAGE_JSON_ASSET {
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "{source} is not an {IMAGE_JSON_ASSET} — name the image by its record, as \
                     seismic-images writes it, with the release's other files beside it"
                ))
            }
        };
        if source.starts_with("https://") {
            let (base, name) = source.rsplit_once('/').expect("the scheme has a slash");
            named(name)?;
            return Ok(Self::at(base));
        }
        let path = Path::new(source);
        named(&path.file_name().unwrap_or_default().to_string_lossy())?;
        let dir = match path.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir,
            _ => Path::new("."),
        };
        Ok(Self {
            base: Base::Dir(dir.to_path_buf()),
            gh: Some("gh".to_string()),
        })
    }

    /// The same release, its provenance taken on trust: `SHA256SUMS` is
    /// still what every asset is checked against, but nothing checks who
    /// built it.
    pub fn unattested(self) -> Self {
        self.verified_with(None)
    }

    pub(crate) fn verified_with(mut self, gh: Option<&str>) -> Self {
        self.gh = gh.map(str::to_string);
        self
    }

    /// Whether `SHA256SUMS`'s build provenance is verified before use.
    pub fn is_attested(&self) -> bool {
        self.gh.is_some()
    }

    /// Where `asset` is: a URL, or a local path.
    pub fn url(&self, asset: &str) -> String {
        match &self.base {
            Base::Url(base) => format!("{base}/{asset}"),
            Base::Dir(dir) => dir.join(asset).display().to_string(),
        }
    }
}

/// The release's `SHA256SUMS`: asset name → digest, as `sha256sum` writes
/// it (bare filenames, so the file is checked from wherever the assets were
/// downloaded to).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sha256Sums(BTreeMap<String, [u8; 32]>);

impl Sha256Sums {
    pub fn parse(text: &[u8]) -> anyhow::Result<Self> {
        let text = std::str::from_utf8(text).context("SHA256SUMS is not UTF-8")?;
        let mut sums = BTreeMap::new();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let Some((digest, name)) = line.split_once(char::is_whitespace) else {
                bail!("SHA256SUMS line has no name: {line:?}");
            };
            // `sha256sum` marks binary mode with a leading `*`.
            let name = name.trim_start().trim_start_matches('*');
            let mut bytes = [0u8; 32];
            if digest.len() != 64 || hex::decode_to_slice(digest, &mut bytes).is_err() {
                bail!("SHA256SUMS line has no sha256 digest: {line:?}");
            }
            sums.insert(name.to_string(), bytes);
        }
        if sums.is_empty() {
            bail!("SHA256SUMS lists nothing");
        }
        Ok(Self(sums))
    }

    pub fn lists(&self, asset: &str) -> bool {
        self.0.contains_key(asset)
    }

    pub fn digest(&self, asset: &str) -> Option<[u8; 32]> {
        self.0.get(asset).copied()
    }

    /// `bytes` are `asset` as the release hashed it, or an error naming both
    /// digests. An asset the file does not list is an error too: a release
    /// that stopped hashing something is not one to trust silently.
    pub fn verify(&self, asset: &str, bytes: &[u8]) -> anyhow::Result<()> {
        let Some(expected) = self.0.get(asset) else {
            bail!("SHA256SUMS lists no {asset}");
        };
        let actual: [u8; 32] = Sha256::digest(bytes).into();
        if actual != *expected {
            bail!(
                "{asset} does not match the release's SHA256SUMS: expected {}, got {}",
                hex::encode(expected),
                hex::encode(actual)
            );
        }
        Ok(())
    }
}

/// seismic-images' `image.json`: the image's record, as the release carries
/// it and as `init` copies it into `inputs/`.
///
/// Only what this CLI reads is typed. Each target's other keys — the blob
/// URL, the storage account — are the provisioner's to read from the same
/// file, and travel through untouched.
#[derive(Debug, Clone, Deserialize)]
pub struct ImageRecord {
    /// The image's name: the release tag, the stem of `measurement_id`.
    pub image: String,
    /// The seismic-images commit that built it.
    pub commit: String,
    /// The source pins it was built from (`seismic_reth`, `summit`, `enclave`).
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
    /// Per attestation type / cloud: where the bytes are and which
    /// measurements asset describes them.
    pub targets: BTreeMap<String, ImageTarget>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageTarget {
    /// The measurements asset for this target, by its name in the release.
    pub measurements: String,
    #[serde(flatten)]
    pub rest: serde_json::Map<String, serde_json::Value>,
}

impl ImageRecord {
    pub fn parse(bytes: &[u8], source: &str) -> anyhow::Result<Self> {
        serde_json::from_slice(bytes)
            .with_context(|| format!("{source} is not seismic-images' image.json"))
    }

    /// The image's GitHub release: the image's name is its tag.
    pub fn release(&self) -> ImageRelease {
        ImageRelease::new(&self.image)
    }

    /// The VHD name the image's measurements are stamped with.
    pub fn vhd(&self) -> String {
        format!("{}.vhd", self.image)
    }

    /// The measurements asset for `target` (an attestation type, `azure-tdx`).
    pub fn measurements_asset(&self, target: &str) -> anyhow::Result<&str> {
        match self.targets.get(target) {
            Some(t) => Ok(&t.measurements),
            None => bail!(
                "image {} is not published for {target}; its targets: {}",
                self.image,
                self.targets
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    /// The record `init` left in a network directory.
    pub fn read(dir: &NetworkDir) -> anyhow::Result<Self> {
        let path = dir.input_image();
        if !path.is_file() {
            bail!(
                "{} not found — `init --image-json` writes it, and without it the image's own \
                 binaries cannot be fetched; pass --reth-bin and --summit-bin",
                path.display()
            );
        }
        let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse(&bytes, &path.display().to_string())
    }
}

/// The client release assets download with: https on the request and on
/// every redirect hop (GitHub redirects release downloads to its object
/// store), and a bound on a stalled transfer rather than a total budget a
/// binary could not fit in.
pub fn download_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(seismic_tee_common::http::USER_AGENT)
        .connect_timeout(seismic_tee_common::http::CONNECT_TIMEOUT)
        .timeout(DOWNLOAD_TIMEOUT)
        .redirect(crate::init::https_only_redirects())
        .build()?)
}

/// GET one asset of a URL release.
async fn get(
    client: &reqwest::Client,
    release: &ImageRelease,
    asset: &str,
) -> anyhow::Result<reqwest::Response> {
    let url = release.url(asset);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("failed to fetch {url}: {e}"))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("no {asset} at {url}{MISSING_HINT}");
    }
    response
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("failed to fetch {url}: {e}"))
}

/// What a missing asset means, since the fix is to name another image.
const MISSING_HINT: &str = " — a seismic-images release carries every founding input beside its \
                            image.json, and so does a build's build/ once `make founding-inputs` \
                            and `make image-json` have run";

/// One small asset's bytes, from wherever the release is.
async fn read(
    client: &reqwest::Client,
    release: &ImageRelease,
    asset: &str,
) -> anyhow::Result<Vec<u8>> {
    let dir = match &release.base {
        Base::Url(_) => {
            let url = release.url(asset);
            return Ok(get(client, release, asset)
                .await?
                .bytes()
                .await
                .with_context(|| format!("reading {url}"))?
                .to_vec());
        }
        Base::Dir(dir) => dir,
    };
    let path = dir.join(asset);
    if !path.is_file() {
        bail!("no {asset} at {}{MISSING_HINT}", path.display());
    }
    std::fs::read(&path).with_context(|| format!("reading {}", path.display()))
}

/// A `SHA256SUMS` refused for its build provenance. Typed so that each
/// caller can attach its own way out: `init` has `--allow-unattested`,
/// `assemble` has binaries of the founder's own.
#[derive(Debug)]
pub struct Unattested(String);

impl std::fmt::Display for Unattested {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Unattested {}

/// The release's `SHA256SUMS`, the record every other fetch is checked
/// against — so the one asset whose build provenance is verified here, before
/// it is trusted with anything.
pub async fn fetch_sums(
    client: &reqwest::Client,
    release: &ImageRelease,
) -> anyhow::Result<Sha256Sums> {
    let bytes = read(client, release, SHA256SUMS_ASSET).await?;
    verify_provenance(release, &bytes).await?;
    Sha256Sums::parse(&bytes).with_context(|| release.url(SHA256SUMS_ASSET))
}

/// `sums` carry a build provenance attestation signed as [`RELEASE_SIGNER`],
/// as `gh attestation verify` checks one: the signature, the certificate's
/// chain to Sigstore's root and its identity, and the transparency-log entry.
/// No checksum-only fallback: a founding on a release whose provenance could
/// not be checked is a founding on whatever the release holds, so skipping
/// the check is the founder's explicit choice ([`ImageRelease::unattested`]),
/// never a fallback.
async fn verify_provenance(release: &ImageRelease, sums: &[u8]) -> anyhow::Result<()> {
    const GH_HINT: &str = "install the GitHub CLI (https://cli.github.com) and run `gh auth login`";
    let Some(gh) = &release.gh else {
        return Ok(());
    };
    let file = crate::shell_outs::temp_file(sums, "")?;
    let path = crate::shell_outs::path_str(file.path());
    let verify = [
        gh.as_str(),
        "attestation",
        "verify",
        path,
        "--repo",
        RELEASE_REPO,
    ];
    let pinned = [&verify[..], &["--cert-identity", RELEASE_SIGNER]].concat();
    let Err(refusal) = crate::shell_outs::run(&pinned, GH_HINT).await else {
        return Ok(());
    };
    // gh's refusal of an identity says only that verification failed. Asked
    // again without the pin, it reports whom the certificates do name — for
    // the message alone; the pinned check above is the one that decided.
    let unpinned = [&verify[..], &["--format", "json"]].concat();
    let found = match crate::shell_outs::run(&unpinned, GH_HINT).await {
        Ok(json) => format!("its attestation is signed by {}", signers(&json)),
        Err(_) => refusal.to_string(),
    };
    Err(Unattested(format!(
        "refusing {}: it is not attested as built by {RELEASE_SIGNER} — {found}",
        release.url(SHA256SUMS_ASSET)
    ))
    .into())
}

/// The signing identities in `gh attestation verify --format json` output.
fn signers(json: &[u8]) -> String {
    let names: Vec<String> = serde_json::from_slice::<Vec<serde_json::Value>>(json)
        .unwrap_or_default()
        .iter()
        .filter_map(|verified| {
            verified
                .pointer("/verificationResult/signature/certificate/subjectAlternativeName")?
                .as_str()
                .map(str::to_string)
        })
        .collect();
    if names.is_empty() {
        "an identity gh did not report".to_string()
    } else {
        names.join(", ")
    }
}

/// Fetch a small asset into memory and check it against `sums`. An asset
/// `SHA256SUMS` does not list is refused rather than taken on trust: a
/// release that stopped hashing something is not one to found on.
pub async fn fetch_checked(
    client: &reqwest::Client,
    release: &ImageRelease,
    sums: &Sha256Sums,
    asset: &str,
) -> anyhow::Result<Vec<u8>> {
    let url = release.url(asset);
    let bytes = read(client, release, asset).await?;
    // Flat, not layered: the mismatch is the message a founder must read,
    // and anyhow renders only the outermost context in a one-line error.
    if let Err(mismatch) = sums.verify(asset, &bytes) {
        bail!("{mismatch} ({url})");
    }
    Ok(bytes)
}

/// `$XDG_CACHE_HOME/seismic/images`, else `~/.cache/seismic/images`: where
/// the image binaries are kept between runs, one directory per image.
pub fn default_cache_dir() -> anyhow::Result<PathBuf> {
    cache_dir(std::env::var_os("XDG_CACHE_HOME"), std::env::var_os("HOME"))
}

fn cache_dir(
    xdg: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> anyhow::Result<PathBuf> {
    let base = match xdg.filter(|v| !v.is_empty()) {
        Some(xdg) => PathBuf::from(xdg),
        None => match home.filter(|v| !v.is_empty()) {
            Some(home) => PathBuf::from(home).join(".cache"),
            None => bail!(
                "neither XDG_CACHE_HOME nor HOME is set, so there is nowhere to keep the image's \
                 binaries; pass --reth-bin and --summit-bin"
            ),
        },
    };
    Ok(base.join("seismic").join("images"))
}

/// Whether this machine can run the image's binaries: x86-64 Linux, and not
/// x86-64 by way of Rosetta.
///
/// The binaries are the image's own — x86-64, dynamically linked against
/// Debian trixie's glibc (symbols up to 2.38) and OpenSSL 3 — so a Mac, an
/// arm64 box, or a glibc older than 2.38 cannot run them. The architecture
/// is asked of the kernel rather than the compiler: inside a Lima VM on
/// Apple silicon an x86-64 build of this CLI runs under Rosetta and `uname`
/// says x86_64, but the image's `summit` has crashed inside Rosetta there,
/// so a registered Rosetta handler counts as "not x86-64" too. Returns what
/// is wrong, for the caller to pair with the way out.
pub fn host_runs_image_binaries() -> Result<(), String> {
    if !cfg!(target_os = "linux") {
        return Err(format!("this is {}, not Linux", std::env::consts::OS));
    }
    let arch = std::fs::read_to_string("/proc/sys/kernel/arch")
        .map(|a| a.trim().to_string())
        .unwrap_or_else(|_| std::env::consts::ARCH.to_string());
    if arch != "x86_64" {
        return Err(format!("this machine is {arch}, not x86-64"));
    }
    if Path::new("/proc/sys/fs/binfmt_misc/rosetta").exists() {
        return Err("this is x86-64 only through Rosetta, where the image's binaries crash".into());
    }
    Ok(())
}

/// The image's `asset` (one of the two binaries), on disk in `dir` (the
/// image's own directory under the cache), verified against `sums` — a
/// cached copy is re-hashed on every use, so a file tampered with or
/// truncated on disk is refetched rather than run. Downloads stream to a
/// `.part` beside the final name and are hashed as they arrive; the file
/// only takes its name once the digest matches, so a `<dir>/seismic-reth`
/// that exists is one that verified.
pub async fn fetch_binary(
    client: &reqwest::Client,
    release: &ImageRelease,
    sums: &Sha256Sums,
    asset: &str,
    dir: &Path,
) -> anyhow::Result<PathBuf> {
    let path = dir.join(asset);
    if path.is_file() {
        let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        match sums.verify(asset, &bytes) {
            Ok(()) => return Ok(path),
            Err(e) => eprintln!(
                "cached {} does not verify ({e}); fetching it again",
                path.display()
            ),
        }
    }
    if !sums.lists(asset) {
        bail!("{} lists no {asset}", release.url(SHA256SUMS_ASSET));
    }
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;

    let url = release.url(asset);
    eprintln!("fetching {url}");
    let part = dir.join(format!("{asset}.part"));
    let mut file =
        std::fs::File::create(&part).with_context(|| format!("creating {}", part.display()))?;
    let mut hasher = Sha256::new();
    let mut write = |chunk: &[u8]| {
        hasher.update(chunk);
        file.write_all(chunk)
            .with_context(|| format!("writing {}", part.display()))
    };
    match &release.base {
        Base::Url(_) => {
            let mut response = get(client, release, asset).await?;
            while let Some(chunk) = response
                .chunk()
                .await
                .with_context(|| format!("reading {url}"))?
            {
                write(&chunk)?;
            }
        }
        Base::Dir(_) => write(&read(client, release, asset).await?)?,
    }
    file.flush()
        .with_context(|| format!("writing {}", part.display()))?;
    drop(file);
    let actual: [u8; 32] = hasher.finalize().into();
    let expected = sums.digest(asset).expect("listed: checked above");
    if actual != expected {
        let _ = std::fs::remove_file(&part);
        bail!(
            "{url} does not match the release's SHA256SUMS: expected {}, got {} — nothing kept",
            hex::encode(expected),
            hex::encode(actual)
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&part, std::fs::Permissions::from_mode(0o755))
            .with_context(|| format!("marking {} executable", part.display()))?;
    }
    std::fs::rename(&part, &path)
        .with_context(|| format!("moving {} into place", part.display()))?;
    Ok(path)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeMap;

    use seismic_tee_common::test_support::FileServer;

    use super::*;

    pub(crate) const TAG: &str = "seismic_2026-09-22.2ee71c";

    /// A release as seismic-images publishes it, served locally: `image.json`
    /// naming the target, the measurements stamped for the tag, both
    /// templates, two "binaries", and SHA256SUMS over every one of them and
    /// the UKI.
    pub(crate) fn release_files(tag: &str) -> BTreeMap<String, Vec<u8>> {
        let image_json = serde_json::json!({
            "image": tag,
            "commit": "2ee71cadfef6164454f71e0ca3c27d0a18fa0527",
            "sources": {"seismic_reth": "39d04d1", "summit": "5e59220", "enclave": "2422116"},
            "targets": {"azure-tdx": {
                "vhd_blob_url": format!("https://seismicimages.blob.core.windows.net/dev/{tag}.vhd"),
                "storage_account_id": "/subscriptions/x/resourceGroups/seismic-images/providers/Microsoft.Storage/storageAccounts/seismicimages",
                "measurements": "measurements.azure-tdx.json",
                "efi_sha256": "aa".repeat(32),
            }},
        });
        let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::from([
            (
                IMAGE_JSON_ASSET.to_string(),
                serde_json::to_vec_pretty(&image_json).unwrap(),
            ),
            (
                "measurements.azure-tdx.json".to_string(),
                format!(r#"{{"measurement_id": "{tag}.vhd", "measurements": {{"4": {{"expected": "ab"}}}}}}"#)
                    .into_bytes(),
            ),
            (
                RETH_GENESIS_ASSET.to_string(),
                br#"{"config": {"chainId": 5124}}"#.to_vec(),
            ),
            (
                SUMMIT_STARTER_ASSET.to_string(),
                b"# starter params\nnamespace = \"\"\nleader_timeout_ms = 2000\n".to_vec(),
            ),
            (
                RETH_BIN_ASSET.to_string(),
                b"#!/bin/sh\necho 0x1111111111111111111111111111111111111111111111111111111111111111\n".to_vec(),
            ),
            (
                SUMMIT_BIN_ASSET.to_string(),
                b"#!/bin/sh\necho 0x2222222222222222222222222222222222222222222222222222222222222222\n".to_vec(),
            ),
        ]);
        resum(&mut files, tag);
        files
    }

    /// (Re)write `files`' SHA256SUMS over every other file and image `tag`'s
    /// UKI, as seismic-images does.
    pub(crate) fn resum(files: &mut BTreeMap<String, Vec<u8>>, tag: &str) {
        files.remove(SHA256SUMS_ASSET);
        let sums: String = files
            .iter()
            .map(|(name, bytes)| format!("{}  {name}\n", hex::encode(Sha256::digest(bytes))))
            .chain(std::iter::once(format!("{}  {tag}.efi\n", "ab".repeat(32))))
            .collect();
        files.insert(SHA256SUMS_ASSET.to_string(), sums.into_bytes());
    }

    /// Serve `files` as release `tag`, at the paths GitHub would.
    pub(crate) fn serve_release(
        tag: &str,
        files: BTreeMap<String, Vec<u8>>,
    ) -> (FileServer, ImageRelease) {
        let served = files
            .into_iter()
            .map(|(name, bytes)| (format!("/{tag}/{name}"), bytes))
            .collect();
        let server = FileServer::serve(served);
        let release = ImageRelease::at(&format!("{}/{tag}", server.url)).unattested();
        (server, release)
    }

    /// `files` written into `dir`, the way a local build leaves `build/`,
    /// and the release beside its image.json. Returns the image.json's path
    /// too, as `--image-json` would name it.
    pub(crate) fn local_release(
        dir: &Path,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> (String, ImageRelease) {
        std::fs::create_dir_all(dir).unwrap();
        for (name, bytes) in files {
            std::fs::write(dir.join(name), bytes).unwrap();
        }
        let image_json = dir.join(IMAGE_JSON_ASSET).display().to_string();
        let release = ImageRelease::beside(&image_json).unwrap().unattested();
        (image_json, release)
    }

    /// A stand-in `gh` in `dir`: appends its arguments to `dir/args`, keeps
    /// the file it was asked about as `dir/subject`, then runs `pinned` when
    /// asked with `--cert-identity` and `unpinned` otherwise.
    pub(crate) fn fake_gh(dir: &Path, pinned: &str, unpinned: &str) -> String {
        let gh = dir.join("gh");
        let d = dir.display();
        std::fs::write(
            &gh,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" >> {d}/args\ncat \"$3\" > {d}/subject\n\
                 case \"$*\" in *--cert-identity*) {pinned} ;; *) {unpinned} ;; esac\n"
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        gh.display().to_string()
    }

    /// SHA256SUMS is taken only with its provenance verified: gh is asked
    /// about exactly the bytes served, pinned to the publishing workflow and
    /// its branch. A refusal names the release, the signer expected and the
    /// signer found (or gh's own error, when nothing verifies at all); no gh
    /// is a refusal too, never a fallback to the checksums alone.
    #[tokio::test]
    async fn sha256sums_must_carry_the_publishing_workflows_provenance() {
        let files = release_files(TAG);
        let (_server, release) = serve_release(TAG, files.clone());
        let client = download_client().unwrap();
        let with = |gh: &str| release.clone().verified_with(Some(gh));

        let dir = tempfile::tempdir().unwrap();
        let gh = fake_gh(dir.path(), "exit 0", "exit 1");
        fetch_sums(&client, &with(&gh)).await.unwrap();
        let args = std::fs::read_to_string(dir.path().join("args")).unwrap();
        let args: Vec<&str> = args.lines().collect();
        assert_eq!(args[..2], ["attestation", "verify"]);
        assert_eq!(
            args[3..],
            ["--repo", RELEASE_REPO, "--cert-identity", RELEASE_SIGNER]
        );
        assert_eq!(
            std::fs::read(dir.path().join("subject")).unwrap(),
            files[SHA256SUMS_ASSET]
        );

        // Attested by this repository, but by another workflow.
        let other = "https://github.com/SeismicSystems/seismic-images/.github/workflows/other.yml@refs/heads/seismic";
        let json = format!(
            r#"[{{"verificationResult":{{"signature":{{"certificate":{{"subjectAlternativeName":"{other}"}}}}}}}}]"#
        );
        let dir = tempfile::tempdir().unwrap();
        let gh = fake_gh(
            dir.path(),
            "echo 'Error: verifying with issuer' >&2; exit 1",
            &format!("echo '{json}'"),
        );
        let err = fetch_sums(&client, &with(&gh)).await.unwrap_err();
        assert!(err.is::<Unattested>(), "{err}");
        let err = err.to_string();
        assert!(err.contains(&release.url(SHA256SUMS_ASSET)), "{err}");
        assert!(err.contains(RELEASE_SIGNER), "{err}");
        assert!(err.contains(&format!("signed by {other}")), "{err}");

        // Not attested at all: gh's own word for it.
        let dir = tempfile::tempdir().unwrap();
        let fail = "echo 'Error: no attestations found' >&2; exit 1";
        let gh = fake_gh(dir.path(), fail, fail);
        let err = fetch_sums(&client, &with(&gh))
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains(RELEASE_SIGNER), "{err}");
        assert!(err.contains("no attestations found"), "{err}");

        let err = fetch_sums(&client, &with("/nonexistent/gh"))
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains(RELEASE_SIGNER), "{err}");
        assert!(err.contains("https://cli.github.com"), "{err}");
    }

    /// The check is on the bytes wherever they came from: a local build's
    /// SHA256SUMS goes to gh as a release's does.
    #[tokio::test]
    async fn a_local_releases_sha256sums_is_held_to_provenance_too() {
        let build = tempfile::tempdir().unwrap();
        let files = release_files(TAG);
        let (_, release) = local_release(build.path(), &files);
        let dir = tempfile::tempdir().unwrap();
        let fail = "echo 'Error: no attestations found' >&2; exit 1";
        let gh = fake_gh(dir.path(), fail, fail);
        let err = fetch_sums(
            &download_client().unwrap(),
            &release.verified_with(Some(&gh)),
        )
        .await
        .unwrap_err();
        assert!(err.is::<Unattested>(), "{err}");
        assert_eq!(
            std::fs::read(dir.path().join("subject")).unwrap(),
            files[SHA256SUMS_ASSET]
        );
    }

    #[test]
    fn a_release_is_the_directory_its_image_json_sits_in() {
        let release = ImageRelease::new(TAG);
        assert_eq!(
            release.url("summit"),
            format!("{RELEASES_URL}/{TAG}/summit")
        );
        let url = format!("{RELEASES_URL}/{TAG}/image.json");
        assert_eq!(ImageRelease::beside(&url).unwrap(), release);
        assert_eq!(ImageRelease::at("http://x/").url("a"), "http://x/a");

        let local = ImageRelease::beside("build/image.json").unwrap();
        assert_eq!(local.url("SHA256SUMS"), "build/SHA256SUMS");
        assert!(local.is_attested());
        assert!(!local.unattested().is_attested());
        assert_eq!(ImageRelease::beside("image.json").unwrap().url("a"), "./a");

        let err = ImageRelease::beside("http://example.test/image.json")
            .unwrap_err()
            .to_string();
        assert!(err.contains("insecure URL rejected"), "{err}");
        for other in ["build/measurements.azure-tdx.json", "https://example.test"] {
            let err = ImageRelease::beside(other).unwrap_err().to_string();
            assert!(err.contains("is not an image.json"), "{err}");
        }
    }

    #[test]
    fn sha256sums_parse_as_sha256sum_writes_them() {
        let text = format!("{}  a.bin\n{} *b.bin\n\n", "ab".repeat(32), "cd".repeat(32));
        let sums = Sha256Sums::parse(text.as_bytes()).unwrap();
        assert!(sums.lists("a.bin") && sums.lists("b.bin"));
        assert!(!sums.lists("c.bin"));

        let err = sums
            .verify("a.bin", b"not those bytes")
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match"), "{err}");
        assert!(err.contains(&"ab".repeat(32)), "{err}");
        let err = sums.verify("c.bin", b"").unwrap_err().to_string();
        assert!(err.contains("lists no c.bin"), "{err}");

        for bad in ["", "zz  a\n", "abcd  a\n", &"ab".repeat(32)] {
            assert!(Sha256Sums::parse(bad.as_bytes()).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_record_names_its_release_and_each_targets_measurements() {
        let files = release_files(TAG);
        let record = ImageRecord::parse(&files[IMAGE_JSON_ASSET], "image.json").unwrap();
        assert_eq!(record.release(), ImageRelease::new(TAG));
        assert_eq!(record.vhd(), format!("{TAG}.vhd"));
        assert_eq!(
            record.measurements_asset("azure-tdx").unwrap(),
            "measurements.azure-tdx.json"
        );
        // The provisioner's keys travel through untouched.
        assert!(
            record.targets["azure-tdx"]
                .rest
                .contains_key("vhd_blob_url")
        );
        let err = record
            .measurements_asset("gcp-tdx")
            .unwrap_err()
            .to_string();
        assert!(err.contains("not published for gcp-tdx"), "{err}");
        assert!(err.contains("azure-tdx"), "{err}");

        let err = ImageRecord::parse(b"{\"image\": 1}", "x/image.json")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("x/image.json is not seismic-images' image.json"),
            "{err}"
        );
    }

    #[test]
    fn the_cache_lives_under_xdg_cache_home_else_home() {
        assert_eq!(
            cache_dir(Some("/c".into()), Some("/h".into())).unwrap(),
            PathBuf::from("/c/seismic/images")
        );
        assert_eq!(
            cache_dir(Some("".into()), Some("/h".into())).unwrap(),
            PathBuf::from("/h/.cache/seismic/images")
        );
        assert!(cache_dir(None, None).is_err());
    }

    #[tokio::test]
    async fn assets_are_fetched_and_held_to_sha256sums() {
        let (server, release) = serve_release(TAG, release_files(TAG));
        let client = download_client().unwrap();
        let sums = fetch_sums(&client, &release).await.unwrap();
        let starter = fetch_checked(&client, &release, &sums, SUMMIT_STARTER_ASSET)
            .await
            .unwrap();
        assert!(starter.starts_with(b"# starter"));
        // The measurements are an asset like any other: hashed, so checked.
        let measurements = fetch_checked(&client, &release, &sums, "measurements.azure-tdx.json")
            .await
            .unwrap();
        assert!(measurements.starts_with(b"{\"measurement_id\""));
        // An asset SHA256SUMS does not list is refused, not taken on trust.
        let unlisted =
            Sha256Sums::parse(format!("{}  other\n", "ab".repeat(32)).as_bytes()).unwrap();
        let err = fetch_checked(&client, &release, &unlisted, SUMMIT_STARTER_ASSET)
            .await
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("lists no summit-genesis-starter.toml"),
            "{err}"
        );
        // A missing asset names the asset and where it was looked for.
        let err = fetch_checked(&client, &release, &sums, "absent")
            .await
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(&format!("no absent at {}", release.url("absent"))),
            "{err}"
        );
        drop(server);

        // A tampered asset is refused by name.
        let mut files = release_files(TAG);
        files.insert(RETH_GENESIS_ASSET.to_string(), b"{}".to_vec());
        let (_server, release) = serve_release(TAG, files);
        let sums = fetch_sums(&client, &release).await.unwrap();
        let err = fetch_checked(&client, &release, &sums, RETH_GENESIS_ASSET)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("reth-genesis.json does not match"), "{err}");
        // …and says where the bytes came from, in the one line to_string gives.
        assert!(err.contains(&release.url(RETH_GENESIS_ASSET)), "{err}");
    }

    /// A release that is not there is named by where SHA256SUMS was looked
    /// for, over https and on disk alike, with what should be there.
    #[tokio::test]
    async fn a_missing_release_is_named_as_such() {
        let (server, _) = serve_release("other", release_files("other"));
        let client = download_client().unwrap();
        let absent = ImageRelease::at(&format!("{}/{TAG}", server.url)).unattested();
        let err = fetch_sums(&client, &absent).await.unwrap_err().to_string();
        assert!(
            err.contains(&format!("no SHA256SUMS at {}", absent.url("SHA256SUMS"))),
            "{err}"
        );
        assert!(err.contains("make image-json"), "{err}");

        let empty = tempfile::tempdir().unwrap();
        let image_json = empty.path().join("image.json");
        let absent = ImageRelease::beside(image_json.to_str().unwrap())
            .unwrap()
            .unattested();
        let err = fetch_sums(&client, &absent).await.unwrap_err().to_string();
        assert!(
            err.contains(&format!("no SHA256SUMS at {}", absent.url("SHA256SUMS"))),
            "{err}"
        );
    }

    /// A local build's assets are read and checked as a release's are,
    /// binaries included.
    #[tokio::test]
    async fn a_local_releases_assets_are_held_to_its_sha256sums() {
        let build = tempfile::tempdir().unwrap();
        let (_, release) = local_release(build.path(), &release_files(TAG));
        let client = download_client().unwrap();
        let sums = fetch_sums(&client, &release).await.unwrap();
        let record = fetch_checked(&client, &release, &sums, IMAGE_JSON_ASSET)
            .await
            .unwrap();
        assert_eq!(ImageRecord::parse(&record, "r").unwrap().image, TAG);
        let cache = tempfile::tempdir().unwrap();
        let summit = fetch_binary(&client, &release, &sums, SUMMIT_BIN_ASSET, cache.path())
            .await
            .unwrap();
        assert_eq!(
            std::fs::read(summit).unwrap(),
            std::fs::read(build.path().join(SUMMIT_BIN_ASSET)).unwrap()
        );

        std::fs::write(build.path().join(RETH_GENESIS_ASSET), b"{}").unwrap();
        let err = fetch_checked(&client, &release, &sums, RETH_GENESIS_ASSET)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("reth-genesis.json does not match"), "{err}");
        assert!(err.contains(&release.url(RETH_GENESIS_ASSET)), "{err}");
    }

    /// A binary is downloaded once, verified, made executable and kept; the
    /// next run re-hashes the cached copy rather than trusting the name, so
    /// a file altered on disk is fetched again, and an upstream that no
    /// longer matches SHA256SUMS leaves nothing behind.
    #[tokio::test]
    async fn binaries_are_cached_verified_and_executable() {
        let (server, release) = serve_release(TAG, release_files(TAG));
        let client = download_client().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let cached = cache.path().join(TAG);
        let sums = fetch_sums(&client, &release).await.unwrap();

        let path = fetch_binary(&client, &release, &sums, SUMMIT_BIN_ASSET, &cached)
            .await
            .unwrap();
        assert_eq!(path, cached.join("summit"));
        assert!(!cached.join("summit.part").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o111,
                0o111
            );
        }
        let fetched_once = server
            .requests()
            .iter()
            .filter(|p| p.ends_with("/summit"))
            .count();
        assert_eq!(fetched_once, 1);

        // Cached: no second request.
        fetch_binary(&client, &release, &sums, SUMMIT_BIN_ASSET, &cached)
            .await
            .unwrap();
        assert_eq!(
            server
                .requests()
                .iter()
                .filter(|p| p.ends_with("/summit"))
                .count(),
            1
        );

        // Altered on disk: fetched again.
        std::fs::write(&path, b"#!/bin/sh\necho tampered\n").unwrap();
        fetch_binary(&client, &release, &sums, SUMMIT_BIN_ASSET, &cached)
            .await
            .unwrap();
        assert_eq!(
            server
                .requests()
                .iter()
                .filter(|p| p.ends_with("/summit"))
                .count(),
            2
        );
        sums.verify(SUMMIT_BIN_ASSET, &std::fs::read(&path).unwrap())
            .unwrap();
        drop(server);

        // Upstream no longer matches: refused, nothing kept.
        let mut files = release_files(TAG);
        files.insert(RETH_BIN_ASSET.to_string(), b"#!/bin/sh\nexit 1\n".to_vec());
        let (_server, release) = serve_release(TAG, files);
        let err = fetch_binary(&client, &release, &sums, RETH_BIN_ASSET, &cached)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("seismic-reth does not match"), "{err}");
        assert!(!cached.join("seismic-reth").exists());
        assert!(!cached.join("seismic-reth.part").exists());
    }
}
