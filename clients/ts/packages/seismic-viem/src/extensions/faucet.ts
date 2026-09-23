import type { Hex, PublicClient } from 'viem'

import { txExplorerUrl } from '@sviem/explorer.ts'

export type CheckFaucetParams = {
  address: Hex
  publicClient: PublicClient
  faucetUrl: string
}

export type CheckFaucetResult = { sent: true; hash: Hex; txUrl?: string }

const TXHASH_PREFIX = 'Txhash: '
const HASH_HEX_LENGTH = 66

/**
 * Extract a tx hash from a faucet response message of the form
 * "Txhash: 0x..". Returns null when the message has no hash.
 * Throws when the prefix is present but the hash is malformed.
 */
export const parseFaucetResponseHash = (msg: string): Hex | null => {
  if (!msg.startsWith(TXHASH_PREFIX)) {
    return null
  }
  const hash = msg.slice(TXHASH_PREFIX.length)
  if (!hash.startsWith('0x') || hash.length !== HASH_HEX_LENGTH) {
    throw new Error(`Invalid hash from faucet claim: ${hash}`)
  }
  return hash as Hex
}

/**
 * Request funds without inspecting the recipient's balance. Claim eligibility
 * belongs to the faucet server, not the public balance RPC.
 *
 * Requires the legacy POST /api/claim API returning { msg: 'Txhash: 0x...' }.
 * This is not an adapter for the public faucet's authenticated /api/claim/new API.
 * Each invocation attempts a claim; rejection is surfaced as an error.
 */
export const checkFaucet = async ({
  address,
  publicClient,
  faucetUrl,
}: CheckFaucetParams): Promise<CheckFaucetResult> => {
  const response = await fetch(`${faucetUrl}/api/claim`, {
    method: 'POST',
    body: JSON.stringify({ address }),
  })
  if (!response.ok) {
    throw new Error(
      `Faucet request failed with status ${response.status}: ${await response.text()}`
    )
  }
  const { msg } = await response.json()
  const hash = parseFaucetResponseHash(msg)
  if (!hash) {
    throw new Error(`Faucet claim failed: ${msg}`)
  }
  const txUrl = txExplorerUrl({ chain: publicClient.chain, txHash: hash })
  if (txUrl) {
    console.debug(`Faucet sent funds to ${address}: ${txUrl}`)
  }
  // only return after the tx is confirmed, to prevent double-requesting
  await publicClient.waitForTransactionReceipt({ hash })
  return { sent: true, hash, txUrl: txUrl ?? undefined }
}
