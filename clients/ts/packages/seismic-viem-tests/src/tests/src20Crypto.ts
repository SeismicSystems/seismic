import { expect } from 'bun:test'
import { AesGcmCrypto, computeKeyHash, parseEncryptedData } from 'seismic-viem'
import type { Hex } from 'viem'
import { keccak256, numberToHex } from 'viem'

const AES_KEY_ZEROS =
  '0x0000000000000000000000000000000000000000000000000000000000000000' as Hex

const AES_KEY_ONE =
  '0x0000000000000000000000000000000000000000000000000000000000000001' as Hex

const AES_KEY_TWO =
  '0x0000000000000000000000000000000000000000000000000000000000000002' as Hex

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

export const testParseEncryptedDataRoundtrip = async () => {
  const cipher = new AesGcmCrypto(AES_KEY_ZEROS)
  const tokenAmount = 1_000_000n
  const plaintextHex = numberToHex(tokenAmount, { size: 32 })
  const nonceHex = cipher.createNonce(42)

  const ciphertext = await cipher.encrypt(plaintextHex, nonceHex)
  const packedBlob = `${ciphertext}${nonceHex.slice(2)}` as Hex

  const parsed = parseEncryptedData(packedBlob)

  expect(parsed.nonce).toBe(nonceHex)
  expect(parsed.ciphertext).toBe(ciphertext)

  const decrypted = await cipher.decrypt(parsed.ciphertext, parsed.nonce)
  expect(BigInt(decrypted)).toBe(tokenAmount)
}

export const testParseEncryptedDataRoundtripLargeAmount = async () => {
  const cipher = new AesGcmCrypto(AES_KEY_ONE)
  const tokenAmount =
    0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffen
  const plaintextHex = numberToHex(tokenAmount, { size: 32 })
  const nonceHex = cipher.createNonce(999)

  const ciphertext = await cipher.encrypt(plaintextHex, nonceHex)
  const packedBlob = `${ciphertext}${nonceHex.slice(2)}` as Hex

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
