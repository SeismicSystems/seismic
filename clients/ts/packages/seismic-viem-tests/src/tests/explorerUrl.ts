import { expect } from 'bun:test'
import {
  addressExplorerUrl,
  blockExplorerUrl,
  getExplorerUrl,
  sanvil,
  tokenExplorerUrl,
  txExplorerUrl,
} from 'seismic-viem'
import { defineChain } from 'viem'

const TEST_EXPLORER_URL = 'https://explorer.test'

const chainWithExplorer = defineChain({
  id: 99999,
  name: 'TestChain',
  nativeCurrency: { decimals: 18, name: 'Ether', symbol: 'ETH' },
  rpcUrls: { default: { http: ['http://unused.invalid'] } },
  blockExplorers: {
    default: { name: 'TestExplorer', url: TEST_EXPLORER_URL },
  },
})

const chainWithoutExplorer = defineChain({
  id: 99998,
  name: 'NoExplorerChain',
  nativeCurrency: { decimals: 18, name: 'Ether', symbol: 'ETH' },
  rpcUrls: { default: { http: ['http://unused.invalid'] } },
})

const SANVIL_TEST_ADDRESS = '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266'

const SAMPLE_TX_HASH =
  '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'

const SAMPLE_BLOCK_NUMBER = 42

export const testGetExplorerUrlReturnsNullForUndefinedChain = () => {
  const result = getExplorerUrl(undefined)
  expect(result).toBeNull()
}

export const testGetExplorerUrlReturnsNullForChainWithoutExplorer = () => {
  const result = getExplorerUrl(chainWithoutExplorer)
  expect(result).toBeNull()
}

export const testGetExplorerUrlReturnsBaseUrlWithoutOptions = () => {
  const result = getExplorerUrl(chainWithExplorer)
  expect(result).toBe(TEST_EXPLORER_URL)
}

export const testGetExplorerUrlBuildsItemUrl = () => {
  const result = getExplorerUrl(chainWithExplorer, {
    item: 'tx',
    id: SAMPLE_TX_HASH,
  })
  expect(result).toBe(`${TEST_EXPLORER_URL}/tx/${SAMPLE_TX_HASH}`)
}

export const testGetExplorerUrlBuildsItemUrlWithTab = () => {
  const result = getExplorerUrl(chainWithExplorer, {
    item: 'tx',
    id: SAMPLE_TX_HASH,
    tab: 'logs',
  })
  expect(result).toBe(`${TEST_EXPLORER_URL}/tx/${SAMPLE_TX_HASH}?tab=logs`)
}

export const testTxExplorerUrlBuildsCorrectUrl = () => {
  const result = txExplorerUrl({
    chain: chainWithExplorer,
    txHash: SAMPLE_TX_HASH,
  })
  expect(result).toBe(`${TEST_EXPLORER_URL}/tx/${SAMPLE_TX_HASH}`)
}

export const testTxExplorerUrlWithTab = () => {
  const result = txExplorerUrl({
    chain: chainWithExplorer,
    txHash: SAMPLE_TX_HASH,
    tab: 'internal',
  })
  expect(result).toBe(`${TEST_EXPLORER_URL}/tx/${SAMPLE_TX_HASH}?tab=internal`)
}

export const testTxExplorerUrlReturnsNullWithoutChain = () => {
  const result = txExplorerUrl({ txHash: SAMPLE_TX_HASH })
  expect(result).toBeNull()
}

export const testAddressExplorerUrlBuildsCorrectUrl = () => {
  const result = addressExplorerUrl({
    chain: chainWithExplorer,
    address: SANVIL_TEST_ADDRESS,
  })
  expect(result).toBe(`${TEST_EXPLORER_URL}/address/${SANVIL_TEST_ADDRESS}`)
}

export const testAddressExplorerUrlWithTab = () => {
  const result = addressExplorerUrl({
    chain: chainWithExplorer,
    address: SANVIL_TEST_ADDRESS,
    tab: 'tokens',
  })
  expect(result).toBe(
    `${TEST_EXPLORER_URL}/address/${SANVIL_TEST_ADDRESS}?tab=tokens`
  )
}

export const testAddressExplorerUrlReturnsNullWithoutExplorer = () => {
  const result = addressExplorerUrl({
    chain: chainWithoutExplorer,
    address: SANVIL_TEST_ADDRESS,
  })
  expect(result).toBeNull()
}

export const testBlockExplorerUrlBuildsCorrectUrl = () => {
  const result = blockExplorerUrl({
    chain: chainWithExplorer,
    blockNumber: SAMPLE_BLOCK_NUMBER,
  })
  expect(result).toBe(`${TEST_EXPLORER_URL}/block/${SAMPLE_BLOCK_NUMBER}`)
}

export const testBlockExplorerUrlWithTab = () => {
  const result = blockExplorerUrl({
    chain: chainWithExplorer,
    blockNumber: SAMPLE_BLOCK_NUMBER,
    tab: 'txs',
  })
  expect(result).toBe(
    `${TEST_EXPLORER_URL}/block/${SAMPLE_BLOCK_NUMBER}?tab=txs`
  )
}

export const testTokenExplorerUrlBuildsCorrectUrl = () => {
  const result = tokenExplorerUrl({
    chain: chainWithExplorer,
    address: SANVIL_TEST_ADDRESS,
  })
  expect(result).toBe(`${TEST_EXPLORER_URL}/token/${SANVIL_TEST_ADDRESS}`)
}

export const testTokenExplorerUrlWithTab = () => {
  const result = tokenExplorerUrl({
    chain: chainWithExplorer,
    address: SANVIL_TEST_ADDRESS,
    tab: 'holders',
  })
  expect(result).toBe(
    `${TEST_EXPLORER_URL}/token/${SANVIL_TEST_ADDRESS}?tab=holders`
  )
}

export const testSanvilHasNoExplorer = () => {
  const result = txExplorerUrl({ chain: sanvil, txHash: SAMPLE_TX_HASH })
  expect(result).toBeNull()
}
