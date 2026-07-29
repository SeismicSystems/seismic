import {
  createShieldedPublicClient,
  createShieldedWalletClient,
  localSeismicDevnet,
  sanvil,
} from 'seismic-viem'
import type { ShieldedPublicClient, ShieldedWalletClient } from 'seismic-viem'
import type {
  Account,
  Chain,
  Hex,
  HttpTransport,
  WebSocketTransport,
} from 'viem'
import { http, webSocket } from 'viem'

/**
 * Receipt-polling interval for clients talking to a local dev node.
 *
 * Local nodes (sanvil, seismic-reth in dev mode) include a tx in a block
 * near-instantly, so receipts are ready long before viem's default 4s
 * polling tick fires — with the default, every receipt wait costs a full
 * tick and dominates integration-suite wall-clock time.
 */
export const LOCAL_POLLING_INTERVAL_MS = 100

const LOCAL_CHAIN_IDS: Set<number> = new Set([sanvil.id, localSeismicDevnet.id])

/** Fast polling on local dev chains; viem's default (4s) everywhere else,
 * so runs against shared endpoints (e.g. a live testnet) stay polite. */
const pollingInterval = (chain: Chain): number | undefined =>
  LOCAL_CHAIN_IDS.has(chain.id) ? LOCAL_POLLING_INTERVAL_MS : undefined

export type PublicClientArgs = {
  chain: Chain
  url: string
}

export type WsPublicClientArgs = {
  chain: Chain
  wsUrl: string
}

export type WalletClientArgs = PublicClientArgs & {
  account: Account
  encryptionSk?: Hex
}

/** Shielded public client over HTTP, tuned for the target chain. */
export const httpPublicClient = ({
  chain,
  url,
}: PublicClientArgs): ShieldedPublicClient<HttpTransport, Chain> =>
  createShieldedPublicClient({
    chain,
    transport: http(url),
    pollingInterval: pollingInterval(chain),
  })

/** Shielded public client over WebSocket, tuned for the target chain. */
export const wsPublicClient = ({
  chain,
  wsUrl,
}: WsPublicClientArgs): ShieldedPublicClient<WebSocketTransport, Chain> =>
  createShieldedPublicClient({
    chain,
    transport: webSocket(wsUrl),
    pollingInterval: pollingInterval(chain),
  })

/** Shielded wallet client over HTTP, tuned for the target chain. */
export const httpWalletClient = ({
  chain,
  url,
  account,
  encryptionSk,
}: WalletClientArgs): Promise<
  ShieldedWalletClient<HttpTransport, Chain, Account>
> =>
  createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
    encryptionSk,
    pollingInterval: pollingInterval(chain),
  })
