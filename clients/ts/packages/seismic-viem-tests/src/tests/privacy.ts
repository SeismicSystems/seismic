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

type PrivacyTestArgs = {
  chain: Chain
  url: string
  account: Account
}

/**
 * Verify that a seismic transaction's on-chain input differs from the
 * plaintext calldata, confirming that calldata is encrypted.
 */
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

  // Use dwrite to get both plaintext and the on-chain tx hash
  const { txHash, plaintextTx, shieldedTx } = await contract.dwrite.setNumber([
    42n,
  ])
  await publicClient.waitForTransactionReceipt({ hash: txHash })

  // Fetch the on-chain transaction
  const onChainTx = await publicClient.getTransaction({ hash: txHash })

  // The plaintext calldata must exist
  expect(plaintextTx.data).toBeDefined()

  // The on-chain input (encrypted) must differ from the plaintext calldata
  expect(onChainTx.input).not.toBe(plaintextTx.data!)

  // The on-chain input should match the shielded (encrypted) data
  expect(onChainTx.input).toBe(shieldedTx.data!)
}

/**
 * Verify that getStorageAt is blocked on the shielded public client,
 * even for a deployed contract. This ensures private state can't be
 * read via raw storage access.
 */
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

  // First set a value so slot 0 isn't trivially zero
  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address: contractAddress,
    client: walletClient,
  })
  const setTx = await contract.write.setNumber([99n])
  await publicClient.waitForTransactionReceipt({ hash: setTx })

  // Attempting to read storage directly should throw
  await expect(
    publicClient.getStorageAt({
      address: contractAddress,
      slot: '0x0000000000000000000000000000000000000000000000000000000000000000',
    })
  ).rejects.toThrow('Cannot call getStorageAt with a shielded public client')
}
