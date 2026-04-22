// Seismic chain definitions only. Seismic tx typing/serialization lives under
// `src/tx/`, and this module wires those formatters into the exported chains.
import { defineChain } from 'viem'
import type { Chain } from 'viem'

import { seismicChainFormatters } from '@sviem/tx/seismicRpc.ts'

export type CreateSeismicDevnetParams = {
  nodeHost: string
  explorerUrl?: string
}

export type CreateSeismicTestnetParams = {
  nodeHost: string
  explorerUrl?: string
}

/**
 * Creates a Seismic chain configuration.
 *
 * @param {CreateSeismicDevnetParams} params - The parameters for creating a Seismic chain.
 *   - `nodeHost` (string) - The hostname for the node (e.g. `testnet-1.seismictest.net`).
 *   - `explorerUrl` (string, optional) - Block explorer URL.
 *
 * @throws {Error} Throws if `nodeHost` is not provided.
 *
 * @returns {Chain} A chain configuration object containing:
 *   - Chain ID: 5124.
 *   - Network name: 'Seismic'.
 *   - Native ETH currency configuration.
 *   - RPC URLs (HTTP and WebSocket endpoints).
 *   - Block explorer configuration (if applicable).
 *   - Seismic-specific transaction formatters.
 *
 * @example
 * ```typescript
 * const chain = createSeismicDevnet({ nodeHost: 'testnet-1.seismictest.net' });
 * ```
 */
export const createSeismicDevnet = /*#__PURE__*/ ({
  nodeHost,
  explorerUrl,
}: CreateSeismicDevnetParams): Chain => {
  if (!nodeHost) {
    throw new Error(
      'Must set `nodeHost` argument, e.g. testnet-1.seismictest.net'
    )
  }

  return defineChain({
    id: 5124,
    name: 'Seismic',
    nativeCurrency: { decimals: 18, name: 'Ether', symbol: 'ETH' },
    rpcUrls: {
      default: {
        http: [`https://${nodeHost}/rpc`],
        webSocket: [`wss://${nodeHost}/ws`],
      },
    },
    blockExplorers: explorerUrl
      ? {
          default: {
            name: 'SeismicScan',
            url: explorerUrl,
          },
        }
      : undefined,
    formatters: seismicChainFormatters,
    testnet: true,
  })
}

export const createSeismicAzTestnet = (n: number) =>
  createSeismicDevnet({
    nodeHost: `az-${n}.seismictest.net`,
    explorerUrl: 'https://seismic-testnet.socialscan.io',
  })

export const createSeismicGcpTestnet = (n: number) =>
  createSeismicDevnet({
    nodeHost: `gcp-${n}.seismictest.net`,
    explorerUrl: 'https://seismic-testnet.socialscan.io',
  })

export const createSeismicTestnet = (n: number) =>
  createSeismicDevnet({
    nodeHost: `testnet-${n}.seismictest.net`,
    explorerUrl: 'https://seismic-testnet.socialscan.io',
  })

/**
 * The first seismic testnet
 *
 * Nodes coordinate using summit, Seismic's consensus client
 */

export const seismicTestnetGcp1 = createSeismicGcpTestnet(1)
export const seismicTestnetGcp2 = createSeismicGcpTestnet(2)

export const seismicTestnet1 = createSeismicTestnet(1)
export const seismicTestnet2 = createSeismicTestnet(2)

export const seismicTestnet = seismicTestnet1

/**
 * For connecting to a locally-running seismic-reth instance on --dev mode
 */
export const localSeismicDevnet = /*#__PURE__*/ defineChain({
  id: 5124,
  name: 'Seismic',
  nativeCurrency: { decimals: 18, name: 'Ether', symbol: 'ETH' },
  rpcUrls: {
    default: {
      http: ['http://127.0.0.1:8545'],
      ws: ['ws://127.0.0.1:8546'],
    },
  },
  formatters: seismicChainFormatters,
  testnet: true,
})

/**
 * For connecting to a locally-running seismic anvil instance.
 * Use {@link https://seismic-2.gitbook.io/seismic-book/getting-started/publish-your-docs#sforge-sanvil-and-ssolc sfoundryup}  to install this
 */
export const sanvil = /*#__PURE__*/ defineChain({
  id: 31_337,
  name: 'Anvil',
  nativeCurrency: { decimals: 18, name: 'Ether', symbol: 'ETH' },
  rpcUrls: {
    default: {
      http: ['http://127.0.0.1:8545'],
      webSocket: ['ws://127.0.0.1:8545'],
    },
  },
  formatters: seismicChainFormatters,
  testnet: true,
})
