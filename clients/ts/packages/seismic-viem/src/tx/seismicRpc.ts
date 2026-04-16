// RPC boundary helpers for Seismic txs. This module adapts the canonical
// in-memory Seismic tx/request shape to viem/JSON-RPC formatting, including the
// auth-list `contractAddress` -> RPC `address` translation.
import { formatTransactionRequest, toHex } from 'viem'
import type {
  BlockIdentifier,
  BlockNumber,
  BlockTag,
  ChainFormatters,
  ExactPartial,
  RpcSchema,
  RpcStateOverride,
  TransactionRequestEIP7702,
} from 'viem'

import {
  SEISMIC_TX_TYPE,
  type SeismicTransactionRequest,
} from '@sviem/tx/seismicTx.ts'

export const estimateGasRpcSchema = {
  Method: 'eth_estimateGas',
  Parameters: ['SeismicTransactionRequest'] as
    | [SeismicTransactionRequest]
    | [SeismicTransactionRequest, BlockNumber]
    | [SeismicTransactionRequest, BlockNumber, RpcStateOverride],
  ReturnType: 'Quantity',
}

export const callRpcSchema = {
  Method: 'eth_call',
  Parameters: ['SeismicTransactionRequest'] as
    | [ExactPartial<SeismicTransactionRequest>]
    | [
        ExactPartial<SeismicTransactionRequest>,
        BlockNumber | BlockTag | BlockIdentifier,
      ]
    | [
        ExactPartial<SeismicTransactionRequest>,
        BlockNumber | BlockTag | BlockIdentifier,
        RpcStateOverride,
      ],
  ReturnType: 'Hex',
}

export const seismicRpcSchema: RpcSchema = [estimateGasRpcSchema, callRpcSchema]

const hasSeismicFields = (request: SeismicTransactionRequest) => {
  return (
    request.encryptionPubkey !== undefined &&
    request.encryptionNonce !== undefined &&
    request.messageVersion !== undefined &&
    request.recentBlockHash !== undefined &&
    request.expiresAtBlock !== undefined &&
    request.signedRead !== undefined
  )
}

const formatAuthorizationList = (
  authorizationList: NonNullable<TransactionRequestEIP7702['authorizationList']>
) =>
  authorizationList.map((authorization) => ({
    address: authorization.contractAddress,
    r: authorization.r,
    s: authorization.s,
    chainId: toHex(authorization.chainId),
    nonce: toHex(authorization.nonce),
    ...(typeof authorization.yParity !== 'undefined'
      ? { yParity: toHex(authorization.yParity) }
      : {}),
    ...(typeof authorization.v !== 'undefined' &&
    typeof authorization.yParity === 'undefined'
      ? { v: toHex(authorization.v) }
      : {}),
  }))

const formatSeismicRpcRequest = (request: SeismicTransactionRequest) => {
  if (request.type === 'seismic') {
    const { authorizationList, ...legacyCompatibleRequest } = request
    const seismicFmt = formatTransactionRequest({
      ...legacyCompatibleRequest,
      type: 'legacy',
    })
    return {
      ...seismicFmt,
      type: SEISMIC_TX_TYPE,
      ...(authorizationList
        ? { authorizationList: formatAuthorizationList(authorizationList) }
        : {}),
    }
  }

  const { type, ...fmt } = formatTransactionRequest(request)
  if (hasSeismicFields(request)) {
    return { ...fmt, type: SEISMIC_TX_TYPE }
  }
  return { ...fmt, type }
}

/**
 * Chain formatters for Seismic transactions, providing formatting utilities for
 * transaction requests.
 * @property {SeismicTransactionRequest} transactionRequest - Formatter
 * configuration for transaction requests
 * @property {Function} transactionRequest.format - Formats a Seismic
 * transaction request into the required RPC format
 * @param {SeismicTransactionRequest} request - The transaction request to be
 * formatted
 * @returns {Object} A formatted transaction request object containing:
 *   - All properties from the formatted RPC request
 *   - `type` (optional) - Set to '0x4a' if encryption public key is present
 *   - `data` (optional) - Transaction data if present
 *   - `encryptionPubkey` (optional) - Public key for transaction encryption
 *   - `chainId` (optional) - Chain ID for the transaction
 * @remarks
 * This function is called by viem's call, estimateGas, and sendTransaction.
 * We can use this to parse transaction request before sending it to the node
 */
export const seismicChainFormatters: ChainFormatters = {
  transactionRequest: {
    format: (request: SeismicTransactionRequest) => {
      // @ts-expect-error: anvil requires chainId to be set but estimateGas
      // doesn't set it
      const chainId = request.chainId
      const formattedRpcRequest = formatSeismicRpcRequest(request)

      if (request.type === 'seismic') {
        if (!request.encryptionNonce) {
          throw new Error(
            'Encryption nonce is required for seismic transactions'
          )
        }
        if (!request.encryptionPubkey) {
          throw new Error(
            'Encryption public key is required for seismic transactions'
          )
        }
        if (request.messageVersion !== 0 && request.messageVersion !== 2) {
          throw new Error(
            'Message version must be set to 0 or 2 for seismic transactions'
          )
        }
        if (!request.recentBlockHash) {
          throw new Error(
            'recentBlockHash is required for seismic transaction requests'
          )
        }
        if (!request.expiresAtBlock) {
          throw new Error(
            'expiresAtBlock is required for seismic transaction requests'
          )
        }
        if (request.signedRead === undefined) {
          throw new Error(
            'signedRead is required for seismic transaction requests'
          )
        }
      }

      return {
        ...formattedRpcRequest,
        chainId,
        encryptionPubkey: request.encryptionPubkey,
        encryptionNonce: request.encryptionNonce,
        messageVersion: request.messageVersion,
        recentBlockHash: request.recentBlockHash,
        expiresAtBlock: request.expiresAtBlock,
        signedRead: request.signedRead,
      }
    },
    type: 'transactionRequest',
  },
}
