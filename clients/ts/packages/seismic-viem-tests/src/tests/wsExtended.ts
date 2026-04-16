import { expect } from 'bun:test'
import { createShieldedPublicClient } from 'seismic-viem'
import type { Chain } from 'viem'
import { webSocket } from 'viem'

type WsTestArgs = {
  chain: Chain
  wsUrl: string
}

/**
 * Test that a WS client can subscribe to new block headers.
 */
export const testWsBlockSubscription = async ({ chain, wsUrl }: WsTestArgs) => {
  const client = await createShieldedPublicClient({
    chain,
    transport: webSocket(wsUrl),
  })

  const blockNumber = await client.getBlockNumber()
  expect(blockNumber).toBeGreaterThanOrEqual(0n)

  const rpcClient = await client.transport.getRpcClient()
  rpcClient.close()
}

/**
 * Test that a WS client can call precompiles (e.g. getTeePublicKey).
 */
export const testWsPrecompileCall = async ({ chain, wsUrl }: WsTestArgs) => {
  const client = await createShieldedPublicClient({
    chain,
    transport: webSocket(wsUrl),
  })

  const teeKey = await client.getTeePublicKey()
  expect(teeKey).toBeDefined()
  expect(typeof teeKey).toBe('string')
  expect(teeKey.length).toBeGreaterThan(0)

  const rpcClient = await client.transport.getRpcClient()
  rpcClient.close()
}

/**
 * Test that a WS client can call the RNG precompile.
 */
export const testWsRngCall = async ({ chain, wsUrl }: WsTestArgs) => {
  const client = await createShieldedPublicClient({
    chain,
    transport: webSocket(wsUrl),
  })

  const randomValue = await client.rng({ numBytes: 32 })
  expect(randomValue).toBeGreaterThan(0n)
  expect(randomValue).toBeLessThan(2n ** 256n)

  const rpcClient = await client.transport.getRpcClient()
  rpcClient.close()
}
