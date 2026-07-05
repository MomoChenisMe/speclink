// JS Store bridge tests (spec: 宿主 Store 物件生效 / 缺方法建構即失敗 /
// 錯誤以語義化例外傳遞的 store 方法前綴).
import { beforeAll, afterAll, describe, expect, it } from 'vitest'
import { execSync } from 'node:child_process'
import { existsSync, rmSync } from 'node:fs'

import { createEngine } from '../index.js'
import { cliBin, cliJson, fixtureProject, makeFsFixture, memoryStore, repoRoot } from './helpers'

let fsFixture: string

beforeAll(() => {
  if (!existsSync(cliBin)) {
    execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
  }
  fsFixture = makeFsFixture()
}, 600_000)

afterAll(() => {
  if (fsFixture) rmSync(fsFixture, { recursive: true, force: true })
})

describe('createEngine (host Store object)', () => {
  it('serves dispatch(["list",…]) from an async host store, fields matching the CLI', async () => {
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    const sdk = await engine.dispatch(['list', '--sort', 'name', '--json'])
    // The memory store mirrors the fs fixture content, so the CLI on the fs
    // fixture is the field-name and field-value authority (camelCase).
    const cli = cliJson(fsFixture, ['list', '--sort', 'name', '--json'])
    expect(sdk).toEqual(cli)
  })

  it('serves dispatch(["status",…]) from an async host store, fields matching the CLI', async () => {
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    const sdk = await engine.dispatch(['status', '--change', 'alpha', '--json'])
    const cli = cliJson(fsFixture, ['status', '--change', 'alpha', '--json'])
    expect(sdk).toEqual(cli)
  })

  it('throws synchronously at construction listing every missing method name', () => {
    const store = memoryStore(fixtureProject()) as Record<string, unknown>
    delete store.writeArtifact
    delete store.archiveChange
    let thrown: unknown
    try {
      createEngine({ store })
    } catch (e) {
      thrown = e
    }
    expect(thrown).toBeInstanceOf(Error)
    const message = (thrown as Error).message
    expect(message).toContain('writeArtifact')
    expect(message).toContain('archiveChange')
  })

  it('a rejecting store method rejects dispatch with the method name prefixed', async () => {
    const store = memoryStore(fixtureProject())
    store.readArtifact = async () => {
      throw new Error('db down')
    }
    const engine = createEngine({ store })
    await expect(engine.dispatch(['list', '--json'])).rejects.toThrow(/readArtifact.*db down/)
  })

  it('a store method returning a plain value (not a Promise) also works', async () => {
    const store = memoryStore(fixtureProject())
    const asyncRead = store.readLanguage
    // Replace one method with a synchronous variant; the bridge must accept T | Promise<T>.
    store.updatedAtSecs = ((name: string) =>
      name === 'beta' ? 200 : 100) as unknown as typeof store.updatedAtSecs
    void asyncRead
    const engine = createEngine({ store })
    const sdk = (await engine.dispatch(['list', '--json'])) as { changes: { name: string }[] }
    // Default sort is modified (updatedAtSecs desc): beta (200) before alpha (100).
    expect(sdk.changes.map((c) => c.name)).toEqual(['beta', 'alpha'])
  })
})
