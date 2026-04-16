import { expect } from 'bun:test'
import type { Hex } from 'viem'
import { parseEther } from 'viem/utils'

import { SAMPLE_TX_HASH } from '@sviem-tests/constants.ts'

/**
 * Inline copy of parseMinBalance from @sviem/faucet.ts
 * since it's not exported.
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
  const result = parseMinBalance(999n, 1)
  expect(result).toBe(999n)
}

export const testParseMinBalanceHandlesNumericWei = () => {
  const result = parseMinBalance(5000)
  expect(result).toBe(5000n)
}

/**
 * Inline copy of hash extraction logic from checkFaucet
 * since it's not exported.
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
  const result = extractFaucetHash(`Txhash: ${SAMPLE_TX_HASH}`)
  expect(result.valid).toBe(true)
  if (result.valid) {
    expect(result.hash).toBe(SAMPLE_TX_HASH)
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
  const hashWithout0x = SAMPLE_TX_HASH.slice(2)
  const result = extractFaucetHash(`Txhash: ${hashWithout0x}`)
  expect(result.valid).toBe(false)
}
