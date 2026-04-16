// Canonical Seismic tx types and serialization. This module defines the
// in-memory Seismic tx/request shape used across signed calls, shielded writes,
// typed-data signing, and raw Seismic tx serialization.
import { concatHex, hexToBigInt, toHex, toRlp } from 'viem'
import type {
  Address,
  Hex,
  OneOf,
  SerializeTransactionFn,
  Signature,
  TransactionRequest,
  TransactionRequestEIP7702,
  TransactionRequestLegacy,
  TransactionSerializable,
  TransactionSerializableEIP7702,
  TransactionSerializableLegacy,
} from 'viem'

import { toYParitySignatureArray } from '@sviem/viem-internal/signature.ts'

export const SEISMIC_TX_TYPE = 74 // '0x4a'

/**
 * The additional fields added to a Seismic transaction
 * @interface SeismicTxExtras
 * @property {Hex} [encryptionPubkey] - The public key used to encrypt the calldata. This uses AES encryption, where the user's keypair is combined with the network's keypair
 * @property {number} [messageVersion] - The version of the message being sent. Used for signing transactions via messages. Normal transactions use messageVersion = 0. Txs signed with EIP-712 use messageVersion = 2
 */
type SeismicTxExtrasBlank = {
  encryptionPubkey?: undefined
  encryptionNonce?: undefined
  messageVersion?: undefined
  recentBlockHash?: undefined
  expiresAtBlock?: undefined
  signedRead?: undefined
}

export type SeismicTxExtras = {
  encryptionPubkey?: Hex | undefined
  encryptionNonce?: Hex | undefined
  messageVersion?: number | undefined
  recentBlockHash?: Hex | undefined
  expiresAtBlock?: bigint | undefined
  signedRead?: boolean | undefined
}

export type SeismicBlockParams = {
  recentBlockHash: Hex
  expiresAtBlock: bigint
}

export type SeismicElements = {
  encryptionPubkey: Hex
  encryptionNonce: Hex
  messageVersion: number
  signedRead: boolean
} & SeismicBlockParams

/**
 * Advanced overrides for Seismic metadata generation.
 *
 * Most callers should omit these and let the client derive safe defaults from
 * chain state. These are primarily useful for:
 *
 * - deterministic tests and debugging
 * - reproducing an exact encrypted or signed request shape
 * - explicit expiry-window control
 * - low-level interoperability with other Seismic tooling
 *
 * These options are only meaningful on Seismic signed or shielded paths. They
 * do not apply to transparent reads or writes.
 */
export type SeismicSecurityParams = {
  /**
   * Expiry window used when deriving `expiresAtBlock` from a recent block.
   * Defaults to the standard 100-block window.
   * Ignored when `expiresAtBlock` is provided explicitly.
   */
  blocksWindow?: bigint
  /**
   * Override the AEAD encryption nonce used for the Seismic payload.
   * Mainly useful for deterministic tests or reproducing an exact request.
   * When omitted, a random nonce is generated.
   */
  encryptionNonce?: Hex
  /**
   * Override the recent block hash used for freshness and replay protection.
   * If provided without `expiresAtBlock`, expiry is derived from this block plus
   * `blocksWindow`.
   * When omitted, the latest block hash is used.
   */
  recentBlockHash?: Hex
  /**
   * Override the block height after which the signed or shielded request is no
   * longer valid.
   * When omitted, expiry is derived from the chosen recent block plus
   * `blocksWindow`.
   */
  expiresAtBlock?: bigint
}

/**
 * Represents a Seismic transaction request, extending viem's base
 * {@link https://viem.sh/docs/glossary/types#transactionrequest TransactionRequest}
 * with {@link SeismicTxExtras}
 *
 * @interface SeismicTransactionRequest
 * @extends {TransactionRequest}
 * @extends {SeismicTxExtras}
 */
export type SeismicTransactionRequest =
  | (TransactionRequest & SeismicTxExtrasBlank)
  | (Omit<TransactionRequestLegacy, 'type'> &
      SeismicTxExtras & {
        authorizationList?: TransactionRequestEIP7702['authorizationList']
        type: 'seismic'
      })

