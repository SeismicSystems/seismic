import { Chain } from 'viem'

import { wsPublicClient } from '@sviem-tests/clients.ts'

export const testWsConnection = async ({
  chain,
  wsUrl,
}: {
  chain: Chain
  wsUrl: string
}) => {
  const client = wsPublicClient({ chain, wsUrl })
  await client.getTeePublicKey()
  const rpcClient = await client.transport.getRpcClient()
  rpcClient.close()
}
