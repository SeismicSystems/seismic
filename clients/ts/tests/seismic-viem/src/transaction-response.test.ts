import { describe, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import type { Address, Hex, RpcTransaction } from 'viem'
import { createPublicClient, custom, formatTransaction, getAddress } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

import {
  createSeismicDevnet,
  localSeismicDevnet,
  sanvil,
} from '@sviem/chain.ts'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
} from '@sviem/client.ts'
import type { GasPayment } from '@sviem/tx/gasPayment.ts'
import type { RpcSeismicTransaction } from '@sviem/tx/response.ts'
import { seismicChainFormatters } from '@sviem/tx/seismicRpc.ts'

const fixture = JSON.parse(
  readFileSync(
    new URL('../../../../test-vectors/gas-payment.json', import.meta.url),
    'utf8'
  )
) as {
  privateKey: Hex
  vectors: {
    tx: Omit<
      RpcSeismicTransaction,
      | 'type'
      | 'blockHash'
      | 'blockNumber'
      | 'transactionIndex'
      | 'hash'
      | 'from'
      | 'r'
      | 's'
      | 'v'
      | 'yParity'
    >
    txHash: Hex
    signature: { r: Hex; s: Hex; yParity: Hex }
  }[]
}
const blockHash: Hex = `0x${'ab'.repeat(32)}`
const sender: Address = '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266'

function rawTransaction(
  vector: (typeof fixture.vectors)[number]
): RpcSeismicTransaction {
  return {
    ...vector.tx,
    ...vector.signature,
    type: '0x4A',
    to: vector.tx.to ?? null,
    blockHash,
    blockNumber: '0x1',
    transactionIndex: '0x0',
    hash: vector.txHash,
    from: sender,
    v: vector.signature.yParity,
  }
}

function transportFor(raw: RpcSeismicTransaction | RpcTransaction) {
  return custom({
    request: async ({ method, params }) => {
      if (method === 'seismic_getTeePublicKey') {
        return `0x${fixture.vectors[0].tx.encryptionPubkey}`
      }
      if (
        method === 'eth_getTransactionByHash' ||
        method === 'eth_getTransactionByBlockHashAndIndex' ||
        method === 'eth_getTransactionByBlockNumberAndIndex'
      ) {
        return structuredClone(raw)
      }
      if (
        method === 'eth_getBlockByHash' ||
        method === 'eth_getBlockByNumber'
      ) {
        return {
          hash: blockHash,
          number: '0x1',
          transactions: params?.[1] ? [structuredClone(raw)] : [raw.hash],
        }
      }
      throw new Error(`Unexpected response fixture RPC: ${method}`)
    },
  })
}

const chains = [
  localSeismicDevnet,
  createSeismicDevnet({ nodeHost: 'fixture.invalid' }),
  sanvil,
]

