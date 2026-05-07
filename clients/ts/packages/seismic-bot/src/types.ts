import type { Address, Chain, Hex } from 'viem'

export type Key = { pk: Hex; silent?: boolean }

export type FaucetConfig = {
  chain: Chain
  privateKeys: Key[]
  extraAddresses: Address[]
}

export type Faucets = Record<string, FaucetConfig>
