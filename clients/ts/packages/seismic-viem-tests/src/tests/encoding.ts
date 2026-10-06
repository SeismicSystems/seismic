import { expect } from 'bun:test'
import {
  buildTxSeismicMetadata,
  serializeSeismicTransaction,
  signSeismicTxTypedData,
} from 'seismic-viem'
import { compressPublicKey } from 'seismic-viem'
import type { TransactionSerializableSeismic } from 'seismic-viem'
import type { Account, Chain, Hex, TransactionSerializableLegacy } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'
import { prepareTransactionRequest } from 'viem/actions'
import { anvil } from 'viem/chains'

import { httpWalletClient } from '@sviem-tests/clients.ts'

type EncodingParams = {
  chain: Chain
  url: string
  encryptionSk: Hex
  encryptionPubkey: Hex
  account: Account
}

export const testSeismicTxEncoding = async ({
  chain,
  url,
  account,
  encryptionSk,
  encryptionPubkey,
}: EncodingParams) => {
  expect(encryptionPubkey).toBe(
    compressPublicKey(privateKeyToAccount(encryptionSk).publicKey)
  )
  const client = await httpWalletClient({ chain, url, account, encryptionSk })

  const plaintext =
    '0xfc3c2cf4943c327f19af0efaf3b07201f608dd5c8e3954399a919b72588d3872b6819ac3d13d3656cbb38833a39ffd1e73963196a1ddfa9e4a5d595fdbebb875'
  const encryptionNonce = '0x46a2b6020bba77fcb1e676a6'
  const metadata = await buildTxSeismicMetadata(client, {
    account,
    nonce: 2,
    to: '0xd3e8763675e4c425df46cc3b5c0f6cbdac396046',
    value: 1000000000000000n,
    encryptionNonce,
    recentBlockHash:
      '0x934207181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f90',
    expiresAtBlock: 100n,
  })

  const encryptedCalldata = await client.encrypt(plaintext, metadata)
  const tx: TransactionSerializableLegacy = {
    chainId: chain.id,
    nonce: metadata.legacyFields.nonce,
    gasPrice: 1000000000n,
    gas: 100000n,
    to: metadata.legacyFields.to,
    value: metadata.legacyFields.value,
    data: encryptedCalldata,
  }

  const preparedTx = await prepareTransactionRequest(client, tx)
  const seismicTx: TransactionSerializableSeismic = {
    ...preparedTx,
    ...metadata.seismicElements,
    gasPayment: { type: 'auto' },
    type: 'seismic',
  }
  // Match messageVersion=2: sign EIP-712, then exercise the raw codec.
  const { signature } = await signSeismicTxTypedData(client, seismicTx)
  const serializedTransaction = serializeSeismicTransaction(seismicTx, {
    r: signature.r,
    s: signature.s,
    yParity: Number(signature.yParity) as 0 | 1,
  })

  // const signature = {
  //   r: '0x1e7a28fd3647ab10173d940fe7e561f7b06185d3d6a93b83b2f210055dd27f04',
  //   s: '0x779d1157c4734323923df2f41073ecb016719a577ce774ef4478c9b443caacb3',
  //   v: '28',
  //   yParity: 1,
  // }

  const expected =
    chain.id === anvil.id
      ? '0x4af90116827a6902843b9aca00830186a0c2808094d3e8763675e4c425df46cc3b5c0f6cbdac39604687038d7ea4c68000a1028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a08c46a2b6020bba77fcb1e676a602a0934207181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f906480b850bf645e68de8096b62950fac2d5bceb71ab1a085aed2e973a8b4f961ca77209f99116130edecd27c39fc62e1b3c05ff42d9e4382f987fc55c2011f8e4f2e662045c27cf78e5c395d6d53d08d452d6dc38c080a0ab59ee17f17b5cb47b313dd2847c34a493fddd5712ba53424cd3054cc5a24965a047c1c63c7fe2163fee4c3264cf43df2ac9904c822cd6d4cd50c2e1cb40cff90f'
      : '0x4af9011682140402843b9aca00830186a0c2808094d3e8763675e4c425df46cc3b5c0f6cbdac39604687038d7ea4c68000a1028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a08c46a2b6020bba77fcb1e676a602a0934207181885f6859ca848f5f01091d1957444a920a2bfb262fa043c6c239f906480b850bf645e68de8096b62950fac2d5bceb71ab1a085aed2e973a8b4f961ca77209f99116130edecd27c39fc62e1b3c05ff42d9e4382f987fc55c2011f8e4f2e6620462173f479fc03f28c1b7f00e8f75df88c001a015a3fdf63b097ea66062ea0daccec7a5ddc4bdedd2cf835fa09aeb8b3509f618a0686b7c4e08a4f8f808b16fa2fa024b5505ea33c1d0ddb33f30c4884400c56960'
  // @ts-ignore
  expect(serializedTransaction).toBe(expected)
}
