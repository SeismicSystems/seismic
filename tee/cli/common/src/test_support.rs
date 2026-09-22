//! HTTP/1.1 servers small enough to live in the tests: canned responses,
//! a directory of files, recorded requests, and a way to get a refused port.
//!
//! Every network interaction the CLI has is a request to a node and a
//! read of the reply, so a test needs no more than this to exercise the real
//! client code path — including what happens when the far end is not there
//! yet. Shared by the node and network crates' tests through the
//! `test-support` feature; nothing here is compiled into a release binary.

use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use serde_json::Value;
use sha2::{Digest as _, Sha256};

/// The enclave schema crate's `fixtures/network-manifest-v1.json`, byte for
/// byte: a valid v1 manifest (chain 5124, namespace `seismic-devnet-3`) for
/// the gates and consumers here to read.
pub const FIXTURE_MANIFEST: &[u8] = br#"{
  "eth": {
    "chain_id": 5124,
    "genesis_hash": "0x78ab9057bb67f95a6182969c5d755ac02802c98c0d2f0d8daeb52f4bddc60be5"
  },
  "manifest_version": 1,
  "measurements": {
    "bootstrap_policy_hash": "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    "contracts": {
      "authority": "0x1000000000000000000000000000000000000002",
      "registry": "0x1000000000000000000000000000000000000001"
    }
  },
  "name": "seismic-devnet-3",
  "summit": {
    "genesis_config_digest": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "namespace": "seismic-devnet-3"
  }
}
"#;

/// A valid manifest whose `bootstrap_policy_hash` commits to `policy` — an
/// artifact set as `assemble` writes it.
pub fn manifest_pinning(policy: &[u8]) -> Vec<u8> {
    let digest: [u8; 32] = Sha256::digest(policy).into();
    String::from_utf8_lossy(FIXTURE_MANIFEST)
        .replace(&"cc".repeat(32), &hex::encode(digest))
        .into_bytes()
}

pub fn write_file(dir: &tempfile::TempDir, name: &str, contents: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, contents).expect("writing test input");
    path
}

/// One request as the server saw it.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

/// A server that answers each connection with the next canned response, in
/// order, and then goes away — so the connection after the last response is
/// refused, which is how a one-shot listener like tdx-init behaves.
///
/// A canned JSON-RPC response answers with the request's own `id`, as any
/// server does (the client checks it), so a fixture can spell any `id` and
/// still match whichever request it ends up answering.
pub struct FakeServer {
    /// `http://127.0.0.1:<port>`, with no trailing slash.
    pub url: String,
    requests: Arc<Mutex<Vec<Request>>>,
    handle: Option<JoinHandle<()>>,
}

impl FakeServer {
    pub fn serve(responses: Vec<(u16, String)>) -> Self {
        let listener = bind_loopback();
        let addr = listener.local_addr().unwrap();
        Self::on(addr, move || listener, responses)
    }

    /// [`Self::serve`], bound to a caller-chosen loopback port rather than an
    /// ephemeral one — for a client whose URL is not a parameter, like
    /// [`crate::NodeDescriptor::attestation_rpc_url`]'s fixed `:7878`.
    pub fn serve_at(port: u16, responses: Vec<(u16, String)>) -> Self {
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        let listener = TcpListener::bind(addr).expect("bind the requested loopback port");
        Self::on(addr, move || listener, responses)
    }

    /// A server whose port is known now but that only starts listening after
    /// `delay` — a tdx-init still behind its LUKS setup. Connections before
    /// then are refused.
    pub fn serve_after(delay: Duration, responses: Vec<(u16, String)>) -> Self {
        let listener = bind_loopback();
        let addr = listener.local_addr().unwrap();
        // Unbound *before* this returns, on this thread. Were the drop left to
        // the server thread, a request racing it could land while the port is
        // still bound: accepted into the backlog, then reset when the listener
        // goes — "connection reset", not "refused", and a client is right not
        // to retry a request that may have been received.
        drop(listener);
        Self::on(
            addr,
            move || {
                std::thread::sleep(delay);
                TcpListener::bind(addr).expect("rebind the same loopback port")
            },
            responses,
        )
    }

