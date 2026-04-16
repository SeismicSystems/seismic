import { expect } from 'bun:test'
import { serializeSeismicTransaction } from 'seismic-viem'
import type { Hex } from 'viem'

import {
  ENCODING_TEST_ENCRYPTION_NONCE,
  ENCODING_TEST_RECENT_BLOCK_HASH,
  ENCODING_TEST_TO,
  ENCRYPTION_PK,
  SANVIL_CHAIN_ID,
} from '@sviem-tests/constants.ts'

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
  data: '0xdeadbeef' as Hex,
  encryptionPubkey: ENCRYPTION_PK,
  encryptionNonce: ENCODING_TEST_ENCRYPTION_NONCE,
  messageVersion: 0,
  recentBlockHash: ENCODING_TEST_RECENT_BLOCK_HASH,
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
