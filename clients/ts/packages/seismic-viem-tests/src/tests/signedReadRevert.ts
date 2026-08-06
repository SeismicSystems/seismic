/**
 * Regression tests for signed-read revert confidentiality.
 *
 * A contract's revert data can embed private state (e.g. a custom error like
 * `revert InsufficientBalance(actualBalance)`), just like a successful return
 * value can. The node therefore encrypts the revert output of a signed read
 * under the caller's key instead of returning it in cleartext, and the client
 * transparently decrypts it so revert reasons still decode as usual.
 *
 * These tests assert both halves of that contract:
 *  1. wire level: the raw RPC error from the node never contains the secret
 *     in cleartext, and its `data` decrypts to the real revert data
 *  2. client level: `signedCall` / signed gas estimation surface the decoded
 *     revert reason to the signer via automatic decryption
 *
 * Mirrors the node-side e2e test
 * `test_eth_call_signed_read_revert_leaks_private_data` in seismic-reth.
 */
import { expect } from 'bun:test'
import {
  buildTxSeismicMetadata,
  randomEncryptionNonce,
  serializeSeismicTransaction,
} from 'seismic-viem'
import type { Account, Chain, Hex } from 'viem'
import { BaseError, decodeErrorResult } from 'viem'

import { httpPublicClient, httpWalletClient } from '@sviem-tests/clients.ts'

type SignedReadRevertTestArgs = {
  chain: Chain
  url: string
  account: Account
}

/**
 * RevertLeak test contract: reverts with a decimal-formatted private
 * (`suint256`) value baked into the revert reason string.
 *
 * Solidity source (compiled with seismic solc, evm-version=mercury):
 *
 *   contract RevertLeak {
 *       suint256 private secret;
 *       function setSecret(suint256 value) public { secret = value; }
 *       function revertWithSecret() public view {
 *           uint256 revealed = uint256(secret);
 *           revert(string(abi.encodePacked("secret=", toString(revealed))));
 *       }
 *       function toString(uint256 value) internal pure returns (string memory) { ... }
 *   }
 */
const revertLeakBytecode: Hex =
  '0x6080604052348015600e575f5ffd5b506105e78061001c5f395ff3fe608060405234801561000f575f5ffd5b5060043610610034575f3560e01c80638987b12814610038578063e0f7bec414610054575b5f5ffd5b610052600480360381019061004d9190610260565b61005e565b005b61005c610067565b005b805f8190b15050565b5f5fb09050610075816100d0565b6040516020016100859190610327565b6040516020818303038152906040526040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100c791906103a0565b60405180910390fd5b60605f8203610116576040518060400160405280600181526020017f30000000000000000000000000000000000000000000000000000000000000008152509050610224565b5f8290505f5b5f821461014557808061012e906103f6565b915050600a8261013e919061046a565b915061011c565b5f8167ffffffffffffffff8111156101605761015f61049a565b5b6040519080825280601f01601f1916602001820160405280156101925781602001600182028036833780820191505090505b5090505b5f851461021d576001826101aa91906104c7565b9150600a856101b991906104fa565b60306101c5919061052a565b60f81b8183815181106101db576101da61055d565b5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff191690815f1a905350600a85610216919061046a565b9450610196565b8093505050505b919050565b5f5ffd5b5f819050919050565b61023f8161022d565b8114610249575f5ffd5b50565b5f8135905061025a81610236565b92915050565b5f6020828403121561027557610274610229565b5b5f6102828482850161024c565b91505092915050565b5f81905092915050565b7f7365637265743d000000000000000000000000000000000000000000000000005f82015250565b5f6102c960078361028b565b91506102d482610295565b600782019050919050565b5f81519050919050565b8281835e5f83830152505050565b5f610301826102df565b61030b818561028b565b935061031b8185602086016102e9565b80840191505092915050565b5f610331826102bd565b915061033d82846102f7565b915081905092915050565b5f82825260208201905092915050565b5f601f19601f8301169050919050565b5f610372826102df565b61037c8185610348565b935061038c8185602086016102e9565b61039581610358565b840191505092915050565b5f6020820190508181035f8301526103b88184610368565b905092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f819050919050565b5f610400826103ed565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8203610432576104316103c0565b5b600182019050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b5f610474826103ed565b915061047f836103ed565b92508261048f5761048e61043d565b5b828204905092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b5f6104d1826103ed565b91506104dc836103ed565b92508282039050818111156104f4576104f36103c0565b5b92915050565b5f610504826103ed565b915061050f836103ed565b92508261051f5761051e61043d565b5b828206905092915050565b5f610534826103ed565b915061053f836103ed565b9250828201905080821115610557576105566103c0565b5b92915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffdfea2646970667358221220efc2a5dedbd6d373c0639d2ee3f210d4206d14702d22025bd9da8b583389b2f664736f6c637829302e382e33312d646576656c6f702e323032352e31312e31322b636f6d6d69742e3464313362633133005a'

