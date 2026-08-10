import { expect } from 'bun:test'
import { AesGcmCrypto } from 'seismic-viem'
import type { Hex } from 'viem'

const AES_KEY = `0x${'11'.repeat(32)}` as Hex

const cipher = () => new AesGcmCrypto(AES_KEY)

/**
 * Reference vectors for the node's `U96` encryption nonce
 * (`TxSeismicElements::encryption_nonce` in seismic-alloy), which is
 * big-endian across all 12 bytes.
 *
 * Produced with the Python client's encoding, `int.to_bytes(12, 'big')`.
 */
const VECTORS: ReadonlyArray<readonly [bigint, Hex]> = [
  [0n, '0x000000000000000000000000'],
  [5n, '0x000000000000000000000005'],
  [42n, '0x00000000000000000000002a'],
  [2n ** 63n, '0x000000008000000000000000'],
  [2n ** 64n, '0x000000010000000000000000'],
  [2n ** 64n + 5n, '0x000000010000000000000005'],
  [2n ** 95n, '0x800000000000000000000000'],
  [2n ** 96n - 1n, '0xffffffffffffffffffffffff'],
]

/** Every value in the 96-bit range must encode like the node and the Python client. */
export const testCreateNonceMatchesU96Encoding = () => {
  for (const [value, expected] of VECTORS) {
    expect(cipher().createNonce(value)).toBe(expected)
  }
}

/**
 * Values differing only above bit 63 must not collide.
 *
 * The nonce field is 96 bits wide, so writing only the low 64 silently maps
 * `n` and `n + 2^64` to the same nonce — reusing an AES-GCM nonce under one
 * key is a hard failure of the mode.
 */
export const testCreateNonceDoesNotCollideAboveU64 = () => {
  const c = cipher()
  expect(c.createNonce(5n)).not.toBe(c.createNonce(2n ** 64n + 5n))
  expect(c.createNonce(0n)).not.toBe(c.createNonce(2n ** 64n))
}

/** Values that already fit in 64 bits keep their previous encoding. */
export const testCreateNonceIsUnchangedBelowU64 = () => {
  const c = cipher()
  expect(c.createNonce(0)).toBe('0x000000000000000000000000')
  expect(c.createNonce(42)).toBe('0x00000000000000000000002a')
  expect(c.createNonce(2n ** 64n - 1n)).toBe('0x00000000ffffffffffffffff')
}

/** Out-of-range values must be rejected, not silently truncated. */
export const testCreateNonceRejectsOversizedValues = () => {
  expect(() => cipher().createNonce(2n ** 96n)).toThrow('96 bits')
  expect(() => cipher().createNonce(2n ** 128n)).toThrow('96 bits')
}

/** Negative values must be rejected, matching the Python client's OverflowError. */
export const testCreateNonceRejectsNegativeValues = () => {
  expect(() => cipher().createNonce(-1)).toThrow('negative')
}

/**
 * `encrypt` takes the same numeric nonce path, so a truncating conversion
 * would encrypt under a different nonce than the one the node records.
 */
export const testEncryptUsesFullWidthNumericNonce = async () => {
  const plaintext = '0xdeadbeef' as Hex
  const c = cipher()

  const viaNumber = await c.encrypt(plaintext, 2n ** 64n + 5n)
  const viaHex = await c.encrypt(plaintext, '0x000000010000000000000005' as Hex)

  expect(viaNumber).toBe(viaHex)

  // And it must differ from the value the truncating conversion produced.
  const truncated = await c.encrypt(
    plaintext,
    '0x000000000000000000000005' as Hex
  )
  expect(viaNumber).not.toBe(truncated)
}
