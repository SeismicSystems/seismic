import { expect } from 'bun:test'
import { aesGcmDecryptPrecompile, aesGcmEncryptPrecompile } from 'seismic-viem'
import { hexToString } from 'viem'
import type { Hex } from 'viem'

/**
 * The AES-GCM precompiles return raw output bytes, not ABI-encoded data, so
 * there is nothing here for viem's `trim` to strip. `trim` removes leading
 * (and trailing) zero bytes, and AES-GCM ciphertext is indistinguishable from
 * random, so roughly 1 ciphertext in 256 begins with a `0x00` that belongs to
 * the ciphertext. These tests pin the decoded result to the exact bytes the
 * precompile returned.
 */

const CIPHERTEXT_BYTES = 32

const ciphertextWithLeadingByte = (first: number): Hex =>
  `0x${first.toString(16).padStart(2, '0')}${'ab'.repeat(
    CIPHERTEXT_BYTES - 1
  )}` as Hex

export const testAesGcmEncryptDecodePreservesEveryLeadingByte = () => {
  for (let first = 0; first < 256; first++) {
    const raw = ciphertextWithLeadingByte(first)
    const decoded = aesGcmEncryptPrecompile.decodeResult(raw)
    expect(decoded).toBe(raw)
    expect(decoded.length).toBe(raw.length)
  }
}

export const testAesGcmEncryptDecodePreservesLeadingZeroByte = () => {
  const raw: Hex = `0x00${'cd'.repeat(CIPHERTEXT_BYTES - 1)}`
  expect(aesGcmEncryptPrecompile.decodeResult(raw)).toBe(raw)
}

export const testAesGcmEncryptDecodePreservesTrailingZeroByte = () => {
  const raw: Hex = `0x${'cd'.repeat(CIPHERTEXT_BYTES - 1)}00`
  expect(aesGcmEncryptPrecompile.decodeResult(raw)).toBe(raw)
}

export const testAesGcmEncryptDecodePreservesInteriorZeroBytes = () => {
  const raw: Hex = `0xab${'00'.repeat(CIPHERTEXT_BYTES - 2)}cd`
  expect(aesGcmEncryptPrecompile.decodeResult(raw)).toBe(raw)
}

export const testAesGcmDecryptDecodePreservesLeadingNul = () => {
  // `0x00616263` decodes to the plaintext "\0abc" — the leading NUL is data.
  const raw: Hex = '0x00616263'
  expect(aesGcmDecryptPrecompile.decodeResult(raw)).toBe(hexToString(raw))
}
