import type { Address, Hex, TransactionSerializableEIP7702 } from 'viem'
import {
  bytesToHex,
  concatHex,
  createPublicClient,
  getAddress,
  hexToBigInt,
  hexToBytes,
  http,
  isAddress,
  toHex,
  toRlp,
  trim,
  zeroAddress,
} from 'viem'
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts'

import { randomBytes } from '@noble/ciphers/webcrypto'
import { gcm } from '@noble/ciphers/webcrypto'
import { secp256k1 } from '@noble/curves/secp256k1'
import { hkdf } from '@noble/hashes/hkdf'
import { sha256 } from '@noble/hashes/sha256'

/** Public, signed fee selection. Explicit Native/Token never fall back. */
export type GasPayment =
  | { type: 'auto' }
  | { type: 'native' }
  | { type: 'token'; token: Address }

export const SEISMIC_TX_TYPE = 0x4a
const DEFAULT_SEISMIC_BLOCKS_WINDOW = 100n

// ── Helpers (inlined from seismic-viem to keep this package standalone) ──

// Keep selector validation/encoding aligned with seismic-viem's gasPayment.ts.
// Omission is resolved before signing; signed bytes always include this field.
const normalizeGasPayment = (value: unknown = undefined): GasPayment => {
  if (value === undefined) return { type: 'auto' }
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error('Invalid gasPayment: expected a tagged object')
  }
  const fields = value as Record<string, unknown>
  const keys = Object.keys(fields)
  if (!Object.prototype.hasOwnProperty.call(fields, 'type')) {
    throw new Error('Invalid gasPayment: missing type')
  }
  if (fields.type === 'auto' || fields.type === 'native') {
    if (keys.length !== 1) {
      throw new Error(
        'Invalid gasPayment: auto/native cannot carry extra fields'
      )
    }
    return { type: fields.type }
  }
  if (
    fields.type === 'token' &&
    keys.length === 2 &&
    Object.prototype.hasOwnProperty.call(fields, 'token') &&
    typeof fields.token === 'string' &&
    isAddress(fields.token, { strict: false }) &&
    fields.token.toLowerCase() !== zeroAddress
  ) {
    return { type: 'token', token: getAddress(fields.token) }
  }
  throw new Error('Invalid gasPayment: expected a nonzero token address')
}

const gasPaymentRlp = (payment: GasPayment): Hex[] => {
  switch (payment.type) {
    case 'auto':
      return ['0x', '0x']
    case 'native':
      return ['0x01', '0x']
    case 'token':
      return ['0x02', payment.token]
  }
}

const compressPublicKey = (uncompressedKey: Hex): Hex => {
  const cleanKey = uncompressedKey.replace('0x', '')
  if (cleanKey.length !== 130) {
    throw new Error('Invalid uncompressed public key length')
  }
  const pt = secp256k1.ProjectivePoint.fromHex(cleanKey)
  return bytesToHex(pt.toRawBytes(true))
}

const randomEncryptionNonce = (): Hex => {
  let nonce = bytesToHex(randomBytes(12))
  while (nonce !== trim(nonce)) {
    nonce = bytesToHex(randomBytes(12))
  }
  return nonce
}

const toYParitySignatureArray = (signature?: {
  v: bigint
  r: Hex
  s: Hex
}): Hex[] => {
  if (!signature) return []
  const { v, r, s } = signature
  const trimR = trim(r)
  const trimS = trim(s)
  const yParity = v === 0n || v === 27n ? '0x' : toHex(1)
  return [
    yParity,
    trimR === '0x00' ? '0x' : trimR,
    trimS === '0x00' ? '0x' : trimS,
  ] as Hex[]
}

// ── Key derivation ──────────────────────────────────────────────────

// Only the request key: this package encrypts write-tx calldata and never
// decrypts TEE responses. The request uses the original "aes-gcm key" label.
// A signed-read feature would need the response label
// ("seismic/response/aes-256-gcm/v1") as well.
const deriveRequestAesKey = (
  privateKey: Hex,
  networkPublicKey: string
): Hex => {
  const privHex = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey
  const sharedPoint = secp256k1
    .getSharedSecret(privHex, networkPublicKey, false)
    .slice(1)

  const version = (sharedPoint[63] & 0x01) | 0x02
  const compressed = sha256
    .create()
    .update(new Uint8Array([version]))
    .update(sharedPoint.slice(0, 32))
    .digest()

  const derived = hkdf(
    sha256,
    compressed,
    new Uint8Array(0),
    new TextEncoder().encode('aes-gcm key'),
    32
  )
  return bytesToHex(derived)
}

// ── AAD encoding ────────────────────────────────────────────────────

