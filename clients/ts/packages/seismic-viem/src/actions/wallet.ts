import type {
  Abi,
  Account,
  Address,
  Chain,
  ContractFunctionArgs,
  ContractFunctionName,
  ReadContractParameters,
  ReadContractReturnType,
  SendTransactionParameters,
  SendTransactionReturnType,
  Transport,
  WriteContractParameters,
  WriteContractReturnType,
} from 'viem'
import { readContract, writeContract } from 'viem/actions'

import { SeismicSecurityParams } from '@sviem/chain.ts'
import { ShieldedWalletClient } from '@sviem/client.ts'
import { hasShieldedParams } from '@sviem/contract/abi.ts'
import {
  signedReadContract,
  transparentReadContract,
} from '@sviem/contract/read.ts'
import {
  ShieldedWriteContractDebugResult,
  shieldedWriteContract,
  shieldedWriteContractDebug,
  transparentWriteContract,
} from '@sviem/contract/write.ts'
import type {
  SendSeismicTransactionParameters,
  SendSeismicTransactionRequest,
  SendSeismicTransactionReturnType,
} from '@sviem/sendTransaction.ts'
import {
  sendShieldedTransaction,
  sendTransparentTransaction,
} from '@sviem/sendTransaction.ts'
import { signedCall } from '@sviem/signedCall.ts'
import type { SignedCall } from '@sviem/signedCall.ts'

/**
 * Defines the actions available for a shielded wallet client.
 *
 * These actions provide functionality for interacting with shielded contracts,
 * making signed calls, sending shielded transactions, and retrieving encryption keys.
 *
 * @template TChain - The type of the blockchain chain (extends `Chain` or `undefined`).
 * @template TAccount - The type of the account (extends `Account` or `undefined`).
 *
 * @property writeContract - Executes a write operation on a shielded contract.
 * Takes parameters specific to the contract and returns the transaction result.
 *
 * @property readContract - Reads data from a shielded contract using signed read methods.
 * Returns the contract's data as defined by the provided arguments.
 *
 * @property signedCall - Executes a signed call on the blockchain, allowing for
 * advanced interactions with shielded contracts or transactions.
 *
 * @property sendShieldedTransaction - Sends a shielded transaction using encrypted payloads
 * and advanced features such as blobs and authorization lists.
 *
 * @param args - The parameters required for sending the transaction.
 *
 * @returns A promise that resolves to the result of the shielded transaction.
 *
 * @property getEncryption - Retrieves the encryption key for the shielded wallet client.
 * @returns {Hex} The encryption key in hexadecimal format.
 */
