import { expect } from 'bun:test'
import type { TransactionSerializableSeismic } from 'seismic-viem'
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

const validSeismicTx: TransactionSerializableSeismic = {
  type: 'seismic',
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

type SeismicTxField = keyof typeof validSeismicTx

const expectThrowsWithoutField = (
  field: SeismicTxField,
  message: string
): void => {
  const { [field]: _omitted, ...tx } = validSeismicTx
  expect(() => serializeSeismicTransaction(tx)).toThrow(message)
}

export const testSerializeMissingChainId = () =>
  expectThrowsWithoutField('chainId', 'Seismic transactions require chainId')

export const testSerializeMissingNonce = () =>
  expectThrowsWithoutField('nonce', 'Seismic transactions require nonce')

export const testSerializeMissingGasPrice = () =>
  expectThrowsWithoutField('gasPrice', 'Seismic transactions require gasPrice')

export const testSerializeMissingGas = () =>
  expectThrowsWithoutField('gas', 'Seismic transactions require gas')

export const testSerializeMissingTo = () =>
  expectThrowsWithoutField('to', 'Seismic transactions require to')

export const testSerializeMissingEncryptionPubkey = () =>
  expectThrowsWithoutField(
    'encryptionPubkey',
    'Seismic transactions require encryptionPubkey'
  )

export const testSerializeMissingEncryptionNonce = () =>
  expectThrowsWithoutField(
    'encryptionNonce',
    'Seismic transactions require encryptionNonce'
  )

export const testSerializeMissingRecentBlockHash = () =>
  expectThrowsWithoutField(
    'recentBlockHash',
    'Seismic transactions require recentBlockHash'
  )

export const testSerializeMissingExpiresAtBlock = () =>
  expectThrowsWithoutField(
    'expiresAtBlock',
    'Seismic transactions require expiresAtBlock'
  )

export const testSerializeMissingData = () =>
  expectThrowsWithoutField('data', 'Seismic transactions require input')

export const testSerializeValidTxDoesNotThrow = () => {
  expect(() => serializeSeismicTransaction(validSeismicTx)).not.toThrow()
}
