import { type Hex, bytesToHex, hexToBytes } from 'viem'

import { gcm } from '@noble/ciphers/webcrypto'
import { secp256k1 } from '@noble/curves/secp256k1'
import { hkdf } from '@noble/hashes/hkdf'
import { sha256 } from '@noble/hashes/sha256'
import type { EncryptionNonce } from '@sviem/crypto/nonce.ts'

export class AesGcmCrypto {
  private readonly NONCE_LENGTH = 12 // 96 bits is the recommended nonce length for GCM
  private readonly U64_SIZE = 8 // Size of u64 in bytes

  constructor(private readonly key: Hex) {
    const keyBuffer = hexToBytes(key)
    if (keyBuffer.length !== 32) {
      throw new Error('Key must be 32 bytes (256 bits)')
    }
  }

  /**
   * Creates a nonce from a u64 number, matching Rust's implementation
   * @param num - The number to convert (will be treated as u64)
   */
  private numberToNonce(num: bigint | number): Uint8Array {
    let value = BigInt(num)

    // Create a buffer for the full nonce (12 bytes)
    const nonceBuffer = new Uint8Array(this.NONCE_LENGTH)
    // Write the u64 value in big-endian format to the last 8 bytes
    for (
      let i = this.NONCE_LENGTH - 1;
      i >= this.NONCE_LENGTH - this.U64_SIZE;
      i--
    ) {
      nonceBuffer[i] = Number(value & 0xffn)
      value = value >> 8n
    }

    // First 4 bytes remain as zeros
    return nonceBuffer
  }

  /**
   * Validates and converts a hex nonce to buffer
   * @param nonce - The nonce in hex format
   */
  private validateAndConvertNonce(nonce: Hex): Uint8Array {
    const nonceBuffer = hexToBytes(nonce)
    if (nonceBuffer.length !== this.NONCE_LENGTH) {
      throw new Error('Nonce must be 12 bytes')
    }
    return nonceBuffer
  }

  /**
   * Creates a nonce from a number in a way compatible with the Rust backend
   */
  public createNonce(num: number | bigint): Hex {
    return bytesToHex(this.numberToNonce(num))
  }

  /**
   * Encrypts data using either a number-based nonce or hex nonce
   */
  public async encrypt(
    plaintext: Hex | null | undefined,
    nonce: EncryptionNonce,
    aad?: Uint8Array
  ): Promise<Hex> {
    if (!plaintext || plaintext === '0x') {
      return '0x'
    }

    // Handle the nonce based on its type
    const nonceBuffer = new Uint8Array(
      typeof nonce === 'string'
        ? this.validateAndConvertNonce(nonce as Hex)
        : this.numberToNonce(nonce)
    )

    const key = hexToBytes(this.key)
    const plaintextBytes = hexToBytes(plaintext)
    const ciphertextBytes = await gcm(key, nonceBuffer, aad).encrypt(
      plaintextBytes
    )
    return bytesToHex(ciphertextBytes)
  }

  /**
   * Decrypts data using either a number-based nonce or hex nonce
   * NOTE: not tested or called in any real way
   */
  public async decrypt(
    ciphertext: Hex | null | undefined,
    nonce: EncryptionNonce,
    aad?: Uint8Array
  ): Promise<Hex> {
    if (!ciphertext || ciphertext === '0x') {
      return '0x'
    }

    // Handle the nonce based on its type
    const nonceBuffer = new Uint8Array(
      typeof nonce === 'string'
        ? this.validateAndConvertNonce(nonce as Hex)
        : this.numberToNonce(nonce)
    )

    const key = hexToBytes(this.key)
    const ciphertextBytes = hexToBytes(ciphertext)
    const plaintextBytes = await gcm(key, nonceBuffer, aad).decrypt(
      ciphertextBytes
    )
    return bytesToHex(plaintextBytes)
  }
}

type AesInputKeys = { privateKey: Hex; networkPublicKey: string }

export const sharedSecretPoint = ({
  privateKey,
  networkPublicKey,
}: AesInputKeys): Uint8Array => {
  const privateKeyHex = privateKey.startsWith('0x')
    ? privateKey.slice(2)
    : privateKey
  // return non-compressed point, stripping prefix (which will be 4)
  return secp256k1
    .getSharedSecret(privateKeyHex, networkPublicKey, false)
    .slice(1)
}

export const sharedKeyFromPoint = (sharedSecret: Uint8Array): string => {
  // Get the last byte (index 63) and apply the bitwise operations
  const version = (sharedSecret[63] & 0x01) | 0x02
  // Extra (non-standard) stuff that Rust does
  // Create a buffer for version and x-coordinate
  const finalSecret = sha256
    .create()
    .update(new Uint8Array([version]))
    .update(sharedSecret.slice(0, 32))
    .digest()
  return bytesToHex(finalSecret).slice(2)
}

export const generateSharedKey = (inputs: AesInputKeys): string => {
  const sharedSecret = sharedSecretPoint(inputs)
  return sharedKeyFromPoint(sharedSecret)
}

/**
 * Domain-separation labels for AES keys derived from an ECDH shared secret.
 * HKDF info labels must match the Rust `AesKeyDomain` enum (in
 * seismic-crypto); keys from different domains are independent.
 */
export const AesKeyDomain = {
  /** Client-to-TEE transaction I/O: encrypted calldata, signed-read requests */
  TxRequest: 'tx-request',
  /** TEE-to-client transaction I/O: signed-read return data */
  TxResponse: 'tx-response',
  /** The on-chain ECDH precompile's derivation (consensus-frozen label) */
  EcdhPrecompile: 'ecdh-precompile',
} as const

export type AesKeyDomain = (typeof AesKeyDomain)[keyof typeof AesKeyDomain]

const DOMAIN_HKDF_INFO: Record<AesKeyDomain, string> = {
  // Request keeps the original 'aes-gcm key' label for backward compat;
  // only the response moved to a new one. See the Rust AesKeyDomain enum.
  [AesKeyDomain.TxRequest]: 'aes-gcm key',
  [AesKeyDomain.TxResponse]: 'seismic/response/aes-256-gcm/v1',
  [AesKeyDomain.EcdhPrecompile]: 'aes-gcm key',
}

/**
 * Derive a domain-separated AES-256 key from an ECDH shared secret using
 * HKDF-SHA256.
 */
export const deriveAesKey = (
  sharedSecret: string,
  domain: AesKeyDomain
): Hex => {
  const derivedKey = hkdf(
    sha256,
    hexToBytes(`0x${sharedSecret}`),
    new Uint8Array(0),
    new TextEncoder().encode(DOMAIN_HKDF_INFO[domain]),
    32
  )
  return bytesToHex(derivedKey)
}

/** Run ECDH and derive a domain-separated AES-256 key. */
export const generateAesKey = (
  aesKeys: AesInputKeys,
  domain: AesKeyDomain
): Hex => {
  const sharedSecret = generateSharedKey(aesKeys)
  return deriveAesKey(sharedSecret, domain)
}
