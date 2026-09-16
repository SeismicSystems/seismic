import { Hex } from 'viem'

import {
  encodeSeismicMetadataAsAAD,
  encodeSeismicResponseAAD,
} from '@sviem/crypto/aead.ts'
import { AesGcmCrypto, splitResponseIv } from '@sviem/crypto/aes.ts'
import type { TxSeismicMetadata } from '@sviem/tx/metadata.ts'

export type EncryptionActions = {
  getEncryption: () => Hex
  getResponseEncryption: () => Hex
  getEncryptionPublicKey: () => Hex
  encrypt: (
    plaintext: Hex | undefined,
    metadata: TxSeismicMetadata
  ) => Promise<Hex>
  decrypt: (
    ciphertext: Hex | undefined,
    metadata: TxSeismicMetadata
  ) => Promise<Hex>
}

export const encryptionActions = (
  requestEncryption: Hex,
  responseEncryption: Hex,
  encryptionPublicKey: Hex
): EncryptionActions => {
  return {
    getEncryption: () => requestEncryption,
    getResponseEncryption: () => responseEncryption,
    getEncryptionPublicKey: () => encryptionPublicKey,
    encrypt: async (
      plaintext: Hex | undefined,
      metadata: TxSeismicMetadata
    ) => {
      const aesCipher = new AesGcmCrypto(requestEncryption)
      const aad = encodeSeismicMetadataAsAAD(metadata)
      const ciphertext = await aesCipher.encrypt(
        plaintext,
        metadata.seismicElements.encryptionNonce,
        aad
      )
      return ciphertext
    },
    decrypt: async (
      ciphertext: Hex | undefined,
      metadata: TxSeismicMetadata
    ) => {
      const { version, iv, body } = splitResponseIv(ciphertext ?? '0x')
      const aesCipher = new AesGcmCrypto(responseEncryption)
      const aad = encodeSeismicResponseAAD(metadata, version)
      return await aesCipher.decrypt(body, iv, aad)
    },
  }
}