const encodeAAD = (fields: {
  sender: Address
  chainId: number
  nonce: number
  to: Address | null
  value: bigint
  encryptionPubkey: Hex
  encryptionNonce: Hex
  messageVersion: number
  recentBlockHash: Hex
  expiresAtBlock: bigint
  signedRead: boolean
}): Uint8Array => {
  const rlpFields: Hex[] = [
    fields.sender,
    toHex(fields.chainId),
    fields.nonce === 0 ? '0x' : toHex(fields.nonce),
    fields.to ?? '0x',
    fields.value === 0n ? '0x' : toHex(fields.value),
    fields.encryptionPubkey,
    fields.encryptionNonce === '0x00' || fields.encryptionNonce === '0x0'
      ? '0x'
      : fields.encryptionNonce,
    fields.messageVersion === 0 ? '0x' : toHex(fields.messageVersion),
    fields.recentBlockHash,
    toHex(fields.expiresAtBlock),
    fields.signedRead ? '0x01' : '0x',
  ]
  return toRlp(rlpFields as any, 'bytes')
}

// ── AES-GCM encrypt ─────────────────────────────────────────────────

const aesGcmEncrypt = async (
  key: Hex,
  nonce: Hex,
  plaintext: Hex,
  aad: Uint8Array
): Promise<Hex> => {
  if (!plaintext || plaintext === '0x') return '0x'
  const nonceBytes = hexToBytes(nonce)
  if (nonceBytes.length !== 12) {
    throw new Error('Nonce must be 12 bytes')
  }
  const ciphertext = await gcm(hexToBytes(key), nonceBytes, aad).encrypt(
    hexToBytes(plaintext)
  )
  return bytesToHex(ciphertext)
}

// ── Serializer ──────────────────────────────────────────────────────

export const serializeSeismicTx = (
  tx: {
    chainId: number
    nonce: number
    gasPrice: bigint
    gas: bigint
    /** Omission resolves to Auto before serialization/signing. */
    gasPayment?: GasPayment
    to: Address | null
    value: bigint
    encryptionPubkey: Hex
    encryptionNonce: Hex
    messageVersion: number
    recentBlockHash: Hex
    expiresAtBlock: bigint
    signedRead: boolean
    data: Hex
    authorizationList?: TransactionSerializableEIP7702['authorizationList']
  },
  signature?: { v: bigint; r: Hex; s: Hex }
): Hex => {
  const rlpArray = [
    tx.chainId ? toHex(tx.chainId) : '0x',
    tx.nonce ? toHex(tx.nonce) : '0x',
    tx.gasPrice ? toHex(tx.gasPrice) : '0x',
    tx.gas ? toHex(tx.gas) : '0x',
    gasPaymentRlp(normalizeGasPayment(tx.gasPayment)),
    tx.to ?? '0x',
    tx.value ? toHex(tx.value) : '0x',
    tx.encryptionPubkey ?? '0x',
    hexToBigInt(tx.encryptionNonce) === 0n
      ? '0x'
      : toHex(hexToBigInt(tx.encryptionNonce)),
    tx.messageVersion === 0 ? '0x' : toHex(tx.messageVersion),
    tx.recentBlockHash,
    tx.expiresAtBlock ? toHex(tx.expiresAtBlock) : '0x',
    tx.signedRead ? '0x01' : '0x',
    tx.data ?? '0x',
    (tx.authorizationList ?? []).map((auth) => [
      auth.chainId ? toHex(auth.chainId) : '0x',
      auth.contractAddress,
      auth.nonce ? toHex(auth.nonce) : '0x',
      auth.yParity ? toHex(auth.yParity) : '0x',
      hexToBigInt(auth.r) ? toHex(hexToBigInt(auth.r)) : '0x',
      hexToBigInt(auth.s) ? toHex(hexToBigInt(auth.s)) : '0x',
    ]),
    ...toYParitySignatureArray(signature),
  ]
  return concatHex([
    toHex(SEISMIC_TX_TYPE),
    toRlp(rlpArray as Parameters<typeof toRlp>[0]),
  ])
}

// ── Public API ──────────────────────────────────────────────────────

export type EncryptSeismicTxParams = {
  /** Standard viem transaction fields */
  tx: {
    to: Address
    data: Hex
    value?: bigint
    nonce: number
    gasPrice: bigint
    gas: bigint
    chainId: number
    /** Public, signed fee selection. Defaults to Auto before signing. */
    gasPayment?: GasPayment
    authorizationList?: TransactionSerializableEIP7702['authorizationList']
  }
  /** Sender address (must match the signer) */
  sender: Address
  /** RPC URL of the Seismic node */
  rpcUrl: string
  /** Optional: your own encryption private key (ephemeral one generated if omitted) */
  encryptionPrivateKey?: Hex
  /**
   * Optional: how many blocks until this tx expires.
   * Defaults to `DEFAULT_SEISMIC_BLOCKS_WINDOW`.
   */
  blocksWindow?: bigint
}

