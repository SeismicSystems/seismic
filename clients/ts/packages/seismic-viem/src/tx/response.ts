// Lookup responses are distinct from signing inputs: quantities are formatted
// like viem quantities, while the signed gas-payment choice remains public.
import type {
  Block,
  Hash,
  Hex,
  RpcBlock,
  RpcTransaction,
  Transaction,
  TransactionBase,
  TransactionEIP7702,
} from 'viem'
import { formatBlock, formatTransaction } from 'viem'

import type { GasPayment } from '@sviem/tx/gasPayment.ts'
import { normalizeGasPayment } from '@sviem/tx/gasPayment.ts'

/** A formatted Seismic transaction, including its signed payment preference. */
export type SeismicTransaction<
  quantity = bigint,
  index = number,
  isPending extends boolean = boolean,
> = TransactionBase<quantity, index, isPending> & {
  type: 'seismic'
  typeHex: '0x4a' | '0x4A'
  chainId: index
  gasPrice: quantity
  gasPayment: GasPayment
  encryptionPubkey: Hex
  encryptionNonce: quantity
  messageVersion: index
  recentBlockHash: Hash
  expiresAtBlock: quantity
  signedRead: boolean
  authorizationList?: TransactionEIP7702['authorizationList']
}

/** Existing lookup APIs can return either Seismic or ordinary Ethereum txs. */
export type SeismicTransactionResponse =
  | SeismicTransaction
  | (Transaction & { gasPayment?: undefined })

/** JSON-RPC quantities stay hex; secp256k1's public-key serde omits the prefix. */
export type RpcSeismicTransaction = Omit<
  SeismicTransaction<Hex, Hex>,
  'type' | 'typeHex' | 'encryptionPubkey' | 'authorizationList' | 'yParity'
> & {
  type: '0x4a' | '0x4A'
  encryptionPubkey: string
  yParity?: Hex
  authorizationList?: RpcTransaction['authorizationList']
}

export type RpcSeismicTransactionResponse =
  | RpcTransaction
  | RpcSeismicTransaction

type RpcSeismicBlock = Omit<RpcBlock, 'transactions'> & {
  transactions: (Hash | RpcSeismicTransactionResponse)[]
}

type SeismicBlock = Omit<Block, 'transactions'> & {
  transactions: (Hash | SeismicTransactionResponse)[]
}

export const formatSeismicTransactionResponse = (
  transaction: RpcSeismicTransactionResponse
): SeismicTransactionResponse => {
  // viem handles shared quantities, pending fields, signatures and EIP-7702.
  // Its standard type-name table does not include 0x4a; override that below.
  const formatted = formatTransaction(transaction as RpcTransaction)
  if (transaction.type !== '0x4a' && transaction.type !== '0x4A')
    return formatted
  if (transaction.gasPayment === undefined) {
    throw new Error('Missing gasPayment in Seismic transaction response')
  }
  if (formatted.yParity === undefined) {
    throw new Error('Missing signature parity in Seismic transaction response')
  }
  return {
    ...formatted,
    type: 'seismic',
    typeHex: transaction.type,
    chainId: Number(transaction.chainId),
    gasPrice: BigInt(transaction.gasPrice),
    yParity: formatted.yParity,
    gasPayment: normalizeGasPayment(transaction.gasPayment),
    encryptionPubkey: `0x${transaction.encryptionPubkey.replace(/^0x/, '')}`,
    encryptionNonce: BigInt(transaction.encryptionNonce),
    messageVersion: Number(transaction.messageVersion),
    recentBlockHash: transaction.recentBlockHash,
    expiresAtBlock: BigInt(transaction.expiresAtBlock),
    signedRead: transaction.signedRead,
  }
}

export const formatSeismicBlockResponse = (
  block: RpcSeismicBlock
): SeismicBlock => ({
  ...formatBlock({ ...block, transactions: [] }),
  transactions: (block.transactions ?? []).map((transaction) =>
    typeof transaction === 'string'
      ? transaction
      : formatSeismicTransactionResponse(transaction)
  ),
})
