import { expect } from 'bun:test'
import { getShieldedContract } from 'seismic-viem'
import type { Account, Chain } from 'viem'

import { httpPublicClient, httpWalletClient } from '@sviem-tests/clients.ts'
import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { deploySeismicCounter } from '@sviem-tests/tests/contract/deploy.ts'

type PrivacyTestArgs = {
  chain: Chain
  url: string
  account: Account
}

const COUNTER_VALUE_ENCRYPTED = 42n

export const testSeismicTxCalldataIsEncrypted = async ({
  chain,
  url,
  account,
}: PrivacyTestArgs) => {
  const publicClient = httpPublicClient({ chain, url })
  const walletClient = await httpWalletClient({ chain, url, account })
  const address = await deploySeismicCounter({ publicClient, walletClient })

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  const { txHash, plaintextTx } = await contract.dwrite.setNumber([
    COUNTER_VALUE_ENCRYPTED,
  ])
  await publicClient.waitForTransactionReceipt({ hash: txHash })

  const onChainTx = await publicClient.getTransaction({ hash: txHash })
  const plaintextData = plaintextTx.data
  expect(plaintextData).toBeDefined()

  // The on-chain calldata must not equal the plaintext, and must not
  // even leak the 4-byte function selector — that would expose which
  // function was called.
  const functionSelector = plaintextData!.slice(0, 10)
  expect(onChainTx.input).not.toBe(plaintextData!)
  expect(onChainTx.input.startsWith(functionSelector)).toBe(false)
}
