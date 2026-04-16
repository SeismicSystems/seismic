import { expect, test } from 'bun:test'
import { serializeSeismicTx } from 'seismic-encrypt'
import {
  type TransactionSerializableSeismic,
  serializeSeismicTransaction,
} from 'seismic-viem'
import type { Signature, TransactionSerializableEIP7702 } from 'viem'

// This is a cross-package parity check: seismic-encrypt and seismic-viem
// should serialize the same Seismic tx bytes for the same auth-list input.
test('serializeSeismicTx preserves authorizationList encoding', () => {
  const authorizationList: TransactionSerializableEIP7702['authorizationList'] =
    [
      {
        chainId: 31337,
        contractAddress: '0x2222222222222222222222222222222222222222',
        nonce: 7,
        yParity: 1,
        r: '0x3333333333333333333333333333333333333333333333333333333333333333',
        s: '0x4444444444444444444444444444444444444444444444444444444444444444',
      },
    ]

  const seismicTx: Parameters<typeof serializeSeismicTx>[0] = {
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
    authorizationList,
  }

  const viemSeismicTx: TransactionSerializableSeismic = {
    ...seismicTx,
    type: 'seismic',
  }

  const signature: Signature = {
    v: 27n,
    r: '0x5555555555555555555555555555555555555555555555555555555555555555',
    s: '0x6666666666666666666666666666666666666666666666666666666666666666',
  }

  const encryptedBytes = serializeSeismicTx(seismicTx, signature)
  const viemBytes = serializeSeismicTransaction(viemSeismicTx, signature)

  expect(encryptedBytes).toBe(viemBytes)
})
