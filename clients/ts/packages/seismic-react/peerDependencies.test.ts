import { describe, expect, test } from 'bun:test'

const packageJson = await Bun.file(
  new URL('./package.json', import.meta.url)
).json()

describe.each(['react', 'react-dom'])('%s peer dependency', (peer) => {
  const range = packageJson.peerDependencies[peer]

  test.each(['18.3.1', '19.2.0'])('accepts React %s', (version) => {
    expect(Bun.semver.satisfies(version, range)).toBe(true)
  })

  test('does not expand support below React 18', () => {
    expect(Bun.semver.satisfies('17.0.2', range)).toBe(false)
  })
})
