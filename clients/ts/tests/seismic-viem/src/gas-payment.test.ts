import { describe, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import {
  createWalletClient,
  custom,
  fromRlp,
  hashTypedData,
  keccak256,
  recoverAddress,
  slice,
  toHex,
  zeroAddress,
} from 'viem'
import type { Address, Hex, Signature } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

import { testSignedCallBlockSelection } from '@sviem-tests/tests/signedCallBlockSelection.ts'
import { encodeSeismicMetadataAsAAD } from '@sviem/crypto/aead.ts'
import {
  assertAutoGasPayment,
  normalizeGasPayment,
} from '@sviem/tx/gasPayment.ts'
import type { GasPayment } from '@sviem/tx/gasPayment.ts'
import { seismicChainFormatters } from '@sviem/tx/seismicRpc.ts'
import { serializeSeismicTransaction } from '@sviem/tx/seismicTx.ts'
import type { TransactionSerializableSeismic } from '@sviem/tx/seismicTx.ts'
import { signSeismicTxTypedData } from '@sviem/tx/signSeismicTypedData.ts'

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
  typedData: { types: Record<string, { name: string; type: string }[]> }
}
const fixture = JSON.parse(
  readFileSync(
    new URL('../../../../test-vectors/gas-payment.json', import.meta.url),
    'utf8'
  )
) as { privateKey: Hex; vectors: Vector[] }
const signer = privateKeyToAccount(fixture.privateKey)
const client = createWalletClient({
  account: signer,
  transport: custom({
    request: async () => {
      throw new Error('Golden-vector signing must not use RPC')
    },
  }),
})

