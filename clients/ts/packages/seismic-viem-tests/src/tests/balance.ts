import { expect } from 'bun:test'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
  getNativeBalance,
} from 'seismic-viem'
import { createPublicClient, createWalletClient, custom, http } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

import {
  type PublicClientArgs,
  httpPublicClient,
} from '@sviem-tests/clients.ts'
import {
  ENCRYPTION_PK,
  TEST_ACCOUNT_PRIVATE_KEY,
} from '@sviem-tests/constants.ts'

const ADDRESS = privateKeyToAccount(TEST_ACCOUNT_PRIVATE_KEY).address
const EMPTY_ADDRESS = '0x00000000000000000000000000000000000000ab'
const COMPATIBILITY_BALANCE =
  '0x9612084f0316e0ebd5182f398e5195a51b5ca47667d4c9b26c9b26c9b26c9b2'

export const testNativeBalanceModes = async () => {
  const calls: { method: string; params: unknown }[] = []
  const transport = custom({
    request: async ({ method, params }) => {
      calls.push({ method, params })
      if (method === 'eth_getBalance') return COMPATIBILITY_BALANCE
      if (method === 'seismic_getTeePublicKey') return ENCRYPTION_PK
      if (method === 'eth_getAccountInfo') {
        expect(params).toHaveLength(2)
        return {
          balance: params[0] === EMPTY_ADDRESS ? '0x0' : '0x2a',
          nonce: '0x0',
          code: '0x',
        }
      }
      throw new Error(`Unexpected RPC: ${method}`)
    },
  })
  const publicClient = createShieldedPublicClient({ transport })
  const wallet = await createShieldedWalletClient({
    transport,
    publicClient,
    account: privateKeyToAccount(TEST_ACCOUNT_PRIVATE_KEY),
  })

  for (const client of [publicClient, wallet]) {
    for (const address of [ADDRESS, EMPTY_ADDRESS] as const) {
      expect(await client.getBalance({ address })).toBe(
        BigInt(COMPATIBILITY_BALANCE)
      )
      expect(await client.getNativeBalance({ address })).toBe(
        address === EMPTY_ADDRESS ? 0n : 42n
      )
    }
  }
  // Also usable with an ordinary viem public client, as in the funding bot.
  expect(
    await getNativeBalance(createPublicClient({ transport }), {
      address: ADDRESS,
    })
  ).toBe(42n)
  expect(
    calls.filter(({ method }) => method === 'eth_getBalance')
  ).toHaveLength(4)
  expect(
    calls.filter(({ method }) => method === 'eth_getAccountInfo')
  ).toHaveLength(5)
}

export const testNativeBalanceBlockSelection = async () => {
  const blocks: unknown[] = []
  const client = createShieldedPublicClient({
    transport: custom({
      request: async ({ method, params }) => {
        expect(method).toBe('eth_getAccountInfo')
        expect(params).toHaveLength(2)
        expect(params[0]).toBe(ADDRESS)
        blocks.push(params[1])
        return { balance: '0x1', nonce: '0x0', code: '0x' }
      },
    }),
  })
  await client.getNativeBalance({ address: ADDRESS })
  await client.getNativeBalance({ address: ADDRESS, blockNumber: 0n })
  await client.getNativeBalance({ address: ADDRESS, blockNumber: 42n })
  await client.getNativeBalance({ address: ADDRESS, blockTag: 'pending' })
  expect(blocks).toEqual(['latest', '0x0', '0x2a', 'pending'])
}

export const testNativeBalanceNeverFallsBack = async () => {
  for (const error of [
    { code: -32601, message: 'Method not found' },
    { code: -32001, message: 'block not found' },
  ]) {
    const methods: string[] = []
    const client = createShieldedPublicClient({
      transport: custom(
        {
          request: async ({ method }) => {
            methods.push(method)
            if (method === 'eth_getBalance') return COMPATIBILITY_BALANCE
            throw error
          },
        },
        { retryCount: 0 }
      ),
    })
    await expect(
      client.getNativeBalance({ address: ADDRESS })
    ).rejects.toThrow()
    expect(methods).toEqual(['eth_getAccountInfo'])
  }
}

/** Real-node coverage: both reth and Sanvil must expose actual native funds. */
export const testNativeBalanceOnNode = async (args: PublicClientArgs) => {
  const client = httpPublicClient(args)
  expect(await client.getNativeBalance({ address: EMPTY_ADDRESS })).toBe(0n)
  expect(await client.getNativeBalance({ address: ADDRESS })).toBeGreaterThan(
    0n
  )
  expect(
    await client.getNativeBalance({ address: ADDRESS, blockNumber: 0n })
  ).toBeGreaterThan(0n)
  // An ordinary native transfer makes historical/latest state distinguishable.
  const recipient = '0x00000000000000000000000000000000000000ac'
  const blockNumber = await client.getBlockNumber({ cacheTime: 0 })
  const before = await client.getNativeBalance({
    address: recipient,
    blockNumber,
  })
  const wallet = createWalletClient({
    account: privateKeyToAccount(TEST_ACCOUNT_PRIVATE_KEY),
    chain: args.chain,
    transport: http(args.url),
  })
  const hash = await wallet.sendTransaction({
    to: recipient,
    value: 1n,
    gas: 21_000n,
    gasPrice: await client.getGasPrice(),
    type: 'legacy',
  })
  const receipt = await client.waitForTransactionReceipt({ hash })
  expect(receipt.status).toBe('success')
  expect(await client.getNativeBalance({ address: recipient })).toBe(
    before + 1n
  )
  expect(
    await client.getNativeBalance({ address: recipient, blockNumber })
  ).toBe(before)

  const version = await client.request({ method: 'web3_clientVersion' })
  if (!version.startsWith('anvil/')) {
    for (const address of [ADDRESS, EMPTY_ADDRESS] as const) {
      expect(await client.getBalance({ address })).toBe(
        BigInt(COMPATIBILITY_BALANCE)
      )
    }
  }
}
