import { expect } from 'bun:test'
import { createShieldedWalletClient, sanvil, signedCall } from 'seismic-viem'
import type {
  Account,
  BlockTag,
  GetBalanceParameters,
  Hex,
  TypedDataDefinition,
  UnionOmit,
} from 'viem'
import { custom, numberToHex } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

import {
  ENCRYPTION_PK,
  ENCRYPTION_SK,
  TEST_ACCOUNT_PRIVATE_KEY,
} from '@sviem-tests/constants.ts'

export const testSignedCallBlockSelection = async (
  mode: 'local' | 'json-rpc' | 'raw',
  selector: UnionOmit<GetBalanceParameters, 'address'>,
  expectedBlock: Hex | BlockTag
) => {
  const signer = privateKeyToAccount(TEST_ACCOUNT_PRIVATE_KEY)
  // The raw fallback needs a signer outside the local/json-rpc branches.
  // This minimal fixture exercises serialization, not smart-account support.
  const account: Account =
    mode === 'raw'
      ? ({ ...signer, type: 'smart' } as unknown as Account)
      : mode === 'json-rpc'
        ? { address: signer.address, type: 'json-rpc' }
        : signer
  const calls: { method: string; params?: readonly unknown[] }[] = []
  const transport = custom(
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
          case 'eth_signTypedData_v4':
            expect(String(params[0]).toLowerCase()).toBe(
              signer.address.toLowerCase()
            )
            return signer.signTypedData(
              JSON.parse(params[1]) as TypedDataDefinition
            )
          case 'eth_call':
            return '0x'
          default:
            throw new Error(`Unexpected RPC method: ${method}`)
        }
      },
    },
    { retryCount: 0 }
  )
  const client = await createShieldedWalletClient({
    account,
    chain: sanvil,
    transport,
    encryptionSk: ENCRYPTION_SK,
    cacheTime: 0,
  })

  expect(
    await signedCall(
      client,
      {
        to: signer.address,
        data: '0x12345678',
        gas: 100_000n,
        gasPrice: 1n,
        nonce: 0,
        ...selector,
      },
      {
        recentBlockHash: `0x${'11'.repeat(32)}`,
        expiresAtBlock: 100n,
      }
    )
  ).toEqual({ data: undefined })

  const callRequests = calls.filter(({ method }) => method === 'eth_call')
  expect(callRequests).toHaveLength(1)
  expect(callRequests[0].params).toHaveLength(2)
  expect(callRequests[0].params?.[1]).toBe(expectedBlock)
  const envelope = callRequests[0].params?.[0]
  if (mode === 'raw') {
    expect(typeof envelope).toBe('string')
    expect(String(envelope).startsWith('0x4a')).toBe(true)
  } else {
    expect(envelope).toHaveProperty('data')
    expect(envelope).toHaveProperty('signature')
  }
  expect(
    calls.filter(({ method }) => method === 'eth_signTypedData_v4')
  ).toHaveLength(mode === 'json-rpc' ? 1 : 0)
}
