import type {
  Account,
  Address,
  AssertCurrentChainErrorType,
  BaseError,
  Chain,
  DeriveChain,
  ExactPartial,
  FormattedTransactionRequest,
  GetChainIdErrorType,
  GetChainParameter,
  Hash,
  PrepareTransactionRequestErrorType,
  SendTransactionReturnType,
  SendRawTransactionErrorType,
  SendTransactionParameters,
  Transport,
  UnionOmit,
} from 'viem'
import type {
  ParseAccountErrorType,
  SignTransactionErrorType,
} from 'viem/accounts'
import { parseAccount } from 'viem/accounts'
import {
  prepareTransactionRequest,
  sendRawTransaction,
  sendTransaction as viemSendTransaction,
} from 'viem/actions'
import type { RecoverAuthorizationAddressErrorType } from 'viem/experimental'
import type {
  AssertRequestErrorType,
  GetTransactionErrorReturnType,
  RequestErrorType,
} from 'viem/utils'
import { assertRequest, getAction, getTransactionError } from 'viem/utils'

import {
  SeismicSecurityParams,
  SeismicTxExtras,
  TransactionSerializableSeismic,
  serializeSeismicTransaction,
} from '@sviem/chain.ts'
import { ShieldedWalletClient } from '@sviem/client.ts'
import type {
  AccountNotFoundErrorType,
  AccountTypeNotSupportedErrorType,
} from '@sviem/error/account.ts'
import {
  AccountNotFoundError,
  AccountTypeNotSupportedError,
} from '@sviem/error/account.ts'
import { buildTxSeismicMetadata } from '@sviem/metadata.ts'
import {
  TYPED_DATA_MESSAGE_VERSION,
  signSeismicTxTypedData,
} from '@sviem/signSeismicTypedData.ts'
import { GetAccountParameter } from '@sviem/viem-internal/account.ts'
import type { ErrorType } from '@sviem/viem-internal/error.ts'

export type SendSeismicTransactionRequest<
  chain extends Chain | undefined = Chain | undefined,
  chainOverride extends Chain | undefined = Chain | undefined,
  _derivedChain extends Chain | undefined = DeriveChain<chain, chainOverride>,
> = UnionOmit<FormattedTransactionRequest<_derivedChain>, 'from' | 'type'> &
  SeismicTxExtras

export type SendSeismicTransactionParameters<
  chain extends Chain | undefined = Chain | undefined,
  account extends Account | undefined = Account | undefined,
  chainOverride extends Chain | undefined = Chain | undefined,
  request extends SendSeismicTransactionRequest<
    chain,
    chainOverride
  > = SendSeismicTransactionRequest<chain, chainOverride>,
> = request &
  GetAccountParameter<account, Account | Address, true, true> &
  GetChainParameter<chain, chainOverride>

export type AssertSeismicRequestParameters = ExactPartial<
  SendSeismicTransactionParameters<Chain>
>

export type SendSeismicTransactionReturnType = Hash

const DEFAULT_SIGNED_ESTIMATE_GAS_LIMIT = 30_000_000n

export type SendSeismicTransactionErrorType =
  | ParseAccountErrorType
  | GetTransactionErrorReturnType<
    | AccountNotFoundErrorType
    | AccountTypeNotSupportedErrorType
    | AssertCurrentChainErrorType
    | AssertRequestErrorType
    | GetChainIdErrorType
    | PrepareTransactionRequestErrorType
    | SendRawTransactionErrorType
    | RecoverAuthorizationAddressErrorType
    | SignTransactionErrorType
    | RequestErrorType
  >
  | ErrorType

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
      parameters: ['blobVersionedHashes', 'chainId', 'fees', 'nonce', 'type', 'sidecars'],
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

      // Estimate gas and fill fee fields using PLAINTEXT calldata
      // without seismic fields.  This avoids hitting sreth's
      // eth_estimateGas with encrypted data + type 0x4a.
      // We omit `type` so viem auto-detects EIP-1559 vs legacy and
      // fills the appropriate fee fields with proper headroom.
      const prepRequest = {
        accessList,
        authorizationList,
        blobs,
        chainId: metadata.legacyFields.chainId,
        data: plaintextCalldata,
        from: account.address,
        gas,
        gasPrice,
        maxFeePerBlobGas,
        maxFeePerGas,
        maxPriorityFeePerGas,
        nonce: metadata.legacyFields.nonce,
        to,
        value,
      } as any

      const viemPreparedTx = (await prepareTransactionRequest(
        client,
        prepRequest
      )) as Record<string, unknown>

      // Seismic txs use legacy-style gasPrice.  If viem chose EIP-1559
      // fees (maxFeePerGas), use that as gasPrice — it already includes
      // headroom above the current base fee so the tx won't be
      // underpriced when the next block's base fee adjusts.
      const resolvedGasPrice =
        viemPreparedTx.gasPrice ??
        viemPreparedTx.maxFeePerGas ??
        (await client.getGasPrice())

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
      } else {
        const serializedTransaction = await account!.signTransaction!(
          preparedTx,
          { serializer: serializeSeismicTransaction }
        )
        return await sendRawTransaction(client, { serializedTransaction })
      }
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
