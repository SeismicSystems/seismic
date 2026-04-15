import { expect } from 'bun:test'
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
  getShieldedContract,
  hasShieldedParams,
} from 'seismic-viem'
import { http } from 'viem'

import { seismicCounterAbi } from '@sviem-tests/tests/contract/abi.ts'
import { seismicCounterBytecode } from '@sviem-tests/tests/contract/bytecode.ts'
import type { ContractTestArgs } from '@sviem-tests/tests/contract/contract.ts'
import { transparentCounterABI } from '@sviem-tests/tests/transparentContract/abi.ts'
import { transparentCounterBytecode } from '@sviem-tests/tests/transparentContract/bytecode.ts'

const TEST_NUMBER = BigInt(11)

const expectSeismicTx = (type: string | null) => {
  if (!type) {
    throw new Error('Transaction type not found')
  }
  // receipt.type can be hex ("0x4a", "0x4A") or a named string ("0x4A")
  // getTransaction().typeHex is always hex
  expect(type.toLowerCase()).toContain('4a')
}

const expectNonSeismicTx = (type: string | null) => {
  if (!type) {
    throw new Error('Transaction type not found')
  }
  expect(type.toLowerCase()).not.toContain('4a')
}

// ── hasShieldedParams utility ──────────────────────────────────────────

export const testHasShieldedParamsDetectsShielded = async () => {
  // setNumber has suint256 → shielded
  expect(hasShieldedParams(seismicCounterAbi, 'setNumber')).toBe(true)
}

export const testHasShieldedParamsDetectsTransparent = async () => {
  // increment has no shielded params → not shielded
  expect(hasShieldedParams(seismicCounterAbi, 'increment')).toBe(false)
}

export const testHasShieldedParamsViewFunctions = async () => {
  // isOdd() and getNumber() have no inputs at all → not shielded
  expect(hasShieldedParams(seismicCounterAbi, 'isOdd')).toBe(false)
  expect(hasShieldedParams(seismicCounterAbi, 'getNumber')).toBe(false)
}

export const testHasShieldedParamsTransparentContract = async () => {
  // All functions in transparentCounterABI should return false
  expect(hasShieldedParams(transparentCounterABI, 'setNumber')).toBe(false)
  expect(hasShieldedParams(transparentCounterABI, 'increment')).toBe(false)
  expect(hasShieldedParams(transparentCounterABI, 'isOdd')).toBe(false)
  expect(hasShieldedParams(transparentCounterABI, 'number')).toBe(false)
}

// ── Smart contract.write routing ───────────────────────────────────────

/**
 * contract.write on a function with suint256 param should route to shielded write
 * (seismic tx type 0x4a)
 */
export const testSmartWriteShieldedParam = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  // setNumber(suint256) → smart write should detect shielded → seismic tx
  const hash = await contract.write.setNumber([TEST_NUMBER])
  const receipt = await publicClient.waitForTransactionReceipt({ hash })
  expectSeismicTx(receipt.type)
}

/**
 * contract.write on a function with no shielded params should route to transparent write
 * (non-seismic tx type)
 */
export const testSmartWriteTransparentParam = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  // increment() has no shielded params → smart write should use transparent tx
  const hash = await contract.write.increment()
  const { typeHex } = await publicClient.getTransaction({ hash })
  expectNonSeismicTx(typeHex)
}

// ── Smart contract.read routing ────────────────────────────────────────

/**
 * contract.read on a view function with no shielded inputs should route to
 * a transparent read (works even without a wallet account in theory, but
 * here we just verify it returns correct data via transparent path)
 */
export const testSmartReadTransparentParam = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  // isOdd() has no shielded inputs → smart read should use transparent read
  // Contract initializes number to 0, so isOdd should be false
  const isOdd = await contract.read.isOdd()
  expect(isOdd).toBe(false)
}

/**
 * contract.read on a view function that exists on a contract where setNumber
 * has been called, verifying smart read returns the right value after a
 * shielded write + transparent read cycle
 */
export const testSmartReadAfterSmartWrite = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  // Set number to 11 via smart write (shielded, since suint256)
  const setHash = await contract.write.setNumber([TEST_NUMBER])
  await publicClient.waitForTransactionReceipt({ hash: setHash })

  // Smart read isOdd() → transparent read → 11 is odd
  const isOdd = await contract.read.isOdd()
  expect(isOdd).toBe(true)

  // Increment via smart write → transparent (no shielded params) → 12
  const incHash = await contract.write.increment()
  await publicClient.waitForTransactionReceipt({ hash: incHash })

  // Smart read isOdd() → transparent read → 12 is not odd
  const isOdd2 = await contract.read.isOdd()
  expect(isOdd2).toBe(false)
}

// ── Force shielded: contract.swrite / contract.sread ───────────────────

/**
 * contract.swrite always uses seismic tx, even for a function with
 * no shielded params (increment)
 */
