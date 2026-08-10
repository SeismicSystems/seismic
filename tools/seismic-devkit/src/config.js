/**
 * Seismic Network Configuration
 * Privacy-enabled EVM blockchain (Chain ID 5124)
 */

export const SEISMIC_CONFIG = {
  network: {
    name: 'Seismic Testnet',
    chainId: 5124,
    chainIdHex: '0x1404',
    currency: {
      name: 'Seismic Ether',
      symbol: 'ETH',
      decimals: 18,
    },
    rpcUrls: [
      process.env.SEISMIC_RPC_URL,
      'https://testnet-1.seismictest.net/rpc',
      'https://gcp-1.seismictest.net/rpc',
      'https://az-1.seismictest.net/rpc',
    ].filter(Boolean),
    wsUrls: [
      'wss://testnet-1.seismictest.net/ws',
      'wss://gcp-1.seismictest.net/ws',
    ],
    explorerUrl: 'https://seismic-testnet.socialscan.io',
  },
  shieldedTypes: [
    { type: 'suint8', description: 'Shielded 8-bit unsigned integer' },
    { type: 'suint32', description: 'Shielded 32-bit unsigned integer' },
    { type: 'suint256', description: 'Shielded 256-bit unsigned integer' },
    { type: 'sbool', description: 'Shielded boolean flag' },
    { type: 'saddress', description: 'Shielded 20-byte EVM address' },
  ],
};
