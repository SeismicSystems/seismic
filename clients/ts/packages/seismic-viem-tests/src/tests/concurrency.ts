import { expect } from 'bun:test'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
  getShieldedContract,
} from 'seismic-viem'
import type { Account, Chain, Hex } from 'viem'
import { http } from 'viem'

import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { deploySeismicCounter } from '@sviem-tests/tests/contract/deploy.ts'

type ConcurrencyTestArgs = {
  chain: Chain
  url: string
  account: Account
}

const NUM_SEQUENTIAL_WRITES = 3
const ODD_COUNTER_VALUE = 5n
const RECEIPT_TIMEOUT_MS = 30_000

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
  const address = await deploySeismicCounter({ publicClient, walletClient })

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  const hashes: Hex[] = []
  for (let i = 0; i < NUM_SEQUENTIAL_WRITES; i++) {
    const hash = await contract.write.increment()
    hashes.push(hash)
  }

  const receipts = await Promise.all(
    hashes.map((hash) =>
      publicClient.waitForTransactionReceipt({
        hash,
        timeout: RECEIPT_TIMEOUT_MS,
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
  const address = await deploySeismicCounter({ publicClient, walletClient })

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  const setTx = await contract.write.setNumber([ODD_COUNTER_VALUE])
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
