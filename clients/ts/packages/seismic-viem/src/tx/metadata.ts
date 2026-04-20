/**
 * Seismic tx metadata: the inputs needed to construct, encrypt, and sign a
 * Seismic transaction beyond what a caller normally provides.
 *
 * A plain viem `sendTransaction` caller supplies `to`, `value`, `data`, and
 * optionally `nonce` / gas fields. A Seismic tx additionally requires:
 *
 *  - the client's encryption pubkey + a fresh encryption nonce (for AEAD)
 *  - a `recentBlockHash` + `expiresAtBlock` window (to bound replay)
 *  - a `messageVersion` selecting the signing path
 *  - a `signedRead` flag (true for signed eth_call, false for txs)
 *  - the resolved tx nonce and chainId (to bind the AAD)
 *
 * `TxSeismicMetadata` bundles all of the above so that tx-build paths
 * (`sendShielded`, `signedCall`, smart contract writes) can share one
 * resolution routine and one AAD encoding (see `crypto/aead.ts`).
 */
import {
  Account,
  Address,
  Chain,
  GetBlockParameters,
  GetTransactionCountParameters,
  Transport,
} from 'viem'
import { parseAccount } from 'viem/accounts'

import { ShieldedWalletClient } from '@sviem/client.ts'
import { randomEncryptionNonce } from '@sviem/crypto/nonce.ts'
import type {
  SeismicBlockParams,
  SeismicElements,
  SeismicSecurityParams,
} from '@sviem/tx/seismicTx.ts'
import { TYPED_DATA_MESSAGE_VERSION } from '@sviem/tx/signSeismicTypedData.ts'

const DEFAULT_SEISMIC_BLOCKS_WINDOW = 100n

/**
 * Ethereum-compatible fields pulled out of the metadata so they can be bound
 * into the AEAD Additional Authenticated Data. Keeping them in their own
 * sub-object makes the AAD encoding explicit and symmetric with the Rust
 * reference implementation in the node.
 */
export type LegacyFields = {
  chainId: number
  nonce: number
  to: Address | null | undefined
  value: bigint
}

/**
 * Resolved metadata for a single Seismic transaction or signed read.
 *
 * Consumers:
 *  - `sendShielded` / `signedCall` — pass this straight into the encryption
 *    step and the tx builder
 *  - `encryptionActions.encrypt/decrypt` — uses the metadata to derive the AAD
 *    bound to the ciphertext
 *  - `signSeismicTxTypedData` — reads `seismicElements` to populate the
 *    EIP-712 message
 *
 * Build instances with `buildTxSeismicMetadata(...)`.
 */
export type TxSeismicMetadata = {
  sender: Address
  legacyFields: LegacyFields
  seismicElements: SeismicElements
}

const fillNonce = async <
  TChain extends Chain | undefined,
  TAccount extends Account,
>(
  client: ShieldedWalletClient<Transport, TChain, TAccount>,
  parameters: {
    account: Account | null
    nonce: number | undefined
  } & GetBlockParameters
) => {
  let account = parseAccount(parameters.account || client.account)
  const { nonce: nonce_ } = parameters
  if (nonce_ !== undefined) {
    return nonce_
  }

  const { blockNumber, blockTag = 'pending' } = parameters
  let args: GetTransactionCountParameters = {
    address: account.address,
    blockTag,
  }
  if (blockNumber) {
    args = { address: account.address, blockNumber }
  }
  return client.getTransactionCount(args)
}

type BuildTxSeismicMetadataParams = {
  account: Address | Account | null | undefined
  nonce?: number
  to: Address
  value?: bigint
  typedDataTx?: boolean
  signedRead?: boolean
} & SeismicSecurityParams

const inferTypedDataTx = (
  typedDataTx: boolean | undefined,
  account: Account
): boolean => {
  if (typedDataTx !== undefined) {
    return typedDataTx
  }
  return account.type === 'json-rpc' || account.type === 'local'
}

/**
 * Resolves a `TxSeismicMetadata` for a tx or signed read.
 *
 * Fills the Seismic-specific fields that a caller rarely wants to compute by
 * hand:
 *
 *  - `nonce` — fetched via `getTransactionCount({ blockTag: 'pending' })` if
 *    not supplied
 *  - `encryptionNonce` — randomly generated if not supplied
 *  - `recentBlockHash` / `expiresAtBlock` — defaults to the latest block and
 *    `latest.number + blocksWindow` (default window: 100 blocks)
 *  - `messageVersion` — `TYPED_DATA_MESSAGE_VERSION` for json-rpc/local
 *    accounts (so wallets like MetaMask can sign via EIP-712), `0` otherwise
 *
 * Any of the above can be forced via `SeismicSecurityParams` — mainly used
 * by tests that need deterministic metadata or callers that want explicit
 * expiry control.
 *
 * Throws if `expiresAtBlock` is in the past or if `blocksWindow <= 0`.
 */
export const buildTxSeismicMetadata = async <
  TChain extends Chain | undefined,
  TAccount extends Account,
>(
  client: ShieldedWalletClient<Transport, TChain, TAccount>,
  {
    account: paramsAcct,
    nonce,
    to,
    value = 0n,
    encryptionNonce,
    blocksWindow = DEFAULT_SEISMIC_BLOCKS_WINDOW,
    recentBlockHash,
    expiresAtBlock,
    typedDataTx,
    signedRead = false,
  }: BuildTxSeismicMetadataParams
): Promise<TxSeismicMetadata> => {
  const account = parseAccount(paramsAcct || client.account)
  if (!account) {
    throw new Error(`Signed reads must have an account`)
  }
  if (blocksWindow <= 0n) {
    throw new Error(`blocksWindow param must be > 0`)
  }

  const resolveBlockParams = async (): Promise<SeismicBlockParams> => {
    if (recentBlockHash && expiresAtBlock) {
      return { recentBlockHash, expiresAtBlock }
    }
    if (recentBlockHash) {
      const recentBlock = await client.getBlock({ blockHash: recentBlockHash })
      return {
        recentBlockHash: recentBlock.hash,
        expiresAtBlock: recentBlock.number + blocksWindow,
      }
    }
    const latestBlock = await client.getBlock({ blockTag: 'latest' })
    if (expiresAtBlock) {
      if (expiresAtBlock <= latestBlock.number) {
        throw new Error(
          `expiresAtBlock param ${expiresAtBlock} is in the past (latest block is #${latestBlock.number})`
        )
      }
      return { recentBlockHash: latestBlock.hash, expiresAtBlock }
    }
    return {
      recentBlockHash: latestBlock.hash,
      expiresAtBlock: latestBlock.number + blocksWindow,
    }
  }

  const useTypedDataTx = inferTypedDataTx(typedDataTx, account)
  const [nonce_, chainId, blockParams] = await Promise.all([
    fillNonce(client, { account, nonce }),
    client.getChainId(),
    resolveBlockParams(),
  ])

  if (client.chain) {
    if (client.chain.id !== chainId) {
      throw new Error(`Client chain's id does not match eth_chainId response`)
    }
  }

  return {
    sender: account.address,
    legacyFields: {
      chainId,
      nonce: nonce_,
      to,
      value,
    },
    seismicElements: {
      encryptionPubkey: client.getEncryptionPublicKey(),
      encryptionNonce: encryptionNonce ?? randomEncryptionNonce(),
      messageVersion: useTypedDataTx ? TYPED_DATA_MESSAGE_VERSION : 0,
      recentBlockHash: blockParams.recentBlockHash,
      expiresAtBlock: blockParams.expiresAtBlock,
      signedRead,
    },
  }
}
