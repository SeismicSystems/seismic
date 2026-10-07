/**
 * Integration tests for signed eth_estimateGas on shielded transactions.
 *
 * Mirrors the seismic-web3 (Python) and seismic-alloy / seismic-foundry tests:
 *   - Signed estimate gas succeeds and returns a reasonable value.
 *   - Transactions using estimated gas execute successfully.
 *   - Explicit gas bypasses estimation.
 */
import { expect } from 'bun:test'
import { getShieldedContract } from 'seismic-viem'
import type { Account, Chain } from 'viem'
import { hexToBytes } from 'viem'

import { httpPublicClient, httpWalletClient } from '@sviem-tests/clients.ts'
import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { seismicCounterBytecode } from '@sviem-tests/tests/contract/bytecode.ts'

export type EstimateGasTestArgs = {
  chain: Chain
  url: string
  account: Account
}

const deployCounter = async (chain: Chain, url: string, account: Account) => {
  const publicClient = httpPublicClient({ chain, url })
  const walletClient = await httpWalletClient({ chain, url, account })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`
  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const receipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = receipt.contractAddress!
  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })
  return { publicClient, walletClient, contract, address }
}

export const testCheapWriteWithEstimatedGas = async ({
  chain,
  url,
  account,
}: EstimateGasTestArgs) => {
  const publicClient = httpPublicClient({ chain, url })
  const walletClient = await httpWalletClient({ chain, url, account })
  for (const gasPayment of [{ type: 'auto' }, { type: 'native' }] as const) {
    // Identity executes cheaply; no deployment/storage cost hides the floor.
    const hash = await walletClient.sendShieldedTransaction({
      to: '0x0000000000000000000000000000000000000004',
      data: '0x313ce567',
      gasPayment,
    })
    const receipt = await publicClient.waitForTransactionReceipt({
      hash,
      timeout: 30_000,
    })
    expect(receipt.status).toBe('success')
    const tx = await publicClient.getTransaction({ hash })
    const ciphertext = hexToBytes(tx.input)
    expect(ciphertext.length).toBe(20)
    const tokens = ciphertext.reduce(
      (total, byte) => total + (byte === 0 ? 1n : 4n),
      0n
    )
    expect(tx.gas).toBeGreaterThanOrEqual(21_000n + 10n * tokens)
    expect(tx.gas).toBeLessThan(100_000n)
  }
}

export const testWriteWithoutExplicitGasSucceeds = async ({
  chain,
  url,
  account,
}: EstimateGasTestArgs) => {
  const { publicClient, contract } = await deployCounter(chain, url, account)

  const txHash = await contract.write.setNumber([42n])
  const receipt = await publicClient.waitForTransactionReceipt({
    hash: txHash,
    timeout: 30_000,
  })
  expect(receipt.status).toBe('success')
}

export const testWriteUsesEstimatedGasNot30M = async ({
  chain,
  url,
  account,
}: EstimateGasTestArgs) => {
  const { publicClient, walletClient, contract } = await deployCounter(
    chain,
    url,
    account
  )

  const { txHash } = await contract.dwrite.setNumber([77n])
  const receipt = await publicClient.waitForTransactionReceipt({
    hash: txHash,
    timeout: 30_000,
  })
  expect(receipt.status).toBe('success')

  const tx = await publicClient.getTransaction({ hash: txHash })
  expect(tx.gas).toBeLessThan(30_000_000n)
  expect(tx.gas).toBeGreaterThan(21_000n)
}

export const testWriteWithExplicitGasSkipsEstimation = async ({
  chain,
  url,
  account,
}: EstimateGasTestArgs) => {
  const { publicClient, contract } = await deployCounter(chain, url, account)

  const explicitGas = 5_000_000n
  const { txHash } = await contract.dwrite.setNumber([55n], {
    gas: explicitGas,
  })
  const receipt = await publicClient.waitForTransactionReceipt({
    hash: txHash,
    timeout: 30_000,
  })
  expect(receipt.status).toBe('success')

  const tx = await publicClient.getTransaction({ hash: txHash })
  expect(tx.gas).toBe(explicitGas)
}

export const testLifecycleWithEstimatedGas = async ({
  chain,
  url,
  account,
}: EstimateGasTestArgs) => {
  const { publicClient, contract } = await deployCounter(chain, url, account)

  const tx1 = await contract.write.setNumber([11n])
  await publicClient.waitForTransactionReceipt({ hash: tx1, timeout: 30_000 })
  const isOdd1 = await contract.read.isOdd()
  expect(isOdd1).toBe(true)

  const tx2 = await contract.write.increment()
  await publicClient.waitForTransactionReceipt({ hash: tx2, timeout: 30_000 })
  const isOdd2 = await contract.read.isOdd()
  expect(isOdd2).toBe(false)
}