const revertLeakAbi = [
  {
    type: 'function',
    name: 'setSecret',
    inputs: [{ name: 'value', type: 'suint256', internalType: 'suint256' }],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'revertWithSecret',
    inputs: [],
    outputs: [],
    stateMutability: 'view',
  },
] as const

// `revertWithSecret` is `view` in Solidity, but the estimateGas test invokes
// it through the write path (writes are what trigger signed gas estimation).
// Re-declare it as nonpayable so `swriteContract` accepts it; the on-chain
// execution is identical.
const revertLeakWriteAbi = [
  revertLeakAbi[0],
  { ...revertLeakAbi[1], stateMutability: 'nonpayable' },
] as const

const SECRET = 987654321n

// Selector of revertWithSecret()
const REVERT_WITH_SECRET_CALLDATA: Hex = '0xe0f7bec4'

/** Walks the viem error chain and returns the first `data` payload found. */
const extractErrorData = (err: unknown): Hex | undefined => {
  if (!(err instanceof BaseError)) return undefined
  const found = err.walk(
    (e) => typeof (e as { data?: unknown }).data !== 'undefined'
  ) as { data?: Hex | { data?: Hex } } | null
  const data = found?.data
  return typeof data === 'object' ? data?.data : data
}

/** Deploy RevertLeak and set its private secret via a shielded write. */
const deployAndSetSecret = async ({
  chain,
  url,
  account,
}: SignedReadRevertTestArgs) => {
  const publicClient = httpPublicClient({ chain, url })
  const walletClient = await httpWalletClient({ chain, url, account })

  const deployTx = await walletClient.deployContract({
    abi: revertLeakAbi,
    bytecode: revertLeakBytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const setTx = await walletClient.swriteContract({
    address,
    abi: revertLeakAbi,
    functionName: 'setSecret',
    args: [SECRET],
  })
  const setReceipt = await publicClient.waitForTransactionReceipt({
    hash: setTx,
  })
  expect(setReceipt.status).toBe('success')

  return { publicClient, walletClient, address }
}

/**
 * Wire level: send a signed read as raw tx bytes straight to the given RPC
 * method (bypassing the client's automatic revert decryption) and assert on
 * the raw RPC error exactly as the node produced it:
 *  1. the plaintext secret appears nowhere in the error
 *  2. the error `data` decrypts to the real revert reason
 */
const wireLevelRevertTest = async (
  args: SignedReadRevertTestArgs,
  method: 'eth_call' | 'eth_estimateGas'
) => {
  const { publicClient, walletClient, address } = await deployAndSetSecret(args)
  const { account } = args

  // Pin the metadata inputs so the exact same metadata (and thus the same
  // AES key + AAD) can be reconstructed to decrypt the revert ciphertext.
  const encryptionNonce = randomEncryptionNonce()
  const latestBlock = await publicClient.getBlock({ blockTag: 'latest' })
  const metadata = await buildTxSeismicMetadata(walletClient, {
    account: account.address,
    to: address,
    encryptionNonce,
    recentBlockHash: latestBlock.hash,
    expiresAtBlock: latestBlock.number + 100n,
    // Force message_version 0 so the request goes as raw signed tx bytes.
    typedDataTx: false,
    signedRead: true,
  })
  const encryptedCalldata = await walletClient.encrypt(
    REVERT_WITH_SECRET_CALLDATA,
    metadata
  )

  const serializedTransaction = await account.signTransaction!(
    {
      type: 'seismic',
      chainId: metadata.legacyFields.chainId,
      nonce: metadata.legacyFields.nonce,
      gasPrice: await publicClient.getGasPrice(),
      gas: 30_000_000n,
      to: address,
      value: 0n,
      data: encryptedCalldata,
      ...metadata.seismicElements,
    },
    { serializer: serializeSeismicTransaction }
  )

  let caught: unknown
  try {
    await walletClient.request({
      method: method as 'eth_call',
      params:
        method === 'eth_call'
          ? [serializedTransaction, 'latest']
          : ([serializedTransaction] as unknown as [Hex, 'latest']),
    })
  } catch (err) {
    caught = err
  }
  expect(caught, `${method} on revertWithSecret() must reject`).toBeDefined()

  // 1. The plaintext secret must not appear anywhere in the raw RPC error.
  const errText = `${(caught as Error).message}\n${JSON.stringify(caught)}`
  expect(
    errText.includes(`secret=${SECRET}`),
    `${method} leaked the private value in cleartext on the wire: ${errText}`
  ).toBe(false)

  // 2. The signer must be able to decrypt the revert data and recover the
  // real revert reason.
  const ciphertext = extractErrorData(caught)
  expect(
    ciphertext,
    `${method} revert error should carry the encrypted revert output as data`
  ).toBeDefined()
  const decrypted = await walletClient.decrypt(ciphertext!, metadata)
  const { args: decodedArgs } = decodeErrorResult({ data: decrypted })
  expect(decodedArgs?.[0]).toBe(`secret=${SECRET}`)
}

/** Wire-level signed-read revert confidentiality for `eth_call`. */
export const testSignedReadRevertWireFormatIsEncrypted = async (
  args: SignedReadRevertTestArgs
) => wireLevelRevertTest(args, 'eth_call')

/**
 * Wire-level signed-read revert confidentiality for `eth_estimateGas`.
 *
 * NOTE: fails against sanvil as of seismic-foundry 979e9446a — its
 * gas-estimation path bypasses the encrypting call wrapper
 * (`do_estimate_gas_with_state` calls the plain `call_with_state`), so
 * revert output goes out in cleartext.
 */
export const testEstimateGasRevertWireFormatIsEncrypted = async (
  args: SignedReadRevertTestArgs
) => wireLevelRevertTest(args, 'eth_estimateGas')

/**
 * Client level: `signedCall` transparently decrypts the encrypted revert
 * output, so the signer sees the decoded revert reason as on a transparent
 * chain.
 */
export const testSignedCallDecryptsRevertReason = async (
  args: SignedReadRevertTestArgs
) => {
  const { walletClient, address } = await deployAndSetSecret(args)

  let caught: unknown
  try {
    await walletClient.signedCall({
      to: address,
      data: REVERT_WITH_SECRET_CALLDATA,
      account: args.account.address,
    })
  } catch (err) {
    caught = err
  }
  expect(caught, 'revertWithSecret() must revert').toBeDefined()

  // The error data must be the decrypted plaintext revert data...
  const data = extractErrorData(caught)
  expect(
    data,
    'signedCall revert error should carry plaintext revert data'
  ).toBeDefined()
  const { args: decodedArgs } = decodeErrorResult({ data: data! })
  expect(decodedArgs?.[0]).toBe(`secret=${SECRET}`)

  // ...and the surfaced message should contain the decoded reason.
  const message = (caught as Error).message
  expect(
    message.includes(`secret=${SECRET}`),
    `signedCall should surface the decrypted revert reason, got: ${message}`
  ).toBe(true)
}

/**
 * Client level, `eth_estimateGas`: a shielded write without explicit `gas`
 * triggers signed gas estimation, which executes the call on the node. On
 * revert, the client decrypts the revert output so the signer sees the
 * decoded reason (and the wire never carried it in cleartext — covered by
 * the wire-level test above, as both endpoints share the node-side path).
 */
export const testEstimateGasDecryptsRevertReason = async (
  args: SignedReadRevertTestArgs
) => {
  const { walletClient, address } = await deployAndSetSecret(args)

  // No explicit `gas`: routes through signed eth_estimateGas, which reverts.
  let caught: unknown
  try {
    await walletClient.swriteContract({
      address,
      abi: revertLeakWriteAbi,
      functionName: 'revertWithSecret',
    })
  } catch (err) {
    caught = err
  }
  expect(caught, 'estimateGas on revertWithSecret() must reject').toBeDefined()

  const data = extractErrorData(caught)
  expect(
    data,
    'signed estimateGas revert error should carry plaintext revert data'
  ).toBeDefined()
  const { args: decodedArgs } = decodeErrorResult({ data: data! })
  expect(decodedArgs?.[0]).toBe(`secret=${SECRET}`)
}
