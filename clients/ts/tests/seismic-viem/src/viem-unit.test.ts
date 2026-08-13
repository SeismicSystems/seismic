import { describe, expect, test } from 'bun:test'
import { AesGcmCrypto } from 'seismic-viem'
import type { Hex } from 'viem'

import {
  testAddressExplorerUrlBuildsCorrectUrl,
  testAddressExplorerUrlReturnsNullWithoutExplorer,
  testAddressExplorerUrlWithTab,
  testBlockExplorerUrlBuildsCorrectUrl,
  testBlockExplorerUrlWithTab,
  testGetExplorerUrlBuildsItemUrl,
  testGetExplorerUrlBuildsItemUrlWithTab,
  testGetExplorerUrlReturnsBaseUrlWithoutOptions,
  testGetExplorerUrlReturnsNullForChainWithoutExplorer,
  testGetExplorerUrlReturnsNullForUndefinedChain,
  testSanvilHasNoExplorer,
  testTokenExplorerUrlBuildsCorrectUrl,
  testTokenExplorerUrlWithTab,
  testTxExplorerUrlBuildsCorrectUrl,
  testTxExplorerUrlReturnsNullWithoutChain,
  testTxExplorerUrlWithTab,
} from '@sviem-tests/tests/explorerUrl.ts'
import {
  testSerializeMissingChainId,
  testSerializeMissingData,
  testSerializeMissingEncryptionNonce,
  testSerializeMissingEncryptionPubkey,
  testSerializeMissingExpiresAtBlock,
  testSerializeMissingGas,
  testSerializeMissingGasPrice,
  testSerializeMissingNonce,
  testSerializeMissingRecentBlockHash,
  testSerializeMissingTo,
  testSerializeValidTxDoesNotThrow,
} from '@sviem-tests/tests/seismicTxValidation.ts'
import {
  testComputeKeyHashDifferentKeysProduceDifferentHashes,
  testComputeKeyHashIsDeterministic,
  testComputeKeyHashMatchesKeccak256,
  testParseEncryptedDataRoundtrip,
  testParseEncryptedDataRoundtripLargeAmount,
  testParseEncryptedDataThrowsOnEmpty,
  testParseEncryptedDataThrowsOnEmptyString,
} from '@sviem-tests/tests/src20Crypto.ts'
import {
  testEmptyAuthorizationListHash,
  testTypedDataIncludesAuthorizationListHash,
} from '@sviem-tests/tests/typedDataUnit.ts'

describe('Explorer URL utilities', () => {
  test(
    'getExplorerUrl returns null for undefined chain',
    testGetExplorerUrlReturnsNullForUndefinedChain
  )
  test(
    'getExplorerUrl returns null for chain without explorer',
    testGetExplorerUrlReturnsNullForChainWithoutExplorer
  )
  test(
    'getExplorerUrl returns base URL without options',
    testGetExplorerUrlReturnsBaseUrlWithoutOptions
  )
  test('getExplorerUrl builds item URL', testGetExplorerUrlBuildsItemUrl)
  test(
    'getExplorerUrl builds item URL with tab',
    testGetExplorerUrlBuildsItemUrlWithTab
  )
  test('txExplorerUrl builds correct URL', testTxExplorerUrlBuildsCorrectUrl)
  test('txExplorerUrl appends tab param', testTxExplorerUrlWithTab)
  test(
    'txExplorerUrl returns null without chain',
    testTxExplorerUrlReturnsNullWithoutChain
  )
  test(
    'addressExplorerUrl builds correct URL',
    testAddressExplorerUrlBuildsCorrectUrl
  )
  test('addressExplorerUrl appends tab param', testAddressExplorerUrlWithTab)
  test(
    'addressExplorerUrl returns null without explorer',
    testAddressExplorerUrlReturnsNullWithoutExplorer
  )
  test(
    'blockExplorerUrl builds correct URL',
    testBlockExplorerUrlBuildsCorrectUrl
  )
  test('blockExplorerUrl appends tab param', testBlockExplorerUrlWithTab)
  test(
    'tokenExplorerUrl builds correct URL',
    testTokenExplorerUrlBuildsCorrectUrl
  )
  test('tokenExplorerUrl appends tab param', testTokenExplorerUrlWithTab)
  test('sanvil chain has no explorer', testSanvilHasNoExplorer)
})

describe('seismic tx field validation', () => {
  test('throws when chainId is missing', testSerializeMissingChainId)
  test('throws when nonce is missing', testSerializeMissingNonce)
  test('throws when gasPrice is missing', testSerializeMissingGasPrice)
  test('throws when gas is missing', testSerializeMissingGas)
  test('throws when to is missing', testSerializeMissingTo)
  test(
    'throws when encryptionPubkey is missing',
    testSerializeMissingEncryptionPubkey
  )
  test(
    'throws when encryptionNonce is missing',
    testSerializeMissingEncryptionNonce
  )
  test(
    'throws when recentBlockHash is missing',
    testSerializeMissingRecentBlockHash
  )
  test(
    'throws when expiresAtBlock is missing',
    testSerializeMissingExpiresAtBlock
  )
  test('throws when data is missing', testSerializeMissingData)
  test('valid tx does not throw', testSerializeValidTxDoesNotThrow)
})

describe('SRC20 parseEncryptedData', () => {
  test('throws on empty hex (0x)', testParseEncryptedDataThrowsOnEmpty)
  test('throws on empty string', testParseEncryptedDataThrowsOnEmptyString)
  test(
    'encrypt → pack → parse → decrypt roundtrip',
    testParseEncryptedDataRoundtrip
  )
  test(
    'roundtrip with large amount and different key',
    testParseEncryptedDataRoundtripLargeAmount
  )
})

describe('Directory computeKeyHash', () => {
  test('is deterministic', testComputeKeyHashIsDeterministic)
  test('matches keccak256', testComputeKeyHashMatchesKeccak256)
  test(
    'different keys produce different hashes',
    testComputeKeyHashDifferentKeysProduceDifferentHashes
  )
})

describe('Seismic EIP-712 typed data', () => {
  test('hashes an empty authorization list', testEmptyAuthorizationListHash)
  test(
    'includes authorizationListHash',
    testTypedDataIncludesAuthorizationListHash
  )
})

describe('AES-GCM numeric nonce bounds', () => {
  const crypto = new AesGcmCrypto(
    '0x0000000000000000000000000000000000000000000000000000000000000000' as Hex
  )

  test('rejects negative nonces', () => {
    expect(() => crypto.createNonce(-1)).toThrow(
      'Nonce must fit in an unsigned 64-bit integer'
    )
  })

  test('rejects unsafe numeric nonces', () => {
    expect(() => crypto.createNonce(Number.MAX_SAFE_INTEGER + 2)).toThrow(
      'Nonce must be a safe integer'
    )
  })

  test('rejects nonces larger than u64', () => {
    expect(() => crypto.createNonce(0x1_0000_0000_0000_0000n)).toThrow(
      'Nonce must fit in an unsigned 64-bit integer'
    )
  })
})