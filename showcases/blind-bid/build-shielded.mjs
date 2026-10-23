// Сборка нативной версии контракта их компилятором (ssolc, seismic-solidity).
//   node scripts/build-shielded.mjs
//
// Нужен бинарник ssolc: https://github.com/SeismicSystems/seismic-solidity/releases
// (в репозитории рядом лежит E:\Cli work\seismic\tools\ssolc\solc.exe).
// Путь можно задать переменной SSOLC.
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SSOLC = process.env.SSOLC || 'E:\\Cli work\\seismic\\tools\\ssolc\\solc.exe';
const FILE = 'BlindBidShielded.sol';
const CONTRACT = 'BlindBid';

if (!fs.existsSync(SSOLC)) {
  console.error('нет ssolc:', SSOLC, '\nскачай релиз seismic-solidity и укажи путь в SSOLC');
  process.exit(1);
}

const input = {
  language: 'Solidity',
  sources: { [FILE]: { content: fs.readFileSync(path.join(ROOT, 'contracts', FILE), 'utf8') } },
  settings: {
    optimizer: { enabled: true, runs: 200 },
    outputSelection: { '*': { '*': ['abi', 'evm.bytecode.object', 'evm.deployedBytecode.object'] } },
  },
};

const out = JSON.parse(execFileSync(SSOLC, ['--standard-json'], { input: JSON.stringify(input), maxBuffer: 64 * 1024 * 1024 }).toString());
const errors = (out.errors || []).filter((e) => e.severity === 'error');
if (errors.length) {
  errors.forEach((e) => console.log(e.formattedMessage));
  process.exit(1);
}
const c = out.contracts[FILE][CONTRACT];
const artifact = {
  BlindBid: {
    abi: c.abi,
    bytecode: '0x' + c.evm.bytecode.object,
    deployedBytecode: '0x' + c.evm.deployedBytecode.object,
  },
};
const outPath = path.join(ROOT, 'artifacts-shielded.json');
fs.writeFileSync(outPath, JSON.stringify(artifact, null, 2));
console.log(`compiled ${CONTRACT} (${FILE}) — ${c.evm.bytecode.object.length / 2} bytes -> artifacts-shielded.json`);
