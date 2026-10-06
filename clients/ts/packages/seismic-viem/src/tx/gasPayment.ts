import type { Address, Hex } from 'viem'
import { getAddress, isAddress, zeroAddress } from 'viem'

/** Public, signed fee selection. Token selection never falls back. */
export type GasPayment =
  | { type: 'auto' }
  | { type: 'native' }
  | { type: 'token'; token: Address }

export type GasPaymentOptions = { gasPayment?: GasPayment }

/** Resolve request omission before signing, and reject malformed selectors. */
export function normalizeGasPayment(value: unknown = undefined): GasPayment {
  if (value === undefined) return { type: 'auto' }
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error('Invalid gasPayment: expected a tagged object')
  }
  const fields = value as Record<string, unknown>
  const keys = Object.keys(fields)
  if (!Object.prototype.hasOwnProperty.call(fields, 'type')) {
    throw new Error('Invalid gasPayment: missing type')
  }
  if (fields.type === 'auto' || fields.type === 'native') {
    if (keys.length !== 1) {
      throw new Error(
        'Invalid gasPayment: auto/native cannot carry extra fields'
      )
    }
    return { type: fields.type }
  }
  if (
    fields.type === 'token' &&
    keys.length === 2 &&
    Object.prototype.hasOwnProperty.call(fields, 'token') &&
    typeof fields.token === 'string' &&
    isAddress(fields.token, { strict: false }) &&
    fields.token.toLowerCase() !== zeroAddress
  ) {
    return { type: 'token', token: getAddress(fields.token) }
  }
  throw new Error('Invalid gasPayment: expected a nonzero token address')
}

/** Canonical mandatory nested RLP field: [kind, token bytes]. */
export function gasPaymentRlp(value: GasPayment): Hex[] {
  const payment = normalizeGasPayment(value)
  switch (payment.type) {
    case 'auto':
      return ['0x', '0x']
    case 'native':
      return ['0x01', '0x']
    case 'token':
      return ['0x02', payment.token]
  }
}

/** EIP-712 uses a zero-address sentinel, unlike RLP's empty token bytes. */
export function gasPaymentTypedData(value: GasPayment): {
  kind: number
  token: Address
} {
  const payment = normalizeGasPayment(value)
  return {
    kind: payment.type === 'auto' ? 0 : payment.type === 'native' ? 1 : 2,
    token: payment.type === 'token' ? payment.token : zeroAddress,
  }
}

/** Standard Ethereum envelopes cannot authenticate an explicit fee selection. */
export function assertAutoGasPayment(value: unknown = undefined): void {
  if (normalizeGasPayment(value).type !== 'auto') {
    throw new Error(
      'Non-Auto gasPayment requires a Seismic transaction. Use swrite or sendShieldedTransaction.'
    )
  }
}
