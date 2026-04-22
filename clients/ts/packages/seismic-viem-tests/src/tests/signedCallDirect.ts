import { expect } from 'bun:test'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
  getShieldedContract,
} from 'seismic-viem'
import type { Account, Chain } from 'viem'
import { decodeFunctionResult, encodeFunctionData, http } from 'viem'

import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { deploySeismicCounter } from '@sviem-tests/tests/contract/deploy.ts'

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
  const address = await deploySeismicCounter({ publicClient, walletClient })

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })
  const setTx = await contract.write.setNumber([ODD_COUNTER_VALUE])
  await publicClient.waitForTransactionReceipt({ hash: setTx })

  const calldata = encodeFunctionData({
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  const { data } = await walletClient.signedCall({
    to: address,
    data: calldata,
    account: account.address,
  })

  expect(data).toBeDefined()
  const isOdd = decodeFunctionResult({
    abi: seismicCounterAbi,
    functionName: 'isOdd',
    data: data!,
  })
  expect(isOdd).toBe(true)
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
  const address = await deploySeismicCounter({ publicClient, walletClient })

  const calldata = encodeFunctionData({
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })

  const { data } = await walletClient.signedCall(
    {
      to: address,
      data: calldata,
      account: account.address,
    },
    { blocksWindow: LARGE_BLOCKS_WINDOW }
  )

  expect(data).toBeDefined()
  // Counter initializes to 0; 0 is even.
  const isOdd = decodeFunctionResult({
    abi: seismicCounterAbi,
    functionName: 'isOdd',
    data: data!,
  })
  expect(isOdd).toBe(false)
}
