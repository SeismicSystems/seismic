import type {
  Abi,
  Account,
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

import { SeismicSecurityParams } from '@sviem/chain.ts'
import { ShieldedWalletClient } from '@sviem/client.ts'
import {
  signedReadContract,
  smartReadContract,
  transparentReadContract,
} from '@sviem/contract/read.ts'
import {
  ShieldedWriteContractDebugResult,
  shieldedWriteContract,
  shieldedWriteContractDebug,
  smartWriteContract,
  transparentWriteContract,
} from '@sviem/contract/write.ts'
import { sendShieldedTransaction } from '@sviem/tx/sendShielded.ts'
import { sendTransparentTransaction } from '@sviem/tx/sendTransparent.ts'
import { signedCall } from '@sviem/tx/signedCall.ts'
import type { SignedCall } from '@sviem/tx/signedCall.ts'
import type {
  SendSeismicTransactionParameters,
  SendSeismicTransactionRequest,
  SendSeismicTransactionReturnType,
} from '@sviem/tx/types.ts'

/**
 * Defines the actions available for a shielded wallet client.
 *
 * These actions provide functionality for interacting with shielded contracts,
 * making signed calls, sending shielded transactions, and retrieving encryption keys.
 *
 * @template TChain - The type of the blockchain chain (extends `Chain` or `undefined`).
 * @template TAccount - The type of the account (extends `Account` or `undefined`).
 *
 * @property writeContract - Smart write helper. Routes to a shielded write when
 * the target function has shielded params, and to a transparent write otherwise.
 *
 * @property readContract - Smart read helper. Routes to a signed read when the
 * target function has shielded params, and to a transparent read otherwise.
 *
 * @property signedCall - Executes a signed call on the blockchain, allowing for
 * advanced interactions with shielded contracts or transactions.
 *
 * Explicit signed/shielded helpers such as `sreadContract`,
 * `swriteContract`, `dwriteContract`, `signedCall`, and
 * `sendShieldedTransaction` accept optional `securityParams` advanced metadata
 * overrides. Most callers should omit these; they are mainly useful for
 * deterministic tests/debugging, explicit expiry control, and low-level
 * interoperability.
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
    >
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
    args: ReadContractParameters<TAbi, TFunctionName, TArgs>
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
    args: ReadContractParameters<TAbi, TFunctionName, TArgs>
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
    writeContract: (args) => smartWriteContract(client, args),
    swriteContract: (args, securityParams) =>
      shieldedWriteContract(
        client as unknown as Parameters<typeof shieldedWriteContract>[0],
        args as unknown as Parameters<typeof shieldedWriteContract>[1],
        securityParams
      ),
    twriteContract: (args) =>
      transparentWriteContract(
        client as unknown as Parameters<typeof transparentWriteContract>[0],
        args as unknown as Parameters<typeof transparentWriteContract>[1]
      ),
    dwriteContract: (args, securityParams) => {
      const debugResult = shieldedWriteContractDebug(
        client as unknown as Parameters<typeof shieldedWriteContractDebug>[0],
        args as unknown as Parameters<typeof shieldedWriteContractDebug>[1],
        undefined,
        securityParams
      )
      return debugResult as unknown as Promise<
        ShieldedWriteContractDebugResult<TChain | undefined, TAccount>
      >
    },
    readContract: (args) => smartReadContract(client, client, args),
    sreadContract: (args, securityParams) =>
      signedReadContract(
        client as unknown as Parameters<typeof signedReadContract>[0],
        args as unknown as Parameters<typeof signedReadContract>[1],
        securityParams
      ),
    treadContract: (args) => {
      const readArgs = args as Record<string, unknown>
      if (readArgs.account !== undefined) {
        throw new Error(
          'walletClient.treadContract is always transparent. Seismic zeroes out `from` on transparent `eth_call`, so `account` would be ignored on the node and cause silent bugs. Remove `account` or use `walletClient.sreadContract`.'
        )
      }
      return transparentReadContract(
        client as unknown as Parameters<typeof transparentReadContract>[0],
        args as unknown as Parameters<typeof transparentReadContract>[1]
      )
    },
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
