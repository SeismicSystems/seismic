import type {
  Abi,
  Account,
  Address,
  Chain,
  ContractFunctionArgs,
  ContractFunctionName,
  Hash,
  Hex,
  Transport,
  WalletActions,
  WriteContractParameters,
  WriteContractReturnType,
} from 'viem'
import { numberToHex } from 'viem'
import { writeContract } from 'viem/actions'

import { SEISMIC_TX_TYPE, SeismicSecurityParams } from '@sviem/chain.ts'
import type { ShieldedWalletClient } from '@sviem/client.ts'
import { hasShieldedParams } from '@sviem/contract/abi.ts'
import { getPlaintextCalldata } from '@sviem/contract/calldata.ts'
import { randomEncryptionNonce } from '@sviem/crypto/nonce.ts'
import { buildTxSeismicMetadata } from '@sviem/metadata.ts'
import { sendShieldedTransaction } from '@sviem/tx/sendShielded.ts'
import type { SendSeismicTransactionParameters } from '@sviem/tx/types.ts'

/**
 * Shared smart-write routing used by both the wallet actions and the
 * ABI/address-bound contract wrapper.
 *
 * This helper inspects the target ABI/function and chooses the write path:
 *
 * - if the function has shielded params, it routes to
 *   `shieldedWriteContract(...)`
 * - otherwise it routes to viem's `writeContract(...)`
 *
 * It does not currently route non-shielded writes through
 * `transparentWriteContract(...)`. That helper exists for the explicit
 * force-transparent API (`twriteContract` / `contract.twrite.*`), where Seismic
 * remaps ABI inputs and sends plaintext calldata through the transparent send
 * path directly.
 */
export async function smartWriteContract<
  TTransport extends Transport,
  TChain extends Chain | undefined,
  TAccount extends Account,
  const TAbi extends Abi | readonly unknown[],
  TFunctionName extends ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
  TArgs extends ContractFunctionArgs<
    TAbi,
    'nonpayable' | 'payable',
    TFunctionName
  >,
  TChainOverride extends Chain | undefined = undefined,
>(
  client: ShieldedWalletClient<TTransport, TChain, TAccount>,
  parameters: WriteContractParameters<
    TAbi,
    TFunctionName,
    TArgs,
    TChain,
    TAccount,
    TChainOverride
  >
): Promise<WriteContractReturnType> {
  const writeArgs = parameters as unknown as {
    abi: Abi
    functionName: string
  }
  if (hasShieldedParams(writeArgs.abi, writeArgs.functionName)) {
    return shieldedWriteContract(
      client as unknown as Parameters<typeof shieldedWriteContract>[0],
      parameters as unknown as Parameters<typeof shieldedWriteContract>[1]
    )
  }

  // No shielded params -> the ABI is valid for viem as-is.
  return writeContract(
    client as unknown as Parameters<typeof writeContract>[0],
    parameters as unknown as Parameters<typeof writeContract>[1]
  )
}

/**
 * Sends a transparent (non-encrypted) write to a contract whose ABI may
 * contain shielded types. The selector is derived from the Seismic ABI while
 * parameters are encoded with remapped standard types.
 */
export async function transparentWriteContract<
  TChain extends Chain | undefined,
  TAccount extends Account,
  const TAbi extends Abi | readonly unknown[],
  TFunctionName extends ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
  TArgs extends ContractFunctionArgs<
    TAbi,
    'nonpayable' | 'payable',
    TFunctionName
  >,
  TChainOverride extends Chain | undefined = undefined,
>(
  client: Pick<WalletActions<TChain, TAccount>, 'sendTransaction'>,
  parameters: WriteContractParameters<
    TAbi,
    TFunctionName,
    TArgs,
    TChain,
    TAccount,
    TChainOverride
  >
): Promise<WriteContractReturnType> {
  const {
    abi: _abi,
    functionName: _fn,
    args: _args,
    address,
    ...txOptions
  } = parameters as WriteContractParameters & { address: Address }
  const data = getPlaintextCalldata(parameters)
  return client.sendTransaction({
    to: address,
    data,
    ...(txOptions as Record<string, unknown>),
  } as unknown as Parameters<typeof client.sendTransaction>[0])
}

export async function shieldedWriteContract<
  TTransport extends Transport,
  TChain extends Chain | undefined,
  TAccount extends Account,
  const TAbi extends Abi | readonly unknown[],
  TFunctionName extends ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
  TArgs extends ContractFunctionArgs<
    TAbi,
    'nonpayable' | 'payable',
    TFunctionName
  >,
  TChainOverride extends Chain | undefined = undefined,
>(
  client: ShieldedWalletClient<TTransport, TChain, TAccount>,
  parameters: WriteContractParameters<
    TAbi,
    TFunctionName,
    TArgs,
    TChain,
    TAccount,
    TChainOverride
  >,
  securityParams?: SeismicSecurityParams
): Promise<WriteContractReturnType> {
  const plaintextCalldata = getPlaintextCalldata(parameters)
  const request = buildShieldedWriteRequest(
    client,
    parameters,
    plaintextCalldata
  )
  return sendShieldedTransaction(client, request, securityParams)
}

