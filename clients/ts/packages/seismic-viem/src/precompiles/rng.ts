import {
  ByteArray,
  Hex,
  bytesToHex,
  decodeAbiParameters,
  hexToBytes,
  isBytes,
  isHex,
  numberToHex,
  pad,
} from 'viem'

import {
  CallClient,
  Precompile,
  calcLinearGasCostU32,
  callPrecompile,
} from '@sviem/precompiles/precompile.ts'

export const RNG_ADDRESS = '0x0000000000000000000000000000000000000064'
// Gas schedule of the node's RNG precompile:
// https://github.com/SeismicSystems/seismic-revm/blob/seismic/crates/seismic/src/precompiles/rng/precompile.rs
const RNG_BASE_GAS = 3500n
const RNG_PERS_WORD_GAS = 5n
const RNG_ROUND_BASE_GAS = 120n
const RNG_ROUND_WORD_GAS = 24n
// Each HKDF expansion round hashes the 121-byte domain-separation prefix, the
// personalization, the previous 32-byte block and the 1-byte round counter.
const RNG_INFO_PREFIX_LEN = 121
const HKDF_ROUND_OVERHEAD_LEN = 33

const persToBytes = (pers?: Hex | ByteArray): ByteArray => {
  if (!pers) {
    return new Uint8Array()
  }
  if (isHex(pers)) {
    return hexToBytes(pers)
  } else if (isBytes(pers)) {
    return pers
  }
  throw new Error('Invalid pers: must be a hex or bytes array')
}

/**
 * Parameters required for random number generation operations.
 *
 * @type {Object}
 * @property {bigint | number} numBytes - The number of bytes to generate (must be less than or equal to 32).
 * @property {Hex | ByteArray} [pers] - Optional personalization string to seed the random number generator.
 */
export type RngParams = {
  numBytes: bigint | number
  pers?: Hex | ByteArray
}

/**
 * Precompile contract configuration for random number generation.
 *
 * @type {Precompile<RngParams, bigint>}
 * @property {Hex} address - The address of the random number generator precompile contract.
 * @property {Function} gasCost - Function that calculates the gas cost for the RNG operation.
 *   - Calculates an initialization cost based on the length of the personalization string.
 *   - Adds the cost of one HKDF expansion round per 32 bytes requested.
 * @property {Function} encodeParams - Function that encodes the input parameters for the precompile call.
 *   - Validates that numBytes is a number or bigint and is less than or equal to 32.
 *   - Encodes numBytes as a 4-byte hex value.
 *   - If personalization is provided, encodes and appends it to the parameters.
 * @property {Function} decodeResult - Function that decodes the result from the precompile call.
 *   - Pads the result to ensure proper ABI decoding.
 *   - Parses the returned hex value as a uint256.
 *   - Returns the decoded value as a bigint.
 */
export const rngPrecompile: Precompile<RngParams, bigint> = {
  address: RNG_ADDRESS,
  gasCost: ({ numBytes, pers }) => {
    const persLen = persToBytes(pers).length
    const initCost = calcLinearGasCostU32({
      len: persLen,
      base: RNG_BASE_GAS,
      word: RNG_PERS_WORD_GAS,
    })
    const roundCost = calcLinearGasCostU32({
      len: RNG_INFO_PREFIX_LEN + persLen + HKDF_ROUND_OVERHEAD_LEN,
      base: RNG_ROUND_BASE_GAS,
      word: RNG_ROUND_WORD_GAS,
    })
    const rounds = BigInt(Math.ceil(Number(numBytes) / 32))
    return initCost + rounds * roundCost
  },
  encodeParams: ({ numBytes, pers }) => {
    if (typeof numBytes !== 'bigint' && typeof numBytes !== 'number') {
      throw new Error('Invalid length: must be a number or bigint')
    }
    if (BigInt(numBytes) > 32n) {
      throw new Error('Invalid length: must be less than or equal to 32')
    }
    const encodedBytes = numberToHex(numBytes, { size: 4 })
    if (!pers) {
      return encodedBytes
    }
    const encodedPers = bytesToHex(persToBytes(pers))
    return `${encodedBytes}${encodedPers.slice(2)}`
  },
  decodeResult: (result: Hex) => {
    const [output] = decodeAbiParameters([{ type: 'uint256' }], pad(result))
    return output as bigint
  },
}

/**
 * Generates a random number using the RNG precompile.
 *
 * @param {CallClient} client - The public client to use for the precompile call.
 * @param {RngParams} args - The parameters for random number generation.
 *   - `numBytes` {bigint | number} - The number of bytes to generate (must be less than or equal to 32).
 *   - `pers` {Hex | ByteArray} [optional] - Personalization string to seed the random number generator.
 *
 * @throws {Error} Throws if:
 *   - The numBytes argument is not a number or bigint.
 *   - The numBytes argument is greater than 32.
 *   - The precompile call fails.
 *
 * @returns {Promise<bigint>} A promise that resolves to the randomly generated number as a bigint.
 *
 * @example
 * ```typescript
 * // Generate a random uint8
 * const randomUint8 = await rng(client, { numBytes: 1 });
 *
 * // Generate a random uint256
 * const randomUint256 = await rng(client, { numBytes: 32 });
 *
 * // Generate a random number with personalization
 * const randomWithPers = await rng(client, {
 *   numBytes: 16,
 *   pers: "0x1234567890abcdef"
 * });
 * ```
 */
export const rng = async (
  client: CallClient,
  args: RngParams
): Promise<bigint> => {
  return callPrecompile({
    client,
    precompile: rngPrecompile,
    args,
  })
}
