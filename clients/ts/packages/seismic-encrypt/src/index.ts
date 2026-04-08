import type { Address, Hex } from 'viem'
import {
  bytesToHex,
  concatHex,
  createPublicClient,
  hexToBigInt,
  hexToBytes,
  http,
  toHex,
  toRlp,
  trim,
} from 'viem'
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts'

import { randomBytes } from '@noble/ciphers/webcrypto'
import { gcm } from '@noble/ciphers/webcrypto'
import { secp256k1 } from '@noble/curves/secp256k1'
import { hkdf } from '@noble/hashes/hkdf'
import { sha256 } from '@noble/hashes/sha256'

export const SEISMIC_TX_TYPE = 0x4a

// ── Helpers (inlined from seismic-viem to keep this package standalone) ──

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

const deriveAesKey = (privateKey: Hex, networkPublicKey: string): Hex => {
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
    to: Address | null
    value: bigint
    encryptionPubkey: Hex
    encryptionNonce: Hex
    messageVersion: number
    recentBlockHash: Hex
    expiresAtBlock: bigint
    signedRead: boolean
    data: Hex
  },
  signature?: { v: bigint; r: Hex; s: Hex }
): Hex => {
  const rlpArray: Hex[] = [
    toHex(tx.chainId),
    tx.nonce ? toHex(tx.nonce) : '0x',
    tx.gasPrice ? toHex(tx.gasPrice) : '0x',
    tx.gas ? toHex(tx.gas) : '0x',
    tx.to ?? '0x',
    tx.value ? toHex(tx.value) : '0x',
    tx.encryptionPubkey ?? '0x',
    hexToBigInt(tx.encryptionNonce) === 0n ? '0x' : tx.encryptionNonce,
    tx.messageVersion === 0 ? '0x' : toHex(tx.messageVersion),
    tx.recentBlockHash,
    toHex(tx.expiresAtBlock),
    tx.signedRead ? '0x01' : '0x',
    tx.data ?? '0x',
    ...toYParitySignatureArray(signature),
  ]
  return concatHex([toHex(SEISMIC_TX_TYPE), toRlp(rlpArray)])
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
  }
  /** Sender address (must match the signer) */
  sender: Address
  /** RPC URL of the Seismic node */
  rpcUrl: string
  /** Optional: your own encryption private key (ephemeral one generated if omitted) */
  encryptionPrivateKey?: Hex
  /** Optional: how many blocks until this tx expires (default 100) */
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
    to: Address | null
    value: bigint
    data: Hex
    encryptionPubkey: Hex
    encryptionNonce: Hex
    messageVersion: number
    recentBlockHash: Hex
    expiresAtBlock: bigint
    signedRead: boolean
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
  blocksWindow = 100n,
}: EncryptSeismicTxParams): Promise<EncryptSeismicTxResult> => {
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
  const aesKey = deriveAesKey(encPrivKey, teePubkey)
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
    to: tx.to,
    value: tx.value ?? 0n,
    data: encryptedData,
    encryptionPubkey: encPubkey,
    encryptionNonce: encNonce,
    messageVersion: 0 as const,
    recentBlockHash,
    expiresAtBlock,
    signedRead: false as const,
    type: 'seismic' as const,
  }

  const serialize = (signature: { v: bigint; r: Hex; s: Hex }): Hex =>
    serializeSeismicTx(seismicTx, signature)

  const unsignedSerializedTx = serializeSeismicTx(seismicTx)

  return { unsignedSerializedTx, seismicTx, serialize }
}
