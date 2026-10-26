//! Where a node listens, and the client both command groups reach it with.
//!
//! A deployed node is a sealed VM: there is no SSH, so every interaction is an
//! HTTP or JSON-RPC call to one of the ports below, on the node's IP and in
//! plain HTTP: nothing in the image terminates TLS. Two of them are
//! operator-only and reachable only from the CIDR the node's NSG pins
//! (`operator_ip_cidr`). The RPC is reth's own listener, which public clients
//! reach over HTTPS through the network's front and this tooling reaches
//! directly.

use std::net::SocketAddr;
use std::time::Duration;

use crate::descriptor::NodeDescriptor;
use crate::error::Result;

/// tdx-init's one-shot config receiver, up only until a boot's POST is
/// consumed.
pub const TDX_INIT_PORT: u16 = 8080;
/// The attestation service's JSON-RPC: LUKS provisioning status, deploy
/// verification evidence, the root-key handshake.
pub const ATTESTATION_RPC_PORT: u16 = 7878;
/// The attestation service's founding harvest: summit's pubkeys for the life of
/// the boot, and a quote over them until the config POST delivers the network
/// manifest. The same process as [`ATTESTATION_RPC_PORT`], on a port of its own because
/// the NSG filters only by port: this one is operator-only, that one is open
/// to anyone.
pub const HARVEST_PORT: u16 = 7879;
/// reth's Ethereum JSON-RPC.
pub const ETH_RPC_PORT: u16 = 8545;

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
/// and the founding harvest, the endpoints that speak HTTP rather than
/// JSON-RPC. JSON-RPC endpoints are reached through [`crate::rpc`] instead,
/// with the same user agent and request timeout.
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
        format!("http://{}/", SocketAddr::new(self.public_ip, TDX_INIT_PORT))
    }

    /// The attestation service's JSON-RPC endpoint.
    pub fn attestation_rpc_url(&self) -> String {
        format!(
            "http://{}",
            SocketAddr::new(self.public_ip, ATTESTATION_RPC_PORT)
        )
    }

    /// The attestation service's founding-harvest endpoint, which `harvest`
    /// and the launch checks poll.
    pub fn harvest_url(&self) -> String {
        format!("http://{}", SocketAddr::new(self.public_ip, HARVEST_PORT))
    }

    /// reth's Ethereum JSON-RPC, where `seismic_nodeInfo` and block 0 are
    /// read. On the IP rather than the FQDN: the name belongs to the front,
    /// and these reads check the node, by the IP its appraisal names.
    pub fn eth_rpc_url(&self) -> String {
        format!("http://{}", SocketAddr::new(self.public_ip, ETH_RPC_PORT))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            public_ip: "203.0.113.7".parse().unwrap(),
            fqdn: "az-1.seismicdev.net".parse().unwrap(),
        }
    }

    /// Every port addresses the node by IP, the RPC included: the FQDN is
    /// the front's.
    #[test]
    fn every_port_uses_the_ip() {
        let d = descriptor();

        assert_eq!(d.tdx_init_url(), "http://203.0.113.7:8080/");
        assert_eq!(d.attestation_rpc_url(), "http://203.0.113.7:7878");
        assert_eq!(d.harvest_url(), "http://203.0.113.7:7879");
        assert_eq!(d.eth_rpc_url(), "http://203.0.113.7:8545");
    }

    /// An IPv6 address is bracketed in a URL, or its colons read as a port.
    #[test]
    fn an_ipv6_address_is_bracketed() {
        let d = NodeDescriptor {
            public_ip: "2001:db8::1".parse().unwrap(),
            ..descriptor()
        };
        assert_eq!(d.tdx_init_url(), "http://[2001:db8::1]:8080/");
        assert_eq!(d.attestation_rpc_url(), "http://[2001:db8::1]:7878");
        assert_eq!(d.harvest_url(), "http://[2001:db8::1]:7879");
        assert_eq!(d.eth_rpc_url(), "http://[2001:db8::1]:8545");
    }

    #[test]
    fn the_shared_client_builds() {
        client().unwrap();
    }
}
