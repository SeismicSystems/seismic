import { expect } from 'bun:test'
import type { Hex } from 'viem'
import { parseEther } from 'viem/utils'

/**
 * Inline the parseMinBalance logic from @sviem/faucet.ts since it's not
 * exported. These tests verify the balance threshold parsing without
 * needing a running faucet service.
 */
const DEFAULT_MIN_BALANCE_WEI = parseEther('0.5')

const parseMinBalance = (
  minBalanceWei?: bigint | number,
  minBalanceEther?: bigint | number
): bigint => {
  if (minBalanceWei && minBalanceEther) {
    if (BigInt(minBalanceWei) !== parseEther(minBalanceEther.toString())) {
      // warn, but use minBalanceWei
    }
  }
  if (minBalanceWei) {
    return BigInt(minBalanceWei)
  }
  if (minBalanceEther) {
    return parseEther(minBalanceEther.toString())
  }
  return DEFAULT_MIN_BALANCE_WEI
}

// --- parseMinBalance tests ---

export const testParseMinBalanceDefaultsToHalfEther = () => {
  const result = parseMinBalance()
  expect(result).toBe(parseEther('0.5'))
}

export const testParseMinBalanceUsesWeiWhenProvided = () => {
  const result = parseMinBalance(1000n)
  expect(result).toBe(1000n)
}

export const testParseMinBalanceUsesEtherWhenProvided = () => {
  const result = parseMinBalance(undefined, 2)
  expect(result).toBe(parseEther('2'))
}

export const testParseMinBalancePrefersWeiOverEther = () => {
  // When both are provided, minBalanceWei wins
  const result = parseMinBalance(999n, 1)
  expect(result).toBe(999n)
}

export const testParseMinBalanceHandlesNumericWei = () => {
  const result = parseMinBalance(5000)
  expect(result).toBe(5000n)
}

// --- faucet response parsing tests ---

/**
 * Inline the hash extraction logic from checkFaucet to test it
 * without a running faucet.
 */
const extractFaucetHash = (
  msg: string
): { valid: true; hash: Hex } | { valid: false } => {
  if (msg.startsWith('Txhash: ')) {
    const hash = msg.slice(8)
    if (hash.startsWith('0x') && hash.length === 66) {
      return { valid: true, hash: hash as Hex }
    }
  }
  return { valid: false }
}

export const testFaucetHashExtractionValid = () => {
  const hash =
    '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
  const result = extractFaucetHash(`Txhash: ${hash}`)
  expect(result.valid).toBe(true)
  if (result.valid) {
    expect(result.hash).toBe(hash)
  }
}

export const testFaucetHashExtractionInvalidLength = () => {
  const result = extractFaucetHash('Txhash: 0xshort')
  expect(result.valid).toBe(false)
}

export const testFaucetHashExtractionNoPrefix = () => {
  const result = extractFaucetHash('Some other message')
  expect(result.valid).toBe(false)
}

export const testFaucetHashExtractionMissingHexPrefix = () => {
  // 64 hex chars but no 0x prefix → not valid
  const result = extractFaucetHash(
    'Txhash: abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
  )
  expect(result.valid).toBe(false)
}
