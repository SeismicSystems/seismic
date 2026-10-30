import { expect } from 'bun:test'
import {
  getPlaintextCalldata,
  getSeismicAbiItem,
  hasShieldedParams,
} from 'seismic-viem'
import type { Abi } from 'viem'
import { toFunctionSelector } from 'viem'

const overloadedTransferAbi: Abi = [
  {
    type: 'function',
    name: 'transfer',
    inputs: [
      { name: 'to', type: 'address' },
      { name: 'amount', type: 'uint256' },
    ],
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'transfer',
    inputs: [
      { name: 'to', type: 'address' },
      { name: 'amount', type: 'suint256' },
      { name: 'data', type: 'sbytes' },
    ],
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'nonpayable',
  },
]

const overloadedTypeMatchAbi: Abi = [
  {
    type: 'function',
    name: 'lookup',
    inputs: [{ name: 'account', type: 'address' }],
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'lookup',
    inputs: [{ name: 'id', type: 'uint256' }],
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'view',
  },
]

const sampleAddress = '0xd3e8763675e4c425df46cc3b5c0f6cbdac396046'
const sampleBytes = '0xdeadbeef'

export const testResolveOverloadByArgCount = () => {
  const item2Args = getSeismicAbiItem(overloadedTransferAbi, 'transfer', [
    sampleAddress,
    100n,
  ])
  expect(item2Args).toBeDefined()
  expect(item2Args?.inputs.length).toBe(2)
  expect(item2Args?.inputs[1].type).toBe('uint256')

  const item3Args = getSeismicAbiItem(overloadedTransferAbi, 'transfer', [
    sampleAddress,
    100n,
    sampleBytes,
  ])
  expect(item3Args).toBeDefined()
  expect(item3Args?.inputs.length).toBe(3)
  expect(item3Args?.inputs[1].type).toBe('suint256')
  expect(item3Args?.inputs[2].type).toBe('sbytes')
}

export const testHasShieldedParamsDisambiguatesOverloads = () => {
  // Calling transfer with 2 args (address, uint256) is transparent
  expect(
    hasShieldedParams(overloadedTransferAbi, 'transfer', [sampleAddress, 100n])
  ).toBe(false)

  // Calling transfer with 3 args (address, suint256, sbytes) is shielded
  expect(
    hasShieldedParams(overloadedTransferAbi, 'transfer', [
      sampleAddress,
      100n,
      sampleBytes,
    ])
  ).toBe(true)
}

export const testResolveOverloadByTypeMatch = () => {
  const byAddress = getSeismicAbiItem(overloadedTypeMatchAbi, 'lookup', [
    sampleAddress,
  ])
  expect(byAddress).toBeDefined()
  expect(byAddress?.inputs[0].type).toBe('address')

  const byUint = getSeismicAbiItem(overloadedTypeMatchAbi, 'lookup', [12345n])
  expect(byUint).toBeDefined()
  expect(byUint?.inputs[0].type).toBe('uint256')
}

export const testResolveBySelector = () => {
  const selector2Args = toFunctionSelector('transfer(address,uint256)')
  const matched = getSeismicAbiItem(overloadedTransferAbi, selector2Args)
  expect(matched).toBeDefined()
  expect(matched?.inputs.length).toBe(2)
}

export const testBackwardCompatibilityWithoutArgs = () => {
  // When args are omitted, returns the first matching function name in the ABI
  const fallbackItem = getSeismicAbiItem(overloadedTransferAbi, 'transfer')
  expect(fallbackItem).toBeDefined()
  expect(fallbackItem?.inputs.length).toBe(2)

  const fallbackShielded = hasShieldedParams(overloadedTransferAbi, 'transfer')
  expect(fallbackShielded).toBe(false)
}

export const testPlaintextCalldataForOverloads = () => {
  const calldata2Args = getPlaintextCalldata({
    abi: overloadedTransferAbi,
    functionName: 'transfer',
    args: [sampleAddress, 100n],
  })

  const calldata3Args = getPlaintextCalldata({
    abi: overloadedTransferAbi,
    functionName: 'transfer',
    args: [sampleAddress, 100n, sampleBytes],
  })

  // The 4-byte function selectors must be different because transfer(address,uint256) != transfer(address,suint256,sbytes)
  const selector2 = calldata2Args.slice(0, 10)
  const selector3 = calldata3Args.slice(0, 10)
  expect(selector2).not.toBe(selector3)

  // Verify selector2 matches transfer(address,uint256)
  expect(selector2).toBe(toFunctionSelector('transfer(address,uint256)'))
  // Verify selector3 matches transfer(address,suint256,sbytes)
  expect(selector3).toBe(
    toFunctionSelector('transfer(address,suint256,sbytes)')
  )
}
