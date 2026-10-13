#!/bin/sh
# Installs a prebuilt `seismic-tee` — the Seismic TEE deploy CLI — from this
# repo's GitHub releases:
#
#   curl -fsSL https://raw.githubusercontent.com/SeismicSystems/seismic/main/tee/cli/install.sh | sh
#
# Everything is fetched with curl: one GitHub API call to pick the release
# (skipped when the version is spelled out) and two downloads from the
# release. The API call is anonymous unless GH_TOKEN or GITHUB_TOKEN is set,
# in which case it is sent as a bearer token — the anonymous limit is 60
# requests an hour per IP, which a shared CI runner can exhaust and a person
# never will. The token goes to api.github.com only, never to a download URL,
# which redirects to a storage host. `gh` is used for exactly one thing, the
# build provenance check, which has no curl equivalent.
#
# What it does: picks the release, downloads the tarball for this machine and
# the release's SHA256SUMS, checks the tarball against it, verifies the
# binary's build provenance with `gh attestation` when `gh` is installed and
# logged in, and installs `seismic-tee` into ~/.local/bin. Releases are tagged
# `seismic-tee/v<X.Y.Z>` (versions) and `seismic-tee/main-<sha>` (a prerelease
# per merge to main);
# the repo hosts other components too, so "Latest" is never consulted — the
# newest release under the prefix is.
#
# Options go after `sh -s --`, or in environment variables; `--help` lists
# them:
#
#   curl -fsSL .../install.sh | sh -s -- --version main --to ~/bin
#
# This script is for the first install. The binary it installs replaces
# itself afterwards — `seismic-tee --upgrade [VERSION]` fetches this script
# and runs it — so there is no separate updater to keep around.
#
# Prebuilt for linux_amd64, linux_arm64 and darwin_arm64; anything else is
# told to build from source. POSIX sh: nothing here needs bash.
set -eu

REPO=${SEISMIC_TEE_REPO:-SeismicSystems/seismic}
INSTALLER_URL=https://raw.githubusercontent.com/$REPO/main/tee/cli/install.sh
VERSION=${SEISMIC_TEE_VERSION:-}
INSTALL_DIR=${SEISMIC_TEE_INSTALL_DIR:-$HOME/.local/bin}
PREFIX=seismic-tee/
BIN=seismic-tee

# Color is for a human at a terminal: off when the stream is redirected or
# piped, when NO_COLOR is set (https://no-color.org), and for a terminal that
# cannot render it. --no-color clears it below, once the arguments are parsed.
# Each style has the role the CLI's palette gives it (tee/cli/CLAUDE.md); the
# help's are clap's, so it reads like the binary's own --help. Messages go to
# stderr and the help to stdout, so each is decided for its own stream. \033
# stays within POSIX printf's octal escapes; \e is a GNU extension dash does
# not read.
wants_color() { [ -t "$1" ] && [ -z "${NO_COLOR:-}" ] && [ "${TERM:-}" != dumb ]; }
no_color() {
    bold=
    dim=
    passed=
    yellow=
    red=
    plain=
    help_heading=
    help_literal=
    help_note=
    help_plain=
}
no_color
if wants_color 2; then
    bold=$(printf '\033[1m')
    dim=$(printf '\033[2m')
    passed=$(printf '\033[2;32m')
    yellow=$(printf '\033[1;33m')
    red=$(printf '\033[1;31m')
    plain=$(printf '\033[0m')
fi
if wants_color 1; then
    help_heading=$(printf '\033[1;4m')
    help_literal=$(printf '\033[1m')
    help_note=$(printf '\033[2m')
    help_plain=$(printf '\033[0m')
fi

