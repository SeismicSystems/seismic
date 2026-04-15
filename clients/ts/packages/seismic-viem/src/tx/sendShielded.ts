import type {
  Account,
  BaseError,
  Chain,
  ExactPartial,
  SendTransactionParameters,
  Transport,
} from 'viem'
import { parseAccount } from 'viem/accounts'
import { prepareTransactionRequest, sendRawTransaction } from 'viem/actions'
import { assertRequest, getAction, getTransactionError } from 'viem/utils'

import {
  SeismicSecurityParams,
  TransactionSerializableSeismic,
  serializeSeismicTransaction,
} from '@sviem/chain.ts'
import { ShieldedWalletClient } from '@sviem/client.ts'
import {
  AccountNotFoundError,
  AccountTypeNotSupportedError,
} from '@sviem/error/account.ts'
import { estimateShieldedGas } from '@sviem/estimateGas.ts'
import { buildTxSeismicMetadata } from '@sviem/metadata.ts'
import {
  TYPED_DATA_MESSAGE_VERSION,
  signSeismicTxTypedData,
} from '@sviem/signSeismicTypedData.ts'
import type {
  SendSeismicTransactionParameters,
  SendSeismicTransactionRequest,
  SendSeismicTransactionReturnType,
} from '@sviem/tx/types.ts'

/**
 * Sends a shielded transaction on the Seismic network.
 *
 * This function facilitates sending a transaction that includes shielded inputs such as blobs,
 * authorization lists, and encrypted calldata. The transaction is prepared, signed, and
 * submitted to the network based on the provided parameters.
 *
 * @template TChain - The type of the blockchain chain (extends `Chain` or `undefined`).
 * @template TAccount - The type of the account (extends `Account` or `undefined`).
 * @template TRequest - The specific request type for the transaction.
 * @template TChainOverride - The type of the chain override (extends `Chain` or `undefined`).
 *
 * @param client - The client instance used to execute the transaction, including chain, account,
 * and transport configurations.
 * @param parameters - The transaction parameters, including gas, value, blobs, and other details.
 *
 * @returns A promise that resolves to the result of the shielded transaction submission.
 *
 * @throws {AccountNotFoundError} If no account is provided in the client or parameters.
 * @throws {AccountTypeNotSupportedError} If the account type is unsupported for shielded transactions.
 * @throws {Error} If the `data` is invalid or missing.
 *
 * @remarks
 * - Supports various account types, including `json-rpc` and `local`.
 * - Requires a valid `data` in hexadecimal format.
 * - Throws specific errors for unsupported account types or missing account configurations.
 * - Uses the `sendRawTransaction` method for final transaction submission.
 *
 * @example
 * ```typescript
 * const result = await sendShieldedTransaction(client, {
 *   account: { address: '0x1234...' },
 *   chain: seismicChain,
 *   data: '0xabcdef...',
 *   value: 1000n,
 *   gas: 21000n,
 * });
 * console.log('Transaction hash:', result.hash);
 * ```
 */
export async function sendShieldedTransaction<
  TChain extends Chain | undefined,
  TAccount extends Account,
  const TRequest extends SendSeismicTransactionRequest<TChain, TChainOverride>,
  TChainOverride extends Chain | undefined = undefined,
>(
  client: ShieldedWalletClient<Transport, TChain, TAccount>,
  parameters: SendSeismicTransactionParameters<
    TChain,
    TAccount,
    TChainOverride,
    TRequest
  >,
  { blocksWindow = 100n, encryptionNonce }: SeismicSecurityParams = {}
): Promise<SendSeismicTransactionReturnType> {
  const {
    account: account_ = client.account,
    chain = client.chain,
    accessList,
    authorizationList,
    blobs,
    data: plaintextCalldata,
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
      // TODO: link this
      // docsPath: '/docs/actions/wallet/sendSeismicTransaction',
    })
  const account = account_ ? parseAccount(account_) : null
  if (account === null) {
    throw new Error(`Account must not be null to send a Seismic transaction`)
  }

  try {
    const assertRequestParams = {
      account,
      gasPrice,
      maxFeePerGas,
      maxPriorityFeePerGas,
      to: parameters.to,
    } as ExactPartial<SendTransactionParameters>
    assertRequest(assertRequestParams)

    const to = await (async () => {
      if (parameters.to) return parameters.to
      return undefined
    })()

    if (
      account === null ||
      account?.type === 'local' ||
      account?.type === 'json-rpc'
    ) {
      const metadata = await buildTxSeismicMetadata(client, {
        account: account_,
        nonce,
        to: to!,
        value,
        blocksWindow,
        signedRead: false,
        encryptionNonce,
      })
      const encryptedCalldata = await client.encrypt(
        plaintextCalldata,
        metadata
      )

      // Resolve gas price: prefer user-supplied values, then fetch from chain.
      // Seismic txs use legacy-style gasPrice, so if viem fills EIP-1559
      // fields we use maxFeePerGas as gasPrice (it includes headroom).
      const resolvedGasPrice =
        gasPrice ?? maxFeePerGas ?? (await client.getGasPrice())

      // When gas is not provided, sign a temporary tx and send it to
      // eth_estimateGas so the node can authenticate the sender.
      // This prevents caller-spoofing against msg.sender-gated state.
      const resolvedGas =
        gas ??
        (await estimateShieldedGas(client, {
          encryptedData: encryptedCalldata,
          metadata,
          gasPrice: resolvedGasPrice,
        }))

      // Fill remaining fee fields via prepareTransactionRequest.
      // Gas is already resolved so viem won't call eth_estimateGas.
      const prepRequest = {
        accessList,
        authorizationList,
        blobs,
        chainId: metadata.legacyFields.chainId,
        data: plaintextCalldata,
        from: account.address,
        gas: resolvedGas,
        gasPrice,
        maxFeePerBlobGas,
        maxFeePerGas,
        maxPriorityFeePerGas,
        nonce: metadata.legacyFields.nonce,
        to,
        value,
        ...rest,
      } as any

      const viemPreparedTx = (await prepareTransactionRequest(
        client,
        prepRequest
      )) as Record<string, unknown>

      const preparedTx = {
        ...viemPreparedTx,
        ...metadata.seismicElements,
        data: encryptedCalldata,
        gasPrice: resolvedGasPrice,
        type: 'seismic',
      } as TransactionSerializableSeismic

      if (
        metadata.seismicElements.messageVersion === TYPED_DATA_MESSAGE_VERSION
      ) {
        const { typedData, signature } = await signSeismicTxTypedData(
          client,
          preparedTx
        )
        const action = getAction(
          client,
          sendRawTransaction,
          'sendRawTransaction'
        )
        return await action({
          // @ts-ignore
          serializedTransaction: { data: typedData, signature },
        })
      }

      const serializedTransaction = await account.signTransaction!(preparedTx, {
        serializer: serializeSeismicTransaction,
      })
      return await sendRawTransaction(client, { serializedTransaction })
    }

    if (account?.type === 'smart')
      throw new AccountTypeNotSupportedError({
        metaMessages: ['Consider using the sendUserOperation Action instead.'],
        docsPath: '/docs/actions/bundler/sendUserOperation',
        type: 'smart',
      })

    throw new AccountTypeNotSupportedError({
      docsPath: '/docs/actions/wallet/sendSeismicTransaction',
      type: (account as any)?.type,
    })
  } catch (err) {
    if (err instanceof AccountTypeNotSupportedError) throw err
    throw getTransactionError(err as BaseError, {
      ...parameters,
      account,
      chain: chain || undefined,
      // not correct, but not used
      type: 'legacy',
    })
  }
}
