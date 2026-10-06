import type {
  Abi,
  Account,
  Chain,
  ContractFunctionArgs,
  ContractFunctionName,
  ReadContractParameters as ViemReadContractParameters,
  WriteContractParameters as ViemWriteContractParameters,
} from 'viem'

import type { GasPaymentOptions } from '@sviem/tx/gasPayment.ts'

/** Request-level fee choice shared by wallet actions and contract wrappers. */
export type WriteContractParameters<
  TAbi extends Abi | readonly unknown[] = Abi,
  TFunctionName extends ContractFunctionName<
    TAbi,
    'nonpayable' | 'payable'
  > = ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
  TArgs extends ContractFunctionArgs<
    TAbi,
    'nonpayable' | 'payable',
    TFunctionName
  > = ContractFunctionArgs<TAbi, 'nonpayable' | 'payable', TFunctionName>,
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends Account | undefined = Account | undefined,
  TChainOverride extends Chain | undefined = Chain | undefined,
> = ViemWriteContractParameters<
  TAbi,
  TFunctionName,
  TArgs,
  TChain,
  TAccount,
  TChainOverride
> &
  GasPaymentOptions

export type ReadContractParameters<
  TAbi extends Abi | readonly unknown[] = Abi,
  TFunctionName extends ContractFunctionName<
    TAbi,
    'pure' | 'view'
  > = ContractFunctionName<TAbi, 'pure' | 'view'>,
  TArgs extends ContractFunctionArgs<
    TAbi,
    'pure' | 'view',
    TFunctionName
  > = ContractFunctionArgs<TAbi, 'pure' | 'view', TFunctionName>,
> = ViemReadContractParameters<TAbi, TFunctionName, TArgs> & GasPaymentOptions
