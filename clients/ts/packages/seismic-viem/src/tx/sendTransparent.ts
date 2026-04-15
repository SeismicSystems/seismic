import type {
  Account,
  BaseError,
  Chain,
  SendTransactionParameters,
  SendTransactionReturnType,
  Transport,
} from 'viem'
import { parseAccount } from 'viem/accounts'
import {
  prepareTransactionRequest,
  sendRawTransaction,
  sendTransaction as viemSendTransaction,
} from 'viem/actions'
import { assertRequest, getAction, getTransactionError } from 'viem/utils'

import { ShieldedWalletClient } from '@sviem/client.ts'
import { AccountNotFoundError } from '@sviem/error/account.ts'

const DEFAULT_SIGNED_ESTIMATE_GAS_LIMIT = 30_000_000n

/**
 * Executes the transparent transaction path for Seismic wallet clients.
 *
 * This is still an Ethereum-style send, but Seismic needs authenticated gas
 * estimation for some transactions. When the account is `local`, the client can
 * sign a provisional raw transaction and send those bytes to `eth_estimateGas`
 * so the node simulates execution with the real caller context.
 *
 * For `json-rpc` accounts, the SDK does not have the signing key locally, so
 * this falls back to viem's standard `sendTransaction` flow and its unsigned
 * gas-estimation behavior.
 */
export async function sendTransparentTransaction<
  TChain extends Chain | undefined,
  TAccount extends Account,
  TChainOverride extends Chain | undefined = undefined,
>(
  client: ShieldedWalletClient<Transport, TChain, TAccount>,
  parameters: SendTransactionParameters<TChain, TAccount, TChainOverride>
): Promise<SendTransactionReturnType> {
  const {
    account: account_ = client.account,
    chain = client.chain,
    accessList,
    authorizationList,
    blobs,
    data,
    gas,
    gasPrice,
    maxFeePerBlobGas,
    maxFeePerGas,
    maxPriorityFeePerGas,
    nonce,
    value,
    ...rest
  } = parameters
  if (typeof account_ === 'undefined')
    throw new AccountNotFoundError({
      docsPath: '/docs/actions/wallet/sendTransaction',
    })
  const account = account_ ? parseAccount(account_) : null

  try {
    assertRequest(parameters)

    const to = await (async () => {
      if (parameters.to) return parameters.to
      return undefined
    })()

    // Only `local` accounts can do signed transparent gas estimation because
    // the SDK can produce a provisional signed raw transaction locally.
    // For `json-rpc` accounts (e.g. external wallets), we do not have the
    // private key in-process, so we currently fall back to viem's standard
    // unsigned `sendTransaction` behavior.
    if (account?.type !== 'local') {
      return await viemSendTransaction(client, parameters)
    }

    // Fill nonce / fees / type using viem, but intentionally skip viem's gas
    // estimation step. On Seismic, unsigned `eth_estimateGas` sanitizes caller
    // context (notably `from`, and for some request shapes related fields), so
    // unsigned estimation is not equivalent to the final authenticated tx.
    const request = await prepareTransactionRequest(client, {
      account,
      accessList,
      authorizationList,
      blobs,
      chain,
      data,
      gasPrice,
      maxFeePerBlobGas,
      maxFeePerGas,
      maxPriorityFeePerGas,
      nonce,
      nonceManager: account.nonceManager,
      parameters: [
        'blobVersionedHashes',
        'chainId',
        'fees',
        'nonce',
        'type',
        'sidecars',
      ],
      value,
      ...rest,
      to,
    } as any)

    const serializer = chain?.serializers?.transaction
    const gasEstimate =
      gas ??
      BigInt(
        await client.request(
          {
            // Estimate gas from a signed raw transaction so the node can
            // recover the real sender and simulate execution with authenticated
            // caller context. We use a large temporary gas limit here only for
            // the estimation request; the final tx is re-signed below with the
            // actual estimated gas.
            method: 'eth_estimateGas',
            params: [
              await account.signTransaction(
                {
                  ...request,
                  gas: DEFAULT_SIGNED_ESTIMATE_GAS_LIMIT,
                },
                { serializer }
              ),
            ],
          } as any,
          { retryCount: 0 }
        )
      )

    // Re-sign the final transparent transaction with the resolved gas limit
    // and submit it normally as a raw transaction.
    const serializedTransaction = await account.signTransaction(
      {
        ...request,
        gas: gasEstimate,
      },
      { serializer }
    )

    return await getAction(
      client,
      sendRawTransaction,
      'sendRawTransaction'
    )({
      serializedTransaction,
    })
  } catch (err) {
    throw getTransactionError(err as BaseError, {
      ...parameters,
      account,
      chain: parameters.chain || undefined,
    })
  }
}
