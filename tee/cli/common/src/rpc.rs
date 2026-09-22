//! JSON-RPC 2.0 over HTTP, as the attestation service speaks it on `:7878`.
//!
//! The attestation service is the one endpoint a deployed node exposes for
//! the whole of its life, and both command groups read it: `status` (and `configure`'s
//! post-POST wait) poll `getLuksProvisioningStatus`. The deploy-verification
//! challenge (`getDeployVerificationEvidence`) is *not* made here: it belongs
//! to the enclave's verify-quote library, which mints the nonce and checks the
//! answer, and which brings its own client for exactly that call. What this
//! module is for is the plain reads, whose result is data rather than a
//! verdict.
//!
//! The transport is jsonrpsee's HTTP client — the same one verify-quote
//! reaches `:7878` with, so every JSON-RPC call this binary makes goes out the
//! same way. What is *not* used is a generated, typed client: the status
//! methods live on a `NodeStatusRpc` trait inside the attestation-service
//! binary, not in a crate deploy can link, and the enclave declares that
//! surface a raw JSON-RPC contract for its consumers. So a call here returns
//! the `result` undecoded, and the caller reads it tolerantly — a state this
//! build does not know is reported and polled through, never a deserialize
//! error an hour into a watch.

use jsonrpsee::core::ClientError;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::traits::ToRpcParams;
use jsonrpsee::http_client::{HeaderMap, HeaderValue, HttpClient, HttpClientBuilder};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::http::{REQUEST_TIMEOUT, USER_AGENT};

/// One node's JSON-RPC endpoint, ready to be called.
///
/// Built once per node and reused across a poll: the failure an operator sees
/// names the endpoint, and every request carries the same user agent and
/// patience as the plain-HTTP client in [`crate::http`].
#[derive(Debug, Clone)]
pub struct Client {
    url: String,
    inner: HttpClient,
}

impl Client {
    /// A client for the JSON-RPC endpoint at `url`.
    ///
    /// Fails only on a URL the transport cannot use; nothing is sent yet.
    pub fn new(url: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_static(USER_AGENT));
        let inner = HttpClientBuilder::default()
            .request_timeout(REQUEST_TIMEOUT)
            .set_headers(headers)
            .build(url)
            .map_err(|source| Error::RpcTransport {
                url: url.to_string(),
                source,
            })?;
        Ok(Self {
            url: url.to_string(),
            inner,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Call `method` and return its `result`, undecoded.
    ///
    /// Every failure names the endpoint. No answer (connection refused,
    /// timeout, a reply that is not JSON-RPC) is [`Error::RpcTransport`],
    /// which is the normal state of a node still coming up; a server that
    /// answers with an `error` object is [`Error::Rpc`], which is not.
    pub async fn call(&self, method: &str, params: impl ToRpcParams + Send) -> Result<Value> {
        self.inner
            .request(method, params)
            .await
            .map_err(|source| match source {
                ClientError::Call(error) => Error::Rpc {
                    url: self.url.clone(),
                    message: error.to_string(),
                },
                source => Error::RpcTransport {
                    url: self.url.clone(),
                    source,
                },
            })
    }
}