/**
 * Represents a serializable Seismic transaction, extending viem's base
 * {@link https://viem.sh/docs/utilities/parseTransaction#returns TransactionSerializable}
 * with {@link SeismicTxExtras}
 *
 * @interface TransactionSerializableSeismic
 * @extends {TransactionSerializable}
 * @extends {SeismicTxExtras}
 */
export type TransactionSerializableSeismic =
  | (TransactionSerializable & SeismicTxExtrasBlank)
  | (Omit<TransactionSerializableLegacy, 'type'> &
      SeismicTxExtras & {
        authorizationList?: TransactionSerializableEIP7702['authorizationList']
        type: 'seismic'
      })

export type TxSeismic = {
  chainId?: number
  nonce?: bigint
  gasPrice?: bigint
  gasLimit?: bigint
  to?: Address | null
  isCreate?: boolean
  value?: bigint
  input?: Hex
  encryptionPubkey: Hex
  encryptionNonce: Hex
  messageVersion: number
  recentBlockHash: Hex
  expiresAtBlock: bigint
  signedRead: boolean
  authorizationList?: {
    chainId: bigint
    contractAddress: `0x${string}`
    nonce: bigint
    yParity: number
    r: `0x${string}`
    s: `0x${string}`
  }[]
}

export type SeismicTxSerializer = SerializeTransactionFn<
  TransactionSerializableSeismic,
  'seismic'
>

export const serializeSeismicTransaction: SeismicTxSerializer = (
  tx: OneOf<TransactionSerializable | TransactionSerializableSeismic>,
  signature?: Signature
): Hex => {
  const {
    chainId,
    nonce,
    gasPrice,
    gas,
    to,
    data,
    value = 0n,
    encryptionPubkey,
    encryptionNonce,
    messageVersion = 0,
    recentBlockHash,
    expiresAtBlock,
    signedRead = false,
    authorizationList,
  } = tx

  if (chainId === undefined) {
    throw new Error('Seismic transactions require chainId')
  }
  if (nonce === undefined) {
    throw new Error('Seismic transactions require nonce')
  }
  if (gasPrice === undefined) {
    throw new Error('Seismic transactions require gasPrice')
  }
  if (gas === undefined) {
    throw new Error('Seismic transactions require gas')
  }
  if (to === undefined) {
    throw new Error('Seismic transactions require to')
  }
  if (encryptionPubkey === undefined) {
    throw new Error('Seismic transactions require encryptionPubkey')
  }
  if (encryptionNonce === undefined) {
    throw new Error('Seismic transactions require encryptionNonce')
  }
  if (recentBlockHash === undefined) {
    throw new Error('Seismic transactions require recentBlockHash')
  }
  if (expiresAtBlock === undefined) {
    throw new Error('Seismic transactions require expiresAtBlock')
  }
  if (data === undefined) {
    throw new Error('Seismic transactions require input')
  }

  const rlpArray = [
    toHex(chainId),
    nonce ? toHex(nonce) : '0x',
    gasPrice ? toHex(gasPrice) : '0x',
    gas ? toHex(gas) : '0x',
    to ?? '0x',
    value ? toHex(value) : '0x',
    encryptionPubkey ?? '0x',
    hexToBigInt(encryptionNonce) === 0n ? '0x' : encryptionNonce,
    messageVersion === 0 ? '0x' : toHex(messageVersion),
    recentBlockHash,
    toHex(expiresAtBlock),
    signedRead ? '0x01' : '0x',
    data ?? '0x',
    ((authorizationList ?? []) as TxSeismic['authorizationList'] & []).map(
      (auth) => [
        auth.chainId ? toHex(auth.chainId) : '0x',
        auth.contractAddress,
        auth.nonce ? toHex(auth.nonce) : '0x',
        auth.yParity ? toHex(auth.yParity) : '0x',
        auth.r,
        auth.s,
      ]
    ),
    ...toYParitySignatureArray(tx as TransactionSerializableLegacy, signature),
  ]

  const rlpEncoded = toRlp(rlpArray as any)
  return concatHex([toHex(SEISMIC_TX_TYPE), rlpEncoded])
}
