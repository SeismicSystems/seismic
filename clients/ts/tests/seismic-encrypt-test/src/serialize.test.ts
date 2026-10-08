import { describe, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import type {
  GasPayment,
  SeismicSerializableTransaction,
} from 'seismic-encrypt'
import { serializeSeismicTx } from 'seismic-encrypt'
import {
  type TransactionSerializableSeismic,
  serializeSeismicTransaction,
} from 'seismic-viem'
import { fromRlp, keccak256, slice, toHex, zeroAddress } from 'viem'
import type { Address, Hex, Signature } from 'viem'

const payments: GasPayment[] = [
  { type: 'auto' },
  { type: 'native' },
  { type: 'token', token: '0x1111111111111111111111111111111111111111' },
]
const signature: Signature & { v: bigint } = {
  v: 27n,
  r: '0x5555555555555555555555555555555555555555555555555555555555555555',
  s: '0x6666666666666666666666666666666666666666666666666666666666666666',
}

function transaction(): SeismicSerializableTransaction {
  return {
    chainId: 31337,
    nonce: 1,
    gasPrice: 2n,
    gas: 21_000n,
    to: '0x1111111111111111111111111111111111111111',
    value: 3n,
    encryptionPubkey:
      '0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0',
    encryptionNonce: '0x0102030405060708090a0b0c',
    messageVersion: 0,
    recentBlockHash:
      '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    expiresAtBlock: 123n,
    signedRead: false,
    data: '0xdeadbeef',
    authorizationList: [
      {
        chainId: 31337,
        contractAddress: '0x2222222222222222222222222222222222222222',
        nonce: 7,
        yParity: 1,
        r: '0x3333333333333333333333333333333333333333333333333333333333333333',
        s: '0x4444444444444444444444444444444444444444444444444444444444444444',
      },
    ],
  }
}

describe('standalone/viem fresh-chain serialization parity', () => {
  for (const payment of [undefined, ...payments]) {
    test(`${payment?.type ?? 'omitted Auto'} preserves signed/unsigned fields and authorizationList`, () => {
      const tx = { ...transaction(), gasPayment: payment }
      const viemTx: TransactionSerializableSeismic = {
        ...tx,
        type: 'seismic',
      }
      expect(serializeSeismicTx(tx)).toBe(serializeSeismicTransaction(viemTx))
      expect(serializeSeismicTx(tx, signature)).toBe(
        serializeSeismicTransaction(viemTx, signature)
      )
      const fields = fromRlp(slice(serializeSeismicTx(tx), 1), 'hex')
      expect(fields).toHaveLength(15)
      expect(fields[4]).toEqual(
        payment?.type === 'token'
          ? ['0x02', payment.token]
          : [payment?.type === 'native' ? '0x01' : '0x', '0x']
      )
      expect(fields[5]).toBe(tx.to ?? '0x')
      expect(fields[14]).toHaveLength(1)
    })
  }

  test('omission is explicit Auto before signing, never old-layout bytes', () => {
    const tx = transaction()
    expect(serializeSeismicTx(tx, signature)).toBe(
      serializeSeismicTx(
        { ...tx, gasPayment: { type: 'auto' as const } },
        signature
      )
    )
    expect(
      fromRlp(slice(serializeSeismicTx(tx, signature), 1), 'hex')
    ).toHaveLength(18)
  })

  test('selector changes are authenticated by the signing hash and signed bytes', () => {
    const tx = transaction()
    const hashes = payments.map((gasPayment) =>
      keccak256(serializeSeismicTx({ ...tx, gasPayment }))
    )
    const signed = payments.map((gasPayment) =>
      serializeSeismicTx({ ...tx, gasPayment }, signature)
    )
    expect(new Set(hashes).size).toBe(3)
    expect(new Set(signed).size).toBe(3)
  })

  for (const parity of [0, 1]) {
    test(`v and yParity signatures encode identical bytes for parity ${parity}`, () => {
      const tx = transaction()
      const { r, s } = signature
      const yParitySigned = serializeSeismicTx(tx, { r, s, yParity: parity })
      expect(serializeSeismicTx(tx, { r, s, v: BigInt(parity) })).toBe(
        yParitySigned
      )
      expect(serializeSeismicTx(tx, { r, s, v: BigInt(parity + 27) })).toBe(
        yParitySigned
      )
      expect(yParitySigned).toBe(
        serializeSeismicTransaction(
          { ...tx, type: 'seismic' },
          { r, s, yParity: parity }
        )
      )
    })
  }

  test('explicit yParity takes precedence over v', () => {
    const tx = transaction()
    const { r, s } = signature
    expect(serializeSeismicTx(tx, { r, s, v: 27n, yParity: 1 })).toBe(
      serializeSeismicTx(tx, { r, s, yParity: 1 })
    )
  })

  test('invalid recovery identifiers are rejected', () => {
    const tx = transaction()
    const { r, s } = signature
    expect(() => serializeSeismicTx(tx, { r, s, v: 99n })).toThrow(
      'Invalid Seismic signature'
    )
    expect(() => serializeSeismicTx(tx, { r, s, yParity: 2 })).toThrow(
      'Invalid Seismic signature'
    )
  })

  test('Viem-compatible serializer does not encode ordinary Ethereum transactions as Seismic', () => {
    expect(() =>
      serializeSeismicTx({
        type: 'legacy',
        chainId: 1,
        nonce: 0,
        gasPrice: 1n,
        gas: 21_000n,
        to: zeroAddress,
        value: 0n,
        data: '0x',
      })
    ).toThrow('complete Seismic transaction')
  })

  test('zero and leading-zero scalar fields remain canonical', () => {
    const tx = {
      ...transaction(),
      chainId: 0,
      expiresAtBlock: 0n,
      encryptionNonce: '0x000000000000000000000001' as Hex,
      authorizationList: [
        {
          chainId: 0,
          contractAddress:
            '0x2222222222222222222222222222222222222222' as Address,
          nonce: 0,
          yParity: 0,
          r: toHex(1n, { size: 32 }),
          s: toHex(0n, { size: 32 }),
        },
      ],
    }
    expect(serializeSeismicTx(tx, signature)).toBe(
      serializeSeismicTransaction({ ...tx, type: 'seismic' }, signature)
    )
    const fields = fromRlp(slice(serializeSeismicTx(tx), 1), 'hex')
    expect(fields[0]).toBe('0x')
    expect(fields[8]).toBe('0x01')
    expect(fields[11]).toBe('0x')
    expect(fields[14]).toEqual([
      ['0x', tx.authorizationList[0].contractAddress, '0x', '0x', '0x01', '0x'],
    ])
  })
})

for (const invalid of [
  null,
  [],
  {},
  { type: 'unknown' },
  { type: 'token' },
  { type: 'token', token: zeroAddress },
  { type: 'token', token: '0x11' },
  { type: 'token', token: 1 },
  { type: 'native', token: zeroAddress },
  { type: 'auto', extra: true },
  {
    type: 'token',
    token: payments[2].type === 'token' ? payments[2].token : '',
    extra: true,
  },
]) {
  test(`reject malformed selector ${JSON.stringify(invalid)}`, () => {
    const tx = {
      ...transaction(),
      gasPayment: invalid as unknown as GasPayment,
    }
    expect(() => serializeSeismicTx(tx)).toThrow('Invalid gasPayment')
    expect(() => serializeSeismicTx(tx, signature)).toThrow(
      'Invalid gasPayment'
    )
  })
}

type Vector = {
  tx: {
    chainId: Hex
    nonce: Hex
    gasPrice: Hex
    gas: Hex
    gasPayment: GasPayment
    to?: Address | null
    value: Hex
    input: Hex
    encryptionPubkey: string
    encryptionNonce: Hex
    messageVersion: Hex
    recentBlockHash: Hex
    expiresAtBlock: Hex
    signedRead: boolean
    authorizationList: {
      chainId: Hex
      address: Address
      nonce: Hex
      yParity: Hex
      r: Hex
      s: Hex
    }[]
  }
  unsigned: Hex
  signed: Hex
  signingHash: Hex
  txHash: Hex
  signature: { r: Hex; s: Hex; yParity: Hex }
}
const fixture = JSON.parse(
  readFileSync(
    new URL('../../../../test-vectors/gas-payment.json', import.meta.url),
    'utf8'
  )
) as { vectors: Vector[] }

describe('Rust/standalone gas-payment golden vectors', () => {
  fixture.vectors.forEach((vector, index) => {
    test(`vector ${index}: ${vector.tx.gasPayment.type}, version ${Number(vector.tx.messageVersion)}, nonce ${vector.tx.encryptionNonce}`, () => {
      const tx = vector.tx
      const standalone: SeismicSerializableTransaction = {
        chainId: Number(tx.chainId),
        nonce: Number(tx.nonce),
        gasPrice: BigInt(tx.gasPrice),
        gas: BigInt(tx.gas),
        gasPayment: tx.gasPayment,
        to: tx.to ?? null,
        value: BigInt(tx.value),
        data: tx.input,
        encryptionPubkey: `0x${tx.encryptionPubkey}`,
        encryptionNonce: toHex(BigInt(tx.encryptionNonce), { size: 12 }),
        messageVersion: Number(tx.messageVersion),
        recentBlockHash: tx.recentBlockHash,
        expiresAtBlock: BigInt(tx.expiresAtBlock),
        signedRead: tx.signedRead,
        authorizationList: tx.authorizationList.map((auth) => ({
          chainId: Number(auth.chainId),
          contractAddress: auth.address,
          nonce: Number(auth.nonce),
          yParity: Number(auth.yParity),
          r: auth.r,
          s: auth.s,
        })),
      }
      const unsigned = serializeSeismicTx(standalone)
      const signed = serializeSeismicTx(standalone, {
        r: vector.signature.r,
        s: vector.signature.s,
        v: BigInt(vector.signature.yParity),
      })
      expect(unsigned).toBe(vector.unsigned)
      expect(signed).toBe(vector.signed)
      expect(
        serializeSeismicTx(standalone, {
          r: vector.signature.r,
          s: vector.signature.s,
          yParity: Number(vector.signature.yParity),
        })
      ).toBe(vector.signed)
      expect(keccak256(signed)).toBe(vector.txHash)
      if (standalone.messageVersion === 0) {
        expect(keccak256(unsigned)).toBe(vector.signingHash)
      }
    })
  })
})
