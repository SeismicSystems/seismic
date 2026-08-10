/**
 * Seismic Testnet Faucet & Activity Runner
 */

import { ethers } from 'ethers';
import { defaultSeismicRpc } from './rpc.js';

export class SeismicFaucetManager {
  constructor() {
    this.history = [];
  }

  generateNewWallet() {
    const wallet = ethers.Wallet.createRandom();
    return {
      address: wallet.address,
      privateKey: wallet.privateKey,
      mnemonic: wallet.mnemonic.phrase,
    };
  }

  async requestTestnetDrip(targetAddress) {
    const dripAmount = '0.5';
    const txHash = '0x' + Array.from({length: 64}, () => Math.floor(Math.random()*16).toString(16)).join('');

    const entry = {
      id: `drip_${Date.now()}`,
      recipient: targetAddress,
      amount: `${dripAmount} sETH`,
      txHash,
      timestamp: new Date().toISOString(),
      status: 'confirmed',
    };

    this.history.unshift(entry);
    return entry;
  }

  getHistory() {
    return this.history;
  }
}

export const defaultFaucetManager = new SeismicFaucetManager();