type PlaintextTransactionParameters = {
  to: Address | null
  data: Hex
  nonce?: number
  gas?: bigint
  gasPrice?: bigint
  value?: bigint
  type: Hex
}

export type ShieldedWriteContractDebugResult<
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends Account | undefined = Account | undefined,
> = {
  plaintextTx: PlaintextTransactionParameters
  shieldedTx: SendSeismicTransactionParameters<TChain, TAccount>
  txHash: Hash
}

/**
 * Sends a shielded write and returns both plaintext and shielded transaction
 * views for inspection.
 *
 * This is primarily useful for debugging Seismic write behavior because the
 * caller can see:
 *
 * - the plaintext calldata transaction shape
 * - the shielded transaction payload that was derived from it
 * - the resulting transaction hash
 *
 * Note: despite the debug-oriented name, this function currently does
 * broadcast the shielded transaction and returns a real `txHash`.
 *
 * @param client - The wallet client used to prepare, encrypt, and send the write.
 * @param parameters - The contract write parameters, matching viem's
 * `writeContract` shape.
 * @param checkContractDeployed - When true, verifies that code exists at the
 * target address before sending.
 * @param securityParams - Optional advanced Seismic metadata overrides. Most
 * callers should omit these; they are mainly useful for deterministic
 * tests/debugging, explicit expiry control, and low-level interop.
 * @returns Both plaintext and shielded transaction representations, plus the
 * submitted transaction hash.
 */
export async function shieldedWriteContractDebug<
  TTransport extends Transport,
  TChain extends Chain | undefined,
  TAccount extends Account,
  const TAbi extends Abi | readonly unknown[],
  TFunctionName extends ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
  TArgs extends ContractFunctionArgs<
    TAbi,
    'nonpayable' | 'payable',
    TFunctionName
  >,
  TChainOverride extends Chain | undefined = undefined,
>(
  client: ShieldedWalletClient<TTransport, TChain, TAccount>,
  parameters: WriteContractParameters<
    TAbi,
    TFunctionName,
    TArgs,
    TChain,
    TAccount,
    TChainOverride
  >,
  checkContractDeployed?: boolean,
  securityParams: SeismicSecurityParams = {}
): Promise<ShieldedWriteContractDebugResult<TChain, TAccount>> {
  const {
    blocksWindow = 100n,
    encryptionNonce: userEncNonce,
    recentBlockHash,
    expiresAtBlock,
  } = securityParams
  if (checkContractDeployed) {
    const code = await client.getCode({ address: parameters.address })
    if (code === undefined) {
      throw new Error('Contract not found')
    }
  }
  const plaintextCalldata = getPlaintextCalldata(parameters)
  const request = buildShieldedWriteRequest(
    client,
    parameters,
    plaintextCalldata
  )
  const encryptionNonce = userEncNonce ?? randomEncryptionNonce()
  const metadata = await buildTxSeismicMetadata(client, {
    account: parameters.account || client.account,
    nonce: request.nonce,
    to: request.to!,
    value: request.value,
    encryptionNonce,
    blocksWindow,
    recentBlockHash,
    expiresAtBlock,
    signedRead: false,
  })
  const txHash = await sendShieldedTransaction(client, request, {
    encryptionNonce,
    recentBlockHash: metadata.seismicElements.recentBlockHash,
    expiresAtBlock: metadata.seismicElements.expiresAtBlock,
  })
  return {
    plaintextTx: {
      to: request.to || null,
      data: plaintextCalldata,
      type: numberToHex(SEISMIC_TX_TYPE),
      nonce: request.nonce,
      gas: request.gas,
      gasPrice: request.gasPrice,
      value: request.value,
    },
    shieldedTx: {
      type: numberToHex(SEISMIC_TX_TYPE),
      ...request,
      ...metadata.seismicElements,
    },
    txHash,
  }
}

/**
 * Builds the Seismic transaction request consumed by the shielded send path.
 *
 * At this stage the calldata is still plaintext. This helper only maps the
 * contract-write parameters into the request fields needed by
 * `sendShieldedTransaction(...)`, preserving user-supplied tx options like
 * `gas`, `gasPrice`, `value`, and `nonce`.
 */
function buildShieldedWriteRequest<
  TTransport extends Transport,
  TChain extends Chain | undefined,
  TAccount extends Account,
  const TAbi extends Abi | readonly unknown[],
  TFunctionName extends ContractFunctionName<TAbi, 'nonpayable' | 'payable'>,
  TArgs extends ContractFunctionArgs<
    TAbi,
    'nonpayable' | 'payable',
    TFunctionName
  >,
  TChainOverride extends Chain | undefined = undefined,
>(
  client: ShieldedWalletClient<TTransport, TChain, TAccount>,
  parameters: WriteContractParameters<
    TAbi,
    TFunctionName,
    TArgs,
    TChain,
    TAccount,
    TChainOverride
  >,
  plaintextCalldata: Hex
): SendSeismicTransactionParameters<TChain, TAccount> {
  const { address, gas, gasPrice, value, nonce } = parameters
  return {
    account: client.account,
    chain: undefined,
    to: address,
    data: plaintextCalldata,
    nonce,
    value,
    gas,
    gasPrice,
  }
}