    /// Serve `responses` on the listener `listen` yields, which is `addr`.
    fn on(
        addr: SocketAddr,
        listen: impl FnOnce() -> TcpListener + Send + 'static,
        responses: Vec<(u16, String)>,
    ) -> Self {
        let url = format!("http://{addr}");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requests);
        let handle = std::thread::spawn(move || {
            let listener = listen();
            for (status, body) in responses {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let request = read_request(&mut stream);
                let body = echo_request_id(&body, &request.body);
                recorded.lock().unwrap().push(request);
                let response = format!(
                    "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    reason(status),
                    body.len(),
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
            // Dropping the listener here is what makes the next connection
            // refused.
        });
        Self {
            url,
            requests,
            handle: Some(handle),
        }
    }

    pub fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        // The thread ends on its own once its responses are consumed, or when
        // the process exits; never block a test on it.
        drop(self.handle.take());
    }
}

/// A server that answers `GET <path>` with the bytes filed under `path`, as
/// many times as asked, and 404s any other path — a release's asset
/// directory, for the code that fetches an image's founding inputs. Serves
/// until dropped.
pub struct FileServer {
    /// `http://127.0.0.1:<port>`, with no trailing slash.
    pub url: String,
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    requests: Arc<Mutex<Vec<String>>>,
    handle: Option<JoinHandle<()>>,
}

impl FileServer {
    pub fn serve(files: BTreeMap<String, Vec<u8>>) -> Self {
        let listener = bind_loopback();
        let addr = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let (stopped, recorded) = (Arc::clone(&stop), Arc::clone(&requests));
        let handle = std::thread::spawn(move || {
            while let Ok((mut stream, _)) = listener.accept() {
                if stopped.load(Ordering::SeqCst) {
                    return;
                }
                let request = read_request(&mut stream);
                recorded.lock().unwrap().push(request.path.clone());
                let response = match files.get(&request.path) {
                    Some(body) => {
                        let mut response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\
                             Content-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                        .into_bytes();
                        response.extend_from_slice(body);
                        response
                    }
                    None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: \
                              close\r\n\r\n"
                        .to_vec(),
                };
                let _ = stream.write_all(&response);
                let _ = stream.flush();
            }
        });
        Self {
            url: format!("http://{addr}"),
            addr,
            stop,
            requests,
            handle: Some(handle),
        }
    }

    /// The paths requested so far, in order.
    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

impl Drop for FileServer {
    fn drop(&mut self) {
        // `accept` blocks; one connection of our own wakes it to see the flag.
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.addr);
        drop(self.handle.take());
    }
}

/// `response` with its `id` replaced by the request's, when both are JSON-RPC
/// envelopes; anything else (a TOML POST, a plain-text body) passes through.
fn echo_request_id(response: &str, request: &[u8]) -> String {
    let Ok(Value::Object(request)) = serde_json::from_slice::<Value>(request) else {
        return response.to_string();
    };
    let Ok(Value::Object(mut envelope)) = serde_json::from_str::<Value>(response) else {
        return response.to_string();
    };
    match (request.get("id"), envelope.get("id")) {
        (Some(id), Some(_)) => {
            envelope.insert("id".to_string(), id.clone());
            Value::Object(envelope).to_string()
        }
        _ => response.to_string(),
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        409 => "Conflict",
        500 => "Internal Server Error",
        _ => "Status",
    }
}

fn read_request(stream: &mut TcpStream) -> Request {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        let n = stream.read(&mut chunk).expect("read request");
        if n == 0 {
            break buffer.len();
        }
        buffer.extend_from_slice(&chunk[..n]);
        if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
    };
    let head = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = head.lines();
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();

    let mut content_length = 0usize;
    let mut content_type = None;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        match name.to_ascii_lowercase().as_str() {
            "content-length" => content_length = value.trim().parse().unwrap_or(0),
            "content-type" => content_type = Some(value.trim().to_string()),
            _ => {}
        }
    }

    let mut body = buffer[header_end..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut chunk).expect("read body");
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    Request {
        method,
        path,
        content_type,
        body,
    }
}

fn bind_loopback() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").expect("bind a loopback port")
}

/// A loopback URL nothing listens on.
pub fn refused_url() -> String {
    let listener = bind_loopback();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    url
}

/// A JSON-RPC 2.0 success envelope around `result`. The `id` is a
/// placeholder: [`FakeServer`] answers with the request's.
pub fn rpc_result(result: Value) -> String {
    serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": result}).to_string()
}