export type ShieldedWalletActions<
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends Account | undefined = Account | undefined,
> = {
  // `sendTransaction` is intentionally overridden on the Seismic wallet client.
  // Transparent writes ultimately flow through this action, so it is the central
  // place where Seismic-specific gas-estimation behavior is applied.
  // Today only `local` accounts can perform signed transparent `eth_estimateGas`
  // because the client has direct access to the signing key. `json-rpc` accounts
  // fall back to viem's normal unsigned estimation path.
  sendTransaction: <TChainOverride extends Chain | undefined = undefined>(
    args: SendTransactionParameters<TChain, TAccount, TChainOverride>
  ) => Promise<SendTransactionReturnType>
  writeContract: <
    TAbi extends Abi | readonly unknown[],
    TFunctionName extends ContractFunctionName<TAbi, 'payable' | 'nonpayable'>,
    TArgs extends ContractFunctionArgs<
      TAbi,
      'payable' | 'nonpayable',
      TFunctionName
    >,
    TChainOverride extends Chain | undefined = undefined,
  >(
    args: WriteContractParameters<
      TAbi,
      TFunctionName,
      TArgs,
      TChain,
      TAccount,
      TChainOverride
    >,
    securityParams?: SeismicSecurityParams
  ) => Promise<WriteContractReturnType>
  swriteContract: <
    TAbi extends Abi | readonly unknown[],
    TFunctionName extends ContractFunctionName<TAbi, 'payable' | 'nonpayable'>,
    TArgs extends ContractFunctionArgs<
      TAbi,
      'payable' | 'nonpayable',
      TFunctionName
    >,
    TChainOverride extends Chain | undefined = undefined,
  >(
    args: WriteContractParameters<
      TAbi,
      TFunctionName,
      TArgs,
      TChain,
      TAccount,
      TChainOverride
    >,
    securityParams?: SeismicSecurityParams
  ) => Promise<WriteContractReturnType>
  twriteContract: <
    TAbi extends Abi | readonly unknown[],
    TFunctionName extends ContractFunctionName<TAbi, 'payable' | 'nonpayable'>,
    TArgs extends ContractFunctionArgs<
      TAbi,
      'payable' | 'nonpayable',
      TFunctionName
    >,
    TChainOverride extends Chain | undefined = undefined,
  >(
    args: WriteContractParameters<
      TAbi,
      TFunctionName,
      TArgs,
      TChain,
      TAccount,
      TChainOverride
    >
  ) => Promise<WriteContractReturnType>
  dwriteContract: <
    TAbi extends Abi | readonly unknown[],
    TFunctionName extends ContractFunctionName<TAbi, 'payable' | 'nonpayable'>,
    TArgs extends ContractFunctionArgs<
      TAbi,
      'payable' | 'nonpayable',
      TFunctionName
    >,
    TChainOverride extends Chain | undefined = undefined,
  >(
    args: WriteContractParameters<
      TAbi,
      TFunctionName,
      TArgs,
      TChain,
      TAccount,
      TChainOverride
    >,
    securityParams?: SeismicSecurityParams
  ) => Promise<ShieldedWriteContractDebugResult<TChain | undefined, TAccount>>
  readContract: <
    TAbi extends Abi | readonly unknown[],
    TFunctionName extends ContractFunctionName<TAbi, 'pure' | 'view'>,
    TArgs extends ContractFunctionArgs<TAbi, 'pure' | 'view', TFunctionName>,
  >(
    args: ReadContractParameters<TAbi, TFunctionName, TArgs>,
    securityParams?: SeismicSecurityParams
  ) => Promise<ReadContractReturnType>
  sreadContract: <
    TAbi extends Abi | readonly unknown[],
    TFunctionName extends ContractFunctionName<TAbi, 'pure' | 'view'>,
    TArgs extends ContractFunctionArgs<TAbi, 'pure' | 'view', TFunctionName>,
  >(
    args: ReadContractParameters<TAbi, TFunctionName, TArgs>,
    securityParams?: SeismicSecurityParams
  ) => Promise<ReadContractReturnType>
  treadContract: <
    TAbi extends Abi | readonly unknown[],
    TFunctionName extends ContractFunctionName<TAbi, 'pure' | 'view'>,
    TArgs extends ContractFunctionArgs<TAbi, 'pure' | 'view', TFunctionName>,
  >(
    args: ReadContractParameters<TAbi, TFunctionName, TArgs>,
    securityParams?: SeismicSecurityParams
  ) => Promise<ReadContractReturnType>
  signedCall: SignedCall<TChain>
  sendShieldedTransaction: <
    const request extends SendSeismicTransactionRequest<TChain, TChainOverride>,
    TChainOverride extends Chain | undefined = undefined,
  >(
    args: SendSeismicTransactionParameters<
      TChain,
      TAccount,
      TChainOverride,
      request
    >,
    securityParams?: SeismicSecurityParams
  ) => Promise<SendSeismicTransactionReturnType>
}

/**
 * Creates the shielded wallet actions for a given shielded wallet client.
 *
 * This function initializes and returns a set of actions, such as interacting with shielded
 * contracts, making signed calls, sending shielded transactions, and retrieving encryption keys.
 *
 * @template TTransport - The transport type used by the client (extends `Transport`).
 * @template TChain - The blockchain chain type (extends `Chain` or `undefined`).
 * @template TAccount - The account type associated with the client (extends `Account` or `undefined`).
 *
 * @param client - The shielded wallet client instance.
 * @param encryption - The encryption key used by the shielded wallet client.
 *
 * @returns {ShieldedWalletActions<TChain, TAccount>} An object containing the shielded wallet actions.
 *
 * @example
 * ```typescript
 * const actions = shieldedWalletActions(client, '0xabcdef123456...');
 *
 * // Write to a shielded contract
 * const writeResult = await actions.writeContract({
 *   address: '0x1234...',
 *   data: '0xdeadbeef...',
 * });
 *
 * // Read from a shielded contract
 * const readResult = await actions.readContract({
 *   address: '0x1234...',
 *   method: 'getValue',
 * });
 *
 * // Send a shielded transaction
 * const txResult = await actions.sendShieldedTransaction({
 *   account: { address: '0x5678...' },
 *   data: '0xabcdef...',
 *   value: 1000n,
 * });
 *
 * // Get encryption key
 * const encryptionKey = actions.getEncryption();
 * console.log('Encryption Key:', encryptionKey);
 * ```
 */
