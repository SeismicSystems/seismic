/**
 * Shielded Engine Unit Tests
 */

import { defaultShieldedEngine } from '../src/core/shielded.js';

async function runShieldedTests() {
  console.log('Testing Shielded State Encryption & TEE Simulation...');

  // 1. Format shielded uint
  const fmt = defaultShieldedEngine.formatShieldedType('suint256', '5000');
  if (!fmt.isShielded || !fmt.shieldedHex.startsWith('0x')) {
    throw new Error('Shielded type formatting failed');
  }

  // 2. Encrypt & Decrypt Calldata
  const encrypted = defaultShieldedEngine.encryptShieldedCalldata(fmt.shieldedHex);
  const decrypted = defaultShieldedEngine.decryptShieldedCalldata(encrypted);

  if (decrypted.toLowerCase() !== fmt.shieldedHex.toLowerCase()) {
    throw new Error('Decryption did not match original shielded hex');
  }

  console.log('✅ Shielded Encryption & Decryption Passed!');
}

runShieldedTests().catch(e => {
  console.error('❌ Shielded Test Failed:', e);
  process.exit(1);
});