describe('existing transaction lookup and block response surfaces', () => {
  for (const [index, vector] of fixture.vectors.entries()) {
    test(`Rust vector ${index} (${vector.tx.gasPayment.type}): local/remote, ordinary/shielded clients`, async () => {
      const raw = rawTransaction(vector)
      for (const chain of chains) {
        for (const client of [
          createPublicClient({ chain, transport: transportFor(raw) }),
          createShieldedPublicClient({ chain, transport: transportFor(raw) }),
        ]) {
          const transactions = [
            await client.getTransaction({ hash: raw.hash }),
            await client.getTransaction({ blockHash, index: 0 }),
            await client.getTransaction({ blockNumber: 1n, index: 0 }),
            (await client.getBlock({ blockHash, includeTransactions: true }))
              .transactions[0],
            (
              await client.getBlock({
                blockNumber: 1n,
                includeTransactions: true,
              })
            ).transactions[0],
          ]
          for (const tx of transactions) {
            expect(tx.type).toBe('seismic')
            // This also verifies discriminated-union inference; no casts/any.
            if (tx.type !== 'seismic')
              throw new Error('Missing Seismic response type')
            const payment: GasPayment = tx.gasPayment
            expect(payment).toEqual(vector.tx.gasPayment)
            expect(tx.typeHex).toBe(raw.type)
            expect(tx.chainId).toBe(Number(raw.chainId))
            expect(tx.gas).toBe(BigInt(raw.gas))
            expect(tx.gasPrice).toBe(BigInt(raw.gasPrice))
            expect(tx.input).toBe(raw.input)
            expect(tx.encryptionPubkey).toBe(
              `0x${raw.encryptionPubkey.replace(/^0x/, '')}`
            )
            expect(tx.encryptionNonce).toBe(BigInt(raw.encryptionNonce))
            expect(tx.messageVersion).toBe(Number(raw.messageVersion))
            expect(tx.expiresAtBlock).toBe(BigInt(raw.expiresAtBlock))
            expect(tx.signedRead).toBe(raw.signedRead)
            expect(tx.yParity).toBe(Number(raw.yParity))
            expect(
              (tx.authorizationList ?? []).map((auth) => ({
                chainId: auth.chainId,
                nonce: auth.nonce,
                contractAddress: auth.contractAddress,
              }))
            ).toEqual(
              (raw.authorizationList ?? []).map((auth) => ({
                chainId: Number(auth.chainId),
                nonce: Number(auth.nonce),
                contractAddress: auth.address,
              }))
            )
          }
          expect((await client.getBlock({ blockHash })).transactions).toEqual([
            raw.hash,
          ])
        }
      }
    })
  }

  test('pending and CREATE responses retain null fields and zero quantities', async () => {
    const base = rawTransaction(fixture.vectors[0])
    const raw: RpcSeismicTransaction = {
      ...base,
      blockHash: null,
      blockNumber: null,
      transactionIndex: null,
      to: null,
      encryptionNonce: '0x0',
      messageVersion: '0x0',
      expiresAtBlock: '0x0',
    }
    const tx = await createPublicClient({
      chain: localSeismicDevnet,
      transport: transportFor(raw),
    }).getTransaction({ hash: raw.hash })
    expect(tx.blockHash).toBeNull()
    expect(tx.blockNumber).toBeNull()
    expect(tx.transactionIndex).toBeNull()
    expect(tx.to).toBeNull()
    if (tx.type !== 'seismic') throw new Error('Missing Seismic type')
    expect(tx.encryptionNonce).toBe(0n)
    expect(tx.messageVersion).toBe(0)
    expect(tx.expiresAtBlock).toBe(0n)
  })

  const common = {
    hash: fixture.vectors[0].txHash,
    blockHash,
    blockNumber: '0x1',
    transactionIndex: '0x0',
    from: sender,
    to: sender,
    gas: '0x5208',
    value: '0x0',
    nonce: '0x0',
    chainId: '0x1404',
    input: '0x',
    v: '0x1',
    r: '0x01',
    s: '0x02',
  } as const
  const dynamic = {
    ...common,
    maxFeePerGas: '0x2',
    maxPriorityFeePerGas: '0x1',
    accessList: [],
    yParity: '0x1',
  } as const
  const ordinary: RpcTransaction[] = [
    { ...common, type: '0x0', gasPrice: '0x1' },
    { ...common, type: '0x1', gasPrice: '0x1', accessList: [], yParity: '0x1' },
    { ...dynamic, type: '0x2' },
    {
      ...dynamic,
      type: '0x3',
      maxFeePerBlobGas: '0x1',
      blobVersionedHashes: [],
    },
    { ...dynamic, type: '0x4', authorizationList: [] },
  ]
  for (const raw of ordinary) {
    test(`ordinary Ethereum ${raw.type} keeps stock formatting`, async () => {
      for (const chain of chains) {
        const client = createShieldedPublicClient({
          chain,
          transport: transportFor(raw),
        })
        const tx = await client.getTransaction({ hash: raw.hash })
        if (tx.type === 'seismic')
          throw new Error('Ordinary Ethereum type changed')
        expect(formatTransaction(raw)).toEqual(tx)
        expect(tx.gasPayment).toBeUndefined()
        const embedded = (
          await client.getBlock({ blockHash, includeTransactions: true })
        ).transactions[0]
        expect(embedded).toEqual(tx)
      }
    })
  }

  for (const kind of ['auto', 'native', 'token'] as const) {
    test(`shielded wallet lookup preserves ${kind} selection`, async () => {
      const vector = fixture.vectors.find(
        (item) => item.tx.gasPayment.type === kind
      )
      if (!vector) throw new Error(`Missing ${kind} vector`)
      const raw = rawTransaction(vector)
      for (const chain of chains) {
        const wallet = await createShieldedWalletClient({
          chain,
          transport: transportFor(raw),
          account: privateKeyToAccount(fixture.privateKey),
          encryptionSk: fixture.privateKey,
        })
        const tx = await wallet.getTransaction({ hash: raw.hash })
        if (tx.type !== 'seismic') throw new Error('Missing Seismic type')
        const payment: GasPayment = tx.gasPayment
        expect(payment).toEqual(vector.tx.gasPayment)
        const embedded = (
          await wallet.getBlock({ blockHash, includeTransactions: true })
        ).transactions[0]
        expect(embedded).toEqual(tx)
      }
    })
  }

  test('mixed blocks retain ordinary transactions and Seismic metadata', async () => {
    const raw = rawTransaction(fixture.vectors[0])
    const legacy = ordinary[0]
    const client = createPublicClient({
      chain: localSeismicDevnet,
      transport: custom({
        request: async () => ({
          hash: blockHash,
          number: '0x1',
          transactions: [legacy, raw],
        }),
      }),
    })
    const block = await client.getBlock({
      blockHash,
      includeTransactions: true,
    })
    expect(seismicChainFormatters.transaction.format(legacy)).toEqual(
      block.transactions[0]
    )
    expect(seismicChainFormatters.transaction.format(raw)).toEqual(
      block.transactions[1]
    )
  })

  test('token addresses and prefixed public keys are normalized', async () => {
    const raw = rawTransaction(fixture.vectors[0])
    const address = sender.toLowerCase() as Address
    const tx = await createPublicClient({
      chain: localSeismicDevnet,
      transport: transportFor({
        ...raw,
        encryptionPubkey: `0x${raw.encryptionPubkey}`,
        gasPayment: { type: 'token' as const, token: address },
      }),
    }).getTransaction({ hash: raw.hash })
    if (tx.type !== 'seismic') throw new Error('Missing Seismic type')
    expect(tx.encryptionPubkey).toBe(`0x${raw.encryptionPubkey}`)
    expect(tx.gasPayment).toEqual({ type: 'token', token: getAddress(address) })
  })

  for (const type of ['0x4a', '0x4A'] as const) {
    test(`${type} responses derive missing yParity from v`, () => {
      const raw = { ...rawTransaction(fixture.vectors[0]), type }
      delete raw.yParity
      const tx = seismicChainFormatters.transaction.format(raw)
      if (tx.type !== 'seismic') throw new Error('Missing Seismic type')
      expect(tx.typeHex).toBe(type)
      expect(tx.yParity).toBe(Number(raw.v))
    })
  }

  test('missing or malformed Seismic selectors are not defaulted to Auto', () => {
    const raw = rawTransaction(fixture.vectors[0])
    for (const gasPayment of [
      undefined,
      { type: 'unknown' },
      { type: 'token', token: '0x' },
    ]) {
      const malformed = {
        ...raw,
        gasPayment,
      } as unknown as RpcSeismicTransaction
      expect(() =>
        seismicChainFormatters.transaction.format(malformed)
      ).toThrow()
    }
  })
})
