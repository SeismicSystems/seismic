import { expect } from 'bun:test'
import type { SeismicAuthorization } from 'seismic-viem'
import { encodeAuthorizationList } from 'seismic-viem'
import type { Hex } from 'viem'
import { fromRlp } from 'viem'

const DELEGATE = '0x1111111111111111111111111111111111111111' as const

// r/s as viem's `sign()` returns them: padded to 32 bytes, so a value whose
// top byte is zero carries a leading 0x00.
const R_PADDED =
  '0x00a1b2c3d4e5f60718293a4b5c6d7e8f9011223344556677889900aabbccddee' as Hex
const S_PADDED =
  '0x0000ffeeddccbbaa99887766554433221100ffeeddccbbaa9988776655443322' as Hex

const R_TRIMMED =
  '0xa1b2c3d4e5f60718293a4b5c6d7e8f9011223344556677889900aabbccddee' as Hex
const S_TRIMMED =
  '0xffeeddccbbaa99887766554433221100ffeeddccbbaa9988776655443322' as Hex

const decodeItems = (authorizationList: SeismicAuthorization[]): Hex[][] =>
  fromRlp(encodeAuthorizationList(authorizationList), 'hex') as Hex[][]

export const testAuthorizationRSAreTrimmed = () => {
  const [item] = decodeItems([
    {
      chainId: 31337n,
      contractAddress: DELEGATE,
      nonce: 1n,
      yParity: 1,
      r: R_PADDED,
      s: S_PADDED,
    },
  ])
  expect(item).toEqual([
    '0x7a69',
    DELEGATE,
    '0x01',
    '0x01',
    R_TRIMMED,
    S_TRIMMED,
  ])
}

export const testAuthorizationZeroFieldsEncodeEmpty = () => {
  const [item] = decodeItems([
    {
      chainId: 0n,
      contractAddress: DELEGATE,
      nonce: 0n,
      yParity: 0,
      r: R_PADDED,
      s: S_PADDED,
    },
  ])
  expect(item.slice(0, 4)).toEqual(['0x', DELEGATE, '0x', '0x'])
}

export const testAuthorizationAcceptsAddressField = () => {
  const withContractAddress = encodeAuthorizationList([
    {
      chainId: 1n,
      contractAddress: DELEGATE,
      nonce: 2n,
      yParity: 1,
      r: R_PADDED,
      s: S_PADDED,
    },
  ])
  const withAddress = encodeAuthorizationList([
    {
      chainId: 1n,
      address: DELEGATE,
      nonce: 2n,
      yParity: 1,
      r: R_PADDED,
      s: S_PADDED,
    },
  ])
  expect(withAddress).toBe(withContractAddress)
}

export const testAuthorizationDerivesYParityFromV = () => {
  const [v27] = decodeItems([
    {
      chainId: 1n,
      address: DELEGATE,
      nonce: 0n,
      v: 27n,
      r: R_PADDED,
      s: S_PADDED,
    },
  ])
  const [v28] = decodeItems([
    {
      chainId: 1n,
      address: DELEGATE,
      nonce: 0n,
      v: 28n,
      r: R_PADDED,
      s: S_PADDED,
    },
  ])
  expect(v27[3]).toBe('0x')
  expect(v28[3]).toBe('0x01')
}

export const testAuthorizationWithoutAddressThrows = () => {
  expect(() =>
    encodeAuthorizationList([
      { chainId: 1n, nonce: 0n, yParity: 0, r: R_PADDED, s: S_PADDED },
    ])
  ).toThrow('Seismic authorization requires an address')
}
