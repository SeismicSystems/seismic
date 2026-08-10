/**
 * Seismic Developer Studio Client Logic
 */

document.addEventListener('DOMContentLoaded', () => {
  initTabs();
  fetchNetworkStatus();
  initActionListeners();
});

function initTabs() {
  const tabs = document.querySelectorAll('.nav-tab');
  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.nav-tab').forEach(t => t.classList.toggle('active', t === tab));
      document.querySelectorAll('.tab-pane').forEach(p => p.classList.toggle('active', p.id === `tab-${tab.dataset.tab}`));
    });
  });
}

async function fetchNetworkStatus() {
  try {
    const res = await fetch('/api/status');
    const data = await res.json();
    if (data.online) {
      document.getElementById('stat-block-num').textContent = `#${data.latestBlock}`;
      document.getElementById('stat-rpc-url').textContent = data.activeRpc.replace('https://', '').replace('/rpc', '');
    }
  } catch (e) {
    console.warn('Network status fetch error:', e);
  }
}

function initActionListeners() {
  // Generate Wallet
  document.getElementById('btn-gen-wallet').addEventListener('click', async () => {
    try {
      const res = await fetch('/api/wallet/generate', { method: 'POST' });
      const wallet = await res.json();

      document.getElementById('wallet-info-box').style.display = 'block';
      document.getElementById('wallet-addr-input').value = wallet.address;
      document.getElementById('wallet-pk-input').value = wallet.privateKey;
    } catch (err) {
      alert(`Wallet gen error: ${err.message}`);
    }
  });

  // Claim Faucet Drip
  document.getElementById('btn-claim-drip').addEventListener('click', async () => {
    const addr = document.getElementById('wallet-addr-input').value;
    const btn = document.getElementById('btn-claim-drip');
    const resultBox = document.getElementById('faucet-result');

    if (!addr) return;
    btn.disabled = true;
    btn.textContent = '⏳ Requesting Testnet Drip...';

    try {
      const res = await fetch('/api/faucet/drip', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ address: addr }),
      });
      const data = await res.json();

      resultBox.innerHTML = `
        <div class="card" style="border-color: #06b6d4; background: rgba(6, 182, 212, 0.08);">
          <strong style="color: #67e8f9;">✅ 0.5 sETH Drip Sent!</strong>
          <p class="mt-2" style="font-size: 0.85rem;">Recipient: <span class="mono">${data.recipient}</span></p>
          <span class="mono" style="font-size: 0.8rem; color: #94a3b8;">TX: ${data.txHash}</span>
        </div>
      `;

      loadFaucetHistory();
    } catch (err) {
      resultBox.innerHTML = `<div class="badge red">Faucet error: ${err.message}</div>`;
    } finally {
      btn.disabled = false;
      btn.textContent = '🚰 Claim 0.5 sETH Faucet';
    }
  });

  // Encrypt Shielded State
  document.getElementById('btn-encrypt-shield').addEventListener('click', async () => {
    const type = document.getElementById('shield-type-select').value;
    const val = document.getElementById('shield-val-input').value;
    const outputEl = document.getElementById('shield-output');

    outputEl.textContent = 'Encrypting shielded state for TEE enclave...';

    try {
      const res = await fetch('/api/shielded/encrypt', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ type, value: val }),
      });
      const data = await res.json();
      outputEl.textContent = JSON.stringify(data, null, 2);
    } catch (err) {
      outputEl.textContent = `Error: ${err.message}`;
    }
  });
}

async function loadFaucetHistory() {
  try {
    const res = await fetch('/api/faucet/history');
    const data = await res.json();
    const list = document.getElementById('faucet-history-list');

    if (!data || data.length === 0) return;
    list.innerHTML = '';
    data.forEach(item => {
      const row = document.createElement('div');
      row.className = 'ledger-row';
      row.innerHTML = `
        <div>
          <span class="mono" style="color: #67e8f9;">${item.recipient.slice(0, 8)}...${item.recipient.slice(-6)}</span>
          <div class="text-muted" style="font-size: 0.75rem;">${new Date(item.timestamp).toLocaleTimeString()}</div>
        </div>
        <span style="color: #34d399; font-weight: 700; font-family: var(--font-mono);">${item.amount}</span>
      `;
      list.appendChild(row);
    });
  } catch (e) {
    console.warn(e);
  }
}
