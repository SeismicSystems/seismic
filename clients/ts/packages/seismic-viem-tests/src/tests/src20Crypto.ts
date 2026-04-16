import { expect } from 'bun:test'
import { AesGcmCrypto, computeKeyHash } from 'seismic-viem'
import type { Hex } from 'viem'
import { keccak256, numberToHex } from 'viem'

// 32-byte AES-256 key (all zeros — matches the key used in precompiles.ts)
const AES_KEY_ZEROS =
  '0x0000000000000000000000000000000000000000000000000000000000000000' as Hex

const AES_KEY_ONE =
  '0x0000000000000000000000000000000000000000000000000000000000000001' as Hex

const AES_KEY_TWO =
  '0x0000000000000000000000000000000000000000000000000000000000000002' as Hex

// 24 hex chars = 12 bytes, matching @sviem/actions/src20/crypto.ts
const NONCE_HEX_LENGTH = 24

/**
 * Inline copy of parseEncryptedData since it's not exported from
 * seismic-viem. Mirrors @sviem/actions/src20/crypto.ts exactly.
 */

function parseEncryptedData(encryptedData: Hex): {
  ciphertext: Hex
  nonce: Hex
} {
  if (!encryptedData || encryptedData === '0x' || encryptedData.length <= 2) {
    throw new Error('Empty encrypted data - recipient has no key')
  }
  const nonce = `0x${encryptedData.slice(-NONCE_HEX_LENGTH)}` as Hex
  const ciphertext = encryptedData.slice(0, -NONCE_HEX_LENGTH) as Hex
  return { ciphertext, nonce }
}

export const testParseEncryptedDataThrowsOnEmpty = () => {
  expect(() => parseEncryptedData('0x' as Hex)).toThrow(
    'Empty encrypted data - recipient has no key'
  )
}

export const testParseEncryptedDataThrowsOnEmptyString = () => {
  expect(() => parseEncryptedData('' as Hex)).toThrow(
    'Empty encrypted data - recipient has no key'
  )
}

/**
 * Encrypt real data with AesGcmCrypto, pack it the way SRC20 events do
 * (ciphertext || nonce), then parse and decrypt — verify the plaintext
 * survives the roundtrip.
 */
export const testParseEncryptedDataRoundtrip = async () => {
  const cipher = new AesGcmCrypto(AES_KEY_ZEROS)

  // Encode a token amount the way SRC20 does: uint256 as 32-byte hex
  const tokenAmount = 1_000_000n
  const plaintextHex = numberToHex(tokenAmount, { size: 32 })

  // Use a numeric nonce (the way AesGcmCrypto.createNonce works)
  const nonceNum = 42n
  const nonceHex = cipher.createNonce(Number(nonceNum))

  // Encrypt
  const ciphertext = await cipher.encrypt(plaintextHex, nonceHex)

  // Build the packed blob: ciphertext_hex + nonce_hex (no 0x on nonce)
  // This matches how SRC20 Transfer events encode encryptedAmount.
  const nonceWithout0x = nonceHex.slice(2)
  const packedBlob = `${ciphertext}${nonceWithout0x}` as Hex

  // Parse — this is the function under test
  const parsed = parseEncryptedData(packedBlob)

  // The parsed nonce should match the original nonce
  expect(parsed.nonce).toBe(nonceHex)

  // The parsed ciphertext should match the original ciphertext
  expect(parsed.ciphertext).toBe(ciphertext)

  // Decrypt with the parsed values — should recover the original amount
  const decrypted = await cipher.decrypt(parsed.ciphertext, parsed.nonce)
  expect(BigInt(decrypted)).toBe(tokenAmount)
}

/**
 * Same roundtrip but with a different key and a larger amount,
 * to verify the split doesn't depend on specific ciphertext length.
 */
export const testParseEncryptedDataRoundtripLargeAmount = async () => {
  const cipher = new AesGcmCrypto(AES_KEY_ONE)

  // Max uint256 - 1
  const tokenAmount =
    0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffen
  const plaintextHex = numberToHex(tokenAmount, { size: 32 })

  const nonceNum = 999n
  const nonceHex = cipher.createNonce(Number(nonceNum))

  const ciphertext = await cipher.encrypt(plaintextHex, nonceHex)
  const nonceWithout0x = nonceHex.slice(2)
  const packedBlob = `${ciphertext}${nonceWithout0x}` as Hex

  const parsed = parseEncryptedData(packedBlob)

  const decrypted = await cipher.decrypt(parsed.ciphertext, parsed.nonce)
  expect(BigInt(decrypted)).toBe(tokenAmount)
}

export const testComputeKeyHashIsDeterministic = () => {
  const hash1 = computeKeyHash(AES_KEY_ONE)
  const hash2 = computeKeyHash(AES_KEY_ONE)
  expect(hash1).toBe(hash2)
}

export const testComputeKeyHashMatchesKeccak256 = () => {
  const hash = computeKeyHash(AES_KEY_ONE)
  const expected = keccak256(AES_KEY_ONE)
  expect(hash).toBe(expected)
}

export const testComputeKeyHashDifferentKeysProduceDifferentHashes = () => {
  const hash1 = computeKeyHash(AES_KEY_ONE)
  const hash2 = computeKeyHash(AES_KEY_TWO)
  expect(hash1).not.toBe(hash2)
}
