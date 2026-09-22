//! The image a network is founded on, by its seismic-images release.
//!
//! seismic-images publishes each image it builds as a GitHub release tagged
//! with the image's name, `seismic_<date>.<commit>`, carrying everything a
//! founding takes from the image: `image.json` (which image, what it was
//! built from, and per cloud target where its bytes are), the measurements,
//! the two genesis templates as the image's own code has them, and the
//! `seismic-reth` and `summit` binaries lifted out of the image's initrd —
//! with one `SHA256SUMS` over all of it. So one tag is the whole identity:
//! `init --image <tag>` takes its inputs from that release and copies
//! `image.json` into `inputs/` as the record, and `assemble` reads the tag
//! back from there to run the image's own binaries, verified against the
//! same `SHA256SUMS`, instead of two binaries on PATH at a rev derived by
//! hand across two repos.
//!
//! Every fetch here is of a release asset, spelled from the tag; the
//! founder's own inputs (a locally authored genesis, a dev image's
//! measurements) go through `init`'s path-or-URL arguments instead.

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

/// One release of seismic-images, addressed by its tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRelease {
    tag: String,
    base: String,
}

impl ImageRelease {
    pub fn new(tag: &str) -> Self {
        Self::at(RELEASES_URL, tag)
    }

    /// A release served from somewhere other than GitHub — a test's local
    /// server, a mirror.
    pub fn at(base: &str, tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            base: base.trim_end_matches('/').to_string(),
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// The VHD name the release's measurements are stamped with.
    pub fn vhd(&self) -> String {
        format!("{}.vhd", self.tag)
    }

    pub fn url(&self, asset: &str) -> String {
        format!("{}/{}/{asset}", self.base, self.tag)
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
/// it and as `init --image` copies it into `inputs/`.
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

    /// The release this record came from: the image's name is its tag.
    pub fn release(&self) -> ImageRelease {
        ImageRelease::new(&self.image)
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

    /// The record `init --image` left in a network directory.
    pub fn read(dir: &NetworkDir) -> anyhow::Result<Self> {
        let path = dir.input_image();
        if !path.is_file() {
            bail!(
                "{} not found — the directory was not scaffolded with `init --image <tag>`, so \
                 the image's own binaries cannot be fetched; pass --reth-bin and --summit-bin",
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

/// GET one asset. A 404 says what is missing — the whole release or one of
/// its assets — since the two mean different things to the founder.
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
        bail!(
            "seismic-images release {} has no {asset} ({url}) — CI publishes a release, with \
             every founding input, for each seismic_* image it builds and for nothing else; an \
             older release or a seismic-dev_* image has to be founded from files you hold",
            release.tag()
        );
    }
    response
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("failed to fetch {url}: {e}"))
}

/// The release's `SHA256SUMS`, the record every other fetch is checked against.
pub async fn fetch_sums(
    client: &reqwest::Client,
    release: &ImageRelease,
) -> anyhow::Result<Sha256Sums> {
    let bytes = get(client, release, SHA256SUMS_ASSET)
        .await?
        .bytes()
        .await
        .with_context(|| format!("reading {}", release.url(SHA256SUMS_ASSET)))?;
    Sha256Sums::parse(&bytes).with_context(|| release.url(SHA256SUMS_ASSET))
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
    let bytes = get(client, release, asset)
        .await?
        .bytes()
        .await
        .with_context(|| format!("reading {url}"))?
        .to_vec();
    // Flat, not layered: the mismatch is the message a founder must read,
    // and anyhow renders only the outermost context in a one-line error.
    if let Err(mismatch) = sums.verify(asset, &bytes) {
        bail!("{mismatch} ({url})");
    }
    Ok(bytes)
}

/// `$XDG_CACHE_HOME/seismic/images`, else `~/.cache/seismic/images`: where
/// the image binaries are kept between runs, one directory per tag.
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

/// The image's `asset` (one of the two binaries), on disk under
/// `cache/<tag>/`, verified against `sums` — a cached copy is re-hashed on
/// every use, so a file tampered with or truncated on disk is refetched
/// rather than run. Downloads stream to a `.part` beside the final name and
/// are hashed as they arrive; the file only takes its name once the digest
/// matches, so a `<tag>/seismic-reth` that exists is one that verified.
pub async fn fetch_binary(
    client: &reqwest::Client,
    release: &ImageRelease,
    sums: &Sha256Sums,
    asset: &str,
    cache: &Path,
) -> anyhow::Result<PathBuf> {
    let dir = cache.join(release.tag());
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
        bail!("SHA256SUMS of release {} lists no {asset}", release.tag());
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;

    let url = release.url(asset);
    eprintln!("fetching {url}");
    let mut response = get(client, release, asset).await?;
    let part = dir.join(format!("{asset}.part"));
    let mut file =
        std::fs::File::create(&part).with_context(|| format!("creating {}", part.display()))?;
    let mut hasher = Sha256::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .with_context(|| format!("reading {url}"))?
    {
        hasher.update(&chunk);
        file.write_all(&chunk)
            .with_context(|| format!("writing {}", part.display()))?;
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
        let sums: String = files
            .iter()
            .map(|(name, bytes)| format!("{}  {name}\n", hex::encode(Sha256::digest(bytes))))
            .chain(std::iter::once(format!("{}  {tag}.efi\n", "ab".repeat(32))))
            .collect();
        files.insert(SHA256SUMS_ASSET.to_string(), sums.into_bytes());
        files
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
        let release = ImageRelease::at(&server.url, tag);
        (server, release)
    }

    #[test]
    fn urls_are_spelled_from_the_tag() {
        let release = ImageRelease::new(TAG);
        assert_eq!(
            release.url("summit"),
            format!("{RELEASES_URL}/{TAG}/summit")
        );
        assert_eq!(release.vhd(), format!("{TAG}.vhd"));
        assert_eq!(
            ImageRelease::at("http://x/", TAG).url("a"),
            format!("http://x/{TAG}/a")
        );
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
        // A missing asset names the release and the asset.
        let err = fetch_checked(&client, &release, &sums, "absent")
            .await
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(&format!("release {TAG} has no absent")),
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

    #[tokio::test]
    async fn a_missing_release_is_named_as_such() {
        let (_server, release) = serve_release("other", release_files("other"));
        let client = download_client().unwrap();
        let err = fetch_sums(&client, &ImageRelease::at(&_server.url, TAG))
            .await
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(&format!("release {TAG} has no SHA256SUMS")),
            "{err}"
        );
        assert!(err.contains("seismic-dev_*"), "{err}");
        let _ = release;
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
        let sums = fetch_sums(&client, &release).await.unwrap();

        let path = fetch_binary(&client, &release, &sums, SUMMIT_BIN_ASSET, cache.path())
            .await
            .unwrap();
        assert_eq!(path, cache.path().join(TAG).join("summit"));
        assert!(!cache.path().join(TAG).join("summit.part").exists());
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
        fetch_binary(&client, &release, &sums, SUMMIT_BIN_ASSET, cache.path())
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
        fetch_binary(&client, &release, &sums, SUMMIT_BIN_ASSET, cache.path())
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
        let err = fetch_binary(&client, &release, &sums, RETH_BIN_ASSET, cache.path())
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("seismic-reth does not match"), "{err}");
        assert!(!cache.path().join(TAG).join("seismic-reth").exists());
        assert!(!cache.path().join(TAG).join("seismic-reth.part").exists());
    }
}
