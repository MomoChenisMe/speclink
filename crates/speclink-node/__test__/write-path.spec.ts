// Write-path verbs and the stdin parameter form (spec: 寫入動詞經 stdin 參數 /
// 錯誤以語義化例外傳遞).
import { beforeAll, afterAll, describe, expect, it } from 'vitest'
import { execSync } from 'node:child_process'
import { existsSync, readFileSync, rmSync } from 'node:fs'
import { join } from 'node:path'

import { createEngine } from '../index.js'
import { cliBin, fixtureProject, makeFsFixture, memoryStore, repoRoot } from './helpers'

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

describe('dispatch write path — stdin parameter form', () => {
  it('new artifact --stdin writes through the host store and resolves success', async () => {
    const store = memoryStore(fixtureProject())
    const writes: Array<[string, string]> = []
    const originalWrite = store.writeArtifact.bind(store)
    store.writeArtifact = async (change: string, artifact: string, content: string) => {
      writes.push([change, artifact])
      return originalWrite(change, artifact, content)
    }
    const engine = createEngine({ store })
    const content = '## Context\n\nHost-written design document.\n'
    const result = await engine.dispatch(
      ['new', 'artifact', 'design', '--change', 'alpha', '--stdin', '--json'],
      { stdin: content },
    )
    expect(writes).toContainEqual(['alpha', 'design.md'])
    expect(result).toMatchObject({
      artifact: 'design',
      change: 'alpha',
      status: 'created',
      validated: true,
      warnings: [],
    })
    expect(await store.readArtifact('alpha', 'design.md')).toBe(content)
  })

  it('new artifact --stdin on the fs form creates the file like the CLI', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fsFixture } })
    const content = '## Context\n\nDesign written through the SDK.\n'
    const result = (await engine.dispatch(
      ['new', 'artifact', 'design', '--change', 'alpha', '--stdin', '--json'],
      { stdin: content },
    )) as { artifact: string; change: string; path: string; status: string; validated: boolean }
    expect(result).toMatchObject({
      artifact: 'design',
      change: 'alpha',
      status: 'created',
      validated: true,
    })
    expect(result.path.endsWith('design.md')).toBe(true)
    const onDisk = readFileSync(join(fsFixture, 'openspec', 'changes', 'alpha', 'design.md'), 'utf8')
    expect(onDisk).toBe(content)
  })

  it('content validation failures reject with the semantic CLI message', async () => {
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    await expect(
      engine.dispatch(['new', 'artifact', 'tasks', '--change', 'alpha', '--stdin', '--force'], {
        stdin: 'no checkboxes here\n',
      }),
    ).rejects.toThrow(/Tasks must contain at least one checkbox/)
  })
})

describe('claim — semantic conflict errors', () => {
  it('a held change rejects with a semantic message and the conflict code', async () => {
    const store = memoryStore(fixtureProject()) as ReturnType<typeof memoryStore> & {
      claim?: (name: string) => Promise<unknown>
    }
    store.claim = async (name: string) => {
      const err = new Error(
        `Change '${name}' is held by chiang — coordinate, or re-claim if it was released.`,
      ) as Error & { code: string }
      err.code = 'ownership_lost'
      throw err
    }
    const engine = createEngine({ store })
    let thrown: (Error & { code?: string }) | undefined
    try {
      await engine.dispatch(['claim', 'alpha'])
    } catch (e) {
      thrown = e as Error & { code?: string }
    }
    expect(thrown).toBeInstanceOf(Error)
    expect(thrown!.message).toContain('held by chiang')
    expect(thrown!.code).toBe('ownership_lost')
  })

  it('a granted claim resolves with the host payload', async () => {
    const store = memoryStore(fixtureProject()) as ReturnType<typeof memoryStore> & {
      claim?: (name: string) => Promise<unknown>
    }
    store.claim = async () => ({ claimed: true, claimedBy: 'you' })
    const engine = createEngine({ store })
    expect(await engine.dispatch(['claim', 'alpha'])).toEqual({ claimed: true, claimedBy: 'you' })
  })

  it('claim on the fs store fails loud like the CLI', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fsFixture } })
    await expect(engine.dispatch(['claim', 'alpha'])).rejects.toThrow(/claim requires a remote store/)
  })
})
