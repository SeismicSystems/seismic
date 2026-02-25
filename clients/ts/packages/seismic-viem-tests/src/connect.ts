import { localSeismicDevnet, sanvil } from 'seismic-viem'
import type { Chain } from 'viem'

export type NodeConnection = {
  chain: Chain
  url: string
  wsUrl: string
}

const chainById: Record<number, Chain> = {
  [sanvil.id]: sanvil as Chain,
  [localSeismicDevnet.id]: localSeismicDevnet as Chain,
}

const deriveWsUrl = (httpUrl: string): string => {
  const u = new URL(httpUrl)
  u.protocol = u.protocol === 'https:' ? 'wss:' : 'ws:'
  return u.toString().replace(/\/$/, '')
}

/**
 * Connect to an already-running node, auto-detecting the chain from its chain ID.
 *
 * @param rpcUrl - HTTP RPC URL. Defaults to `RPC_URL` env var or `http://127.0.0.1:8545`.
 * @returns Chain config, HTTP URL, and WebSocket URL.
 * @throws If the node is unreachable or returns an unknown chain ID.
 */
export const connectToNode = async (
  rpcUrl?: string,
): Promise<NodeConnection> => {
  const url = rpcUrl ?? process.env.RPC_URL ?? 'http://127.0.0.1:8545'

  let chainId: number
  try {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'eth_chainId',
        params: [],
      }),
    })
    const json = (await res.json()) as { result: string }
    chainId = Number(json.result)
  } catch (e) {
    throw new Error(
      `Could not reach node at ${url}. Start a node first (e.g. mise run anvil::start).`,
      { cause: e },
    )
  }

  const chain = chainById[chainId]
  if (!chain) {
    throw new Error(
      `Unknown chain ID ${chainId} from node at ${url}. ` +
        `Supported: ${Object.entries(chainById)
          .map(([id, c]) => `${id} (${c.name})`)
          .join(', ')}`,
    )
  }

  const wsUrl = process.env.WS_URL ?? deriveWsUrl(url)

  return { chain, url, wsUrl }
}
