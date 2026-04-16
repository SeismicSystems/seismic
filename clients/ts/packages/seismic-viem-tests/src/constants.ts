import type { Hex } from 'viem'

// First Anvil account private key
export const TEST_ACCOUNT_PRIVATE_KEY =
  '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80' as Hex

// Encryption keypair derived from TEST_ACCOUNT-independent secp256k1 key.
// The SK generates the PK via compressPublicKey(privateKeyToAccount(sk).publicKey).
export const ENCRYPTION_SK =
  '0x311d54d3bf8359c70827122a44a7b4458733adce3c51c6b59d9acfce85e07505' as Hex
export const ENCRYPTION_PK =
  '0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0' as Hex

// Values from the encoding.ts golden-output test
export const ENCODING_TEST_TO =
  '0xd3e8763675e4c425df46cc3b5c0f6cbdac396046' as const
export const ENCODING_TEST_ENCRYPTION_NONCE =
  '0x46a2b6020bba77fcb1e676a6' as Hex
export const ENCODING_TEST_RECENT_BLOCK_HASH =
  '0x934207181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f90' as Hex

// Sanvil (local anvil) chain ID
export const SANVIL_CHAIN_ID = 31337
