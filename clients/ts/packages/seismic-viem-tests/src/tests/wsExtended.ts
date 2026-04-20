import { expect } from 'bun:test'
import { createShieldedPublicClient } from 'seismic-viem'
import type { Chain } from 'viem'
import { webSocket } from 'viem'

type WsTestArgs = {
  chain: Chain
  wsUrl: string
}

const RNG_SIZE_BYTES = 32

export const testWsBlockSubscription = async ({ chain, wsUrl }: WsTestArgs) => {
  const client = await createShieldedPublicClient({
    chain,
    transport: webSocket(wsUrl),
  })

  // If the WS transport weren't wired up this would reject or hang;
  // the assertion proves we got a numeric block height back.
  const blockNumber = await client.getBlockNumber()
  expect(typeof blockNumber).toBe('bigint')

  const rpcClient = await client.transport.getRpcClient()
  rpcClient.close()
}

export const testWsPrecompileCall = async ({ chain, wsUrl }: WsTestArgs) => {
  const client = await createShieldedPublicClient({
    chain,
    transport: webSocket(wsUrl),
  })

  const teeKey = await client.getTeePublicKey()
  expect(teeKey.length).toBeGreaterThan(0)

  const rpcClient = await client.transport.getRpcClient()
  rpcClient.close()
}

export const testWsRngCall = async ({ chain, wsUrl }: WsTestArgs) => {
  const client = await createShieldedPublicClient({
    chain,
    transport: webSocket(wsUrl),
  })

  const randomValue = await client.rng({ numBytes: RNG_SIZE_BYTES })
  expect(randomValue).toBeLessThan(2n ** BigInt(8 * RNG_SIZE_BYTES))

  const rpcClient = await client.transport.getRpcClient()
  rpcClient.close()
}
