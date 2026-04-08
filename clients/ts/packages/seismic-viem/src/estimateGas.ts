/**
 * Signed gas estimation for shielded transactions.
 *
 * Signs a temporary tx (using the block gas limit as placeholder)
 * and sends the signed bytes to {@link eth_estimateGas} so the node
 * can authenticate the sender.  This prevents caller-spoofing
 * attacks against `msg.sender`-gated private state.
 */
import type { Account, Chain, Hex, Transport } from 'viem'

import type { TransactionSerializableSeismic } from '@sviem/chain.ts'
import { serializeSeismicTransaction } from '@sviem/chain.ts'
import type { ShieldedWalletClient } from '@sviem/client.ts'
import type { TxSeismicMetadata } from '@sviem/metadata.ts'
import {
  TYPED_DATA_MESSAGE_VERSION,
  signSeismicTxTypedData,
} from '@sviem/signSeismicTypedData.ts'

export async function estimateShieldedGas<
  TTransport extends Transport,
  TChain extends Chain | undefined,
  TAccount extends Account,
>(
  client: ShieldedWalletClient<TTransport, TChain, TAccount>,
  {
    encryptedData,
    metadata,
    gasPrice,
  }: {
    encryptedData: Hex
    metadata: TxSeismicMetadata
    gasPrice: bigint
  }
): Promise<bigint> {
  const block = await client.getBlock({ blockTag: 'latest' })
  const blockGasLimit = block.gasLimit

  const tempTx: TransactionSerializableSeismic = {
    type: 'seismic',
    chainId: metadata.legacyFields.chainId,
    nonce: metadata.legacyFields.nonce,
    gasPrice,
    gas: blockGasLimit,
    to: metadata.legacyFields.to ?? undefined,
    value: metadata.legacyFields.value,
    data: encryptedData,
    ...metadata.seismicElements,
  }

  const isEip712 =
    metadata.seismicElements.messageVersion === TYPED_DATA_MESSAGE_VERSION

  if (isEip712) {
    const { typedData, signature } = await signSeismicTxTypedData(
      client,
      tempTx
    )
    const response: Hex = await client.request({
      method: 'eth_estimateGas',
      params: [{ data: typedData, signature }],
    })
    return BigInt(response)
  }

  const serializedTransaction = await client.account!.signTransaction!(tempTx, {
    serializer: serializeSeismicTransaction,
  })

  const response: Hex = await client.request({
    method: 'eth_estimateGas',
    params: [serializedTransaction],
  })
  return BigInt(response)
}
