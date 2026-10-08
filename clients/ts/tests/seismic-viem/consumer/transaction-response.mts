// Compile only: verify public built declarations, without workspace aliases.
import {
  createSeismicDevnet,
  createShieldedPublicClient,
  createShieldedWalletClient,
  localSeismicDevnet,
  sanvil,
} from 'seismic-viem'
import type { GasPayment, SeismicTransactionResponse } from 'seismic-viem'
import type { Hex } from 'viem'
import { createPublicClient, http } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

declare const hash: Hex
const account = privateKeyToAccount(
  '0x0000000000000000000000000000000000000000000000000000000000000001'
)
for (const chain of [
  localSeismicDevnet,
  createSeismicDevnet({ nodeHost: 'fixture.invalid' }),
  sanvil,
]) {
  const ordinary = createPublicClient({ chain, transport: http() })
  const shielded = createShieldedPublicClient({ chain, transport: http() })
  const wallet = await createShieldedWalletClient({
    chain,
    transport: http(),
    account,
  })
  for (const client of [ordinary, shielded, wallet]) {
    const tx = await client.getTransaction({ hash })
    const response: SeismicTransactionResponse = tx
    const maybePayment: GasPayment | undefined = tx.gasPayment
    if (tx.type === 'seismic') {
      const payment: GasPayment = tx.gasPayment
      const nonce: bigint = tx.encryptionNonce
      const expiry: bigint = tx.expiresAtBlock
      const version: number = tx.messageVersion
      // @ts-expect-error: response payment is typed, not any/string
      const invalid: string = tx.gasPayment
      void [payment, nonce, expiry, version, invalid]
    } else {
      const noPayment: undefined = tx.gasPayment
      void noPayment
    }
    const indexed = await client.getTransaction({ blockHash: hash, index: 0 })
    if (indexed.type === 'seismic') {
      const payment: GasPayment = indexed.gasPayment
      void payment
    }
    const block = await client.getBlock({
      blockHash: hash,
      includeTransactions: true,
    })
    for (const embedded of block.transactions) {
      if (embedded.type === 'seismic') {
        const payment: GasPayment = embedded.gasPayment
        void payment
      }
    }
    const hashes = await client.getBlock({
      blockHash: hash,
      includeTransactions: false,
    })
    const txHash: Hex = hashes.transactions[0]
    const pending = await client.getTransaction({
      blockTag: 'pending',
      index: 0,
    })
    const pendingHash: null = pending.blockHash
    void [response, maybePayment, txHash, pendingHash]
  }
}
