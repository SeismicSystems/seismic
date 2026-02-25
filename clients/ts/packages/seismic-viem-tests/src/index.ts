export type { NodeConnection } from '@sviem-tests/connect.ts'
export { connectToNode } from '@sviem-tests/connect.ts'

export { testSeismicTxEncoding } from '@sviem-tests/tests/encoding.ts'
export { testSeismicTx } from '@sviem-tests/tests/contract/contract.ts'
export { testDepositContract } from '@sviem-tests/tests/contract/depositContract.ts'
export {
  testContractTreadIsntSeismicTx,
  testShieldedWalletClientTreadIsntSeismicTx,
  testViemReadContractIsntSeismicTx,
} from '@sviem-tests/tests/transparentContract/tread-contract.ts'
export {
  testContractTwriteIsntSeismicTx,
  testShieldedWalletClientTwriteIsntSeismicTx,
  testViemWriteContractIsntSeismicTx,
} from '@sviem-tests/tests/transparentContract/twrite-contract.ts'
export { testAesKeygen } from '@sviem-tests/tests/aesKeygen.ts'
export { testWsConnection } from '@sviem-tests/tests/ws.ts'
export {
  testSeismicCallTypedData,
  testSeismicTxTypedData,
} from '@sviem-tests/tests/typedData.ts'
export {
  testAesGcm,
  testEcdh,
  testHkdfHex,
  testHkdfString,
  testRng,
  testRngWithPers,
  testSecp256k1,
} from '@sviem-tests/tests/precompiles.ts'
export {
  testLegacyTxTrace,
  testSeismicTxTrace,
} from '@sviem-tests/tests/trace.ts'

export { loadDotenv } from '@sviem-tests/util.ts'

/**
 * @deprecated Use {@link connectToNode} instead.
 * The tests in this repo have moved to external management for nodes (sanvil, sreth, etc).
 * This allows for more flexible testing against different nodes (local, testnet, etc) without coupling the tests to node management.
 * This way the tests just need a URL to connect to, such that the test suite can be run against any node (local build, remote testnet, etc).
 * Management of local tests that need an anvil/sreth to be started is now managed by mise (see clients/ts.mise.toml), which uses the process manager (pitchfork) to start/stop nodes as needed.
 */
export type { RunProcessOptions } from '@sviem-tests/process/manage.ts'
export type { RethProcessOptions } from '@sviem-tests/process/chains/reth.ts'
export type {
  NodeProcess,
  NodeProcessOptions,
  SpawnedNode,
} from '@sviem-tests/process/node.ts'
export {
  runProcess,
  killProcess,
  waitForProcessExit,
} from '@sviem-tests/process/manage.ts'
export { setupRethNode } from '@sviem-tests/process/chains/reth.ts'
export { setupAnvilNode } from '@sviem-tests/process/chains/anvil.ts'
export { envChain, setupNode, buildNode } from '@sviem-tests/process/node.ts'