import type { ShieldedPublicClient, ShieldedWalletClient } from 'seismic-viem'
import type { Address } from 'viem'

import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { seismicCounterBytecode } from '@sviem-tests/tests/contract/bytecode.ts'

/**
 * Send the SeismicCounter deploy tx and wait for its receipt.
 * Returns the deployed contract address.
 *
 * Takes clients rather than constructing them so the caller's generic
 * type inference (chain, account) flows through to the contract proxy.
 */
export const deploySeismicCounter = async ({
  publicClient,
  walletClient,
}: {
  publicClient: ShieldedPublicClient
  walletClient: ShieldedWalletClient
}): Promise<Address> => {
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`
  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  return deployReceipt.contractAddress!
}
