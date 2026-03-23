import {
  createPublicClient,
  encodeFunctionData,
  http,
  parseAbi,
} from 'viem'
import { privateKeyToAccount } from 'viem/accounts'
import { encryptSeismicTx } from 'seismic-encrypt'
import type { Address, Hex } from 'viem'

// ── Read env ────────────────────────────────────────────────────────

const required = (name: string): string => {
  const val = process.env[name]
  if (!val) {
    console.error(`Missing required env var: ${name}`)
    console.error('Copy .env.example to .env and fill in the values.')
    process.exit(1)
  }
  return val
}

const RPC_URL = required('RPC_URL')
const PRIVATE_KEY = required('PRIVATE_KEY') as Hex
const TOKEN_ADDRESS = required('TOKEN_ADDRESS') as Address
const RECIPIENT = required('RECIPIENT') as Address
const AMOUNT = BigInt(required('AMOUNT'))
const CHAIN_ID = Number(required('CHAIN_ID'))

// ── ERC20 ABI (just transfer) ───────────────────────────────────────

const erc20Abi = parseAbi([
  'function transfer(address to, uint256 amount) returns (bool)',
  'function balanceOf(address owner) view returns (uint256)',
])

// ── Main ────────────────────────────────────────────────────────────

const main = async () => {
  const account = privateKeyToAccount(PRIVATE_KEY)
  const client = createPublicClient({ transport: http(RPC_URL) })

  console.log(`Sender:   ${account.address}`)
  console.log(`Token:    ${TOKEN_ADDRESS}`)
  console.log(`To:       ${RECIPIENT}`)
  console.log(`Amount:   ${AMOUNT}`)
  console.log(`Chain ID: ${CHAIN_ID}`)
  console.log()

  // 1. Build the calldata — a normal ERC20 transfer
  const data = encodeFunctionData({
    abi: erc20Abi,
    functionName: 'transfer',
    args: [RECIPIENT, AMOUNT],
  })

  // 2. Gather tx fields from the network
  const [nonce, gasPrice] = await Promise.all([
    client.getTransactionCount({ address: account.address }),
    client.getGasPrice(),
  ])

  const tx = {
    to: TOKEN_ADDRESS,
    data,
    nonce,
    gasPrice,
    gas: 100_000n,
    chainId: CHAIN_ID,
  }

  console.log('Plaintext calldata:', data)
  console.log()

  // 3. Encrypt for Seismic
  console.log('Encrypting transaction...')
  const { seismicTx, serialize } = await encryptSeismicTx({
    tx,
    sender: account.address,
    rpcUrl: RPC_URL,
  })

  console.log('Encrypted calldata:', seismicTx.data)
  console.log()

  // 4. Sign with the local account
  const signed = await account.signTransaction(
    { ...seismicTx },
    { serializer: (_tx, sig) => serialize(sig!) },
  )

  // 5. Send via standard eth_sendRawTransaction
  console.log('Sending raw transaction...')
  const hash = await client.sendRawTransaction({
    serializedTransaction: signed,
  })

  console.log(`tx hash: ${hash}`)

  // 6. Wait for receipt
  console.log('Waiting for confirmation...')
  const receipt = await client.waitForTransactionReceipt({ hash })
  console.log(`Status:       ${receipt.status}`)
  console.log(`Block number: ${receipt.blockNumber}`)
  console.log(`Gas used:     ${receipt.gasUsed}`)
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
