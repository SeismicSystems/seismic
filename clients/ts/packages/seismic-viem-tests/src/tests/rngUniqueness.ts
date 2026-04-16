import { expect } from 'bun:test'
import { createShieldedPublicClient } from 'seismic-viem'
import type { Chain } from 'viem'
import { http } from 'viem'

type PublicClientConfig = {
  chain: Chain
  url: string
}

export const testRngUniqueness = async (
  { chain, url }: PublicClientConfig,
  size: number,
  iterations: number = 5
) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })

  const results: bigint[] = []
  for (let i = 0; i < iterations; i++) {
    const value = await publicClient.rng({ numBytes: size })
    expect(value).toBeGreaterThan(0n)
    expect(value).toBeLessThan(2n ** BigInt(8 * size))
    results.push(value)
  }

  const uniqueValues = new Set(results.map((v) => v.toString()))
  expect(uniqueValues.size).toBe(iterations)
}

export const testRngDifferentPersProducesDifferentResults = async ({
  chain,
  url,
}: PublicClientConfig) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })

  const pers1 = new Uint8Array([1, 2, 3, 4])
  const pers2 = new Uint8Array([5, 6, 7, 8])

  const result1 = await publicClient.rng({ numBytes: 32, pers: pers1 })
  const result2 = await publicClient.rng({ numBytes: 32, pers: pers2 })

  expect(result1).not.toBe(result2)
}
