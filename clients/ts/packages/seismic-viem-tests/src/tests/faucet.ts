import { expect } from 'bun:test'
import { parseFaucetResponseHash, parseMinBalance } from 'seismic-viem'
import { parseEther } from 'viem/utils'

import { SAMPLE_TX_HASH } from '@sviem-tests/constants.ts'

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

export const testParseFaucetResponseHashValid = () => {
  const hash = parseFaucetResponseHash(`Txhash: ${SAMPLE_TX_HASH}`)
  expect(hash).toBe(SAMPLE_TX_HASH)
}

export const testParseFaucetResponseHashNoPrefix = () => {
  const hash = parseFaucetResponseHash('Some other message')
  expect(hash).toBeNull()
}

export const testParseFaucetResponseHashThrowsOnInvalidLength = () => {
  expect(() => parseFaucetResponseHash('Txhash: 0xshort')).toThrow(
    'Invalid hash from faucet claim'
  )
}

export const testParseFaucetResponseHashThrowsOnMissingHexPrefix = () => {
  const hashWithout0x = SAMPLE_TX_HASH.slice(2)
  expect(() => parseFaucetResponseHash(`Txhash: ${hashWithout0x}`)).toThrow(
    'Invalid hash from faucet claim'
  )
}