export const shieldedWalletActions = <
  TTransport extends Transport,
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends Account = Account,
>(
  client: ShieldedWalletClient<TTransport, TChain, TAccount>
): ShieldedWalletActions<TChain, TAccount> => {
  return {
    sendTransaction: (args) =>
      sendTransparentTransaction(
        client as unknown as Parameters<typeof sendTransparentTransaction>[0],
        args as unknown as Parameters<typeof sendTransparentTransaction>[1]
      ),
    writeContract: (args) => {
      const writeArgs = args as unknown as {
        abi: Abi
        address: Address
        functionName: string
        args?: readonly unknown[]
      }
      if (hasShieldedParams(writeArgs.abi, writeArgs.functionName)) {
        return shieldedWriteContract(
          client as unknown as Parameters<typeof shieldedWriteContract>[0],
          args as unknown as Parameters<typeof shieldedWriteContract>[1]
        )
      }
      // No shielded params → abi is valid for viem as-is
      return writeContract(
        client as unknown as Parameters<typeof writeContract>[0],
        args as unknown as Parameters<typeof writeContract>[1]
      )
    },
    swriteContract: (args) =>
      shieldedWriteContract(
        client as unknown as Parameters<typeof shieldedWriteContract>[0],
        args as unknown as Parameters<typeof shieldedWriteContract>[1]
      ),
    twriteContract: (args) =>
      transparentWriteContract(
        client as unknown as Parameters<typeof transparentWriteContract>[0],
        args as unknown as Parameters<typeof transparentWriteContract>[1]
      ),
    dwriteContract: (args) => {
      const debugResult = shieldedWriteContractDebug(
        client as unknown as Parameters<typeof shieldedWriteContractDebug>[0],
        args as unknown as Parameters<typeof shieldedWriteContractDebug>[1]
      )
      return debugResult as unknown as Promise<
        ShieldedWriteContractDebugResult<TChain | undefined, TAccount>
      >
    },
    readContract: (args, securityParams) => {
      const readArgs = args as unknown as {
        abi: Abi
        address: Address
        functionName: string
        args?: readonly unknown[]
      }
      if (hasShieldedParams(readArgs.abi as Abi, readArgs.functionName)) {
        return signedReadContract(
          client as unknown as Parameters<typeof signedReadContract>[0],
          args as unknown as Parameters<typeof signedReadContract>[1],
          securityParams
        )
      }
      // No shielded params → abi is valid for viem as-is
      return readContract(
        client as unknown as Parameters<typeof readContract>[0],
        args as unknown as Parameters<typeof readContract>[1]
      )
    },
    sreadContract: (args, securityParams) =>
      signedReadContract(
        client as unknown as Parameters<typeof signedReadContract>[0],
        args as unknown as Parameters<typeof signedReadContract>[1],
        securityParams
      ),
    treadContract: (args) =>
      transparentReadContract(
        client as unknown as Parameters<typeof transparentReadContract>[0],
        args as unknown as Parameters<typeof transparentReadContract>[1]
      ),
    signedCall: (args, securityParams) =>
      signedCall(
        client as unknown as Parameters<typeof signedCall>[0],
        args as unknown as Parameters<typeof signedCall>[1],
        securityParams
      ),
    sendShieldedTransaction: (args, securityParams) =>
      sendShieldedTransaction(
        client as unknown as Parameters<typeof sendShieldedTransaction>[0],
        args as unknown as Parameters<typeof sendShieldedTransaction>[1],
        securityParams
      ),
  }
}
