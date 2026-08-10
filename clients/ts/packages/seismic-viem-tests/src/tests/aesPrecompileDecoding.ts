import { expect } from 'bun:test'
import { aesGcmDecrypt, aesGcmEncrypt } from 'seismic-viem'
import type { Hex } from 'viem'
import { hexToBytes } from 'viem'

const AES_KEY = `0x${'11'.repeat(32)}` as Hex

const ANY_CIPHERTEXT = `0x${'ab'.repeat(16)}` as Hex

/**
 * Minimal stand-in for the precompile call, so these stay unit tests: the
 * precompile returns its raw output bytes, and we assert on what the client
 * hands back to the caller.
 */
const stubClient = (data: Hex) => ({
  call: async (): Promise<{ data: Hex }> => ({ data }),
})

/**
 * AES-GCM output is indistinguishable from random, so roughly 1 in 256 results
 * starts with a zero byte. Those bytes are part of the ciphertext and must
 * survive decoding.
 */
export const testAesGcmEncryptPreservesLeadingZeroByte = async () => {
  const ciphertext = '0x00a1b2c3d4e5f60718293a4b5c6d7e8f' as Hex

  const result = await aesGcmEncrypt(stubClient(ciphertext), {
    aesKey: AES_KEY,
    nonce: 1,
    plaintext: 'hello',
  })

  expect(result).toBe(ciphertext)
  expect(hexToBytes(result).length).toBe(16)
}

/** Multiple leading zero bytes must survive too. */
export const testAesGcmEncryptPreservesMultipleLeadingZeroBytes = async () => {
  const ciphertext = '0x0000f60718293a4b5c6d7e8f90a1b2c3' as Hex

  const result = await aesGcmEncrypt(stubClient(ciphertext), {
    aesKey: AES_KEY,
    nonce: 1,
    plaintext: 'hello',
  })

  expect(result).toBe(ciphertext)
  expect(hexToBytes(result).length).toBe(16)
}

/** Regression guard: ciphertext without leading zeros was never affected. */
export const testAesGcmEncryptPreservesNonZeroLeadingByte = async () => {
  const ciphertext = '0xa1b2c3d4e5f60718293a4b5c6d7e8f90' as Hex

  const result = await aesGcmEncrypt(stubClient(ciphertext), {
    aesKey: AES_KEY,
    nonce: 1,
    plaintext: 'hello',
  })

  expect(result).toBe(ciphertext)
}

/**
 * Encrypting an empty plaintext returns just the 16-byte GCM tag, which is
 * equally likely to begin with a zero byte.
 */
export const testAesGcmEncryptPreservesTagOnlyOutput = async () => {
  const tag = '0x00530f8afbc74536b9a963b4f1c4cb73' as Hex

  const result = await aesGcmEncrypt(stubClient(tag), {
    aesKey: AES_KEY,
    nonce: 1,
    plaintext: '',
  })

  expect(result).toBe(tag)
  expect(hexToBytes(result).length).toBe(16)
}

/** A decrypted plaintext that begins with a NUL byte must keep it. */
export const testAesGcmDecryptPreservesLeadingNulByte = async () => {
  // NUL followed by "hello"
  const plaintextHex = '0x0068656c6c6f' as Hex
  const expected = `${String.fromCharCode(0)}hello`

  const result = await aesGcmDecrypt(stubClient(plaintextHex), {
    aesKey: AES_KEY,
    nonce: 1,
    ciphertext: ANY_CIPHERTEXT,
  })

  expect(result).toBe(expected)
}

/** Regression guard: ordinary text was never affected. */
export const testAesGcmDecryptPreservesOrdinaryText = async () => {
  const plaintextHex = '0x68656c6c6f' as Hex

  const result = await aesGcmDecrypt(stubClient(plaintextHex), {
    aesKey: AES_KEY,
    nonce: 1,
    ciphertext: ANY_CIPHERTEXT,
  })

  expect(result).toBe('hello')
}
