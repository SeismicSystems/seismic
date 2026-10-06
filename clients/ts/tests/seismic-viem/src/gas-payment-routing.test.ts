import { expect, test } from 'bun:test'
import {
  createShieldedWalletClient,
  getShieldedContract,
  sanvil,
} from 'seismic-viem'
import type { GasPayment, ShieldedWalletClient } from 'seismic-viem'
import { custom, numberToHex } from 'viem'
import type { Hex } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

import {
  ENCRYPTION_PK,
  ENCRYPTION_SK,
  TEST_ACCOUNT_PRIVATE_KEY,
} from '@sviem-tests/constants.ts'

const abi = [
  {
    type: 'function',
    name: 'setNumber',
    inputs: [{ name: 'number', type: 'suint256' }],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'increment',
    inputs: [],
    outputs: [],
    stateMutability: 'nonpayable',
  },
] as const
const pinned = {
  recentBlockHash: `0x${'11'.repeat(32)}` as Hex,
  expiresAtBlock: 100n,
}
const address = `0x${'33'.repeat(20)}` as const
const choices: (GasPayment | undefined)[] = [
  undefined,
  { type: 'auto' },
  { type: 'native' },
  { type: 'token', token: `0x${'22'.repeat(20)}` },
]

async function setup() {
  const calls: { method: string; params?: readonly unknown[] }[] = []
  const client = await createShieldedWalletClient({
    account: privateKeyToAccount(TEST_ACCOUNT_PRIVATE_KEY),
    chain: sanvil,
    encryptionSk: ENCRYPTION_SK,
    cacheTime: 0,
    transport: custom(
      {
        request: async ({ method, params }) => {
          calls.push({ method, params })
          switch (method) {
            case 'seismic_getTeePublicKey':
              return ENCRYPTION_PK
            case 'eth_chainId':
              return numberToHex(sanvil.id)
            case 'eth_getTransactionCount':
              return '0x0'
            case 'eth_gasPrice':
              return '0x1'
            case 'eth_getBlockByNumber':
              return {
                number: '0x1',
                hash: pinned.recentBlockHash,
                gasLimit: '0x1000000',
                gasUsed: '0x0',
                transactions: [],
                timestamp: '0x1',
                baseFeePerGas: null,
              }
            case 'eth_estimateGas':
              return '0x5208'
            case 'eth_sendRawTransaction':
              return `0x${'55'.repeat(32)}`
            default:
              throw new Error(`Unexpected RPC ${method}`)
          }
        },
      },
      { retryCount: 0 }
    ),
  })
  // Match the shared test clients' erased chain/account boundary.
  return { client: client as unknown as ShieldedWalletClient, calls }
}

for (const gasPayment of choices) {
  for (const route of [
    'send',
    'wallet',
    'contract',
    'debug',
    'smart',
  ] as const) {
    test(`${route} write and signed estimate retain ${gasPayment?.type ?? 'omitted Auto'}`, async () => {
      const { client, calls } = await setup()
      const options = { chain: undefined, gasPayment, nonce: 0, gasPrice: 1n }
      const contract = getShieldedContract({ abi, address, client })
      if (route === 'send')
        await client.sendShieldedTransaction(
          { to: address, data: '0x1234', ...options },
          pinned
        )
      else if (route === 'wallet')
        await client.swriteContract(
          { abi, address, functionName: 'setNumber', args: [1n], ...options },
          pinned
        )
      else if (route === 'contract')
        await contract.swrite.setNumber([1n], options)
      else if (route === 'smart') await contract.write.setNumber([1n], options)
      else {
        const debug = await client.dwriteContract(
          { abi, address, functionName: 'setNumber', args: [1n], ...options },
          pinned
        )
        expect(debug.plaintextTx.gasPayment).toEqual(
          gasPayment ?? { type: 'auto' }
        )
        expect(debug.shieldedTx.gasPayment).toEqual(
          gasPayment ?? { type: 'auto' }
        )
      }
      const expected = {
        kind:
          !gasPayment || gasPayment.type === 'auto'
            ? 0
            : gasPayment.type === 'native'
              ? 1
              : 2,
        token:
          gasPayment?.type === 'token'
            ? gasPayment.token
            : `0x${'00'.repeat(20)}`,
      }
      const estimates = calls.filter(
        ({ method }) => method === 'eth_estimateGas'
      )
      const sends = calls.filter(
        ({ method }) => method === 'eth_sendRawTransaction'
      )
      expect(estimates).toHaveLength(1)
      expect(sends).toHaveLength(1)
      expect(estimates[0].params?.[0]).toMatchObject({
        data: { message: { gasPayment: expected, signedRead: true } },
      })
      expect(sends[0].params?.[0]).toMatchObject({
        data: { message: { gasPayment: expected, signedRead: false } },
      })
    })
  }
}

for (const gasPayment of choices.slice(2)) {
  test(`transparent routes reject ${gasPayment?.type} before estimate/send`, async () => {
    const { client, calls } = await setup()
    const contract = getShieldedContract({ abi, address, client })
    await expect(
      client.sendTransaction({ chain: undefined, to: address, gasPayment })
    ).rejects.toThrow('requires a Seismic transaction')
    await expect(
      client.writeContract({
        chain: undefined,
        abi,
        address,
        functionName: 'increment',
        gasPayment,
      })
    ).rejects.toThrow('requires a Seismic transaction')
    await expect(
      contract.write.increment({ chain: undefined, gasPayment })
    ).rejects.toThrow('requires a Seismic transaction')
    await expect(
      contract.twrite.increment({ chain: undefined, gasPayment })
    ).rejects.toThrow('requires a Seismic transaction')
    expect(
      calls.filter(
        ({ method }) =>
          method === 'eth_estimateGas' || method === 'eth_sendRawTransaction'
      )
    ).toHaveLength(0)
  })
}

test('ordinary Ethereum signed bytes are unchanged with omitted or explicit Auto', async () => {
  const { client, calls } = await setup()
  const request = {
    type: 'legacy' as const,
    to: address,
    nonce: 0,
    gas: 21_000n,
    gasPrice: 1n,
    value: 1n,
  }
  const expected = await privateKeyToAccount(
    TEST_ACCOUNT_PRIVATE_KEY
  ).signTransaction({
    ...request,
    chainId: sanvil.id,
  })
  await client.sendTransaction({ ...request, chain: undefined })
  await client.sendTransaction({
    ...request,
    chain: undefined,
    gasPayment: { type: 'auto' },
  })
  const sends = calls.filter(
    ({ method }) => method === 'eth_sendRawTransaction'
  )
  expect(sends).toHaveLength(2)
  expect(sends.map(({ params }) => params?.[0])).toEqual([expected, expected])
  expect(
    calls.filter(({ method }) => method === 'eth_estimateGas')
  ).toHaveLength(0)
})