export const testSwriteAlwaysShielded = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  // swrite.increment() → should always be seismic tx even though no shielded params
  const hash = await contract.swrite.increment()
  const receipt = await publicClient.waitForTransactionReceipt({ hash })
  expectSeismicTx(receipt.type)
}

/**
 * contract.sread always uses signed read, even for a function with
 * no shielded params (isOdd). Verify it returns correct data.
 */
export const testSreadAlwaysShielded = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  // sread.isOdd() → force signed read even though no shielded params
  // Contract initializes to 0 → not odd
  const isOdd = await contract.sread.isOdd()
  expect(isOdd).toBe(false)
}

// ── Smart wallet client actions ────────────────────────────────────────

/**
 * walletClient.writeContract auto-detects shielded params:
 * - setNumber(suint256) → seismic tx
 */
export const testSmartWalletWriteShielded = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const hash = await walletClient.writeContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'setNumber',
    args: [TEST_NUMBER],
  })
  const receipt = await publicClient.waitForTransactionReceipt({ hash })
  expectSeismicTx(receipt.type)
}

/**
 * walletClient.writeContract auto-detects transparent params:
 * - increment() → non-seismic tx
 */
export const testSmartWalletWriteTransparent = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const hash = await walletClient.writeContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'increment',
  })
  const { typeHex } = await publicClient.getTransaction({ hash })
  expectNonSeismicTx(typeHex)
}

/**
 * walletClient.readContract auto-detects transparent params:
 * - isOdd() → transparent read
 */
export const testSmartWalletReadTransparent = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  // isOdd() has no shielded inputs → transparent read
  const isOdd = await walletClient.readContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  expect(isOdd).toBe(false)
}

// ── Force shielded wallet client actions ───────────────────────────────

/**
 * walletClient.swriteContract always uses seismic tx, even for transparent functions
 */
export const testSwriteContractAlwaysShielded = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  // swriteContract for increment() → should always be seismic tx
  const hash = await walletClient.swriteContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'increment',
  })
  const receipt = await publicClient.waitForTransactionReceipt({ hash })
  expectSeismicTx(receipt.type)
}

/**
 * walletClient.sreadContract always uses signed read, even for transparent functions
 */
export const testSreadContractAlwaysShielded = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  // sreadContract for isOdd() → should always use signed read
  const isOdd = await walletClient.sreadContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  expect(isOdd).toBe(false)
}

// ── twrite/tread on shielded-typed functions ───────────────────────────

/**
 * contract.twrite on a function with suint256 param should succeed —
 * builds calldata with correct selector from original ABI (suint256),
 * encodes params with remapped types, and sends as standard tx
 */
export const testTwriteRemapsShieldedTypes = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  // twrite.setNumber(suint256) → correct selector + remapped encoding, non-seismic tx
  const hash = await contract.twrite.setNumber([TEST_NUMBER])
  const receipt = await publicClient.waitForTransactionReceipt({ hash })
  expectNonSeismicTx(receipt.type)

  // Verify the value was set correctly
  const isOdd = await contract.tread.isOdd()
  expect(isOdd).toBe(true)
}

/**
 * walletClient.twriteContract on a function with suint256 param should succeed
 */
export const testTwriteContractRemapsShieldedTypes = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  // twriteContract with suint256 → correct selector, non-seismic tx
  const hash = await walletClient.twriteContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'setNumber',
    args: [TEST_NUMBER],
  })
  const { typeHex } = await publicClient.getTransaction({ hash })
  expectNonSeismicTx(typeHex)
}

// ── Smart routing with fully transparent contract ──────────────────────

/**
 * contract.write on a fully transparent contract (no shielded types at all)
 * should always use transparent tx
 */
export const testSmartWriteTransparentContract = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${transparentCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: transparentCounterABI,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: transparentCounterABI,
    address,
    client: walletClient,
  })

  // setNumber(uint256) on transparent contract → transparent tx
  const hash = await contract.write.setNumber([TEST_NUMBER])
  const { typeHex } = await publicClient.getTransaction({ hash })
  expectNonSeismicTx(typeHex)
}

/**
 * contract.read on a fully transparent contract should use transparent read
 */
export const testSmartReadTransparentContract = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${transparentCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: transparentCounterABI,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: transparentCounterABI,
    address,
    client: walletClient,
  })

  // isOdd() on transparent contract → transparent read
  const isOdd = await contract.read.isOdd()
  expect(isOdd).toBe(false)
}

// ── End-to-end smart routing lifecycle ─────────────────────────────────

/**
 * Full lifecycle test: deploy seismic counter, use smart routing for
 * all operations, verify correct tx types at each step, and verify
 * state is consistent throughout
 */
