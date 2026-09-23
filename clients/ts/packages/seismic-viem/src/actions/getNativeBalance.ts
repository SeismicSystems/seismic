import type {
  Address,
  BlockTag,
  Client,
  EIP1193RequestFn,
  GetBalanceParameters,
  Hex,
} from 'viem'
import { numberToHex } from 'viem'

export type GetNativeBalanceParameters = GetBalanceParameters

type NativeBalanceRpcSchema = [
  {
    Method: 'eth_getAccountInfo'
    Parameters: [address: Address, block: Hex | BlockTag]
    ReturnType: { balance: Hex; nonce: Hex; code: Hex }
  },
]

/**
 * Read actual public native funds, never the eth_getBalance compatibility
 * placeholder or an sUSDC-derived balance.
 *
 * Uses the two-argument eth_getAccountInfo endpoint supported by updated reth
 * and Sanvil. Sanvil rejects eth_getBalance's reth-only third argument.
 * Errors are propagated; never fall back to ordinary eth_getBalance.
 */
export const getNativeBalance = async (
  client: Pick<Client, 'request'>,
  { address, blockNumber, blockTag = 'latest' }: GetNativeBalanceParameters
): Promise<bigint> => {
  const request = client.request as EIP1193RequestFn<NativeBalanceRpcSchema>
  const block = blockNumber !== undefined ? numberToHex(blockNumber) : blockTag
  const info = await request({
    method: 'eth_getAccountInfo',
    params: [address, block],
  })
  return BigInt(info.balance)
}
