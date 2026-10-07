import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import * as esm from 'seismic-encrypt'
import { keccak256, recoverAddress, toHex } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

const require = createRequire(import.meta.url)
const cjs = require('seismic-encrypt')
const fixture = JSON.parse(
  readFileSync(
    new URL('../../../../test-vectors/gas-payment.json', import.meta.url),
    'utf8'
  )
)

for (const [name, api] of [
  ['ESM', esm],
  ['CommonJS', cjs],
]) {
  assert.equal(
    typeof api.encryptSeismicTx,
    'function',
    `${name} encryption export`
  )
  assert.equal(
    typeof api.serializeSeismicTx,
    'function',
    `${name} serializer export`
  )
  for (const vector of fixture.vectors) {
    const tx = vector.tx
    const standalone = {
      chainId: Number(tx.chainId),
      nonce: Number(tx.nonce),
      gasPrice: BigInt(tx.gasPrice),
      gas: BigInt(tx.gas),
      gasPayment: tx.gasPayment,
      to: tx.to ?? null,
      value: BigInt(tx.value),
      data: tx.input,
      encryptionPubkey: `0x${tx.encryptionPubkey}`,
      encryptionNonce: toHex(BigInt(tx.encryptionNonce), { size: 12 }),
      messageVersion: Number(tx.messageVersion),
      recentBlockHash: tx.recentBlockHash,
      expiresAtBlock: BigInt(tx.expiresAtBlock),
      signedRead: tx.signedRead,
      authorizationList: tx.authorizationList.map((auth) => ({
        chainId: Number(auth.chainId),
        contractAddress: auth.address,
        nonce: Number(auth.nonce),
        yParity: Number(auth.yParity),
        r: auth.r,
        s: auth.s,
      })),
    }
    assert.equal(api.serializeSeismicTx(standalone), vector.unsigned)
    const { r, s, yParity } = vector.signature
    assert.equal(
      api.serializeSeismicTx(standalone, { r, s, v: BigInt(yParity) }),
      vector.signed
    )
    assert.equal(
      api.serializeSeismicTx(standalone, { r, s, yParity: Number(yParity) }),
      vector.signed
    )
  }
}

// Only two read-only RPC methods are mocked; no network or backend process is used.
const originalFetch = globalThis.fetch
const account = privateKeyToAccount(fixture.privateKey)
globalThis.fetch = async (_input, init) => {
  const request = JSON.parse(init.body)
  let result
  if (request.method === 'seismic_getTeePublicKey') {
    result = `0x${fixture.vectors[0].tx.encryptionPubkey}`
  } else if (request.method === 'eth_getBlockByNumber') {
    result = { hash: `0x${'ab'.repeat(32)}`, number: '0x1', transactions: [] }
  } else {
    throw new Error(`Unexpected consumer RPC: ${request.method}`)
  }
  return new Response(
    JSON.stringify({ jsonrpc: '2.0', id: request.id, result }),
    {
      headers: { 'content-type': 'application/json' },
    }
  )
}
try {
  for (const [name, api] of [
    ['ESM', esm],
    ['CommonJS', cjs],
  ]) {
    const prepared = await api.encryptSeismicTx({
      tx: {
        chainId: 5124,
        nonce: 1,
        gasPrice: 1n,
        gas: 100_000n,
        to: account.address,
        value: 0n,
        data: '0xdeadbeef',
      },
      sender: account.address,
      rpcUrl: 'http://fixture.invalid',
      encryptionPrivateKey: fixture.privateKey,
    })
    assert.equal(prepared.serialize(), prepared.unsignedSerializedTx)
    const directSigned = await account.signTransaction(prepared.seismicTx, {
      serializer: api.serializeSeismicTx,
    })
    let signature
    const callbackSigned = await account.signTransaction(prepared.seismicTx, {
      serializer: (_tx, sig) => {
        if (sig) signature = sig
        return prepared.serialize(sig)
      },
    })
    assert.equal(callbackSigned, directSigned, `${name} signature callback`)
    const recovered = await recoverAddress({
      hash: keccak256(prepared.unsignedSerializedTx),
      signature,
    })
    assert.equal(recovered.toLowerCase(), account.address.toLowerCase())
  }
} finally {
  globalThis.fetch = originalFetch
}
console.log(
  `Built ESM/CommonJS consumer smoke passed: ${fixture.vectors.length} golden vectors each and mocked encryption/Viem signing`
)
