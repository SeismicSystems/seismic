import { beforeAll, describe, test } from 'bun:test'
import { Chain } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

import { connectToNode } from '@sviem-tests/connect.ts'
import {
  ENCRYPTION_PK,
  ENCRYPTION_SK,
  TEST_ACCOUNT_PRIVATE_KEY,
} from '@sviem-tests/constants.ts'
import { testAesKeygen } from '@sviem-tests/tests/aesKeygen.ts'
import {
  testConcurrentReads,
  testConcurrentShieldedTransactions,
} from '@sviem-tests/tests/concurrency.ts'
import { testSeismicTx } from '@sviem-tests/tests/contract/contract.ts'
import { testDepositContract } from '@sviem-tests/tests/contract/depositContract.ts'
import { testSeismicTxEncoding } from '@sviem-tests/tests/encoding.ts'
import {
  testGetStorageAtThrows,
  testSignedCallWithoutToThrows,
} from '@sviem-tests/tests/errorPaths.ts'
import {
  testLifecycleWithEstimatedGas,
  testWriteUsesEstimatedGasNot30M,
  testWriteWithExplicitGasSkipsEstimation,
  testWriteWithoutExplicitGasSucceeds,
} from '@sviem-tests/tests/estimateGas.ts'
import {
  testAesGcm,
  testEcdh,
  testHkdfHex,
  testHkdfString,
  testRng,
  testRngWithPers,
  testSecp256k1,
} from '@sviem-tests/tests/precompiles.ts'
import { testSeismicTxCalldataIsEncrypted } from '@sviem-tests/tests/privacy.ts'
import { testRngDifferentPersProducesDifferentResults } from '@sviem-tests/tests/rngUniqueness.ts'
import { testDwriteContractUsesSecurityParams } from '@sviem-tests/tests/securityParams.ts'
import {
  testSignedCallDirect,
  testSignedCallWithSecurityParams,
} from '@sviem-tests/tests/signedCallDirect.ts'
import {
  testHasShieldedParamsDetectsShielded,
  testHasShieldedParamsDetectsTransparent,
  testHasShieldedParamsTransparentContract,
  testHasShieldedParamsViewFunctions,
  testSmartReadAfterSmartWrite,
  testSmartReadTransparentContract,
  testSmartReadTransparentParam,
  testSmartRoutingLifecycle,
  testSmartWalletActionsLifecycle,
  testSmartWalletReadTransparent,
  testSmartWalletWriteShielded,
  testSmartWalletWriteTransparent,
  testSmartWriteShieldedParam,
  testSmartWriteTransparentContract,
  testSmartWriteTransparentParam,
  testSreadAlwaysShielded,
  testSreadContractAlwaysShielded,
  testSwriteAlwaysShielded,
  testSwriteContractAlwaysShielded,
  testTwriteContractRemapsShieldedTypes,
  testTwriteRemapsShieldedTypes,
} from '@sviem-tests/tests/smartRouting/smartRouting.ts'
import {
  testLegacyTxTrace,
  testSeismicTxTrace,
} from '@sviem-tests/tests/trace.ts'
import {
  testContractTreadIsntSeismicTx,
  testContractTreadRejectsAccountOption,
  testContractTreadWithPublicOnlyClient,
  testShieldedWalletClientTreadIsntSeismicTx,
  testShieldedWalletClientTreadRejectsAccountOption,
  testViemReadContractIsntSeismicTx,
} from '@sviem-tests/tests/transparentContract/tread-contract.ts'
import {
  testContractTwriteIsntSeismicTx,
  testShieldedWalletClientTwriteIsntSeismicTx,
  testViemWriteContractIsntSeismicTx,
} from '@sviem-tests/tests/transparentContract/twrite-contract.ts'
import {
  testSeismicCallTypedData,
  testSeismicTxTypedData,
} from '@sviem-tests/tests/typedData.ts'
import { testWsConnection } from '@sviem-tests/tests/ws.ts'
import { sanvil } from '@sviem/chain.ts'

