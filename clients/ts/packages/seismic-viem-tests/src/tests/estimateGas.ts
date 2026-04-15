/**
 * Integration tests for signed eth_estimateGas on shielded transactions.
 *
 * Mirrors the seismic-web3 (Python) and seismic-alloy / seismic-foundry tests:
 *   - Signed estimate gas succeeds and returns a reasonable value.
 *   - Transactions using estimated gas execute successfully.
 *   - Explicit gas bypasses estimation.
 */
import { expect } from 'bun:test'
import {
  buildTxSeismicMetadata,
  createShieldedPublicClient,
  createShieldedWalletClient,
  estimateShieldedGas,
  getPlaintextCalldata,
  getShieldedContract,
} from 'seismic-viem'
import type { Account, Chain, Hex } from 'viem'
import { http } from 'viem'

import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { seismicCounterBytecode } from '@sviem-tests/tests/contract/bytecode.ts'

export type EstimateGasTestArgs = {
  chain: Chain
  url: string
  account: Account
}

const deployCounter = async (chain: Chain, url: string, account: Account) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
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

export const testEstimateGasReturnsReasonableValue = async ({
  chain,
  url,
  account,
}: EstimateGasTestArgs) => {
  const { walletClient, address } = await deployCounter(chain, url, account)

  // @ts-expect-error: seismicCounterAbi functionName type narrowing
  const plaintextCalldata: Hex = getPlaintextCalldata({
    abi: seismicCounterAbi,
    functionName: 'setNumber',
    args: [42n],
  })

  const metadata = await buildTxSeismicMetadata(walletClient, {
    account: walletClient.account,
    nonce: await walletClient.getTransactionCount({
      address: walletClient.account.address,
    }),
    to: address,
    signedRead: false,
  })
  const encryptedData = await walletClient.encrypt(plaintextCalldata, metadata)
  const gasPrice = await walletClient.getGasPrice()

  const gas = await estimateShieldedGas(walletClient, {
    encryptedData,
    metadata,
    gasPrice,
  })

  expect(gas).toBeGreaterThan(21_000n)
  expect(gas).toBeLessThan(30_000_000n)
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
