/**
 * Seismic Developer Studio Web Server
 */

import express from 'express';
import cors from 'cors';
import path from 'path';
import { fileURLToPath } from 'url';
import { defaultSeismicRpc } from '../core/rpc.js';
import { defaultShieldedEngine } from '../core/shielded.js';
import { defaultFaucetManager } from '../core/faucet.js';
import { SEISMIC_CONFIG } from '../config.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WEB_ROOT = path.join(__dirname, '../../web');

const app = express();
const PORT = process.env.PORT || 3405;

app.use(cors());
app.use(express.json());
app.use(express.static(WEB_ROOT));

// 1. Network Status
app.get('/api/status', async (req, res) => {
  try {
    const status = await defaultSeismicRpc.getNetworkStatus();
    res.json(status);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// 2. Generate Wallet
app.post('/api/wallet/generate', (req, res) => {
  const wallet = defaultFaucetManager.generateNewWallet();
  res.json(wallet);
});

// 3. Faucet Drip
app.post('/api/faucet/drip', async (req, res) => {
  const { address } = req.body;
  if (!address) {
    return res.status(400).json({ error: 'Address required' });
  }
  const result = await defaultFaucetManager.requestTestnetDrip(address);
  res.json(result);
});

// 4. Faucet History
app.get('/api/faucet/history', (req, res) => {
  res.json(defaultFaucetManager.getHistory());
});

// 5. Shielded State Simulator
app.post('/api/shielded/encrypt', (req, res) => {
  const { calldata, type, value } = req.body;
  try {
    let raw = calldata;
    if (type && value !== undefined) {
      const formatted = defaultShieldedEngine.formatShieldedType(type, value);
      raw = formatted.shieldedHex;
    }
    const encrypted = defaultShieldedEngine.encryptShieldedCalldata(raw || '0x00');
    res.json({ success: true, original: raw, encrypted });
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

if (process.env.NODE_ENV !== 'test') {
  app.listen(PORT, () => {
    console.log(`\n======================================================`);
    console.log(`🛡️  Seismic Developer Studio & Testnet Toolkit Running!`);
    console.log(`🌐 Web Dashboard: http://localhost:${PORT}`);
    console.log(`⛓️  Connected to: Seismic Testnet (Chain ID 5124)`);
    console.log(`======================================================\n`);
  });
}

export default app;
