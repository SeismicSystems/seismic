# Public RPC trust (decision record)

> **Decision record — October 2026.** A point-in-time capture of the options
> pass on what a client trusts when it talks to a node's public RPC, and on
> where the node's public HTTPS proxy runs as a result. It is not updated as
> the design moves; the current-state docs it links are the authority on what
> runs today.

**Outcome, in two parts,** in
[SEI-888](https://linear.app/seismic-systems/issue/SEI-888):

1. **Now: the proxy leaves the image (B).** Today's in-image proxy (A) ends TLS
   in the guest under a key bound to no quote, so it adds no trust a client can
   check. Moving TLS to a front outside the CVM costs no trust, shrinks the
   image, and leaves operators free to run whatever front and load balancing
   they want, one hostname over many nodes included. It is an interim step,
   not the end state. The work is
   [SEI-991](https://linear.app/seismic-systems/issue/SEI-991) (the image),
   [SEI-992](https://linear.app/seismic-systems/issue/SEI-992) (the front and
   the firewall rules) and
   [SEI-993](https://linear.app/seismic-systems/issue/SEI-993) (public and
   operator listeners).
2. **Next: the trust gap stays open.** Under A and B alike, a client reading
   public state trusts whichever RPC answers it, and until
   [SEI-646](https://linear.app/seismic-systems/issue/SEI-646) its SDK takes
   `tx_io_pk` from that RPC too. Closing the gap is the second part of this
   record: trust the data, with a light client, and the application key,
   `tx_io_pk`, both of which work through any front. Attesting the transport
   key, which would bring a terminator back into the guest, is
   [a future decision](#attesting-the-transport-key-a-future-decision), under
   [a bar](#bringing-a-terminator-back).

The first part is decided, not built; the second is open.

![A and B stacked, each a row from client through the front to the node's CVM: under A an L7 proxy in the guest ends TLS under a key bound to no quote, with no front; under B an L7 front ends TLS and forwards plain HTTP over the operator's VNet to backends in a guest that runs no proxy, while the operator's CLI, CI or any SDK can also reach the node's IP over plain HTTP, past the front](../diagrams/public-rpc-placements-a-b.svg)

The question came up while replacing nginx and certbot with an unprivileged
Caddy in the image
([seismic-images #81](https://github.com/SeismicSystems/seismic-images/pull/81),
SEI-629 under SEI-843), as "where does the proxy run". It widened once it was
clear that which placements are worth having depends on what clients trust:
the data, an application key, or the transport.

- [The question](#the-question)
- [What TLS on the public port protects](#what-tls-on-the-public-port-protects)
- [Where the proxy runs](#where-the-proxy-runs)
  - [What B changes on the network](#what-b-changes-on-the-network)
  - [What an in-image proxy adds](#what-an-in-image-proxy-adds)
  - [What it costs](#what-it-costs)
  - [What a front alone gives](#what-a-front-alone-gives)
  - [Fronts and their cost](#fronts-and-their-cost)
- [What a client can trust](#what-a-client-can-trust)
  - [Trust the data](#trust-the-data)
  - [Trust the application key](#trust-the-application-key)
  - [Trust the transport key](#trust-the-transport-key)
  - [Trust the session](#trust-the-session)
  - [What it means for the proxy](#what-it-means-for-the-proxy)
- [Attesting the transport key: a future decision](#attesting-the-transport-key-a-future-decision)
  - [Bringing a terminator back](#bringing-a-terminator-back)
  - [Option C: an attested per-node key](#option-c-an-attested-per-node-key)
  - [Option D: an L4 front and a shared, attested TLS key](#option-d-an-l4-front-and-a-shared-attested-tls-key)
  - [D+: pinning the ACME account](#d-pinning-the-acme-account)
- [Settled here](#settled-here)
- [Left open](#left-open)
- [References](#references)

## The question

Two questions, the first answered by the second:

- **Where does the node's public proxy run:** in the image, or only in a front
  outside it? "Proxy" here is whatever answers the public port and routes to
  the backends, at L4 or L7. What matters is where TLS ends.
- **What does a client trust** when it reads from or writes to a node's public
  RPC: the response itself, the key it encrypted to, or the connection it came
  over?

An operator running more than one node needs a front outside the image anyway.
A staking operator may run a validator plus up to `observers_per_validator`
read-only nodes, and wants one hostname over all of them. An in-image proxy
cannot serve that hostname on its own:

- TLS-ALPN-01 fails behind a TLS-terminating load balancer, since the load
  balancer answers the handshake.
- Behind a TCP passthrough load balancer, the challenge lands on a random
  backend.
- DNS-01 would put DNS credentials inside the TEE.

So the placement question is not "proxy inside or outside the CVM", but
**whether the image also needs a proxy of its own, given a front outside it**.

## What TLS on the public port protects

- **Not shielded data.** Confidentiality comes from attested
  application-layer keys ([node.md](../node.md)). The manifest
  pins `tx_io_pk@0`. Later epochs' keys are to be recorded in summit's
  consensus state and checked by clients with a light client
  ([the root-key commitment record](2026-09-root-key-commitment.md),
  [Key records & light clients](https://linear.app/seismic-systems/project/key-records-and-light-clients-9a17291b5b1c)).
- **Nothing against the operator or host.** They control DNS and routing, so
  they pass HTTP-01 and TLS-ALPN-01 for the node's name and get a valid
  certificate whenever they want. Today's in-image certificate key is bound to
  no quote, and nothing watches CT for it.
- **Third parties on the path,** for:
  - browsers: an HTTPS page can only call `https` and `wss`;
  - the integrity of public reads;
  - the privacy of non-shielded queries, such as which addresses a client
    looks up.
- **Not needed for:** the peer planes, the `seismic-tee` CLI's operator calls
  (plain HTTP to `:7878` and `:7879`, with its own attestation checks, and to
  `:8080`, guarded by the firewall), CI smoke tests, the front to node hop,
  and metrics scrapes. The CLI's HTTPS calls, its reth reads, are among
  [a front's costs](#what-a-front-alone-gives).

## Where the proxy runs

- **A. An L7 proxy in the image** (today's node, since seismic-images #81),
  such as Caddy, running ACME for the node's own name. Fits single-node
  operators only.
- **B. An L7 front only.** The image runs no proxy, and the backends bind a
  non-loopback address. Chosen.

A and B are compared in the figure at the top. A's key is bound to no quote,
so where TLS ends tells a client nothing it can check, and the proxy buys
nothing by sitting in the guest. Ending TLS in the guest earns its place only
once its key is attested, the future decision
[below](#attesting-the-transport-key-a-future-decision).

### What B changes on the network

Against today's node, B changes who takes part in three scopes:

- **Public HTTPS** spans the client, the front and the node, not the client and
  the node. DNS picks the front, and the front's path map picks the node and
  port. What is reachable is the front's path map plus each backend listener's
  methods, so the policy lives with the operator, not in the image.
- **Operator HTTP** is new: metrics and summit's full query set, each on its
  own listener, reached from inside the operator's VNet. Today they share
  `:443` with public RPC. The summit queries clients need stay on public
  HTTPS, on a listener of their own behind the front: the light client's
  finalized headers and state proofs ([Key records & light clients](https://linear.app/seismic-systems/project/key-records-and-light-clients-9a17291b5b1c)), and
  what the staking UI reads.
- **Operator VNet** is new: one operator's private network, joined by being in
  their Azure subscription, with firewall rules per port and source set by
  deploy.

### What an in-image proxy adds

- **The measured image decides what is public.** Backends stay on loopback,
  and only the exact routes are reachable. Under B, a firewall rule in deploy
  decides which ports the public reaches, outside the measurement. What the
  image still decides is which ports listen beyond loopback and which methods
  each listener serves, and against the operator, who controls the firewall
  anyway, that is the part that counts.
- **A single-node operator needs no front** (A). Under B, they bring one.
- **No managed front reads non-shielded RPC** (A). A host that wants to
  has to obtain a certificate for the node's name itself. It can, but the
  issuance shows up in Certificate Transparency logs.

### What it costs

- **The most exposed parser on the node runs in the guest that holds
  `root_key`,** where in-guest root can read it (A).
- **With ACME in the guest** (A):
  - The image carries Caddy, a pinned Go toolchain, a fourth source pin and
    the Caddyfile.
  - The config POST carries the operator's domain and email, the injection
    class SEI-776 had to close.
  - Every new hostname takes a certificate at first boot. Let's Encrypt
    allows 50 a week per registered domain, and the nightly found-devnet
    spends 28 (SEI-821, SEI-823). Past the cap, Caddy falls back to ZeroSSL,
    which publishes no per-domain or weekly limit on its free 90-day
    certificates, so the cost is a second CA to depend on rather than a
    failed founding.
- **Changing which routes are public needs a new image** and new measurements
  (A).

### What a front alone gives

- A smaller image: Caddy, its build, the domain config and `domain.env` all
  go.
- One hostname over many nodes. `tx_io_pk` is the same on every node, so any
  node serves shielded transactions. A hostname is both the service's name and,
  through DNS, its locator
  ([networking.md](../networking.md#four-kinds-of-name)), so serving one name
  from many nodes needs a mapping point, which is the front.
- One wildcard certificate, issued by DNS-01 from deploy, covers every devnet
  and CI name. CI can skip TLS and reach node IPs over plain HTTP from
  `operator_ip_cidr`.
- Rate limiting, WAF and DDoS protection sit in the front.
- The public/internal split becomes a firewall rule. Today `:443` serves public
  RPC and the operator's metrics routes on one port, so a firewall on ports
  cannot tell them apart. Under B, metrics and summit's full query listener
  each have their own port, never go through the front, and the firewall
  admits only the monitoring source.

Its costs:

- Every cloud target and every third-party operator brings a front. Operators
  without one get no rate limiting beyond reth's own limits.
- A managed front reads non-shielded RPC in plaintext, and the hop from the
  front to the node is plaintext inside the VNet unless the image terminates
  TLS for it.
- The CLI's reth reads move off the hostname. Today the CLI reads each node's
  reth over `https://<fqdn>/rpc`:
  - a node's `seismic_nodeInfo`, whose enode host it checks against the IP it
    appraised
    ([what is verified about an address](../networking.md#what-is-verified-about-an-address));
  - every node's block 0, in the launch assertions.

  Behind a front over many nodes, each read lands on whichever node the front
  picks. The CLI has to read them from the node's IP instead, so reth's RPC
  also listens on a port the firewall opens to the operator's range.

### Fronts and their cost

East US list prices as of this record. Each front can serve many hostnames
and nodes, so the cost is per operator or per region, not per node. For scale,
a `DC4es_v6` confidential VM is about $147 a month. A validator that serves no
public RPC needs no front at all.

- **Azure Application Gateway Standard_v2:** about $180 a month fixed, plus
  capacity units. WAF_v2 is about $323 a month. No built-in ACME: certificates
  go into Key Vault through a renewal job of our own.
- **Azure Front Door Standard:** about $35 a month, plus per-GB and
  per-request fees. Managed certificates. WebSockets are capped at 2 hours per
  connection, 5 minutes idle, and 3,000 concurrent per profile.
- **A small ordinary VM running Caddy or HAProxy:** about $10–30 a month. It
  sits outside the TEE, which this design allows.
- **Cloudflare's free plan:** no charge. It terminates TLS, carries
  WebSockets, and its origin rules can send each path to its own port. It is a
  third party that reads non-shielded RPC.
- **Azure Standard Load Balancer:** about $18 a month. It passes TCP through
  and does not terminate TLS.
- **For comparison:** AWS ALB is about $16 a month and GCP's global L7 load
  balancer about $18 a month, each plus usage.

## What a client can trust

A client can verify the data it gets, the application key it encrypts to, the
transport key that authenticates its TLS connection, or the TLS session
itself. Down that list each covers less and rests on more. Approaches that
need browsers or TLS itself to change (browser-native attestation, IETF
post-handshake attestation) are left out: neither is deployable today.

Both kinds of key are long-lived, and what separates them is the layer that
uses them:

| | Application key | Transport key |
| --- | --- | --- |
| The key | `tx_io_pk`, an ECDH key | `tls_pk`, the certificate's signing key |
| Used by | the application: the client encrypts each message to it (Noise N) | TLS: it signs the handshake, authenticating the ephemeral key exchange |
| Covers | only messages sent through it: signed reads, shielded writes | everything on the connection, plain RPC included |
| Across a front | yes: the bytes are end to end | no: whatever ends TLS must be in the guest |
| The client's check | its SDK compares the key with its pin | it compares the certificate's key with the attested one, which browsers cannot |
| Forward secrecy | no: a leaked `tx_io_sk` opens past traffic | yes: the ephemeral exchange protects past sessions |

Attesting the session goes one step past the transport key: the quote also
binds the session's own keying material, so it is fresh per connection.

This builds on [trust-model.md](../trust-model.md). Its
[client trust models](../trust-model.md#client-trust-models) say how a client
learns its anchors: `network_id`, finalized heads, and `tx_io_pk`. The four
below say how a client trusts one response, given those anchors. The trust
model's rule, that a verifier trusts only what `network_id` commits to or what
an authority it commits to signs, sorts them:

- **Trusting the data** checks responses against finalized heads, which the
  validator set signs.
- **Trusting the application key** encrypts to `tx_io_pk`, which the manifest
  pins at epoch 0 and key records carry later.
- **Trusting the transport key** adds an anchor of its own. D's `tls_pk`
  derives from `root_key`, so the manifest could pin it as it pins
  `tx_io_pk`, but a browser checks the WebPKI and CT, not the pin. D+ rests on
  whoever owns the DNS zone, outside anything `network_id` commits to.
- **Trusting the session** rests on a quote per connection, so on the platform
  and on the accepted measurement set, which a client must learn from the
  registry.

### Trust the data

Verify the response itself, whoever served it.

| Approach | Covers | Who can verify | Trust root | Status |
| --- | --- | --- | --- | --- |
| Light client and proofs | public state, and whether the chain is current | SDKs and wallets, browsers included | two thirds of each epoch's validators | [Key records & light clients](https://linear.app/seismic-systems/project/key-records-and-light-clients-9a17291b5b1c) |

- The client follows finalized headers and checks each public read against
  them, by Merkle proof (`eth_getProof`, receipts, logs) or by re-executing
  `eth_call` against proven state, as Helios does for Ethereum.
- The RPC becomes untrusted: the front, the operator and the node's TEE alike.
  The trust root is the chain's own.
- It cannot cover shielded state. A proof of a private slot would reveal its
  value, and a call that reads private state cannot be re-executed outside a
  TEE.

### Trust the application key

Encrypt or authenticate to a key only the network holds, so whatever relays
the bytes cannot read or forge them.

| Approach | Covers | Who can verify | Trust root | Status |
| --- | --- | --- | --- | --- |
| Signed reads and shielded writes, to `tx_io_pk` | shielded state | SDKs, browsers included | the TEE and the `tx_io_pk` pin | shipped; the SDK's pin check is [SEI-646](https://linear.app/seismic-systems/issue/SEI-646) |
| An encrypted envelope on every method | which public data a client asks for, and that a network CVM answered | SDKs, browsers included | the TEE and the pin | not built |

- Both use the client-to-network Noise N of
  [trust-model.md](../trust-model.md): the client encrypts to `tx_io_pk`, and
  the response comes back under a key from the same ECDH, so only a holder of
  the network's secret can read the request or produce the response.
- Until SEI-646, the SDKs take `tx_io_pk` from whichever RPC they ask. A fake
  endpoint can hand out its own key, under every placement.
- "A network CVM answered" is not "the chain is current": the host can feed its
  CVM a stale view. Freshness comes from the data.

### Trust the transport key

Attest the key that authenticates the TLS connection, then trust everything
that comes over a connection under it. TLS must end in the guest.

| Approach | Covers | Who can verify | Trust root | Status |
| --- | --- | --- | --- | --- |
| C: the in-image terminator's per-node key, quoted by the attestation service, and CT monitoring ([RA-WEBs](https://arxiv.org/abs/2411.01340) as published) | every response on the connection | clients that pin the key; browsers only detect, after the fact | the TEE and the accepted measurement set, and whoever watches CT | [option C](#option-c-an-attested-per-node-key) |
| D: one key on every node, derived from `root_key` (RA-WEBs over a shared name) | every response on the connection | clients that pin the key; browsers only detect, after the fact | the TEE and the `root_key` commitment, and whoever watches CT | [option D](#option-d-an-l4-front-and-a-shared-attested-tls-key) |
| D+: the ACME account in the TEE, pinned by CAA `accounturi` ([dstack's zero-trust HTTPS](https://arxiv.org/abs/2509.11555)) | every response on the connection | everyone: the CA enforces it | the TEE and the DNS owner | [D+](#d-pinning-the-acme-account) |
| Nested TLS: an ordinary CA certificate outside, and inside it a second handshake under a per-boot key whose self-signed certificate carries the quote ([Flashbots](https://collective.flashbots.net/t/attested-tls-protocol-redesign-for-scale-and-performance/5794), [`attested-tls`](https://github.com/flashbots/attested-tls)) | every response on the connection | clients that verify the inner certificate; not browsers | the TEE and the accepted measurement set | machine to machine; built for load-balanced fleets |

- A CA-signed certificate does not make a connection attested. A name can
  have several certificates, so a client still has to check that the
  certificate's key is the attested one
  ([Flashbots](https://collective.flashbots.net/t/the-evolution-of-remote-attested-tls/5383)).
  Browsers cannot, so only D+ reaches them.
- A client attests the key once, and trusts every later session under it.
- Nested TLS is the per-node-key, shared-name cell of
  [the future decision](#attesting-the-transport-key-a-future-decision)
  without a CA for the attested key, so without the CSR dance: each instance
  behind a load balancer presents its own quoted certificate, checked at the
  handshake and cached after. The outer certificate only shows the domain
  owner's endorsement, as ordinary HTTPS does. It gives up freshness: no
  verifier nonce reaches the quote, so a short certificate validity is the
  only bound on its age.
- None of these says the chain is current, for the same reason as trusting the
  application key.

### Trust the session

Attest the TLS session itself, per connection.

| Approach | Covers | Who can verify | Trust root | Status |
| --- | --- | --- | --- | --- |
| Post-handshake attested TLS ([attested-tls-proxy](https://github.com/flashbots/attested-tls-proxy)) | every response, bound to this session | clients that run its client | the TEE and the accepted measurement set | machine to machine only |

- attested-tls-proxy's quote binds `SHA-256(leaf public key)` and the TLS
  exporter, keying material derived from the session's ephemeral exchange
  ([its protocol](https://github.com/flashbots/attested-tls-proxy/blob/main/crates/attested-tls/README.md#protocol-specification)).
  So it attests the transport key and the session at once, and a quote cannot
  be replayed onto another connection.
- It needs a quote per connection, and a client that can compute the exporter,
  which browser JavaScript cannot.
- Its authors keep it for infrequent, long-lived connections only. A TDX quote
  takes about 39 ms on GCP and does not parallelize, since one quoting enclave
  serves the host, so a quote per connection caps how fast a client can open
  them; they moved their high-throughput services to nested TLS
  ([Flashbots](https://collective.flashbots.net/t/attested-tls-protocol-redesign-for-scale-and-performance/5794)).
  On Azure the cost is larger: evidence generation holds the vTPM
  exclusively, with a fixed wait of about 3 seconds
  ([SEI-778](https://linear.app/seismic-systems/issue/SEI-778)).

Outside the four, a client can query several operators' RPCs and compare.
That trusts that they do not collude, and is a stopgap rather than a security
property.

### What it means for the proxy

- Only trusting the transport key or the session needs TLS to end in the CVM.
  Trusting the data or the application key works over any transport, through
  any front, in today's browsers.
- Tools that speak plain JSON-RPC, such as wallets and explorers, use none of
  the above: they trust their RPC endpoint, as on Ethereum. Their path to
  trusting the data is a wallet that embeds a light client.
- So the proxy placement turns on whether an attested transport key is worth
  an internet-facing terminator in the guest, with a key in it.

## Attesting the transport key: a future decision

Not decided here. If a client ever needs to trust the transport key, TLS ends
in the guest again, in a terminator whose key is attested, and that takes two
choices, independent of each other: which key, and under which name. The key
is a question of trust, and the name one of operations. A third choice is what
terminates TLS in the guest:

- **An L7 proxy,** such as today's Caddy: TLS plus HTTP/1.1, HTTP/2 and
  WebSocket parsing, routing by path.
- **A TLS-only terminator,** such as a small rustls TCP proxy or HAProxy in TCP
  mode: it parses TLS and forwards bytes, routing by port, one TLS listener
  per backend. The smallest parser in the guest, and the natural fit, since
  the point is the key, not HTTP routing. attested-tls-proxy is one, for
  clients that speak its protocol; browsers need an ordinary listener beside
  it.
- **The backends themselves,** with rustls in reth, summit and the attestation
  service: no extra process, but fork patches, and the key in every backend.

![The two choices as a 2×2 grid, each cell showing two nodes: rows are the key, either a different quoted key on each node or one key derived from root_key on both, joined by a dotted line; columns are the name, either one DNS record per node resolving to that node, or one shared record resolving to an L4 load balancer that passes TCP to both nodes; each cell notes where ACME runs, what clients pin and which certificates exist; C and D are the corners analysed in detail](../diagrams/public-rpc-attested-transport-key.svg)

| | Per-node name, no front | Shared name, behind an L4 front |
| --- | --- | --- |
| **Per-node key, quoted** | **C** | per-node keys under one shared name |
| **One key, derived from `root_key`** | one pinnable key under per-node names | **D** |

- **The key sets the anchor, and how far a leak reaches.** A per-node key is
  trusted through its quote, so a client must also follow the registry to know
  which images are accepted, and a leak reaches one node. A key derived from
  `root_key` is a commitment the manifest can pin, as it pins `tx_io_pk`, and
  a leak reaches every node. Only the derived key meets the second test of
  [the bar](#bringing-a-terminator-back).
- **The name sets the operations.** Under a per-node name the guest can run
  ACME itself, as today. Under a shared name behind an L4 front, a challenge
  lands on a random node, so ACME runs outside the guest
  ([the CSR dance](#the-csr-dance)).
- **The two corners not analysed below:**
  - *Per-node keys under one shared name.* Deploy answers DNS-01 for the
    shared name once per node, with each node's own CSR, so each node holds
    its own certificate for the name, and CT shows one key per node. Let's
    Encrypt limits certificates for an identical set of names per week, so
    each node's certificate also names the node, to keep the sets distinct.
  - *One key under per-node names.* Each node runs ACME in the guest, under
    the derived key: one pinnable `tls_pk`, with no front.
- **A node can take more than one cell.** One proxy on `:443` picks the
  certificate by SNI: the node's own name resolves to its IP, as under C, and
  the shared name to the L4 front, as under D. It pays both cells' costs.

B stays the baseline whichever cells are chosen later. A per-node-name cell
needs no front for its names, and B's front keeps the shared public name. A
shared-name cell replaces B's L7 front with an L4 one for its name, and the L7
work moves into the guest's proxy.

### Bringing a terminator back

An in-image terminator, at L4 or L7, comes back only with a written case that:

- names the clients that must trust the transport key because neither the
  data nor the application key serves them, such as machine clients that pin
  a key;
- gives the transport key an anchor `network_id` commits to, such as `tls_pk`
  pinned in the manifest as `tx_io_pk` is;
- states the cost: an internet-facing parser in the guest that holds
  `root_key`, and how far a leaked key reaches.

Hardening today's Caddy in place, on the expectation that a terminator returns,
was weighed and set aside. If C returns, today's Caddy comes back nearly as
it is, plus an attestation endpoint, so removing it now costs little to undo;
if D returns, it has no ACME and no per-node name, and most of today's
configuration would be rewritten. Either way, hardening moves neither the
parser out of the guest nor the certificate into any client's reach. The
Caddyfile's routes and WebSocket rule move to the front's configuration
instead ([SEI-992](https://linear.app/seismic-systems/issue/SEI-992)).

### Option C: an attested per-node key

Option C is A with its TLS key attested. The attestation service reads the public
half of Caddy's certificate from `/persistent/caddy` and returns a quote
binding `SHA-256(leaf public key)` and `network_id`, the way tx-io evidence
binds `tx_io_pk`, cached per key. That is RA-WEBs as published: a per-service
key made in the TEE, ACME run in the TEE, and CT watched for any other key.
Against D:

- **The key is per node,** made by Caddy, so a leak reaches one node, and
  there is no CSR dance: Caddy runs ACME in the guest, as today. Caddy makes a
  new key for every certificate unless `reuse_private_keys` is set
  ([its docs](https://caddyserver.com/docs/caddyfile/directives/tls)), so the
  attested key would change at every renewal without it.
- **There is no front,** so one hostname per node.
- **The anchor is weaker.** A per-node key appears after founding, so the
  manifest cannot pin it. A client verifies a TDX quote and must also know
  which images are accepted, which means following the registry: the attested
  head model of [trust-model.md](../trust-model.md#client-trust-models). D's
  key is a function of `root_key`, so it reaches `network_id` the way
  `tx_io_pk` does.

It keeps everything A costs: Caddy, ACME and the domain config in the guest.
Against [the bar for a terminator](#bringing-a-terminator-back), it fails the
second test unless its clients follow the registry.

### Option D: an L4 front and a shared, attested TLS key

A front breaks TLS ending in the CVM only if it terminates TLS itself. An L4
load balancer passes TCP through: each connection stays on one node for its
life, and the TLS session inside it ends in that node's guest. What an L4 front
does break is the certificate: every node must present one valid for the
shared name, and an ACME challenge sent to the name lands on a random node. D
answers both with one key on every node and ACME run outside the guest.

- **The key.** The custodian derives `tls_sk = HKDF(root_key, "tls",
  root_version)`, the way it derives `tx_io_sk`, so every node holds the same
  key. `tls_pk` sits in the custodian's evidence beside `tx_io_pk`.
- **Outside the guest.** Deploy holds the ACME account key and the DNS
  credentials, and answers DNS-01. Neither enters the guest.
- **Inside the guest.** The TLS terminator: Caddy with a static certificate
  (`tls <cert> <key>`) and no ACME or domain config. It still routes the exact
  paths. The L4 load balancer is outside the image (an Azure Standard Load
  Balancer); the image runs no balancer.
- **The check.** Anyone can run a CT monitor over the network's names: a
  certificate whose key is not `tls_pk` is an impersonation.

#### The CSR dance

How the guest gets a certificate for `tls_pk` without running ACME:

- **Two keys, two holders.** ACME authorizes an account key for the name,
  here by DNS-01, and issues to the key in a CSR, which is signed by the
  certificate's own key; Let's Encrypt checks both signatures
  ([how it works](https://letsencrypt.org/how-it-works/)). So deploy holds the
  account key and the DNS credentials, the guest signs a CSR with `tls_sk`,
  and neither secret crosses.
- **The CSR is no secret, and no proof of who sends it.** It carries no value
  from the CA, so deploy keeps it and reuses it at every renewal, and the
  guest signs once per `root_version` and name set
  ([Let's Encrypt staff](https://community.letsencrypt.org/t/why-does-acme-use-csrs/190463/17)).
- **The certificate is public.** Deploy hands it to every node, which installs
  it only if its key is `tls_pk`.
- **Security rests elsewhere:** only guests hold `tls_sk`, so a certificate for
  it serves only a CVM, and browsers accept only CT-logged certificates, so a
  certificate for any other key is public and a monitor flags it, within the
  logs' maximum merge delay, usually 24 hours. Deploy is trusted for
  availability only.

#### What D costs

- **The proxy in the guest stays:** Caddy and its Go dependency tree, parsing
  internet traffic next to `root_key`, now holding a key derived from it.
- **`tls_sk` is shared.** A compromised terminator on one node leaks it for
  every node, and CT cannot see that leak, since the attacker uses the
  legitimate certificate. The key rotates only with `root_version`, or with
  the epoch if the HKDF label takes one.
- **No L7 at the front.** A WAF, HTTP rate limiting and path routing move back
  into the guest. DDoS protection is L4 only.
- **Two new guest endpoints:** sign a CSR for given names, and take a
  certificate. The certificate is public, so its channel needs no integrity:
  the guest installs it only if its key is `tls_pk`. The names go into a DER
  builder as hostnames, validated like SEI-776's `Hostname`, not into config
  text.
- **Browsers still verify nothing per connection.** Clients outside a browser
  can pin `tls_pk`.

### D+: pinning the ACME account

dstack's zero-trust HTTPS moves ACME into the TEE, and pins the account rather
than the key
([dstack's gateway](https://github.com/Dstack-TEE/dstack/blob/next/docs/dstack-gateway.md)):

- The ACME account key is made in the TEE, and the name's CAA record says
  `issue "letsencrypt.org;accounturi=<that account>"`. A CA then refuses every
  other account, so whoever controls the network path but not DNS cannot get
  a certificate for the name. That is prevention, where D detects.
- For us, the account key would derive from `root_key` like `tls_sk`, so one
  CAA record covers every node.
- DNS-01 puts a DNS credential in the TEE. `dns-persist-01` replaces it with
  one static TXT record naming the account, so the guest never writes DNS. It
  is an IETF draft, live on Let's Encrypt's staging but not production
  ([dstack's notes](https://github.com/Dstack-TEE/dstack/blob/next/docs/certbot-dns-persist-01.md)).
- The guest runs an ACME client again, and calls out to the CA.

The trust root is the DNS owner, who can rewrite CAA at will. For a node, the
operator owns both the host and the zone, so D+ falls back to D's detection.
It holds only for a name someone else owns, such as one foundation-owned
hostname over third-party nodes, which roots clients' trust in a mutable DNS
zone and the foundation rather than in the chain.

## Settled here

- **Attesting the transport key, for now, against B:** B. No cell of the
  key-and-name matrix is worth an internet-facing parser next to `root_key`
  while trusting the data and the key reach browsers without one. Choosing a
  cell is its own decision, under [the bar](#bringing-a-terminator-back).
- **An in-image router behind a TLS-terminating front,** option B in this
  record's first pass, is dropped: it adds a parser in the guest and attests
  nothing, and what it enforced, which routes are public, the image already
  sets through which ports listen beyond loopback and which methods each
  listener serves. The options were relabeled after it went: the front-only
  option, first C, is B, and C and D are the attested ones.
- **A front reading non-shielded RPC:** accepted.
  [trust-model.md](../trust-model.md) states it as an accepted risk: the
  operator could already, by getting a certificate for its node's name.
- **Our own networks' front:** a small ordinary VM running Caddy per network
  ([SEI-992](https://linear.app/seismic-systems/issue/SEI-992)).
- **Third-party operators:** serving public RPC needs a TLS front, and the
  front's Caddyfile ships as the reference config outside the image. A
  validator that serves no public RPC needs no front.

## Left open

- Which cells of [the key-and-name matrix](#attesting-the-transport-key-a-future-decision)
  to build, if any, once a written case clears
  [the bar](#bringing-a-terminator-back)
  ([SEI-994](https://linear.app/seismic-systems/issue/SEI-994)).
- Whether to build the encrypted envelope on every method, for the privacy of
  public queries, once the light client covers their integrity.
- Which summit queries the public listener serves
  ([SEI-993](https://linear.app/seismic-systems/issue/SEI-993)).
- Whether each node keeps its own FQDN behind a front.
  [networking.md's DNS section](../networking.md#dns) assumes one A record and
  one certificate per node.

## References

- [RA-WEBs: Remote Attestation for WEB services](https://arxiv.org/abs/2411.01340):
  attested certificate keys checked against CT, with no browser changes.
- [dstack's zero-trust HTTPS](https://arxiv.org/abs/2509.11555), and
  [its gateway docs](https://github.com/Dstack-TEE/dstack/blob/next/docs/dstack-gateway.md):
  the ACME account in the TEE, pinned by CAA.
- [The evolution of remote attested TLS](https://collective.flashbots.net/t/the-evolution-of-remote-attested-tls/5383)
  (Flashbots): certificate extensions, handshake extensions, then
  post-handshake attestation, and binding the key against binding the session.
- [attested-tls-proxy](https://github.com/flashbots/attested-tls-proxy):
  post-handshake attested TLS, binding both.
- [Attested TLS protocol redesign for scale and performance](https://collective.flashbots.net/t/attested-tls-protocol-redesign-for-scale-and-performance/5794)
  (Flashbots): why session binding does not scale, and nested TLS, with an
  attested inner certificate inside an ordinary outer one.
- [Why does ACME use CSRs?](https://community.letsencrypt.org/t/why-does-acme-use-csrs/190463)
  and [how Let's Encrypt works](https://letsencrypt.org/how-it-works/).
