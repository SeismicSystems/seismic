import type {
  Account,
  Address,
  AssertCurrentChainErrorType,
  Chain,
  DeriveChain,
  ExactPartial,
  FormattedTransactionRequest,
  GetChainIdErrorType,
  GetChainParameter,
  Hash,
  PrepareTransactionRequestErrorType,
  SendRawTransactionErrorType,
  Transport,
  UnionOmit,
} from 'viem'
import type {
  ParseAccountErrorType,
  SignTransactionErrorType,
} from 'viem/accounts'
import type { RecoverAuthorizationAddressErrorType } from 'viem/experimental'
import type {
  AssertRequestErrorType,
  GetTransactionErrorReturnType,
  RequestErrorType,
} from 'viem/utils'

import type {
  AccountNotFoundErrorType,
  AccountTypeNotSupportedErrorType,
} from '@sviem/error/account.ts'
import type { SeismicTxExtras } from '@sviem/tx/seismicTx.ts'
import type { GetAccountParameter } from '@sviem/viem-internal/account.ts'
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

export type { Transport }
