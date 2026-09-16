import { expect } from 'bun:test'
import {
  MIN_RESPONSE_LENGTH,
  RESPONSE_FORMAT_VERSION,
  splitResponseIv,
} from 'seismic-viem'
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
