import {
  type Abi,
  type AbiFunction,
  type AbiParameter,
  fromHex,
  getAbiItem,
  isHex,
  sha256,
  toHex,
} from 'viem'

const isAnyTrue = (bools: boolean[]): boolean =>
  bools.reduce((acc, val) => acc || val, false)

const remapSeismicParam = (
  param: AbiParameter
): { shielded: boolean } & AbiParameter => {
  const ty = param.type
  if (ty.startsWith('tuple')) {
    const { components, ...rest } = param as unknown as AbiParameter & {
      components: AbiParameter[]
    }
    // const components = (param as unknown as { components: AbiParameter[]}).components
    const mappedComponents = components.map(remapSeismicParam)
    const shielded = isAnyTrue(mappedComponents.map(({ shielded }) => shielded))
    return {
      shielded,
      components: mappedComponents,
      ...rest,
    }
  }

  // sint/suint arrays
  if (ty.match(/suint\d+\[\d+\]/) || ty.match(/sint\d+\[\d+\]/)) {
    const baseType = ty.startsWith('suint') ? 'uint' : 'int'
    const bits = ty.match(/\d+/)![0]
    const arraySize = ty.match(/\[\d+\]/)![0]
    return { shielded: true, type: `${baseType}${bits}${arraySize}` }
  }

  // sint/suint single values
  if (ty.startsWith('suint') || ty.startsWith('sint')) {
    const bits = ty.slice(ty.startsWith('suint') ? 5 : 4)
    return {
      shielded: true,
      type: `${ty.startsWith('suint') ? 'uint' : 'int'}${bits}`,
    }
  }

  if (ty === 'sbool') {
    return { shielded: true, type: 'bool' }
  }
  if (ty === 'sbool[]') {
    return { shielded: true, type: 'bool[]' }
  }
  if (ty === 'saddress') {
    return { shielded: true, type: 'address' }
  }
  if (ty === 'saddress[]') {
    return { shielded: true, type: 'address[]' }
  }

  // sbytes (e.g. sbytes8, sbytes32, sbytes)
  if (ty.startsWith('sbytes')) {
    return { shielded: true, type: ty.slice(1) }
  }

  return { shielded: false, type: ty }
}

/**
 * Remaps an entire ABI, converting all shielded types (suint256, saddress, etc.)
 * to their standard equivalents (uint256, address, etc.) so viem can encode them.
 */
export const remapSeismicAbi = (abi: Abi): Abi => {
  return abi.map((item) => {
    if (item.type !== 'function') return item
    return remapSeismicAbiInputs(item)
  }) as Abi
}

/**
 * Resolves an ABI function item from a Seismic contract ABI, supporting
 * function overloads.
 *
 * When `name` is a selector (hex string), it resolves directly by selector.
 * When `args` is provided, shielded types (e.g. `suint256`, `saddress`) are
 * remapped to standard EVM types so viem's overload resolution can correctly
 * match against JavaScript/TypeScript runtime argument types, and the
 * matching original Seismic ABI item is returned.
 *
 * If `args` is omitted or no overload matches, falls back to standard name matching.
 */
export function getSeismicAbiItem(
  abi: Abi,
  name: string,
  args?: readonly unknown[]
): AbiFunction | undefined {
  if (isHex(name, { strict: false })) {
    return getAbiItem({ abi, name }) as AbiFunction | undefined
  }

  if (args !== undefined) {
    const remapped = remapSeismicAbi(abi)
    const matchedRemapped = getAbiItem({
      abi: remapped,
      name,
      args,
    })

    if (matchedRemapped) {
      const matchingIndex = remapped.findIndex(
        (item) => item === matchedRemapped
      )
      if (matchingIndex !== -1) {
        return abi[matchingIndex] as AbiFunction
      }
    }
  }

  return getAbiItem({ abi, name }) as AbiFunction | undefined
}

export function hasShieldedParams(
  abi: Abi,
  functionName: string,
  args?: readonly unknown[]
): boolean {
  const abiItem = getSeismicAbiItem(abi, functionName, args)
  if (!abiItem?.inputs) return false
  return abiItem.inputs.some((param) => remapSeismicParam(param).shielded)
}

export const remapSeismicAbiInputs = (
  abiFunction: AbiFunction
): AbiFunction => {
  return {
    type: abiFunction.type,
    inputs: abiFunction.inputs.map((param) => {
      return remapSeismicParam(param)
    }),
    name: abiFunction.name,
    outputs: abiFunction.outputs,
    stateMutability: abiFunction.stateMutability,
  }
}

/*
 * Utility function to process arguments including fetching the ABI item
 */
export function getProcessedArgs<
  abiItem extends AbiFunction,
  args extends readonly unknown[] | undefined,
>(abiItem: abiItem, args: args): any[] {
  // Check if ABI item has inputs and arguments are provided
  if (abiItem.inputs.length === 0 || !args || args.length === 0) {
    return []
  }

  return abiItem.inputs.map((input, index) => {
    if (input.internalType === 'Suint') {
      const value = args[index] as bigint
      const hash = fromHex(sha256(toHex(value)), { to: 'bigint' })
      return hash
    }
    return args[index]
  })
}
