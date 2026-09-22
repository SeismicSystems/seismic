//! Where a node listens, and the client both command groups reach it with.
//!
//! A deployed node is a sealed VM: there is no SSH, so every interaction is an
//! HTTP or JSON-RPC call to one of the ports below. Two of them are
//! operator-only and reachable only from the CIDR the node's NSG pins
//! (`operator_ip_cidr`); the RPC is public, behind nginx and a real cert.

use std::time::Duration;

use crate::descriptor::NodeDescriptor;
use crate::error::Result;

/// tdx-init's one-shot config receiver, up only until a boot's POST is
/// consumed. Plain HTTP: it runs before certbot has issued anything.
pub const TDX_INIT_PORT: u16 = 8080;
/// The attestation service's JSON-RPC: LUKS provisioning status, deploy
/// verification evidence, the root-key handshake.
pub const ATTESTATION_RPC_PORT: u16 = 7878;
/// summit-key-holder, which serves `{pubkeys, quote}` until the box takes its
/// config POST. Plain HTTP for the same reason as tdx-init: pre-certificate.
pub const SUMMIT_KEY_HOLDER_PORT: u16 = 7879;

/// How long any single request waits before it is treated as unreachable.
///
/// A node coming up is *expected* to refuse connections for minutes, so the
/// waiting is a caller's retry loop over short requests, not a long timeout —
/// which is why this is short and shared.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// How long the TCP connect alone may take before it counts as a *connect*
/// failure.
///
/// A node that is not up yet does not always refuse: while its network stack
/// is still coming up, or behind an NSG that drops rather than rejects, the
/// SYN goes unanswered and the connect hangs. Without this, such a hang would
/// run into [`REQUEST_TIMEOUT`] and surface as a request timeout, which
/// callers rightly do not retry (a POST that timed out mid-flight may have
/// landed). With it, the hang is reported as a connect error — the same class
/// as "connection refused" — and the retry loops wait it out like any other
/// not-up-yet node. Shorter than [`REQUEST_TIMEOUT`] so it is the one that
/// fires.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// What every request from this tooling identifies itself as, so a node's
/// logs distinguish the deploy CLIs from anything else that reached it. Sent
/// by this client and by [`crate::rpc`] alike.
pub const USER_AGENT: &str = concat!("seismic-tee-cli/", env!("CARGO_PKG_VERSION"));

/// The client the plain-HTTP requests go out on: tdx-init's config receiver
/// and summit-key-holder, the endpoints that speak HTTP rather than JSON-RPC.
/// JSON-RPC endpoints are reached through [`crate::rpc`] instead, with the
/// same user agent and request timeout.
pub fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()?)
}

impl NodeDescriptor {
    /// tdx-init's config receiver, on the operator-only port.
    pub fn tdx_init_url(&self) -> String {
        format!("http://{}:{TDX_INIT_PORT}/", self.public_ip)
    }

    /// The attestation service's JSON-RPC endpoint.
    pub fn attestation_rpc_url(&self) -> String {
        format!("http://{}:{ATTESTATION_RPC_PORT}", self.public_ip)
    }

    /// The summit-key-holder endpoint the founding harvest polls.
    pub fn key_holder_url(&self) -> String {
        format!("http://{}:{SUMMIT_KEY_HOLDER_PORT}", self.public_ip)
    }

    /// The node's public Ethereum JSON-RPC: nginx proxies `/rpc` to reth,
    /// which is why this one goes to the FQDN over TLS rather than to the IP.
    pub fn eth_rpc_url(&self) -> String {
        format!("https://{}/rpc", self.fqdn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            public_ip: "203.0.113.7".into(),
            fqdn: "az-1.seismicdev.net".into(),
        }
    }

    /// The operator-only ports address the node by IP; only the public RPC
    /// uses the FQDN, because only it has a certificate to match.
    #[test]
    fn operator_ports_use_the_ip_and_the_rpc_uses_the_fqdn() {
        let d = descriptor();

        assert_eq!(d.tdx_init_url(), "http://203.0.113.7:8080/");
        assert_eq!(d.attestation_rpc_url(), "http://203.0.113.7:7878");
        assert_eq!(d.key_holder_url(), "http://203.0.113.7:7879");
        assert_eq!(d.eth_rpc_url(), "https://az-1.seismicdev.net/rpc");
    }

    #[test]
    fn the_shared_client_builds() {
        client().unwrap();
    }
}
