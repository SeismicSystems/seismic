import { expect, test } from 'bun:test'

test('CommonJS entry point exposes the public API', () => {
  const result = Bun.spawnSync([
    'node',
    '-e',
    "process.stdout.write(JSON.stringify(Object.keys(require('seismic-encrypt')).sort()))",
  ])

  expect(result.exitCode).toBe(0)
  expect(result.stderr.toString()).toBe('')
  expect(JSON.parse(result.stdout.toString())).toEqual([
    'SEISMIC_TX_TYPE',
    'encryptSeismicTx',
    'serializeSeismicTx',
  ])
})
