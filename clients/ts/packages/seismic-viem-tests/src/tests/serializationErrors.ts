import { expect } from 'bun:test'
import { serializeSeismicTransaction } from 'seismic-viem'
import type { Hex } from 'viem'

/**
 * Values sourced from the existing encoding.ts test, which verifies
 * that these produce a known-good serialized output.
 */
const SANVIL_CHAIN_ID = 31337

// Address used in encoding.ts test
const ENCODING_TEST_TO = '0xd3e8763675e4c425df46cc3b5c0f6cbdac396046' as const

// Encryption pubkey from the ENC_PK constant in viem.test.ts
const ENCRYPTION_PUBKEY =
  '0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0' as Hex

// Encryption nonce from the encoding.ts test
const ENCRYPTION_NONCE = '0x46a2b6020bba77fcb1e676a6' as Hex

// Block hash from the encoding.ts test
const RECENT_BLOCK_HASH =
  '0x934207181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f90' as Hex

// Minimal valid calldata (4-byte function selector)
const SAMPLE_CALLDATA = '0xdeadbeef' as Hex

const SAMPLE_GAS_PRICE = 1_000_000_000n // 1 gwei
const SAMPLE_GAS_LIMIT = 100_000n
const SAMPLE_EXPIRES_AT_BLOCK = 100n

const validSeismicTx = {
  type: 'seismic' as const,
  chainId: SANVIL_CHAIN_ID,
  nonce: 1,
  gasPrice: SAMPLE_GAS_PRICE,
  gas: SAMPLE_GAS_LIMIT,
  to: ENCODING_TEST_TO,
  value: 0n,
  data: SAMPLE_CALLDATA,
  encryptionPubkey: ENCRYPTION_PUBKEY,
  encryptionNonce: ENCRYPTION_NONCE,
  messageVersion: 0,
  recentBlockHash: RECENT_BLOCK_HASH,
  expiresAtBlock: SAMPLE_EXPIRES_AT_BLOCK,
  signedRead: false,
}

export const testSerializeMissingChainId = () => {
  const { chainId: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require chainId'
  )
}

export const testSerializeMissingNonce = () => {
  const { nonce: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require nonce'
  )
}

export const testSerializeMissingGasPrice = () => {
  const { gasPrice: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require gasPrice'
  )
}

export const testSerializeMissingGas = () => {
  const { gas: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require gas'
  )
}

export const testSerializeMissingTo = () => {
  const { to: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require to'
  )
}

export const testSerializeMissingEncryptionPubkey = () => {
  const { encryptionPubkey: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require encryptionPubkey'
  )
}

export const testSerializeMissingEncryptionNonce = () => {
  const { encryptionNonce: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require encryptionNonce'
  )
}

export const testSerializeMissingRecentBlockHash = () => {
  const { recentBlockHash: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require recentBlockHash'
  )
}

export const testSerializeMissingExpiresAtBlock = () => {
  const { expiresAtBlock: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require expiresAtBlock'
  )
}

export const testSerializeMissingData = () => {
  const { data: _, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx as any)).toThrow(
    'Seismic transactions require input'
  )
}

export const testSerializeValidTxDoesNotThrow = () => {
  expect(() => serializeSeismicTransaction(validSeismicTx as any)).not.toThrow()
}
