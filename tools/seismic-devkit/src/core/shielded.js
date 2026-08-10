/**
 * Seismic Shielded State & Encryption Primitives
 */

import crypto from 'crypto';

export class ShieldedEngine {
  constructor() {
    this.keypair = this.generateEphemeralKeypair();
  }

  generateEphemeralKeypair() {
    return {
      publicKey: '0x' + crypto.randomBytes(32).toString('hex'),
      privateKey: '0x' + crypto.randomBytes(32).toString('hex'),
    };
  }

  /**
   * Encrypt shielded calldata for Seismic TEE execution
   */
  encryptShieldedCalldata(rawCalldata, userSecret = null) {
    const iv = crypto.randomBytes(16);
    const key = crypto.createHash('sha256').update(userSecret || this.keypair.privateKey).digest();
    const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);

    let encrypted = cipher.update(Buffer.from(rawCalldata.replace('0x', ''), 'hex'));
    encrypted = Buffer.concat([encrypted, cipher.final()]);
    const tag = cipher.getAuthTag();

    const packed = Buffer.concat([iv, tag, encrypted]);
    return '0x' + packed.toString('hex');
  }

  /**
   * Simulate TEE Enclave Decryption
   */
  decryptShieldedCalldata(packedHex, userSecret = null) {
    const packed = Buffer.from(packedHex.replace('0x', ''), 'hex');
    const iv = packed.subarray(0, 16);
    const tag = packed.subarray(16, 32);
    const encrypted = packed.subarray(32);

    const key = crypto.createHash('sha256').update(userSecret || this.keypair.privateKey).digest();
    const decipher = crypto.createDecipheriv('aes-256-gcm', key, iv);
    decipher.setAuthTag(tag);

    let decrypted = decipher.update(encrypted);
    decrypted = Buffer.concat([decrypted, decipher.final()]);
    return '0x' + decrypted.toString('hex');
  }

  /**
   * Format shielded types
   */
  formatShieldedType(typeName, rawValue) {
    switch (typeName) {
      case 'suint8':
      case 'suint32':
      case 'suint256':
        return {
          type: typeName,
          value: rawValue,
          shieldedHex: '0x' + BigInt(rawValue).toString(16).padStart(64, '0'),
          isShielded: true,
        };
      case 'sbool':
        return {
          type: 'sbool',
          value: Boolean(rawValue),
          shieldedHex: Boolean(rawValue) ? '0x' + '1'.padStart(64, '0') : '0x' + '0'.padStart(64, '0'),
          isShielded: true,
        };
      case 'saddress':
        return {
          type: 'saddress',
          value: rawValue,
          shieldedHex: '0x' + rawValue.replace('0x', '').toLowerCase().padStart(64, '0'),
          isShielded: true,
        };
      default:
        throw new Error(`Unsupported shielded type: ${typeName}`);
    }
  }
}

export const defaultShieldedEngine = new ShieldedEngine();
