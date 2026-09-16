import { expect } from 'bun:test'
import { createShieldedWalletClient, sanvil, signedCall } from 'seismic-viem'
import type { Hex } from 'viem'
import { custom } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

const ACCOUNT_PK: Hex =
  '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80'
const ENCRYPTION_SK: Hex =
  '0xa30363336e1bb949185292a2a302de86e447d98f3a43d823c8c234d9e3e5ad77'
const TEE_PK =
  '0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0'
const TARGET: Hex = '0xd3e8763675e4c425df46cc3b5c0f6cbdac396046'
const BLOCK_HASH = `0x${'11'.repeat(32)}` as Hex

const seen: string[] = []

const mockTransport = (ethCallResult: Hex) =>
  custom({
    request: async ({
      method,
      params,
    }: {
      method: string
      params?: unknown
    }) => {
      seen.push(method)
      switch (method) {
        case 'seismic_getTeePublicKey':
          return TEE_PK
        case 'eth_chainId':
          return '0x7a69'
        case 'eth_blockNumber':
          return '0xa'
        case 'eth_getTransactionCount':
          return '0x0'
        case 'eth_gasPrice':
          return '0x3b9aca00'
        case 'eth_estimateGas':
          return '0x5208'
        case 'eth_getBlockByNumber':
          return { number: '0xa', hash: BLOCK_HASH, baseFeePerGas: null }
        case 'eth_call':
          return ethCallResult
        default:
          throw new Error(`unmocked RPC: ${method} ${JSON.stringify(params)}`)
      }
    },
  })

const makeClient = async (ethCallResult: Hex) =>
  await createShieldedWalletClient({
    chain: sanvil,
    transport: mockTransport(ethCallResult),
    account: privateKeyToAccount(ACCOUNT_PK),
    encryptionSk: ENCRYPTION_SK,
  })

/**
 * Guards the layer the SEI-369 truncation bug lived at: a bare `0x` reaching
 * signedCall must fail, not decode as an authenticated empty result.
 */
export const testSignedCallRejectsBareZeroX = async () => {
  const client = await makeClient('0x')
  await expect(signedCall(client, { to: TARGET, data: '0x' })).rejects.toThrow(
    /shorter than the/
  )
}
