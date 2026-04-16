import { expect } from 'bun:test'
import { createShieldedPublicClient } from 'seismic-viem'
import type { Chain } from 'viem'
import { http } from 'viem'

type PublicClientConfig = {
  chain: Chain
  url: string
}

/**
 * Call RNG multiple times and verify that all results are unique.
 * This addresses the gap where RNG tests only checked range bounds
 * but didn't verify uniqueness, meaning a constant value could pass.
 */
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

  // All values should be unique
  const uniqueValues = new Set(results.map((v) => v.toString()))
  expect(uniqueValues.size).toBe(iterations)
}

/**
 * Call RNG with personalization and verify uniqueness.
 * Different personalization strings should produce different outputs.
 */
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

  // Different personalizations should yield different random values
  expect(result1).not.toBe(result2)
}
