import { expect } from 'bun:test'
import {
  createSeismicDevnet,
  localSeismicDevnet,
  sanvil,
  seismicTestnet,
} from 'seismic-viem'

// viem reads WebSocket URLs from `rpcUrls.default.webSocket`; a chain that
// spells the key differently silently has no WebSocket endpoint.
export const testChainsExposeWebSocketUrls = () => {
  expect(localSeismicDevnet.rpcUrls.default.webSocket).toEqual([
    'ws://127.0.0.1:8546',
  ])
  expect(sanvil.rpcUrls.default.webSocket).toEqual(['ws://127.0.0.1:8545'])
  expect(seismicTestnet.rpcUrls.default.webSocket).toEqual([
    'wss://testnet-1.seismictest.net/ws',
  ])
  expect(
    createSeismicDevnet({ nodeHost: 'node.example' }).rpcUrls.default.webSocket
  ).toEqual(['wss://node.example/ws'])
}
