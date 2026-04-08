import type { ExtractAbiFunctionNames } from 'abitype'
import type {
  Abi,
  Account,
  Address,
  Chain,
  Client,
  ContractFunctionArgs,
  ContractFunctionName,
  GetContractParameters,
  GetContractReturnType,
  IsNarrowable,
  IsNever,
  ReadContractParameters,
  Transport,
  UnionOmit,
  WriteContractParameters,
  WriteContractReturnType,
} from 'viem'
import { getContract } from 'viem'
import { readContract, writeContract } from 'viem/actions'

import type { ShieldedWalletClient } from '@sviem/client.ts'
import { hasShieldedParams } from '@sviem/contract/abi.ts'
import {
  SignedReadContractParameters,
  signedReadContract,
  transparentReadContract,
} from '@sviem/contract/read.ts'
import {
  ShieldedWriteContractDebugResult,
  shieldedWriteContract,
  shieldedWriteContractDebug,
  transparentWriteContract,
} from '@sviem/contract/write.ts'
import type { KeyedClient } from '@sviem/viem-internal/client.ts'
import type {
  GetReadFunction,
  GetWriteFunction,
} from '@sviem/viem-internal/function.ts'
import { getFunctionParameters } from '@sviem/viem-internal/function.ts'

type TransparentReadContractReturnType<
  TAbi extends Abi | readonly unknown[],
  TClient extends Client | KeyedClient = Client | KeyedClient,
  _readFunctionNames extends string = TAbi extends Abi
    ? Abi extends TAbi
      ? string
      : ExtractAbiFunctionNames<TAbi, 'pure' | 'view'>
    : string,
  _narrowable extends boolean = IsNarrowable<TAbi, Abi>,
  _publicClient extends Client | unknown = TClient extends {
    public: Client
  }
    ? TClient['public']
    : TClient,
> = _publicClient extends Client
  ? IsNever<_readFunctionNames> extends true
    ? unknown
    : {
        tread: {
          [functionName in _readFunctionNames]: GetReadFunction<
            _narrowable,
            TAbi,
            functionName extends ContractFunctionName<TAbi, 'pure' | 'view'>
              ? functionName
              : never
          >
        }
        sread: {
          [functionName in _readFunctionNames]: GetReadFunction<
            _narrowable,
            TAbi,
            functionName extends ContractFunctionName<TAbi, 'pure' | 'view'>
              ? functionName
              : never
          >
        }
      }
  : unknown

type TransparentWriteContractReturnType<
  TAbi extends Abi | readonly unknown[],
  TClient extends Client | KeyedClient = Client | KeyedClient,
  _writeFunctionNames extends string = TAbi extends Abi
    ? Abi extends TAbi
      ? string
      : ExtractAbiFunctionNames<TAbi, 'nonpayable' | 'payable'>
    : string,
  _narrowable extends boolean = IsNarrowable<TAbi, Abi>,
  _walletClient extends Client | unknown = TClient extends {
    wallet: Client
  }
    ? TClient['wallet']
    : TClient,
> = _walletClient extends Client
  ? IsNever<_writeFunctionNames> extends true
    ? unknown
    : {
        twrite: {
          [functionName in _writeFunctionNames]: GetWriteFunction<
            _narrowable,
            _walletClient['chain'],
            _walletClient['account'],
            TAbi,
            functionName extends ContractFunctionName<
              TAbi,
              'nonpayable' | 'payable'
            >
              ? functionName
              : never,
            WriteContractReturnType
          >
        }
        swrite: {
          [functionName in _writeFunctionNames]: GetWriteFunction<
            _narrowable,
            _walletClient['chain'],
            _walletClient['account'],
            TAbi,
            functionName extends ContractFunctionName<
              TAbi,
              'nonpayable' | 'payable'
            >
              ? functionName
              : never,
            WriteContractReturnType
          >
        }
        dwrite: {
          [functionName in _writeFunctionNames]: GetWriteFunction<
            _narrowable,
            _walletClient['chain'],
            _walletClient['account'],
            TAbi,
            functionName extends ContractFunctionName<
              TAbi,
              'nonpayable' | 'payable'
            >
              ? functionName
              : never,
            ShieldedWriteContractDebugResult<
              _walletClient['chain'],
              _walletClient['account']
            >
          >
        }
      }
  : unknown

