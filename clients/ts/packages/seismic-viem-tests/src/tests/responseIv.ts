import { expect } from 'bun:test'
import {
  MIN_RESPONSE_LENGTH,
  RESPONSE_FORMAT_VERSION,
  encryptionActions,
  splitResponseIv,
} from 'seismic-viem'
import type { TxSeismicMetadata } from 'seismic-viem'
import type { Hex } from 'viem'

const VERSION: Hex = '0x01'
const IV: Hex = '0x7da3a99bf0f90d56551d99ea'
const BODY: Hex = `0x${'ef'.repeat(20)}`

const response = (version: Hex, body: Hex): Hex =>
  `${version}${IV.slice(2)}${body.slice(2)}` as Hex

export const testSplitResponseIvSeparatesVersionIvAndBody = () => {
  const { version, iv, body } = splitResponseIv(response(VERSION, BODY))
  expect(version).toBe(RESPONSE_FORMAT_VERSION)
  expect(iv).toBe(IV)
  expect(body).toBe(BODY)
}

export const testSplitResponseIvAcceptsTagOnlyBody = () => {
  const tagOnly = `0x${'cd'.repeat(16)}` as Hex
  const { iv, body } = splitResponseIv(response(VERSION, tagOnly))
  expect(iv).toBe(IV)
  expect(body).toBe(tagOnly)
}

export const testSplitResponseIvRejectsShortResponse = () => {
  const short = `0x${'ab'.repeat(MIN_RESPONSE_LENGTH - 1)}` as Hex
  expect(() => splitResponseIv(short)).toThrow(/shorter than the/)
}

export const testSplitResponseIvRejectsEmptyResponse = () => {
  expect(() => splitResponseIv('0x')).toThrow(/shorter than the/)
}

export const testSplitResponseIvRejectsUnknownVersion = () => {
  const wrong = response('0x02', BODY)
  expect(() => splitResponseIv(wrong)).toThrow(
    /unsupported signed-read response format 2/
  )
}

const RESPONSE_KEY: Hex = `0x${'5a'.repeat(32)}`
const REQUEST_KEY: Hex = `0x${'a5'.repeat(32)}`
const PUBKEY: Hex =
  '0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0'

const metadata = (): TxSeismicMetadata => ({
  sender: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
  legacyFields: {
    chainId: 31337,
    nonce: 0,
    to: '0xd3e8763675e4c425df46cc3b5c0f6cbdac396046',
    value: 0n,
  },
  seismicElements: {
    encryptionPubkey: PUBKEY,
    encryptionNonce: IV,
    messageVersion: 0,
    recentBlockHash: `0x${'11'.repeat(32)}`,
    expiresAtBlock: 100n,
    signedRead: true,
  },
})

const actions = () => encryptionActions(REQUEST_KEY, RESPONSE_KEY, PUBKEY)

/**
 * Guards the layer the SEI-369 truncation bug lived at: a bare `0x` reaching
 * the caller must fail, not decode as an authenticated empty result.
 */
export const testDecryptRejectsBareZeroX = async () => {
  await expect(actions().decrypt('0x', metadata())).rejects.toThrow(
    /shorter than the/
  )
}

export const testDecryptRejectsUndefinedResponse = async () => {
  await expect(actions().decrypt(undefined, metadata())).rejects.toThrow(
    /shorter than the/
  )
}
