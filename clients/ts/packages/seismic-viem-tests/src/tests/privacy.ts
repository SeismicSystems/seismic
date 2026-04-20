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
import { seismicCounterBytecode } from '@sviem-tests/tests/contract/bytecode.ts'

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

  const { txHash, plaintextTx, shieldedTx } = await contract.dwrite.setNumber([
    COUNTER_VALUE_ENCRYPTED,
  ])
  await publicClient.waitForTransactionReceipt({ hash: txHash })

  const onChainTx = await publicClient.getTransaction({ hash: txHash })

  expect(plaintextTx.data).toBeDefined()
  expect(onChainTx.input).not.toBe(plaintextTx.data!)
  expect(onChainTx.input).toBe(shieldedTx.data!)
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
  const setTx = await contract.write.setNumber([COUNTER_VALUE_STORAGE])
  await publicClient.waitForTransactionReceipt({ hash: setTx })

  await expect(
    publicClient.getStorageAt({
      address: contractAddress,
      slot: STORAGE_SLOT_ZERO,
    })
  ).rejects.toThrow('Cannot call getStorageAt with a shielded public client')
}