/**
 * The same as viem's {@link https://viem.sh/docs/contract/getContract.html#with-wallet-client GetContractReturnType}, with a few differences:
 * - `read` and `write` are "smart" — they auto-detect shielded params and route accordingly
 * - `sread` and `swrite` always use signed reads & seismic transactions (force shielded)
 * - `tread` and `twrite` behave like viem's standard read & write (force transparent)
 * - `dwrite` returns both plaintext and encrypted tx for debugging
 */
export type ShieldedContract<
  TTransport extends Transport = Transport,
  TAddress extends Address = Address,
  TAbi extends Abi | readonly unknown[] = Abi,
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends Account = Account,
  TClient extends
    | ShieldedWalletClient<TTransport, TChain, TAccount>
    | KeyedClient<TTransport, TChain, TAccount> = ShieldedWalletClient<
    TTransport,
    TChain,
    TAccount
  >,
> = GetContractReturnType<TAbi, TClient, TAddress> &
  TransparentReadContractReturnType<TAbi, TClient> &
  TransparentWriteContractReturnType<TAbi, TClient>

/**
 * This function extends viem's base {@link https://viem.sh/docs/contract/getContract.html getContract} functionality by adding:
 * - `read`: smart read — auto-detects shielded params; uses signed read if shielded, transparent read otherwise
 * - `write`: smart write — auto-detects shielded params; uses shielded write if shielded, transparent write otherwise
 * - `sread`: force shielded read — always uses signed read regardless of params
 * - `swrite`: force shielded write — always uses encrypted seismic transaction regardless of params
 * - `tread`: force transparent read — always uses unsigned read (from the zero address)
 * - `twrite`: force transparent write — always uses non-encrypted calldata
 * - `dwrite`: debug write — get plaintext and encrypted transaction without broadcasting
 *
 * @param {GetContractParameters} params - The configuration object.
 *   - `abi` ({@link Abi}) - The contract's ABI.
 *   - `address` ({@link Address}) - The contract's address.
 *   - `client` ({@link ShieldedWalletClient}) - The client instance to use for interacting with the contract.
 *
 * @throws {Error} If the wallet client is not provided for shielded write or signed read operations.
 * @throws {Error} If the wallet client does not have an account configured for signed reads.
 *
 * @example
 * ```typescript
 * const contract = getShieldedContract({
 *   abi: myContractAbi,
 *   address: '0x1234...',
 *   client: shieldedWalletClient,
 * });
 *
 * // Smart write — auto-detects shielded params
 * await contract.write.setNumber([7n], { gas: 50000n }); // shielded if suint256 param
 * await contract.write.increment();                       // transparent if no shielded params
 *
 * // Smart read — auto-detects shielded params
 * const value = await contract.read.getSecret();  // signed read if shielded param
 * const odd = await contract.read.isOdd();         // transparent read if no shielded params
 *
 * // Force shielded
 * await contract.swrite.increment();  // always shielded
 * const val = await contract.sread.isOdd(); // always signed read
 * ```
 *
 * @remarks
 * - The `read` property auto-detects shielded params and routes to signed read or transparent read
 * - The `write` property auto-detects shielded params and routes to shielded write or transparent write
 * - The `sread` property always calls a signed read
 * - The `swrite` property always encrypts calldata via seismic transaction
 * - The `tread` property toggles between public reads and signed reads, depending on whether an `account` is provided
 * - The `twrite` property makes a normal write with transparent calldata
 * - The `dwrite` property returns both plaintext and encrypted tx for debugging
 * - The client must be a {@link ShieldedWalletClient}
 */
