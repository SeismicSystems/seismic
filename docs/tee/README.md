# Seismic in a TEE

Seismic keeps transaction calldata and shielded contract state private,
including from the operators who run its nodes. The design follows from a short
chain of constraints:

1. **Every node sees every input.** A blockchain stays correct because every
   node re-executes every transaction.
2. **So each node runs inside a TDX confidential VM.** The host cannot read the
   guest's memory, and the hardware signs a quote stating which code the guest
   booted.
3. **The host is the adversary.** It owns the guest's disk, network, clock and
   lifecycle, and it can boot as many guests as it likes.
4. **Every node must decrypt the same client traffic.** So the network shares
   one secret, `root_key`, and derives every network key from it.
5. **Joining the network means being handed `root_key`.** Most of the design is
   about who may receive it.
6. **A quote proves execution, not membership.** It shows a genuine guest
   running reviewed code, but anyone can boot the same image and found a clone
   network.
7. **So the network is named by a hash of its founding artifacts**,
   `network_id`, and every attested exchange binds it.

Each doc below covers one topic, so start with the one that matches your
question. For the whole picture in one diagram, start at [the trust
chain](trust-model.md#the-trust-chain).

## The docs

- [architecture.md](architecture.md) — **what a TEE node is.** The processes
  and how their units are grouped, the one TPM user, what the outside can
  reach, key custody and the custodian's socket boundary, the keys by family,
  the root-key handshake cryptography, the LUKS volume, and the boot chain
  from power-on to serving.
  Read it when you touch a service, a key, a port, or the boot order.
- [networking.md](networking.md) — **how nodes find each other.** The three
  planes and each one's identity, discovery and trust posture; the names a node
  carries, in RFC 1498's terms; where every address comes from, who binds it
  and when, how DNS names are made, and what is verified about an address;
  and bootstrap phase by phase. Read it when you touch a plane, bootnodes, an
  address, or DNS.
- [network-manifest.md](network-manifest.md) — **what identifies a network.**
  The fields, `network_id = SHA-256(file bytes)`, the byte-exactness rule,
  what the hash commits to, the transcript bindings, and the root-key pin.
  Read it when you touch the manifest, a binding, or anything that hashes it.
- [network-founding.md](network-founding.md) — **how a network is born.** Where
  validator keys come from, the founding flow from harvest to launch checks,
  how summit's keys reach their keystore, the founding window, and what the
  manifest pins of summit's genesis. Read it before changing the boot chain or
  the founding flow.
- [chain-backed-admission.md](chain-backed-admission.md) — **how a node gets
  in.** The root-key handshake, the admission predicate, the readiness and
  freshness gate, and which repo owns each stage of an image release. Read it
  when working on admission, the registry, or the measurement pipeline.
- [trust-model.md](trust-model.md) — **what is trusted, and what is
  accepted.** The trust chain from the out-of-band roots to every check, what a
  quote does and does not prove, the anchor each actor verifies against, how
  the standard vocabularies (RATS, Noise) map onto the design, the accepted
  risks including the rollback family, and the open decisions. Read it before
  extending trust to a new actor, accepting a new residual, or closing a listed
  decision.
- [decisions/](decisions/) — **dated decision records.** The options weighed
  at one point in time, never updated afterwards. The docs above are the
  authority on the current design.

Normative specs live next to the code that implements them:

- [measurement-admission SPEC.md](https://github.com/SeismicSystems/enclave/blob/seismic/crates/measurement-admission/SPEC.md)
  — the byte-exact admission rules: the Azure TDX v1 schema, policy-document
  format, admission-ID derivation, registry genesis storage, revision lifecycle.
- [seismic-images azure-measurements.md](https://github.com/SeismicSystems/seismic-images/blob/seismic/docs/azure-measurements.md)
  — the Azure guest-measurement review: what each register covers, and why the
  schema pins the ones it does.

## The code

Three services, all in the [enclave](https://github.com/SeismicSystems/enclave)
repo, plus the image that measures them:

| | |
|---|---|
| `tdx-init` | Blocks for the config POST, validates it, fans the artifacts out to `/run/seismic/conf/`. |
| `custodian` | Owns `root_key`. No network listener, unix socket only; wraps and unwraps against verified handshake bindings. |
| `attestation-service` | The only TPM user: mints every quote, the founding harvest's included, runs both halves of the root-key handshake, and makes the admission decision. |
| [seismic-images](https://github.com/SeismicSystems/seismic-images) | The measured TDX image, and `make measure`, whose output becomes the accepted measurement set. Its units carry summit's keys to their keystore: summit's own `keys generate` at boot, then a persist script once LUKS opens. |