const TIMEOUT_MS = 60_000
const CONTRACT_TIMEOUT_MS = 120_000

const account = privateKeyToAccount(TEST_ACCOUNT_PRIVATE_KEY)

let chain: Chain
let url: string
let wsUrl: string

beforeAll(async () => {
  try {
    const conn = await connectToNode()
    chain = conn.chain
    url = conn.url
    wsUrl = conn.wsUrl
  } catch (e) {
    console.error('Setup failed:', e)
    process.exit(1)
  }
})

describe('Seismic Contract', async () => {
  test(
    'deploy & call contracts with seismic tx via private key account',
    async () => await testSeismicTx({ chain, url, account }),
    {
      timeout: CONTRACT_TIMEOUT_MS,
    }
  )

  test(
    'deploy & call contracts with seismic tx via JSON RPC',
    async () => {
      if (chain.id !== sanvil.id) {
        // only run this against anvil
        return
      }
      const jsonRpcAccount = {
        type: 'json-rpc',
        address: account.address,
      }
      // @ts-ignore
      await testSeismicTx({ chain, url, account: jsonRpcAccount })
    },
    {
      timeout: CONTRACT_TIMEOUT_MS,
    }
  )
})

describe('Security params', async () => {
  test(
    'dwriteContract forwards explicit Seismic metadata overrides',
    async () =>
      await testDwriteContractUsesSecurityParams({ chain, url, account }),
    {
      timeout: CONTRACT_TIMEOUT_MS,
    }
  )
})

