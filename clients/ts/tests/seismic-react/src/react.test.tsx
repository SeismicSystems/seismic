import { describe, expect, test } from 'bun:test'
import {
  useShieldedContract,
  useShieldedWallet,
  useShieldedWriteContract,
  useSignedReadContract,
} from 'seismic-react'

import { renderHook } from '@testing-library/react'

// A minimal ABI for testing
const testAbi = [
  {
    inputs: [{ name: 'x', type: 'uint256' }],
    name: 'set',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
  {
    inputs: [],
    name: 'get',
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view',
    type: 'function',
  },
] as const

const TEST_ADDRESS = '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266'

describe('useShieldedWallet', () => {
  test('throws when used outside ShieldedWalletProvider', () => {
    expect(() => {
      renderHook(() => useShieldedWallet())
    }).toThrow('useWalletClient must be used within a WalletClientProvider')
  })
})

describe('useShieldedContract', () => {
  test('throws when used outside ShieldedWalletProvider', () => {
    expect(() => {
      renderHook(() =>
        useShieldedContract({ abi: testAbi, address: TEST_ADDRESS })
      )
    }).toThrow('useWalletClient must be used within a WalletClientProvider')
  })
})

describe('useShieldedWriteContract', () => {
  test('throws when used outside ShieldedWalletProvider', () => {
    expect(() => {
      renderHook(() =>
        useShieldedWriteContract({
          abi: testAbi,
          address: TEST_ADDRESS,
          functionName: 'set',
          args: [1n],
        })
      )
    }).toThrow('useWalletClient must be used within a WalletClientProvider')
  })
})

describe('useSignedReadContract', () => {
  test('throws when used outside ShieldedWalletProvider', () => {
    expect(() => {
      renderHook(() =>
        useSignedReadContract({
          abi: testAbi,
          address: TEST_ADDRESS,
          functionName: 'get',
        })
      )
    }).toThrow('useWalletClient must be used within a WalletClientProvider')
  })
})
