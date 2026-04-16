import { expect } from 'bun:test'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
} from 'seismic-viem'
import type { Account, Chain } from 'viem'
import { http } from 'viem'

import { ZERO_ADDRESS } from '@sviem-tests/constants.ts'

type NodeParams = {
  chain: Chain
  url: string
}

export const testGetStorageAtThrows = async ({ chain, url }: NodeParams) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  await expect(
    publicClient.getStorageAt({
      address: ZERO_ADDRESS,
      slot: '0x0',
    })
  ).rejects.toThrow('Cannot call getStorageAt with a shielded public client')
}

type SignedCallErrorParams = {
  chain: Chain
  url: string
  account: Account
}

export const testSignedCallWithoutToThrows = async ({
  chain,
  url,
  account,
}: SignedCallErrorParams) => {
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  await expect(
    walletClient.signedCall({
      data: '0xdeadbeef',
      account: account.address,
    })
  ).rejects.toThrow("Signed calls must set 'to' address")
}