describe('Signed estimate gas', async () => {
  test(
    'write without explicit gas auto-estimates and succeeds',
    async () =>
      await testWriteWithoutExplicitGasSucceeds({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )

  test(
    'write uses estimated gas, not 30M default',
    async () => await testWriteUsesEstimatedGasNot30M({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )

  test(
    'explicit gas skips estimation',
    async () =>
      await testWriteWithExplicitGasSkipsEstimation({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )

  test(
    'full lifecycle with auto-estimated gas',
    async () => await testLifecycleWithEstimatedGas({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('Deposit Contract', async () => {
  test(
    'deploy & test deposit contract functionality',
    async () => await testDepositContract({ chain, url, account }),
    {
      timeout: CONTRACT_TIMEOUT_MS,
    }
  )
})

describe('twrite should not use seismic tx', async () => {
  test(
    'ShieldedContract.twrite should not use seismic tx',
    async () => await testContractTwriteIsntSeismicTx({ chain, url, account }),
    {
      timeout: TIMEOUT_MS,
    }
  )

  test(
    'viem writeContract should not use seismic tx',
    async () =>
      await testViemWriteContractIsntSeismicTx({ chain, url, account }),
    {
      timeout: TIMEOUT_MS,
    }
  )

  test(
    'ShieldedWalletClient.twriteContract does not use seismic tx',
    async () =>
      await testShieldedWalletClientTwriteIsntSeismicTx({
        chain,
        url,
        account,
      }),
    {
      timeout: TIMEOUT_MS,
    }
  )
})

describe('tread should not use seismic tx', async () => {
  test(
    'ShieldedContract.tread should not use seismic tx',
    async () => await testContractTreadIsntSeismicTx({ chain, url, account }),
    {
      timeout: TIMEOUT_MS,
    }
  )

  test(
    'viem readContract should not use seismic tx',
    async () =>
      await testViemReadContractIsntSeismicTx({ chain, url, account }),
    {
      timeout: TIMEOUT_MS,
    }
  )

  test(
    'ShieldedContract.tread rejects account because it is always transparent',
    async () =>
      await testContractTreadRejectsAccountOption({ chain, url, account }),
    {
      timeout: TIMEOUT_MS,
    }
  )

  test(
    'ShieldedContract.tread works with a public-only keyed client',
    async () =>
      await testContractTreadWithPublicOnlyClient({ chain, url, account }),
    {
      timeout: TIMEOUT_MS,
    }
  )

  test(
    'ShieldedWalletClient.treadContract rejects account because it is always transparent',
    async () =>
      await testShieldedWalletClientTreadRejectsAccountOption({
        chain,
        url,
        account,
      }),
    {
      timeout: TIMEOUT_MS,
    }
  )

  test(
    'ShieldedWalletClient.treadContract does not use seismic tx',
    async () =>
      await testShieldedWalletClientTreadIsntSeismicTx({
        chain,
        url,
        account,
      }),
    {
      timeout: TIMEOUT_MS,
    }
  )
})

describe('Seismic Transaction Encoding', async () => {
  test(
    'node detects and parses seismic transaction',
    async () =>
      await testSeismicTxEncoding({
        chain,
        account,
        url,
        encryptionSk: ENCRYPTION_SK,
        encryptionPubkey: ENCRYPTION_PK,
      }),
    {
      timeout: TIMEOUT_MS,
    }
  )
})

describe('AES', async () => {
  test('generates AES key correctly', testAesKeygen)
})

describe('Websocket Connection', () => {
  test(
    'should connect to the ws',
    async () => {
      await testWsConnection({
        chain,
        wsUrl,
      })
    },
    { timeout: TIMEOUT_MS }
  )
})

describe('Seismic Precompiles', () => {
  test(
    'RNG(1)',
    async () => {
      await testRng({ chain, url }, 1)
    },
    { timeout: TIMEOUT_MS }
  )
  test(
    'RNG(8)',
    async () => {
      await testRng({ chain, url }, 8)
    },
    { timeout: TIMEOUT_MS }
  )
  test(
    'RNG(32)',
    async () => {
      await testRng({ chain, url }, 32)
    },
    { timeout: TIMEOUT_MS }
  )
  test(
    'RNG(32, pers)',
    async () => {
      await testRngWithPers({ chain, url }, 32)
    },
    { timeout: TIMEOUT_MS }
  )
  test(
    'ECDH',
    async () => {
      await testEcdh({ chain, url })
    },
    { timeout: TIMEOUT_MS }
  )
  test(
    'HKDF(string)',
    async () => {
      await testHkdfString({ chain, url })
    },
    { timeout: TIMEOUT_MS }
  )
  test(
    'HKDF(hex)',
    async () => {
      await testHkdfHex({ chain, url })
    },
    { timeout: TIMEOUT_MS }
  )
  test('AES-GCM', async () => testAesGcm({ chain, url }), {
    timeout: TIMEOUT_MS,
  })
  test('secp256k1', async () => testSecp256k1({ chain, url }), {
    timeout: TIMEOUT_MS,
  })
})

describe('Transaction Trace', async () => {
  test(
    'Seismic Tx removes input from trace',
    async () => {
      await testSeismicTxTrace({ chain, url, account })
    },
    {
      timeout: TIMEOUT_MS,
    }
  )
  test(
    'Legacy Tx keeps input in trace',
    async () => {
      await testLegacyTxTrace({ chain, url, account })
    },
    { timeout: TIMEOUT_MS }
  )
})

describe('hasShieldedParams utility', () => {
  test(
    'detects shielded params (suint256)',
    testHasShieldedParamsDetectsShielded
  )
  test(
    'detects transparent params (no shielded types)',
    testHasShieldedParamsDetectsTransparent
  )
  test(
    'view functions with no inputs are not shielded',
    testHasShieldedParamsViewFunctions
  )
  test(
    'transparent contract has no shielded functions',
    testHasShieldedParamsTransparentContract
  )
})

describe('Smart routing: contract.write', () => {
  test(
    'contract.write routes to shielded tx for suint256 param',
    async () => await testSmartWriteShieldedParam({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'contract.write routes to transparent tx for no shielded params',
    async () => await testSmartWriteTransparentParam({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'contract.write on transparent contract uses transparent tx',
    async () =>
      await testSmartWriteTransparentContract({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('Smart routing: contract.read', () => {
  test(
    'contract.read routes to transparent read for no shielded inputs',
    async () => await testSmartReadTransparentParam({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'contract.read returns correct data after smart write cycle',
    async () => await testSmartReadAfterSmartWrite({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'contract.read on transparent contract uses transparent read',
    async () => await testSmartReadTransparentContract({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('twrite/tread remap shielded types', () => {
  test(
    'contract.twrite remaps suint256 and sends non-seismic tx',
    async () => await testTwriteRemapsShieldedTypes({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'walletClient.twriteContract remaps suint256 and sends non-seismic tx',
    async () =>
      await testTwriteContractRemapsShieldedTypes({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('Force shielded: contract.swrite / contract.sread', () => {
  test(
    'contract.swrite always uses seismic tx even for transparent functions',
    async () => await testSwriteAlwaysShielded({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'contract.sread always uses signed read even for transparent functions',
    async () => await testSreadAlwaysShielded({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('Smart routing: wallet client actions', () => {
  test(
    'walletClient.writeContract routes to shielded for suint256',
    async () => await testSmartWalletWriteShielded({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'walletClient.writeContract routes to transparent for no shielded params',
    async () => await testSmartWalletWriteTransparent({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'walletClient.readContract routes to transparent for no shielded inputs',
    async () => await testSmartWalletReadTransparent({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('Force shielded: wallet client actions', () => {
  test(
    'walletClient.swriteContract always uses seismic tx',
    async () => await testSwriteContractAlwaysShielded({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'walletClient.sreadContract always uses signed read',
    async () => await testSreadContractAlwaysShielded({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('Smart routing: end-to-end lifecycle', () => {
  test(
    'full lifecycle via contract proxy with all routing modes',
    async () => await testSmartRoutingLifecycle({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'full lifecycle via wallet client actions with all routing modes',
    async () => await testSmartWalletActionsLifecycle({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('Error paths', () => {
  test(
    'getStorageAt throws on shielded public client',
    async () => await testGetStorageAtThrows({ chain, url }),
    { timeout: TIMEOUT_MS }
  )
  test(
    'signedCall without to address throws',
    async () => await testSignedCallWithoutToThrows({ chain, url, account }),
    { timeout: TIMEOUT_MS }
  )
})

describe('Privacy invariants', () => {
  test(
    'seismic tx calldata is encrypted on-chain',
    async () => await testSeismicTxCalldataIsEncrypted({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('RNG uniqueness', () => {
  test(
    'RNG with different personalization produces different results',
    async () =>
      await testRngDifferentPersProducesDifferentResults({ chain, url }),
    { timeout: TIMEOUT_MS }
  )
})

describe('SignedCall standalone', () => {
  test(
    'signedCall directly reads contract state',
    async () => await testSignedCallDirect({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'signedCall with custom security params',
    async () => await testSignedCallWithSecurityParams({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

describe('Concurrent transactions', () => {
  test(
    'multiple sequential shielded writes all succeed',
    async () =>
      await testConcurrentShieldedTransactions({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
  test(
    'concurrent reads via different methods return consistent results',
    async () => await testConcurrentReads({ chain, url, account }),
    { timeout: CONTRACT_TIMEOUT_MS }
  )
})

// Typed Data tests are placed last because they use EIP-712 messageVersion=2
// signing which may fail on older sreth versions, and a failure here would
// corrupt the shared account nonce for all subsequent tests.
describe('Typed Data', async () => {
  test(
    'client can sign a seismic typed message',
    async () =>
      await testSeismicCallTypedData({
        chain,
        account,
        url,
        encryptionSk: ENCRYPTION_SK,
        encryptionPubkey: ENCRYPTION_PK,
      }),
    { timeout: TIMEOUT_MS }
  )

  test(
    'client can sign via eth_signTypedData',
    async () =>
      await testSeismicTxTypedData({
        account,
        chain,
        url,
        encryptionSk: ENCRYPTION_SK,
        encryptionPubkey: ENCRYPTION_PK,
      }),
    { timeout: TIMEOUT_MS }
  )
})