export const testSmartRoutingLifecycle = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  const contract = getShieldedContract({
    abi: seismicCounterAbi,
    address,
    client: walletClient,
  })

  // Step 1: Initial read — isOdd() → smart read (transparent) → false (number=0)
  const isOdd0 = await contract.read.isOdd()
  expect(isOdd0).toBe(false)

  // Step 2: Smart write setNumber(11) → shielded (suint256)
  const setHash = await contract.write.setNumber([BigInt(11)])
  const setReceipt = await publicClient.waitForTransactionReceipt({
    hash: setHash,
  })
  expectSeismicTx(setReceipt.type)

  // Step 3: Smart read isOdd() → transparent → true (11 is odd)
  const isOdd1 = await contract.read.isOdd()
  expect(isOdd1).toBe(true)

  // Step 4: Smart write increment() → transparent (no shielded params)
  const incHash = await contract.write.increment()
  const incReceipt = await publicClient.waitForTransactionReceipt({
    hash: incHash,
  })
  expectNonSeismicTx(incReceipt.type)

  // Step 5: Smart read isOdd() → transparent → false (12 is not odd)
  const isOdd2 = await contract.read.isOdd()
  expect(isOdd2).toBe(false)

  // Step 6: Force shielded read via sread — same result
  const isOdd2_s = await contract.sread.isOdd()
  expect(isOdd2_s).toBe(false)

  // Step 7: Force transparent read via tread — same result
  const isOdd2_t = await contract.tread.isOdd()
  expect(isOdd2_t).toBe(false)

  // Step 8: Force shielded write increment() → seismic tx (even though no shielded params)
  const swriteHash = await contract.swrite.increment()
  const swriteReceipt = await publicClient.waitForTransactionReceipt({
    hash: swriteHash,
  })
  expectSeismicTx(swriteReceipt.type)

  // Step 9: Force transparent write setNumber(suint256) → non-seismic tx
  // (twrite remaps suint256 → uint256 so viem can encode it, then sends unencrypted)
  const twriteHash = await contract.twrite.setNumber([BigInt(7)])
  const twriteReceipt = await publicClient.waitForTransactionReceipt({
    hash: twriteHash,
  })
  expectNonSeismicTx(twriteReceipt.type)

  // Step 10: Final smart read → 7 is odd
  const isOdd3 = await contract.read.isOdd()
  expect(isOdd3).toBe(true)
}

/**
 * Full lifecycle test using wallet client actions instead of contract proxy
 */
export const testSmartWalletActionsLifecycle = async ({
  chain,
  url,
  account,
}: ContractTestArgs) => {
  const publicClient = createShieldedPublicClient({
    chain,
    transport: http(url),
  })
  const walletClient = await createShieldedWalletClient({
    chain,
    transport: http(url),
    account,
  })
  const bytecode: `0x${string}` = `0x${seismicCounterBytecode.object.replace(/^0x/, '')}`

  const deployTx = await walletClient.deployContract({
    abi: seismicCounterAbi,
    bytecode,
    chain: walletClient.chain,
  })
  const deployReceipt = await publicClient.waitForTransactionReceipt({
    hash: deployTx,
  })
  const address = deployReceipt.contractAddress!

  // Step 1: Smart read isOdd() → transparent → false (number=0)
  const isOdd0 = await walletClient.readContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  expect(isOdd0).toBe(false)

  // Step 2: Smart write setNumber(suint256) → shielded
  const setHash = await walletClient.writeContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'setNumber',
    args: [BigInt(11)],
  })
  const setReceipt = await publicClient.waitForTransactionReceipt({
    hash: setHash,
  })
  expectSeismicTx(setReceipt.type)

  // Step 3: Smart write increment() → transparent
  const incHash = await walletClient.writeContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'increment',
  })
  const incReceipt = await publicClient.waitForTransactionReceipt({
    hash: incHash,
  })
  expectNonSeismicTx(incReceipt.type)

  // Step 4: Smart read → false (12 is not odd)
  const isOdd1 = await walletClient.readContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  expect(isOdd1).toBe(false)

  // Step 5: Force shielded write increment() → seismic tx
  const swriteHash = await walletClient.swriteContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'increment',
  })
  const swriteReceipt = await publicClient.waitForTransactionReceipt({
    hash: swriteHash,
  })
  expectSeismicTx(swriteReceipt.type)

  // Step 6: Force shielded read → true (13 is odd)
  const isOdd2 = await walletClient.sreadContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  expect(isOdd2).toBe(true)

  // Step 7: Force transparent read → same result
  const isOdd2_t = await walletClient.treadContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'isOdd',
  })
  expect(isOdd2_t).toBe(true)

  // Step 8: Force transparent write → non-seismic tx
  const twriteHash = await walletClient.twriteContract({
    address,
    abi: seismicCounterAbi,
    functionName: 'increment',
  })
  const twriteReceipt = await publicClient.waitForTransactionReceipt({
    hash: twriteHash,
  })
  expectNonSeismicTx(twriteReceipt.type)
}
