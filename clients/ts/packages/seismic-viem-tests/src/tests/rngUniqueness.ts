import { expect } from 'bun:test'
import type { Chain } from 'viem'

import { httpPublicClient } from '@sviem-tests/clients.ts'

type PublicClientConfig = {
  chain: Chain
  url: string
}

export const testRngDifferentPersProducesDifferentResults = async ({
  chain,
  url,
}: PublicClientConfig) => {
  const publicClient = httpPublicClient({ chain, url })

  const pers1 = new Uint8Array([1, 2, 3, 4])
  const pers2 = new Uint8Array([5, 6, 7, 8])

  const result1 = await publicClient.rng({ numBytes: 32, pers: pers1 })
  const result2 = await publicClient.rng({ numBytes: 32, pers: pers2 })

  expect(result1).not.toBe(result2)
}
