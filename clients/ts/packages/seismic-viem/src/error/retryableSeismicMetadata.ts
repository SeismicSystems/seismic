const RETRYABLE_SEISMIC_METADATA_PATTERNS = [
  /recent_block_hash .* not found in the last \d+ blocks/i,
  /transaction expired: current block .* > expires_at_block/i,
]

const toMessage = (value: unknown): string | undefined => {
  if (typeof value === 'string') {
    return value
  }
  if (typeof value === 'number' || typeof value === 'bigint') {
    return String(value)
  }
  try {
    return JSON.stringify(value)
  } catch {
    return undefined
  }
}

export const isRetryableSeismicMetadataError = (error: unknown): boolean => {
  const messages: string[] = []
  let current = error

  for (let depth = 0; depth < 8 && current; depth++) {
    if (current instanceof Error) {
      if (current.message) {
        messages.push(current.message)
      }
      const maybeCurrent = current as Error & {
        shortMessage?: unknown
        details?: unknown
        cause?: unknown
      }
      const shortMessage = toMessage(maybeCurrent.shortMessage)
      if (shortMessage) {
        messages.push(shortMessage)
      }
      const details = toMessage(maybeCurrent.details)
      if (details) {
        messages.push(details)
      }
      current = maybeCurrent.cause
      continue
    }

    if (typeof current === 'object') {
      const maybeCurrent = current as {
        message?: unknown
        shortMessage?: unknown
        details?: unknown
        cause?: unknown
      }
      const message = toMessage(maybeCurrent.message)
      if (message) {
        messages.push(message)
      }
      const shortMessage = toMessage(maybeCurrent.shortMessage)
      if (shortMessage) {
        messages.push(shortMessage)
      }
      const details = toMessage(maybeCurrent.details)
      if (details) {
        messages.push(details)
      }
      current = maybeCurrent.cause
      continue
    }

    const message = toMessage(current)
    if (message) {
      messages.push(message)
    }
    break
  }

  return messages.some((message) =>
    RETRYABLE_SEISMIC_METADATA_PATTERNS.some((pattern) => pattern.test(message))
  )
}
