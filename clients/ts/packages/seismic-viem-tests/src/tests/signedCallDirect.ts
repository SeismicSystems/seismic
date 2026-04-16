import { expect } from 'bun:test'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
  getShieldedContract,
} from 'seismic-viem'
import type { Account, Chain } from 'viem'
import { encodeFunctionData, http } from 'viem'

import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { seismicCounterBytecode } from '@sviem-tests/tests/contract/bytecode.ts'

type SignedCallTestArgs = {
  chain: Chain
  url: string
  account: Account
}

/**
 * Test signedCall directly as a standalone action (not through contract.read).
 * Deploy a contract, set a value, then use signedCall to read it.
 */
export const testSignedCallDirect = async ({
  chain,
  url,
  account,
}: SignedCallTestArgs) => {
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

  // Set the number to 7
  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address: contractAddress,
    client: walletClient,
  })
  const setTx = await contract.write.setNumber([7n])
  await publicClient.waitForTransactionReceipt({ hash: setTx })

  // Use signedCall directly to read isOdd()
  const calldata = encodeFunctionData({
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  const { data } = await walletClient.signedCall({
    to: contractAddress,
    data: calldata,
    account: account.address,
  })

  // 7 is odd, so we expect a truthy response
  // The response is ABI-encoded bool
  expect(data).toBeDefined()
  // ABI-encoded true is 0x...0001
  expect(data!.endsWith('1')).toBe(true)
}

/**
 * Test signedCall with explicit security params (custom blocksWindow).
 */
export const testSignedCallWithSecurityParams = async ({
  chain,
  url,
  account,
}: SignedCallTestArgs) => {
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

  const calldata = encodeFunctionData({
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })

  // Call with a large blocksWindow — should succeed
  const { data } = await walletClient.signedCall(
    {
      to: contractAddress,
      data: calldata,
      account: account.address,
    },
    { blocksWindow: 200n }
  )
  expect(data).toBeDefined()
}
