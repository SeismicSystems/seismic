import { afterEach, expect, spyOn, test } from 'bun:test'
import type {
  EncryptSeismicTxParams,
  GasPayment,
  SeismicTxSerializer,
} from 'seismic-encrypt'
import { encryptSeismicTx, serializeSeismicTx } from 'seismic-encrypt'
import { serializeSeismicTransaction } from 'seismic-viem'
import { fromRlp, getAddress, keccak256, slice } from 'viem'
import type { Hex } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

const params: EncryptSeismicTxParams = {
  tx: {
    chainId: 5124,
    nonce: 1,
    gasPrice: 2n,
    gas: 100_000n,
    to: '0x1111111111111111111111111111111111111111',
    data: '0xdeadbeef',
    value: 3n,
  },
  sender: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
  rpcUrl: 'http://fixture.invalid',
  encryptionPrivateKey:
    '0x0000000000000000000000000000000000000000000000000000000000000001',
}
const payments: GasPayment[] = [
  { type: 'auto' },
  { type: 'native' },
  { type: 'token', token: '0xabcdefabcdefabcdefabcdefabcdefabcdefabcd' },
]
const signature = {
  v: 27n,
  r: `0x${'11'.repeat(32)}` as Hex,
  s: `0x${'22'.repeat(32)}` as Hex,
}

function mockRpc() {
  return spyOn(globalThis, 'fetch').mockImplementation(async (_input, init) => {
    if (typeof init?.body !== 'string') {
      throw new Error('Expected JSON-RPC request body')
    }
    const request = JSON.parse(init.body) as { id: number; method: string }
    const result = (() => {
      switch (request.method) {
        case 'seismic_getTeePublicKey':
          return '0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0'
        case 'eth_getBlockByNumber':
          return {
            hash: `0x${'ab'.repeat(32)}`,
            number: '0x10',
            gasLimit: '0x1c9c380',
            transactions: [],
          }
        default:
          throw new Error(`Unexpected RPC method: ${request.method}`)
      }
    })()
    return new Response(
      JSON.stringify({ jsonrpc: '2.0', id: request.id, result }),
      {
        headers: { 'content-type': 'application/json' },
      }
    )
  })
}

const restore: (() => void)[] = []
afterEach(() => {
  for (const cleanup of restore) cleanup()
  restore.length = 0
})

for (const gasPayment of [undefined, ...payments]) {
  test(`encryptSeismicTx resolves ${gasPayment?.type ?? 'omitted Auto'} and preserves it in both serializers`, async () => {
    const rpc = mockRpc()
    restore.push(() => rpc.mockRestore())
    const result = await encryptSeismicTx({
      ...params,
      tx: { ...params.tx, gasPayment },
    })
    const expected: GasPayment =
      gasPayment?.type === 'token'
        ? { type: 'token', token: getAddress(gasPayment.token) }
        : (gasPayment ?? { type: 'auto' })
    expect(result.seismicTx.gasPayment).toEqual(expected)
    expect(result.seismicTx.gas).toBe(params.tx.gas)
    expect(result.seismicTx.value).toBe(params.tx.value!)
    expect(result.seismicTx.data).not.toBe(params.tx.data)
    expect(result.unsignedSerializedTx).toBe(
      serializeSeismicTransaction(result.seismicTx)
    )
    expect(result.serialize(signature)).toBe(
      serializeSeismicTransaction(result.seismicTx, signature)
    )
    const fields = fromRlp(slice(result.unsignedSerializedTx, 1), 'hex')
    expect(fields).toHaveLength(15)
    expect(fields[4]).toEqual(
      gasPayment?.type === 'token'
        ? ['0x02', gasPayment.token]
        : [gasPayment?.type === 'native' ? '0x01' : '0x', '0x']
    )
    expect(rpc).toHaveBeenCalledTimes(2)
  })
}

test('documented Viem signing and returned signature callback agree', async () => {
  const rpc = mockRpc()
  restore.push(() => rpc.mockRestore())
  const account = privateKeyToAccount(params.encryptionPrivateKey!)
  const result = await encryptSeismicTx({ ...params, sender: account.address })
  expect(result.serialize()).toBe(result.unsignedSerializedTx)
  expect(result.serialize({ r: signature.r, s: signature.s, yParity: 0 })).toBe(
    result.serialize(signature)
  )
  const signed = await account.signTransaction<SeismicTxSerializer>(
    result.seismicTx,
    { serializer: serializeSeismicTx }
  )
  const helperSerializer: SeismicTxSerializer = (_tx, sig) =>
    result.serialize(sig)
  const helperSigned = await account.signTransaction<SeismicTxSerializer>(
    result.seismicTx,
    { serializer: helperSerializer }
  )
  expect(helperSigned).toBe(signed)
  expect(fromRlp(slice(signed, 1), 'hex')).toHaveLength(18)
  expect(rpc).toHaveBeenCalledTimes(2)
})

test('changing only gasPayment leaves ciphertext/AAD unchanged, but changes signed bytes', async () => {
  const rpc = mockRpc()
  restore.push(() => rpc.mockRestore())
  // A fixed key/nonce is safe only in this isolated test, never production.
  const random = spyOn(globalThis.crypto, 'getRandomValues').mockImplementation(
    (array) => {
      if (array) {
        new Uint8Array(array.buffer, array.byteOffset, array.byteLength).fill(1)
      }
      return array
    }
  )
  restore.push(() => random.mockRestore())
  const results = []
  for (const gasPayment of payments) {
    results.push(
      await encryptSeismicTx({ ...params, tx: { ...params.tx, gasPayment } })
    )
  }
  expect(
    new Set(results.map((result) => result.seismicTx.encryptionNonce)).size
  ).toBe(1)
  expect(new Set(results.map((result) => result.seismicTx.data)).size).toBe(1)
  expect(
    new Set(results.map((result) => keccak256(result.unsignedSerializedTx)))
      .size
  ).toBe(3)
  expect(
    new Set(results.map((result) => result.serialize(signature))).size
  ).toBe(3)
})

test('invalid selection is rejected before any RPC or encryption', async () => {
  const rpc = mockRpc()
  restore.push(() => rpc.mockRestore())
  for (const invalid of [
    null,
    { type: 'token', token: '0x0000000000000000000000000000000000000000' },
    { type: 'auto', token: params.tx.to },
    { type: 'unknown' },
  ]) {
    await expect(
      encryptSeismicTx({
        ...params,
        tx: { ...params.tx, gasPayment: invalid as unknown as GasPayment },
      })
    ).rejects.toThrow('Invalid gasPayment')
  }
  expect(rpc).not.toHaveBeenCalled()
})

test('empty-calldata transfer keeps its input and mandatory selection', async () => {
  const rpc = mockRpc()
  restore.push(() => rpc.mockRestore())
  const result = await encryptSeismicTx({
    ...params,
    tx: { ...params.tx, data: '0x', gasPayment: payments[1] },
  })
  expect(result.seismicTx.data).toBe('0x')
  expect(result.seismicTx.gasPayment).toEqual({ type: 'native' })
  expect(result.serialize(signature)).toBe(
    serializeSeismicTx(result.seismicTx, signature)
  )
})
