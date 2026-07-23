import { expect } from 'bun:test'
import { encodeAuthorizationList, signSeismicTxTypedData } from 'seismic-viem'
import { createWalletClient, http, keccak256 } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

const EMPTY_AUTHORIZATION_LIST_HASH =
  '0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347'

const account = privateKeyToAccount(
  '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80'
)

type SeismicTypedDataView = {
  types: {
    TxSeismic: readonly { name: string; type: string }[]
  }
  message: {
    authorizationListHash: string
  }
}

export const testEmptyAuthorizationListHash = () => {
  expect(keccak256(encodeAuthorizationList())).toBe(
    EMPTY_AUTHORIZATION_LIST_HASH
  )
}

export const testTypedDataIncludesAuthorizationListHash = async () => {
  const client = createWalletClient({
    account,
    transport: http('http://127.0.0.1:8545'),
  })
  const { typedData } = await signSeismicTxTypedData(client, {
    type: 'seismic',
    chainId: 31337,
    nonce: 0,
    gasPrice: 1n,
    gas: 100_000n,
    to: '0x1111111111111111111111111111111111111111',
    value: 0n,
    data: '0x',
    encryptionPubkey:
      '0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0',
    encryptionNonce: '0x46a2b6020bba77fcb1e676a6',
    messageVersion: 2,
    recentBlockHash:
      '0x934207181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f90',
    expiresAtBlock: 100n,
    signedRead: false,
  })

  const view = typedData as unknown as SeismicTypedDataView
  expect(view.types.TxSeismic.at(-1)).toEqual({
    name: 'authorizationListHash',
    type: 'bytes32',
  })
  expect(view.message.authorizationListHash).toBe(EMPTY_AUTHORIZATION_LIST_HASH)
}
