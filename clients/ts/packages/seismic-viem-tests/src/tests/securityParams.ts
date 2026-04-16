import { expect } from 'bun:test'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
} from 'seismic-viem'
import { http } from 'viem'

import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { seismicCounterBytecode } from '@sviem-tests/tests/contract/bytecode.ts'
import type { ContractTestArgs } from '@sviem-tests/tests/contract/contract.ts'

export const testDwriteContractUsesSecurityParams = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
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
  const address = deployReceipt.contractAddress!

  const latestBlock = await publicClient.getBlock({ blockTag: 'latest' })
  const encryptionNonce = '0xaaaaaaaaaaaaaaaaaaaaaaaa'
  const expiresAtBlock = latestBlock.number + 10n

  const { txHash, shieldedTx } = await walletClient.dwriteContract(
    {
      address,
      abi: seismicCounterAbi,
      functionName: 'setNumber',
      args: [11n],
    },
    {
      encryptionNonce,
      recentBlockHash: latestBlock.hash,
      expiresAtBlock,
    }
  )

  expect(shieldedTx.encryptionNonce).toBe(encryptionNonce)
  expect(shieldedTx.recentBlockHash).toBe(latestBlock.hash)
  expect(shieldedTx.expiresAtBlock).toBe(expiresAtBlock)

  const receipt = await publicClient.waitForTransactionReceipt({ hash: txHash })
  expect(receipt.status).toBe('success')
}