export function getShieldedContract<
  TTransport extends Transport,
  TAddress extends Address,
  const TAbi extends Abi | readonly unknown[],
  const TClient extends
    | ShieldedWalletClient<TTransport, TChain, TAccount>
    | KeyedClient<TTransport, TChain, TAccount>,
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends Account = Account,
>({
  abi,
  address,
  client,
}: GetContractParameters<
  TTransport,
  TChain,
  TAccount,
  TAbi,
  TClient,
  TAddress
>): ShieldedContract<TTransport, TAddress, TAbi, TChain, TAccount, TClient> {
  const viemContract = getContract({ abi, address, client })

  const walletClient:
    | ShieldedWalletClient<TTransport, TChain, TAccount>
    | undefined = (() => {
    if (!client) return undefined
    if ('wallet' in client)
      return client.wallet as ShieldedWalletClient<TTransport, TChain, TAccount>
    return client as ShieldedWalletClient<TTransport, TChain, TAccount>
  })()

  const transparentWriteAction = new Proxy(
    {},
    {
      get(_, functionName: string) {
        return (
          ...parameters: [
            args?: readonly unknown[],
            options?: UnionOmit<
              WriteContractParameters,
              'abi' | 'address' | 'functionName' | 'args'
            >,
          ]
        ) => {
          if (walletClient === undefined) {
            throw new Error('Must provide wallet client to call Contract.write')
          }
          const { args, options } = getFunctionParameters(parameters)
          return transparentWriteContract(
            walletClient as unknown as Parameters<
              typeof transparentWriteContract
            >[0],
            {
              abi,
              address,
              functionName,
              args,
              ...(options as Record<string, unknown>),
            } as unknown as Parameters<typeof transparentWriteContract>[1]
          )
        }
      },
    }
  )

  function shieldedWrite<
    functionName extends ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
    args extends ContractFunctionArgs<
      TAbi,
      'payable' | 'nonpayable',
      functionName
    > = ContractFunctionArgs<TAbi, 'payable' | 'nonpayable', functionName>,
  >({
    functionName,
    args,
    ...options
  }: WriteContractParameters<TAbi, functionName, args, TChain, TAccount>) {
    if (walletClient === undefined) {
      throw new Error('Must provide wallet client to call Contract.write')
    }

    return shieldedWriteContract(
      walletClient as unknown as Parameters<typeof shieldedWriteContract>[0],
      {
        abi,
        address,
        functionName,
        args,
        ...(options as Record<string, unknown>),
      } as unknown as Parameters<typeof shieldedWriteContract>[1]
    )
  }

  function shieldedWriteDebug<
    functionName extends ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
    args extends ContractFunctionArgs<
      TAbi,
      'payable' | 'nonpayable',
      functionName
    > = ContractFunctionArgs<TAbi, 'payable' | 'nonpayable', functionName>,
  >({
    functionName,
    args,
    ...options
  }: WriteContractParameters<TAbi, functionName, args, TChain, TAccount>) {
    if (walletClient === undefined) {
      throw new Error('Must provide wallet client to call Contract.dwrite')
    }

    return shieldedWriteContractDebug(
      walletClient as unknown as Parameters<
        typeof shieldedWriteContractDebug
      >[0],
      {
        abi,
        address,
        functionName,
        args,
        ...(options as Record<string, unknown>),
      } as unknown as Parameters<typeof shieldedWriteContractDebug>[1]
    )
  }

  const shieldedWriteAction = new Proxy(
    {},
    {
      get(_, functionName: string) {
        return (
          ...parameters: [
            args?: readonly unknown[],
            options?: UnionOmit<
              WriteContractParameters,
              'abi' | 'address' | 'functionName' | 'args'
            >,
          ]
        ) => {
          const { args, options } = getFunctionParameters(parameters)
          return shieldedWrite({
            abi,
            address,
            functionName,
            args,
            ...(options as Record<string, unknown>),
          } as unknown as WriteContractParameters<
            TAbi,
            ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
            ContractFunctionArgs<TAbi, 'nonpayable' | 'payable'>,
            TChain,
            TAccount
          >)
        }
      },
    }
  )

  const shieldedWriteDebugAction = new Proxy(
    {},
    {
      get(_, functionName: string) {
        return (
          ...parameters: [
            args?: readonly unknown[],
            options?: UnionOmit<
              WriteContractParameters,
              'abi' | 'address' | 'functionName' | 'args'
            >,
          ]
        ) => {
          const { args, options } = getFunctionParameters(parameters)
          return shieldedWriteDebug({
            abi,
            address,
            functionName,
            args,
            ...(options as Record<string, unknown>),
          } as unknown as WriteContractParameters<
            TAbi,
            ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
            ContractFunctionArgs<TAbi, 'nonpayable' | 'payable'>,
            TChain,
            TAccount
          >)
        }
      },
    }
  )

  function signedRead<
    TFunctionName extends ContractFunctionName<TAbi, 'pure' | 'view'>,
    TArgs extends ContractFunctionArgs<
      TAbi,
      'pure' | 'view',
      TFunctionName
    > = ContractFunctionArgs<TAbi, 'pure' | 'view', TFunctionName>,
  >(params: SignedReadContractParameters<TAbi, TFunctionName, TArgs>) {
    if (walletClient === undefined) {
      throw new Error('Must provide wallet client to call signed read')
    }
    return signedReadContract(walletClient, params)
  }

  const transparentReadAction = new Proxy(
    {},
    {
      get(_, functionName: string) {
        return (
          ...parameters: [
            args?: readonly unknown[] | undefined,
            options?: UnionOmit<
              ReadContractParameters,
              'abi' | 'address' | 'functionName' | 'args'
            >,
          ]
        ) => {
          const { args, options } = getFunctionParameters(parameters)
          const opts = options as Record<string, unknown>
          if (opts?.account) {
            return signedRead({
              abi,
              address,
              functionName,
              args,
              ...opts,
            } as unknown as SignedReadContractParameters<
              TAbi,
              ContractFunctionName<TAbi, 'pure' | 'view'>,
              ContractFunctionArgs<TAbi, 'pure' | 'view'>
            >)
          }
          return transparentReadContract(
            walletClient as unknown as Parameters<
              typeof transparentReadContract
            >[0],
            {
              abi,
              address,
              functionName,
              args,
              ...(opts as Record<string, unknown>),
            } as unknown as Parameters<typeof transparentReadContract>[1]
          )
        }
      },
    }
  )

  const signedReadAction = new Proxy(
    {},
    {
      get(_, functionName: string) {
        return (
          ...parameters: [
            args?: readonly unknown[] | undefined,
            options?: UnionOmit<
              ReadContractParameters,
              'abi' | 'address' | 'functionName' | 'args'
            >,
          ]
        ) => {
          if (!walletClient?.account) {
            console.error(JSON.stringify(walletClient, null, 2))
            throw new Error(
              'Wallet must have an account to perform signed reads'
            )
          }
          const params = getFunctionParameters(parameters)
          const { args } = params
          const {
            options: { account: _account, ...options },
          } = params as {
            options: { account: Account | undefined } & Record<string, unknown>
          }
          return signedRead({
            abi,
            address,
            functionName,
            args,
            // Force account to be their account
            // Node will reject it if they stick in a different account here,
            // because of the AEAD in encryption
            account: walletClient.account,
            ...options,
          } as unknown as SignedReadContractParameters<
            TAbi,
            ContractFunctionName<TAbi, 'pure' | 'view'>,
            ContractFunctionArgs<TAbi, 'pure' | 'view'>
          >)
        }
      },
    }
  )

  const smartReadAction = new Proxy(
    {},
    {
      get(_, functionName: string) {
        return (
          ...parameters: [
            args?: readonly unknown[] | undefined,
            options?: UnionOmit<
              ReadContractParameters,
              'abi' | 'address' | 'functionName' | 'args'
            >,
          ]
        ) => {
          const isShielded = hasShieldedParams(abi as Abi, functionName)
          if (isShielded) {
            if (!walletClient?.account) {
              throw new Error(
                'Wallet must have an account to perform signed reads'
              )
            }
            const params = getFunctionParameters(parameters)
            const { args } = params
            const {
              options: { account: _account, ...options },
            } = params as {
              options: { account: Account | undefined } & Record<
                string,
                unknown
              >
            }
            return signedRead({
              abi,
              address,
              functionName,
              args,
              account: walletClient.account,
              ...options,
            } as unknown as SignedReadContractParameters<
              TAbi,
              ContractFunctionName<TAbi, 'pure' | 'view'>,
              ContractFunctionArgs<TAbi, 'pure' | 'view'>
            >)
          } else {
            const { args, options } = getFunctionParameters(parameters)
            // No shielded params → abi is valid for viem as-is
            // @ts-expect-error: walletClient satisfies Client for readContract
            return readContract(walletClient, {
              abi,
              address,
              functionName,
              args,
              ...(options as Record<string, unknown>),
            })
          }
        }
      },
    }
  )

  const smartWriteAction = new Proxy(
    {},
    {
      get(_, functionName: string) {
        return (
          ...parameters: [
            args?: readonly unknown[],
            options?: UnionOmit<
              WriteContractParameters,
              'abi' | 'address' | 'functionName' | 'args'
            >,
          ]
        ) => {
          const isShielded = hasShieldedParams(abi as Abi, functionName)
          if (isShielded) {
            const { args, options } = getFunctionParameters(parameters)
            return shieldedWrite({
              abi,
              address,
              functionName,
              args,
              ...(options as Record<string, unknown>),
            } as unknown as WriteContractParameters<
              TAbi,
              ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
              ContractFunctionArgs<TAbi, 'nonpayable' | 'payable'>,
              TChain,
              TAccount
            >)
          } else {
            if (walletClient === undefined) {
              throw new Error(
                'Must provide wallet client to call Contract.write'
              )
            }
            // No shielded params → abi is valid for viem as-is
            const { args, options } = getFunctionParameters(parameters)
            return writeContract(
              walletClient as unknown as Parameters<typeof writeContract>[0],
              {
                abi,
                address,
                functionName,
                args,
                ...(options as Record<string, unknown>),
              } as unknown as Parameters<typeof writeContract>[1]
            )
          }
        }
      },
    }
  )

  const contract: {
    [_ in
      | 'abi'
      | 'address'
      | 'createEventFilter'
      | 'estimateGas'
      | 'getEvents'
      | 'simulate'
      | 'watchEvent'
      | 'read'
      | 'sread'
      | 'tread'
      | 'write'
      | 'swrite'
      | 'twrite'
      | 'dwrite']?: unknown
  } = viemContract
  // Transparent writes use the standard writeContract
  contract.twrite = transparentWriteAction
  // Transparent reads use signed reads,
  // but signing is only activated if they supply an account parameter
  contract.tread = transparentReadAction
  // Force shielded writes always use seismic transactions
  contract.swrite = shieldedWriteAction
  // Force shielded reads always use signed reads
  contract.sread = signedReadAction
  // Smart writes auto-detect shielded params
  contract.write = smartWriteAction
  // Smart reads auto-detect shielded params
  contract.read = smartReadAction
  // Debug writes use seismic debug transactions
  contract.dwrite = shieldedWriteDebugAction
  return contract as ShieldedContract<
    TTransport,
    TAddress,
    TAbi,
    TChain,
    TAccount,
    TClient
  >
}