say() { printf '%s\n' "$*" >&2; }
# One line of $1's style; the escapes stay out of the words.
styled() {
    style=$1
    shift
    say "$style$*$plain"
}
warn() { say "${yellow}warning:${plain} $*"; }
fail() {
    say "${red}error:${plain} $*"
    exit 1
}
# The full help on stdout for --help; the one-line form on stderr, ahead of
# the error, for a bad option.
usage() {
    say "Usage: install.sh [--version VERSION] [--to DIR] [--no-color]; --help for more"
}
help() {
    h=$help_heading l=$help_literal n=$help_note r=$help_plain
    cat <<EOF
Installs a prebuilt seismic-tee, the Seismic TEE deploy CLI, from the GitHub
releases of $REPO.

${h}Usage:${r} ${l}curl -fsSL $INSTALLER_URL | sh -s --${r} [OPTIONS]

${h}Options:${r}
  ${l}--version${r} VERSION  Which build. Default: the newest release.
                       ${l}X.Y.Z${r}, ${l}vX.Y.Z${r}  a release
                       ${l}main${r}           the newest release of main
                       ${l}main-${r}<sha>     one release of main, by its full commit
  ${l}--to${r} DIR           Where the binary goes. Default: ~/.local/bin
  ${l}--no-color${r}         Plain messages.
  ${l}-h${r}, ${l}--help${r}         This text.

${h}Environment:${r}
  ${l}SEISMIC_TEE_VERSION${r}, ${l}SEISMIC_TEE_INSTALL_DIR${r}  the same as the two options
  ${l}SEISMIC_TEE_REPO${r}                              another <owner>/<repo>
  ${l}GH_TOKEN${r} or ${l}GITHUB_TOKEN${r}                      sent to api.github.com only,
                                                for its rate limit
  ${l}NO_COLOR${r}                                      the same as --no-color

${h}Examples:${r}
  ${n}# the newest release, into ~/.local/bin${r}
  curl -fsSL $INSTALLER_URL | sh

  ${n}# the newest release of main, for a change that is merged but not in a version yet${r}
  curl -fsSL $INSTALLER_URL |
      sh -s -- --version main

  ${n}# one release, into a directory you choose${r}
  curl -fsSL $INSTALLER_URL |
      sh -s -- --version v0.1.0 --to /opt/seismic/bin

  ${n}# the same in the environment, for a CI job that sets it once${r}
  curl -fsSL $INSTALLER_URL |
      SEISMIC_TEE_VERSION=v0.1.0 SEISMIC_TEE_INSTALL_DIR=/opt/seismic/bin sh

Every download is checked against the release's SHA256SUMS, and the binary's
build provenance against GitHub's attestation when gh is installed and logged
in. Once installed, the binary upgrades (or downgrades) itself:
${l}seismic-tee --upgrade${r} [VERSION], with VERSION as above.
EOF
}

while [ $# -gt 0 ]; do
    case $1 in
        --version)
            [ $# -ge 2 ] || fail "--version needs a value"
            VERSION=$2
            shift 2
            ;;
        --version=*) VERSION=${1#--version=}; shift ;;
        --to)
            [ $# -ge 2 ] || fail "--to needs a value"
            INSTALL_DIR=$2
            shift 2
            ;;
        --to=*) INSTALL_DIR=${1#--to=}; shift ;;
        --no-color)
            no_color
            shift
            ;;
        -h | --help)
            show_help=1
            shift
            ;;
        *)
            usage
            fail "unknown option: $1"
            ;;
    esac
done
# After the loop, so a --no-color anywhere on the line reaches the help.
if [ -n "${show_help:-}" ]; then
    help
    exit 0
fi

# --- Which build -------------------------------------------------------------

os=$(uname -s)
arch=$(uname -m)
case $os in
    Linux) os=linux ;;
    Darwin) os=darwin ;;
    *) fail "unsupported operating system: $os" ;;
esac
case $arch in
    x86_64 | amd64) arch=amd64 ;;
    aarch64 | arm64) arch=arm64 ;;
    *) fail "unsupported architecture: $arch" ;;
esac
# A shell under Rosetta reports x86_64 on an Apple Silicon Mac; the native
# build is the one to install there. Only macOS has the sysctl.
if [ "$os" = darwin ] && [ "$arch" = amd64 ] && [ "$(sysctl -n sysctl.proc_translated 2> /dev/null)" = 1 ]; then
    arch=arm64
fi
platform=${os}_${arch}
case $platform in
    linux_amd64 | linux_arm64 | darwin_arm64) ;;
    *) fail "no prebuilt $BIN for $platform; build it from source: cargo install --git https://github.com/$REPO $BIN" ;;
esac

# --- Transport ---------------------------------------------------------------

command -v curl > /dev/null 2>&1 || fail "curl is required"

# GET a GitHub API path; JSON on stdout. Authenticated when a token is in
# the environment (GH_TOKEN, else GITHUB_TOKEN), for the rate limit alone.
token=${GH_TOKEN:-${GITHUB_TOKEN:-}}
api() {
    if [ -n "$token" ]; then
        curl -fsSL -H 'Accept: application/vnd.github+json' -H "Authorization: Bearer $token" "https://api.github.com/$1"
    else
        curl -fsSL -H 'Accept: application/vnd.github+json' "https://api.github.com/$1"
    fi
}

