import { expect } from 'bun:test'
import { getShieldedContract } from 'seismic-viem'
import type { Account, Chain } from 'viem'
import { decodeFunctionResult, encodeFunctionData, parseEther } from 'viem'

import { httpPublicClient, httpWalletClient } from '@sviem-tests/clients.ts'
import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { deploySeismicCounter } from '@sviem-tests/tests/contract/deploy.ts'
import { depositContractAbi } from '@sviem-tests/tests/contract/depositContractAbi.ts'
import { depositContractBytecode } from '@sviem-tests/tests/contract/depositContractBytecode.ts'

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
  const publicClient = httpPublicClient({ chain, url })
  const walletClient = await httpWalletClient({ chain, url, account })
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
  const publicClient = httpPublicClient({ chain, url })
  const walletClient = await httpWalletClient({ chain, url, account })
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

// A signed read binds nonce and value into the AEAD metadata. Passing either
// explicitly must produce the same values in the signed request, otherwise
// the node cannot authenticate the calldata.
export const testSignedCallWithExplicitNonce = async ({
  chain,
  url,
  account,
}: SignedCallTestArgs) => {
  const publicClient = httpPublicClient({ chain, url })
  const walletClient = await httpWalletClient({ chain, url, account })
  const address = await deploySeismicCounter({ publicClient, walletClient })

  const calldata = encodeFunctionData({
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  const nonce = await publicClient.getTransactionCount({
    address: account.address,
    blockTag: 'pending',
  })

  const { data } = await walletClient.signedCall({
    to: address,
    data: calldata,
    account: account.address,
    nonce,
  })

  expect(data).toBeDefined()
  const isOdd = decodeFunctionResult({
    abi: seismicCounterAbi,
    functionName: 'isOdd',
    data: data!,
  })
  expect(isOdd).toBe(false)
}

// The deposit contract rejects a value that is not a whole number of gwei
// after it has already checked the argument lengths, so seeing that reason
// proves both that the calldata decrypted and that msg.value was the value
// we asked for.
export const testSignedCallWithValue = async ({
  chain,
  url,
  account,
}: SignedCallTestArgs) => {
  const publicClient = httpPublicClient({ chain, url })
  const walletClient = await httpWalletClient({ chain, url, account })

  const bytecode: `0x${string}` = `0x${depositContractBytecode.object.replace(/^0x/, '')}`
  const deployTx = await walletClient.deployContract({
    abi: depositContractAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const { contractAddress } = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })

  const calldata = encodeFunctionData({
    abi: depositContractAbi,
    functionName: 'deposit',
    args: [
      `0x${'11'.repeat(32)}`,
      `0x${'22'.repeat(48)}`,
      `0x01${'00'.repeat(11)}${account.address.slice(2)}`,
      `0x${'33'.repeat(64)}`,
      `0x${'44'.repeat(96)}`,
      `0x${'55'.repeat(32)}`,
    ],
  })

  let caught: unknown
  try {
    await walletClient.signedCall({
      to: contractAddress!,
      data: calldata,
      account: account.address,
      value: parseEther('1') + 1n,
    })
  } catch (err) {
    caught = err
  }
  expect(caught, 'deposit() must revert').toBeDefined()
  const message = (caught as Error).message
  expect(
    message.includes('DepositContract: deposit value not multiple of gwei'),
    `signedCall should execute with the requested value, got: ${message}`
  ).toBe(true)
}