export type EncryptSeismicTxResult = {
  /** The unsigned serialized seismic tx — sign this with your wallet, then sendRawTransaction */
  unsignedSerializedTx: Hex
  /** Individual fields if you want to sign with account.signTransaction + custom serializer */
  seismicTx: {
    chainId: number
    nonce: number
    gasPrice: bigint
    gas: bigint
    /** Resolved selector, always included in the signed wire format. */
    gasPayment: GasPayment
    to: Address | null
    value: bigint
    data: Hex
    encryptionPubkey: Hex
    encryptionNonce: Hex
    messageVersion: number
    recentBlockHash: Hex
    expiresAtBlock: bigint
    signedRead: boolean
    authorizationList?: TransactionSerializableEIP7702['authorizationList']
    type: 'seismic'
  }
  /** Serialize + concat type prefix. Pass a viem Signature to get the final signed bytes. */
  serialize: (signature: { v: bigint; r: Hex; s: Hex }) => Hex
}

/**
 * Takes a stock viem-style transaction and returns an encrypted Seismic
 * transaction (type 0x4a) ready to be signed and sent via
 * `eth_sendRawTransaction`.
 *
 * This is a standalone function — it does NOT require a ShieldedWalletClient.
 * It only needs an RPC URL (to fetch the TEE pubkey and latest block).
 *
 * Usage:
 * ```ts
 * const { seismicTx, serialize } = await encryptSeismicTx({ tx, sender, rpcUrl })
 *
 * // Option A: sign with a local account
 * const signed = await account.signTransaction(
 *   { ...seismicTx },
 *   { serializer: (_tx, sig) => serialize(sig!) },
 * )
 * await publicClient.sendRawTransaction({ serializedTransaction: signed })
 *
 * // Option B: manual signing (you provide v, r, s)
 * const finalBytes = serialize({ v, r, s })
 * await publicClient.sendRawTransaction({ serializedTransaction: finalBytes })
 * ```
 */
export const encryptSeismicTx = async ({
  tx,
  sender,
  rpcUrl,
  encryptionPrivateKey,
  blocksWindow = DEFAULT_SEISMIC_BLOCKS_WINDOW,
}: EncryptSeismicTxParams): Promise<EncryptSeismicTxResult> => {
  const gasPayment = normalizeGasPayment(tx.gasPayment)
  const client = createPublicClient({ transport: http(rpcUrl) })

  // 1. Fetch TEE pubkey and latest block in parallel
  const [teeKeyRaw, latestBlock] = await Promise.all([
    client.request({ method: 'seismic_getTeePublicKey' as any }) as Promise<
      Hex | string
    >,
    client.getBlock({ blockTag: 'latest' }),
  ])
  const teePubkey = (teeKeyRaw as string).startsWith('0x')
    ? (teeKeyRaw as string).slice(2)
    : (teeKeyRaw as string)

  // 2. Derive encryption keys
  const encPrivKey = encryptionPrivateKey ?? generatePrivateKey()
  const aesKey = deriveRequestAesKey(encPrivKey, teePubkey)
  const uncompressedPk = privateKeyToAccount(encPrivKey).publicKey
  const encPubkey = compressPublicKey(uncompressedPk)

  // 3. Build seismic-specific fields
  const encNonce = randomEncryptionNonce()
  const recentBlockHash = latestBlock.hash
  const expiresAtBlock = latestBlock.number + blocksWindow

  // 4. Encode AAD and encrypt
  const aadFields = {
    sender,
    chainId: tx.chainId,
    nonce: tx.nonce,
    to: tx.to,
    value: tx.value ?? 0n,
    encryptionPubkey: encPubkey,
    encryptionNonce: encNonce,
    messageVersion: 0,
    recentBlockHash,
    expiresAtBlock,
    signedRead: false,
  }
  const aad = encodeAAD(aadFields)
  const encryptedData = await aesGcmEncrypt(aesKey, encNonce, tx.data, aad)

  // 5. Build the full seismic tx object
  const seismicTx = {
    chainId: tx.chainId,
    nonce: tx.nonce,
    gasPrice: tx.gasPrice,
    gas: tx.gas,
    gasPayment,
    to: tx.to,
    value: tx.value ?? 0n,
    data: encryptedData,
    encryptionPubkey: encPubkey,
    encryptionNonce: encNonce,
    messageVersion: 0 as const,
    recentBlockHash,
    expiresAtBlock,
    signedRead: false as const,
    authorizationList: tx.authorizationList,
    type: 'seismic' as const,
  }

  const serialize = (signature: { v: bigint; r: Hex; s: Hex }): Hex =>
    serializeSeismicTx(seismicTx, signature)

  const unsignedSerializedTx = serializeSeismicTx(seismicTx)

  return { unsignedSerializedTx, seismicTx, serialize }
}
