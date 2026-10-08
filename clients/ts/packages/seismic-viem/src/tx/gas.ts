import type { Hex } from 'viem'
import { hexToBytes } from 'viem'

/**
 * Minimum gas limit accepted by the current Seismic pool for encrypted input.
 * Mirrors the Prague rules: intrinsic gas (including CREATE/initcode and
 * EIP-7702 charges) and the EIP-7623 calldata floor. Seismic ignores access lists.
 * The Prague bound is conservative on older nodes. This is not an execution
 * estimate and must be calculated from the final write ciphertext, not its
 * separately encrypted signed-read estimation twin.
 */
export function seismicPoolGasMinimum(
  encryptedData: Hex,
  isCreate = false,
  authorizationCount = 0
): bigint {
  const bytes = hexToBytes(encryptedData)
  let tokens = 0n
  for (const byte of bytes) tokens += byte === 0 ? 1n : 4n

  const intrinsic =
    (isCreate ? 53_000n + 2n * ((BigInt(bytes.length) + 31n) / 32n) : 21_000n) +
    4n * tokens +
    25_000n * BigInt(authorizationCount)
  const floor = 21_000n + 10n * tokens
  return intrinsic > floor ? intrinsic : floor
}