# Download release asset $1 of $tag into $tmp.
download() {
    curl -fsSL -o "$tmp/$1" "https://github.com/$REPO/releases/download/$tag/$1" ||
        fail "could not download $1 from release $tag of $REPO"
}

# The newest release (the API lists newest first) whose tag starts with $1.
# One page only: this repo's releases include a `seismic-tee/main-<sha>`
# prerelease per merge and the other components' releases, so once more than
# a page of those have landed since the newest version tag, a bare install
# finds nothing. Page (or prune old prereleases) if that ever happens.
newest_tag() {
    api "repos/$REPO/releases?per_page=100" |
        grep -o "\"tag_name\": *\"$1[^\"]*\"" |
        head -n 1 |
        sed 's/.*"\([^"]*\)"$/\1/'
}

# --- Which release -----------------------------------------------------------

case $VERSION in
    "")
        tag=$(newest_tag "${PREFIX}v") || true
        [ -n "$tag" ] || fail "$REPO has no ${PREFIX}v* release"
        ;;
    main)
        tag=$(newest_tag "${PREFIX}main-") || true
        [ -n "$tag" ] || fail "$REPO has no ${PREFIX}main-* prerelease"
        ;;
    main-*) tag=$PREFIX$VERSION ;;
    v[0-9]*) tag=$PREFIX$VERSION ;;
    [0-9]*) tag=${PREFIX}v$VERSION ;;
    *) fail "unrecognized version: $VERSION (want X.Y.Z, vX.Y.Z, main-<sha> or main)" ;;
esac
version=${tag#"$PREFIX"}
asset=${BIN}_${version}_${platform}.tar.gz
styled "$bold" "installing $BIN $version for $platform (detected)"
styled "$dim" "  from https://github.com/$REPO/releases/tag/$tag"

# --- Download, check, install ------------------------------------------------

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

download "$asset"
download SHA256SUMS

expected=$(awk -v asset="$asset" '$2 == asset { print $1 }' "$tmp/SHA256SUMS")
[ -n "$expected" ] || fail "SHA256SUMS of $tag lists no $asset"
if command -v sha256sum > /dev/null 2>&1; then
    actual=$(sha256sum "$tmp/$asset" | cut -d' ' -f1)
else
    actual=$(shasum -a 256 "$tmp/$asset" | cut -d' ' -f1)
fi
[ "$actual" = "$expected" ] || fail "$asset does not match SHA256SUMS: expected $expected, got $actual"
styled "$passed" "  checksum matches the release's SHA256SUMS"

tar -xzf "$tmp/$asset" -C "$tmp" "$BIN"

# The tarball's checksum says the download is the release's; the attestation
# says the release's binary was built by this repo's workflow from the commit
# the release names. Verified when gh can (gh 2.49+, logged in), skipped with
# a warning otherwise — never silently.
if command -v gh > /dev/null 2>&1 && gh auth status > /dev/null 2>&1 && gh attestation --help > /dev/null 2>&1; then
    gh attestation verify "$tmp/$BIN" --repo "$REPO" > /dev/null ||
        fail "$BIN from $tag has no valid build provenance attestation from $REPO"
    styled "$passed" "  build provenance verified: attested by $REPO's release workflow"
else
    warn "build provenance not verified (needs gh 2.49+, logged in); the checksum matched"
fi

mkdir -p "$INSTALL_DIR"
cp "$tmp/$BIN" "$INSTALL_DIR/$BIN.tmp.$$"
chmod 755 "$INSTALL_DIR/$BIN.tmp.$$"
mv -f "$INSTALL_DIR/$BIN.tmp.$$" "$INSTALL_DIR/$BIN"

say "installed $("$INSTALL_DIR/$BIN" --version) to $INSTALL_DIR/$BIN"
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) warn "$INSTALL_DIR is not on your PATH; add it: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
# Reference for later, not part of this run's report: dimmed, in the shape of
# the CLI's own Examples: blocks.
styled "$dim" "
Upgrade: the binary replaces itself
  $BIN --upgrade               the newest release
  $BIN --upgrade main          the newest release of main
  $BIN --upgrade vX.Y.Z        one release; naming an older one downgrades
  $BIN --upgrade main-<sha>    one release of main, by its full commit"
