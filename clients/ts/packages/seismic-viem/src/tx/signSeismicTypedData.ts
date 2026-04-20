/**
 * EIP-712 typed-data signing path for Seismic transactions.
 *
 * Used when the client's account is a wallet it does not directly control
 * (MetaMask, WalletConnect, Ledger, Trezor, etc.) and therefore cannot hand
 * us a locally-signed raw Seismic tx. Instead, we build an EIP-712 typed
 * message whose schema matches what the Seismic node validates, ask the
 * wallet to sign that, and forward the `{ typedData, signature }` pair to
 * the node via `eth_sendRawTransaction` / `eth_call` (the node
 * reconstructs and verifies the tx from those two pieces).
 *
 * Local (private-key) accounts use a different path entirely — they sign
 * the serialized Seismic tx directly. See `tx/sendShielded.ts` and
 * `tx/signedCall.ts` for the branching.
 */
import {
  Account,
  Chain,
  Client,
  Hex,
  Transport,
  TypedData,
  numberToHex,
  parseSignature,
} from 'viem'
import { SignTypedDataParameters, signTypedData } from 'viem/actions'

import {
  type TransactionSerializableSeismic,
  type TxSeismic,
} from '@sviem/tx/seismicTx.ts'

/**
 * Seismic `messageVersion` selector embedded in the tx.
 *
 *  - `0` — raw serialized Seismic tx signed by a local account
 *  - `1` — reserved for personal_sign (ledger/trezor friendly)
 *  - `2` — EIP-712 typed-data signing (this module's path)
 */
export const TYPED_DATA_MESSAGE_VERSION: number = 2

const seismicTxTypedData = <
  typedData extends TypedData | Record<string, unknown>,
  primaryType extends keyof typedData | 'EIP712Domain',
  account extends Account | undefined,
>(
  tx: TransactionSerializableSeismic
): SignTypedDataParameters<typedData, primaryType, account> => {
  if (tx.chainId === undefined) {
    throw new Error('Seismic transactions require chainId argument')
  }
  if (tx.encryptionPubkey === undefined) {
    throw new Error('Seismic transactions require encryptionPubkey argument')
  }
  if (tx.encryptionNonce === undefined) {
    throw new Error('Seismic transactions require encryptionNonce argument')
  }
  if (tx.recentBlockHash === undefined) {
    throw new Error('Seismic transactions require recentBlockHash')
  }
  if (tx.expiresAtBlock === undefined) {
    throw new Error('Seismic transactions require expiresAtBlock')
  }
  if (tx.signedRead === undefined) {
    throw new Error('Seismic transactions require signedRead')
  }

  const isCreate = tx.to === undefined || tx.to === null
  const message: TxSeismic = {
    chainId: tx.chainId,
    nonce: tx.nonce !== undefined ? BigInt(tx.nonce) : undefined,
    gasPrice: tx.gasPrice && BigInt(tx.gasPrice),
    gasLimit: tx.gas && BigInt(tx.gas),
    to: isCreate ? '0x0000000000000000000000000000000000000000' : tx.to,
    isCreate,
    value: tx.value ? BigInt(tx.value) : 0n,
    input: tx.data ?? '0x',
    encryptionPubkey: tx.encryptionPubkey,
    encryptionNonce: tx.encryptionNonce,
    messageVersion: TYPED_DATA_MESSAGE_VERSION,
    recentBlockHash: tx.recentBlockHash,
    expiresAtBlock: tx.expiresAtBlock,
    signedRead: tx.signedRead,
  }

  // @ts-ignore
  return {
    types: {
      EIP712Domain: [
        { name: 'name', type: 'string' },
        { name: 'version', type: 'string' },
        { name: 'chainId', type: 'uint256' },
        { name: 'verifyingContract', type: 'address' },
      ],
      TxSeismic: [
        { name: 'chainId', type: 'uint64' },
        { name: 'nonce', type: 'uint64' },
        { name: 'gasPrice', type: 'uint128' },
        { name: 'gasLimit', type: 'uint64' },
        { name: 'to', type: 'address' },
        { name: 'isCreate', type: 'bool' },
        { name: 'value', type: 'uint256' },
        // compressed secp256k1 public key (33 bytes)
        { name: 'input', type: 'bytes' },
        { name: 'encryptionPubkey', type: 'bytes' },
        { name: 'encryptionNonce', type: 'uint96' },
        { name: 'messageVersion', type: 'uint8' },
        { name: 'recentBlockHash', type: 'bytes32' },
        { name: 'expiresAtBlock', type: 'uint64' },
        { name: 'signedRead', type: 'bool' },
      ],
    },
    primaryType: 'TxSeismic',
    domain: {
      name: 'Seismic Transaction',
      version: `${TYPED_DATA_MESSAGE_VERSION}`,
      chainId: tx.chainId,
      // no verifying contract since this happens in RPC
      verifyingContract: '0x0000000000000000000000000000000000000000',
    },
    message,
  }
}

type PrimitiveSignature = {
  r: Hex
  s: Hex
  yParity: Hex
}

/**
 * Builds the EIP-712 typed-data envelope for a Seismic tx and asks the
 * wallet to sign it via `eth_signTypedData_v4`.
 *
 * Returns both the typed-data payload and the parsed signature (`r`, `s`,
 * `yParity`). Callers forward the pair to the node as the tx's
 * `serializedTransaction` — the node reconstructs the Seismic tx from the
 * typed data and recovers the sender from the signature.
 *
 * Throws if the tx is missing any field required by the EIP-712 schema
 * (`chainId`, `encryptionPubkey`, `encryptionNonce`, `recentBlockHash`,
 * `expiresAtBlock`, `signedRead`) — `buildTxSeismicMetadata` fills these.
 */
export const signSeismicTxTypedData = async <
  TTransport extends Transport,
  TChain extends Chain | undefined,
  TAccount extends Account,
  typedData extends TypedData | Record<string, unknown>,
  primaryType extends keyof typedData | 'EIP712Domain',
>(
  client: Client<TTransport, TChain, TAccount>,
  tx: TransactionSerializableSeismic
): Promise<{
  typedData: SignTypedDataParameters<typedData, primaryType, TAccount>
  signature: PrimitiveSignature
}> => {
  const typedData = seismicTxTypedData(tx)
  const encodedSignature = await signTypedData(client, typedData)
  const { r, s, yParity } = parseSignature(encodedSignature)
  return {
    // @ts-ignore
    typedData,
    signature: {
      r,
      s,
      yParity: numberToHex(yParity),
    },
  }
}
