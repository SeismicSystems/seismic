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

const NUM_SEQUENTIAL_WRITES = 3

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

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address: contractAddress,
    client: walletClient,
  })

  const hashes = []
  for (let i = 0; i < NUM_SEQUENTIAL_WRITES; i++) {
    const hash = await contract.write.increment()
    hashes.push(hash)
  }

  const receipts = await Promise.all(
    hashes.map((hash) =>
      publicClient.waitForTransactionReceipt({
        hash: hash as `0x${string}`,
        timeout: 30_000,
      })
    )
  )

  for (const receipt of receipts) {
    expect(receipt.status).toBe('success')
  }

  const uniqueHashes = new Set(hashes)
  expect(uniqueHashes.size).toBe(NUM_SEQUENTIAL_WRITES)

  // 0 + 3 increments = 3, which is odd
  const isOdd = await contract.tread.isOdd()
  expect(isOdd).toBe(true)
}

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

  const setTx = await contract.write.setNumber([5n])
  await publicClient.waitForTransactionReceipt({ hash: setTx })

  const [signedResult, transparentResult, smartResult] = await Promise.all([
    contract.sread.isOdd(),
    contract.tread.isOdd(),
    contract.read.isOdd(),
  ])

  expect(signedResult).toBe(true)
  expect(transparentResult).toBe(true)
  expect(smartResult).toBe(true)
}