function transaction(
  vector: Vector
): Extract<TransactionSerializableSeismic, { type: 'seismic' }> {
  const tx = vector.tx
  return {
    type: 'seismic',
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
}

const choices: GasPayment[] = [
  { type: 'auto' },
  { type: 'native' },
  { type: 'token', token: `0x${'11'.repeat(20)}` },
]

describe('Rust/TS gas-payment golden vectors', () => {
  fixture.vectors.forEach((vector, index) => {
    test(`vector ${index}: ${vector.tx.gasPayment.type}, version ${Number(vector.tx.messageVersion)}, nonce ${vector.tx.encryptionNonce}, ${vector.tx.to ? 'call' : 'create/auth'}`, async () => {
      const tx = transaction(vector)
      const unsigned = serializeSeismicTransaction(tx)
      expect(unsigned).toBe(vector.unsigned)
      const signature: Signature = {
        r: vector.signature.r,
        s: vector.signature.s,
        yParity: Number(vector.signature.yParity) as 0 | 1,
      }
      expect(serializeSeismicTransaction(tx, signature)).toBe(vector.signed)
      expect(keccak256(vector.signed)).toBe(vector.txHash)
      let digest: Hex
      let changedDigest: Hex
      const changed = {
        ...tx,
        gasPayment: choices[tx.gasPayment?.type === 'auto' ? 1 : 0],
      }
      if (Number(vector.tx.messageVersion) === 2) {
        const signed = await signSeismicTxTypedData(client, tx)
        expect(signed.typedData.types).toEqual(vector.typedData.types)
        expect(signed.signature).toEqual({
          r: toHex(BigInt(vector.signature.r), { size: 32 }),
          s: toHex(BigInt(vector.signature.s), { size: 32 }),
          yParity: toHex(Number(vector.signature.yParity)),
        })
        digest = hashTypedData(
          signed.typedData as Parameters<typeof hashTypedData>[0]
        )
        changedDigest = hashTypedData(
          (await signSeismicTxTypedData(client, changed))
            .typedData as Parameters<typeof hashTypedData>[0]
        )
      } else {
        digest = keccak256(unsigned)
        expect(
          String(
            await signer.signTransaction(tx, {
              serializer: serializeSeismicTransaction,
            })
          )
        ).toBe(vector.signed)
        changedDigest = keccak256(serializeSeismicTransaction(changed))
      }
      expect(digest).toBe(vector.signingHash)
      expect(
        (await recoverAddress({ hash: digest, signature })).toLowerCase()
      ).toBe(signer.address.toLowerCase())
      expect(
        (await recoverAddress({ hash: changedDigest, signature })).toLowerCase()
      ).not.toBe(signer.address.toLowerCase())
      // AAD remains selector-free, and retains its fixed-width nonce encoding.
      const metadata = {
        sender: signer.address,
        legacyFields: {
          chainId: Number(vector.tx.chainId),
          nonce: Number(vector.tx.nonce),
          to: tx.to ?? null,
          value: BigInt(vector.tx.value),
        },
        seismicElements: {
          encryptionPubkey: tx.encryptionPubkey!,
          encryptionNonce: tx.encryptionNonce!,
          messageVersion: Number(vector.tx.messageVersion),
          recentBlockHash: vector.tx.recentBlockHash,
          expiresAtBlock: BigInt(vector.tx.expiresAtBlock),
          signedRead: vector.tx.signedRead,
        },
      }
      const aad = encodeSeismicMetadataAsAAD({
        ...metadata,
        gasPayment: tx.gasPayment,
      } as typeof metadata)
      const changedAad = encodeSeismicMetadataAsAAD({
        ...metadata,
        gasPayment: changed.gasPayment,
      } as typeof metadata)
      expect(aad).toEqual(changedAad)
      const decoded = fromRlp(toHex(aad), 'hex')
      expect(decoded).toHaveLength(11)
      expect(decoded[6]).toBe(tx.encryptionNonce!)
    })
  })
})

test('omission is Auto before signing, never an omitted wire field', () => {
  const tx = transaction(fixture.vectors[0])
  const { gasPayment: _payment, ...omitted } = tx
  expect(serializeSeismicTransaction(omitted)).toBe(fixture.vectors[0].unsigned)
  expect(
    fromRlp(slice(serializeSeismicTransaction(omitted), 1), 'hex')[4]
  ).toEqual(['0x', '0x'])
})

for (const invalid of [
  null,
  [],
  {},
  { type: 'unknown' },
  { type: 'token' },
  { type: 'token', token: zeroAddress },
  { type: 'token', token: '0x11' },
  { type: 'native', token: zeroAddress },
  { type: 'auto', extra: true },
  {
    type: 'token',
    token: choices[2].type === 'token' ? choices[2].token : zeroAddress,
    extra: true,
  },
]) {
  test(`reject malformed gasPayment ${JSON.stringify(invalid)}`, () =>
    expect(() => normalizeGasPayment(invalid)).toThrow())
}

test('transparent Ethereum requests retain their format and reject explicit choices', () => {
  assertAutoGasPayment()
  assertAutoGasPayment({ type: 'auto' })
  for (const choice of choices.slice(1))
    expect(() => assertAutoGasPayment(choice)).toThrow(
      'requires a Seismic transaction'
    )
  const format = seismicChainFormatters.transactionRequest!.format
  const standard = {
    type: 'legacy' as const,
    to: signer.address,
    value: 1n,
    nonce: 0,
  }
  expect(format({ ...standard, gasPayment: { type: 'auto' } })).toEqual(
    format(standard)
  )
  expect(() =>
    // @ts-expect-error: explicit payment choices require a Seismic envelope
    format({ ...standard, gasPayment: { type: 'native' } })
  ).toThrow()
  const tx = transaction(fixture.vectors[1])
  expect(format(tx)).toMatchObject({ gasPayment: { type: 'auto' } })
})

for (const mode of ['local', 'json-rpc', 'raw'] as const) {
  for (const gasPayment of choices) {
    test(`${mode} signed call forwards ${gasPayment.type}`, () =>
      testSignedCallBlockSelection(mode, {}, 'latest', 0, gasPayment))
  }
}
