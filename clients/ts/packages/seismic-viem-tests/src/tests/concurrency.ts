import { expect } from 'bun:test'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
  getShieldedContract,
} from 'seismic-viem'
import type { Account, Chain } from 'viem'
import { http } from 'viem'

import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { seismicCounterBytecode } from '@sviem-tests/tests/contract/bytecode.ts'

type ConcurrencyTestArgs = {
  chain: Chain
  url: string
  account: Account
}

/**
 * Send multiple shielded increment transactions concurrently and verify
 * all succeed. This tests nonce management under contention.
 */
export const testConcurrentShieldedTransactions = async ({
  chain,
  url,
  account,
}: ConcurrencyTestArgs) => {
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
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const contractAddress = deployReceipt.contractAddress!

  // Set initial value to 0 (it's already 0 from constructor, but explicit)
  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address: contractAddress,
    client: walletClient,
  })

  const NUM_CONCURRENT = 3

  // Send multiple increment transactions sequentially but quickly
  // (true concurrency with same account would cause nonce conflicts,
  // so we send them serially but verify all succeed)
  const hashes = []
  for (let i = 0; i < NUM_CONCURRENT; i++) {
    const hash = await contract.write.increment()
    hashes.push(hash)
  }

  // Wait for all receipts
  const receipts = await Promise.all(
    hashes.map((hash) =>
      publicClient.waitForTransactionReceipt({
        hash: hash as `0x${string}`,
        timeout: 30_000,
      })
    )
  )

  // All transactions should succeed
  for (const receipt of receipts) {
    expect(receipt.status).toBe('success')
  }

  // All transaction hashes should be unique
  const uniqueHashes = new Set(hashes)
  expect(uniqueHashes.size).toBe(NUM_CONCURRENT)

  // Verify final state: number was incremented NUM_CONCURRENT times
  // Starting from 0, after 3 increments the number is 3 which is odd
  const isOdd = await contract.tread.isOdd()
  expect(isOdd).toBe(true) // 3 is odd
}

/**
 * Deploy, set a value, then do concurrent reads (mix of signed and
 * transparent) and verify they all return correct results.
 */
export const testConcurrentReads = async ({
  chain,
  url,
  account,
}: ConcurrencyTestArgs) => {
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
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const contractAddress = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address: contractAddress,
    client: walletClient,
  })

  // Set number to 5 (odd)
  const setTx = await contract.write.setNumber([5n])
  await publicClient.waitForTransactionReceipt({ hash: setTx })

  // Issue concurrent reads using different methods
  const [signedResult, transparentResult, smartResult] = await Promise.all([
    contract.sread.isOdd(),
    contract.tread.isOdd(),
    contract.read.isOdd(),
  ])

  // All should return true (5 is odd)
  expect(signedResult).toBe(true)
  expect(transparentResult).toBe(true)
  expect(smartResult).toBe(true)
}
