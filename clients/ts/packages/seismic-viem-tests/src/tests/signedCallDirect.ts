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

const ODD_COUNTER_VALUE = 7n
const LARGE_BLOCKS_WINDOW = 200n

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

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address: contractAddress,
    client: walletClient,
  })
  const setTx = await contract.write.setNumber([ODD_COUNTER_VALUE])
  await publicClient.waitForTransactionReceipt({ hash: setTx })

  const calldata = encodeFunctionData({
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  const { data } = await walletClient.signedCall({
    to: contractAddress,
    data: calldata,
    account: account.address,
  })

  expect(data).toBeDefined()
  expect(data!.endsWith('1')).toBe(true)
}

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

  const { data } = await walletClient.signedCall(
    {
      to: contractAddress,
      data: calldata,
      account: account.address,
    },
    { blocksWindow: LARGE_BLOCKS_WINDOW }
  )
  expect(data).toBeDefined()
}
