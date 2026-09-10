import { describe, expect, test } from 'bun:test'

const packageJson = await Bun.file(
  new URL('./package.json', import.meta.url)
).json()

const peers = ['react', 'react-dom'] as const

describe('React peer dependency contract', () => {
  test('keeps react and react-dom on the same compatibility range', () => {
    expect(packageJson.peerDependencies.react).toBe(
      packageJson.peerDependencies['react-dom']
    )
  })

  describe.each(peers)('%s peer dependency', (peer) => {
    const range = packageJson.peerDependencies[peer]

    test.each(['18.0.0', '18.3.1', '19.0.0', '19.2.0'])(
      'accepts supported React version %s',
      (version) => {
        expect(Bun.semver.satisfies(version, range)).toBe(true)
      }
    )

    test.each(['17.0.2', '20.0.0'])(
      'rejects unsupported React version %s',
      (version) => {
        expect(Bun.semver.satisfies(version, range)).toBe(false)
      }
    )
  })
})
