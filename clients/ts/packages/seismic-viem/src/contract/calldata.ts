import type {
  Abi,
  AbiFunction,
  Account,
  Chain,
  ContractFunctionArgs,
  ContractFunctionName,
  Hex,
  WriteContractParameters,
} from 'viem'
import { encodeAbiParameters, getAbiItem, toFunctionSelector } from 'viem'
import { formatAbiItem } from 'viem/utils'

import { remapSeismicAbiInputs } from '@sviem/contract/abi.ts'

/**
 * Builds the plaintext calldata for a Seismic contract write before any
 * shielding/encryption is applied.
 *
 * The selector is derived from the original Seismic ABI item so the method id
 * stays aligned with the contract's declared function signature. The arguments
 * are then encoded using a remapped ABI where shielded parameter types are
 * converted into standard ABI-compatible types. This produces the raw calldata
 * shape shared by both:
 *
 * - transparent writes, which send it as-is
 * - shielded writes, which encrypt it before submission
 */
export const getPlaintextCalldata = <
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
  parameters: WriteContractParameters<
    TAbi,
    TFunctionName,
    TArgs,
    TChain,
    TAccount,
    TChainOverride
  >
): Hex => {
  const { abi, functionName, args = [] } = parameters as WriteContractParameters
  const seismicAbi = getAbiItem({ abi, name: functionName }) as AbiFunction
  const selector = toFunctionSelector(formatAbiItem(seismicAbi))
  const ethAbi = remapSeismicAbiInputs(seismicAbi)
  const encodedParams = encodeAbiParameters(ethAbi.inputs, args).slice(2)
  return `${selector}${encodedParams}` as Hex
}
