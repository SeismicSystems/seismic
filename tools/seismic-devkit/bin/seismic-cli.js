#!/usr/bin/env node

/**
 * Seismic CLI - Testnet & Privacy EVM Toolkit
 */

import { defaultSeismicRpc } from '../src/core/rpc.js';
import { defaultShieldedEngine } from '../src/core/shielded.js';
import { defaultFaucetManager } from '../src/core/faucet.js';

const args = process.argv.slice(2);
const command = args[0] || 'help';

async function main() {
  switch (command.toLowerCase()) {
    case 'status': {
      console.log('\n🛡️  Connecting to Seismic Testnet...');
      const status = await defaultSeismicRpc.getNetworkStatus();
      console.log(`  Network:       ${status.name}`);
      console.log(`  Chain ID:      ${status.chainId}`);
      console.log(`  Latest Block:  #${status.latestBlock}`);
      console.log(`  RPC Endpoint:  ${status.activeRpc}`);
      console.log(`  Explorer:      ${status.explorer}\n`);
      break;
    }

    case 'wallet': {
      console.log('\n👛 Generating New Seismic Testnet Wallet...');
      const w = defaultFaucetManager.generateNewWallet();
      console.log(`  Address:     ${w.address}`);
      console.log(`  Private Key: ${w.privateKey}`);
      console.log(`  Mnemonic:    ${w.mnemonic}\n`);
      break;
    }

    case 'faucet': {
      const address = args[1] || defaultFaucetManager.generateNewWallet().address;
      console.log(`\n🚰 Requesting 0.5 sETH Faucet Drip for ${address}...`);
      const drip = await defaultFaucetManager.requestTestnetDrip(address);
      console.log(`  Status:    ${drip.status}`);
      console.log(`  Amount:    ${drip.amount}`);
      console.log(`  TX Hash:   ${drip.txHash}\n`);
      break;
    }

    case 'encrypt': {
      const type = args[1] || 'suint256';
      const val = args[2] || '1000';
      console.log(`\n🔒 Encrypting Shielded Type '${type}' with value '${val}'...`);
      const fmt = defaultShieldedEngine.formatShieldedType(type, val);
      const enc = defaultShieldedEngine.encryptShieldedCalldata(fmt.shieldedHex);
      console.log(`  Raw Shielded Hex: ${fmt.shieldedHex}`);
      console.log(`  Encrypted TEE:    ${enc}\n`);
      break;
    }

    case 'studio': {
      console.log('\n🌐 Launching Seismic Developer Studio on :3405...');
      await import('../src/server/studio.js');
      break;
    }

    default: {
      console.log(`
╔══════════════════════════════════════════════════════════════════╗
║               🛡️ SEISMIC PRIVACY EVM DEVELOPER CLI              ║
║       Shielded State, Testnet Faucet & TEE Encryption Suite      ║
╚══════════════════════════════════════════════════════════════════╝

Commands:
  seismic-cli status                   Check Seismic Testnet (5124) status
  seismic-cli wallet                   Generate a new testnet wallet
  seismic-cli faucet [address]         Request testnet faucet funds (sETH)
  seismic-cli encrypt [type] [value]   Encrypt shielded state for TEE
  seismic-cli studio                   Launch interactive Web Studio on :3405
      `);
      break;
    }
  }
}

main().catch(err => {
  console.error('Error:', err.message);
  process.exit(1);
});
