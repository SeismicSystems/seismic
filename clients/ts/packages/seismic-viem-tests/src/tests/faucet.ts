import { expect, spyOn } from 'bun:test'
import { checkFaucet, parseFaucetResponseHash } from 'seismic-viem'
import type { TransactionReceipt } from 'viem'
import { createPublicClient, custom } from 'viem'

const ADDRESS = '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266'
const HASH = `0x${'ab'.repeat(32)}` as const
const FAUCET_URL = 'https://faucet.example'

const faucetClient = () =>
  createPublicClient({
    transport: custom({
      request: async ({ method }) => {
        throw new Error(`Unexpected RPC: ${method}`)
      },
    }),
  })

export const testCheckFaucetWithoutBalanceCheck = async () => {
  const publicClient = faucetClient()
  const getBalance = spyOn(publicClient, 'getBalance').mockResolvedValue(
    2n ** 255n
  )
  const wait = spyOn(
    publicClient,
    'waitForTransactionReceipt'
  ).mockResolvedValue({
    transactionHash: HASH,
    status: 'success',
  } as TransactionReceipt)
  const fetchClaim = spyOn(globalThis, 'fetch').mockResolvedValue(
    Response.json({ msg: `Txhash: ${HASH}` })
  )
  try {
    const result = await checkFaucet({
      address: ADDRESS,
      publicClient,
      faucetUrl: FAUCET_URL,
    })
    expect(result).toEqual({ sent: true, hash: HASH, txUrl: undefined })
    expect(getBalance).not.toHaveBeenCalled()
    expect(fetchClaim).toHaveBeenCalledTimes(1)
    expect(fetchClaim).toHaveBeenCalledWith(`${FAUCET_URL}/api/claim`, {
      method: 'POST',
      body: JSON.stringify({ address: ADDRESS }),
    })
    expect(wait).toHaveBeenCalledWith({ hash: HASH })
  } finally {
    getBalance.mockRestore()
    wait.mockRestore()
    fetchClaim.mockRestore()
  }
}

export const testCheckFaucetWaitsForConfirmation = async () => {
  const publicClient = faucetClient()
  let confirm = () => {}
  const confirmation = new Promise<void>((resolve) => {
    confirm = resolve
  })
  let waiting = () => {}
  const startedWaiting = new Promise<void>((resolve) => {
    waiting = resolve
  })
  const wait = spyOn(
    publicClient,
    'waitForTransactionReceipt'
  ).mockImplementation(async () => {
    waiting()
    await confirmation
    return { transactionHash: HASH, status: 'success' } as TransactionReceipt
  })
  const fetchClaim = spyOn(globalThis, 'fetch').mockResolvedValue(
    Response.json({ msg: `Txhash: ${HASH}` })
  )
  try {
    let settled = false
    const claim = checkFaucet({
      address: ADDRESS,
      publicClient,
      faucetUrl: FAUCET_URL,
    }).then((result) => {
      settled = true
      return result
    })
    await startedWaiting
    expect(settled).toBe(false)
    confirm()
    expect((await claim).sent).toBe(true)
  } finally {
    confirm()
    wait.mockRestore()
    fetchClaim.mockRestore()
  }
}

export const testCheckFaucetSurfacesRejection = async () => {
  const publicClient = faucetClient()
  const wait = spyOn(publicClient, 'waitForTransactionReceipt')
  const fetchClaim = spyOn(globalThis, 'fetch')
  try {
    for (const [response, message] of [
      [new Response('Cooldown', { status: 429 }), 'status 429: Cooldown'],
      [
        Response.json({ msg: 'Already claimed' }),
        'Faucet claim failed: Already claimed',
      ],
      [
        Response.json({ msg: 'Txhash: 0xshort' }),
        'Invalid hash from faucet claim',
      ],
    ] as const) {
      fetchClaim.mockResolvedValueOnce(response)
      await expect(
        checkFaucet({ address: ADDRESS, publicClient, faucetUrl: FAUCET_URL })
      ).rejects.toThrow(message)
    }
    expect(wait).not.toHaveBeenCalled()
  } finally {
    wait.mockRestore()
    fetchClaim.mockRestore()
  }
}

export const testParseFaucetResponseHashValid = () => {
  expect(parseFaucetResponseHash(`Txhash: ${HASH}`)).toBe(HASH)
}

export const testParseFaucetResponseHashNoPrefix = () => {
  expect(parseFaucetResponseHash('Some other message')).toBeNull()
}

export const testParseFaucetResponseHashThrowsOnInvalidLength = () => {
  expect(() => parseFaucetResponseHash('Txhash: 0xshort')).toThrow(
    'Invalid hash from faucet claim'
  )
}

export const testParseFaucetResponseHashThrowsOnMissingHexPrefix = () => {
  expect(() => parseFaucetResponseHash(`Txhash: ${HASH.slice(2)}`)).toThrow(
    'Invalid hash from faucet claim'
  )
}
