/**
 * Seismic RPC Provider Tests
 */

import { defaultSeismicRpc } from '../src/core/rpc.js';
import { SEISMIC_CONFIG } from '../src/config.js';

async function runRpcTests() {
  console.log('Testing Seismic RPC Connectivity (Chain ID 5124)...');

  const status = await defaultSeismicRpc.getNetworkStatus();
  if (status.chainId !== SEISMIC_CONFIG.network.chainId) {
    throw new Error(`Expected Chain ID 5124, got ${status.chainId}`);
  }

  console.log(`✅ Seismic Network Status: Online (Chain ID: ${status.chainId})`);
}

runRpcTests().catch(e => {
  console.error('❌ RPC Test Failed:', e);
  process.exit(1);
});
