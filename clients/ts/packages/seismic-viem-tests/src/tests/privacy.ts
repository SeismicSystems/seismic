import { expect } from 'bun:test'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
  getShieldedContract,
} from 'seismic-viem'
import type { Account, Chain } from 'viem'
import { http } from 'viem'

import { STORAGE_SLOT_ZERO } from '@sviem-tests/constants.ts'
import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { deploySeismicCounter } from '@sviem-tests/tests/contract/deploy.ts'

type PrivacyTestArgs = {
  chain: Chain
  url: string
  account: Account
}

const COUNTER_VALUE_ENCRYPTED = 42n
const COUNTER_VALUE_STORAGE = 99n

export const testSeismicTxCalldataIsEncrypted = async ({
  chain,
  url,
  account,
}: PrivacyTestArgs) => {
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

export const testGetStorageAtBlockedForContract = async ({
  chain,
  url,
  account,
}: PrivacyTestArgs) => {
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
  const setTx = await contract.write.setNumber([COUNTER_VALUE_STORAGE])
  await publicClient.waitForTransactionReceipt({ hash: setTx })

  await expect(
    publicClient.getStorageAt({
      address,
      slot: STORAGE_SLOT_ZERO,
    })
  ).rejects.toThrow('Cannot call getStorageAt with a shielded public client')
}
