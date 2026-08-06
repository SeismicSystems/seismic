/**
 * Decryption of signed-read revert data.
 *
 * A signed read (or signed gas estimate) proves possession of the caller's
 * key, so the node encrypts the revert output of the simulated call under
 * that key — a contract's revert data can embed private state (e.g.
 * `revert InsufficientBalance(actualBalance)`) just like a successful return
 * value can. The node surfaces only a generic `execution reverted` message
 * and puts the ciphertext in the error's `data` field.
 *
 * `decryptRevertError` restores the plaintext revert data client-side using
 * the same metadata (AES key + AAD) that encrypted the request, so callers
 * see decoded revert reasons exactly as they would on a transparent chain.
 */
import type { Hex } from 'viem'
import { RawContractError, decodeErrorResult } from 'viem'

import type { TxSeismicMetadata } from '@sviem/tx/metadata.ts'
import { getRevertErrorData } from '@sviem/viem-internal/call.ts'

type DecryptClient = {
  decrypt: (
    ciphertext: Hex | undefined,
    metadata: TxSeismicMetadata
  ) => Promise<Hex>
}

/** Best-effort human-readable message for decrypted revert data. */
const revertMessage = (data: Hex): string => {
  try {
    const { errorName, args } = decodeErrorResult({ data })
    if (errorName === 'Error' && typeof args?.[0] === 'string') {
      return `execution reverted: ${args[0]}`
    }
  } catch {
    // Custom error or raw bytes: leave the generic message; the data field
    // still carries the full plaintext for ABI-aware decoding upstream.
  }
  return 'execution reverted'
}

/**
 * If `err` carries encrypted signed-read revert data, returns a
 * [`RawContractError`] with the decrypted revert data (and a decoded reason
 * message when the data is a standard `Error(string)` revert), suitable for
 * viem's regular contract-error decoding.
 *
 * Returns `err` unchanged when there is nothing to decrypt (no metadata, no
 * revert data) or when decryption fails — e.g. plaintext revert data from a
 * node without signed-read revert encryption, or error data unrelated to
 * revert output. AES-GCM authentication makes a wrong-input decrypt fail
 * loudly rather than produce garbage.
 */
export const decryptRevertError = async (
  client: DecryptClient,
  err: unknown,
  metadata: TxSeismicMetadata | undefined
): Promise<unknown> => {
  if (!metadata) return err
  const data = getRevertErrorData(err)
  if (!data || data === '0x') return err

  let decrypted: Hex
  try {
    decrypted = await client.decrypt(data, metadata)
  } catch {
    return err
  }
  const message = revertMessage(decrypted)
  const decryptedErr = new RawContractError({ data: decrypted, message })
  // viem's `getNodeError` derives the `ExecutionRevertedError` reason from
  // `details`, which `RawContractError` doesn't populate on its own.
  decryptedErr.details = message
  return decryptedErr
}
