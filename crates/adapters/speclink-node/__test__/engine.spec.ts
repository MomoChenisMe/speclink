// Parity tests: the fs-form engine must return the same objects the CLI
// prints with --json on the same fixture project (spec: fs 形式與 CLI 行為對等).
import { beforeAll, afterAll, describe, expect, it } from 'vitest'
import { execSync } from 'node:child_process'
import { existsSync, rmSync } from 'node:fs'

import { createEngine } from '../index.js'
import { cliBin, cliJson, makeFsFixture, repoRoot } from './helpers'

let fixture: string
/** The fixture root for a table row, which is built before `beforeAll` runs. */
const fixtureRoot = () => '/any/project'

beforeAll(() => {
  if (!existsSync(cliBin)) {
    execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
  }
  fixture = makeFsFixture()
}, 600_000)

afterAll(() => {
  if (fixture) rmSync(fixture, { recursive: true, force: true })
})

describe('createEngine (fs form) — CLI parity', () => {
  it('dispatch(["list","--json"]) matches the CLI field by field', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fixture } })
    const sdk = await engine.dispatch(['list', '--json'])
    const cli = cliJson(fixture, ['list', '--json'])
    expect(sdk).toEqual(cli)
  })

  it('dispatch(["list","--specs","--json"]) matches the CLI', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fixture } })
    const sdk = await engine.dispatch(['list', '--specs', '--json'])
    const cli = cliJson(fixture, ['list', '--specs', '--json'])
    expect(sdk).toEqual(cli)
  })

  it('dispatch(["status","--change","alpha","--json"]) matches the CLI field by field', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fixture } })
    const sdk = await engine.dispatch(['status', '--change', 'alpha', '--json'])
    const cli = cliJson(fixture, ['status', '--change', 'alpha', '--json'])
    expect(sdk).toEqual(cli)
  })

  it('dispatch(["status","--change","beta","--json"]) matches the CLI (all tasks done)', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fixture } })
    const sdk = await engine.dispatch(['status', '--change', 'beta', '--json'])
    const cli = cliJson(fixture, ['status', '--change', 'beta', '--json'])
    expect(sdk).toEqual(cli)
  })
})

// spec node-sdk「createEngine 只接受本機資料夾」
describe('createEngine accepts the local folder only', () => {
  /**
   * An object that has every method a caller could name (listChanges,
   * readArtifact, …): each one records its call, and every property read is
   * recorded too. Only `type` is absent.
   */
  function recordingStore() {
    const calls: string[] = []
    const reads: string[] = []
    const store = new Proxy(
      {},
      {
        get(_target, key) {
          if (typeof key !== 'string') return undefined
          reads.push(key)
          if (key === 'type') return undefined
          return () => {
            calls.push(key)
            return null
          }
        },
      },
    )
    return { store, calls, reads }
  }

  it('an object with methods throws at once, names createHost, and none of its methods runs', () => {
    const { store, calls, reads } = recordingStore()
    let engine: unknown
    let thrown: unknown
    try {
      engine = createEngine({ store: store as never })
    } catch (e) {
      thrown = e
    }
    expect(thrown).toBeInstanceOf(TypeError)
    expect((thrown as Error).message).toBe(
      "createEngine: only { type: 'fs', root } is accepted — use createHost for a custom store (list, read, commit)",
    )
    expect(engine).toBeUndefined()
    expect(calls).toEqual([])
    expect(reads).toEqual(['type'])
  })

  it("{ type: 'fs' } with no root throws at once and says root must be a string", () => {
    let engine: unknown
    let thrown: unknown
    try {
      engine = createEngine({ store: { type: 'fs' } as never })
    } catch (e) {
      thrown = e
    }
    expect(thrown).toBeInstanceOf(TypeError)
    expect((thrown as Error).message).toBe("createEngine: fs store requires a string 'root'")
    expect(engine).toBeUndefined()
  })
})

// 稽核（sharp edges）：createEngine 的輸入檢查。錯的形狀都在建構當下大聲失敗，
// 訊息指出正確的寫法與 createHost。
describe('createEngine input checks', () => {
  const ONLY_FS = "createEngine: only { type: 'fs', root } is accepted — use createHost for a custom store (list, read, commit)"
  const NOT_AN_OBJECT =
    "createEngine: options.store must be an object — pass { type: 'fs', root }, or use createHost for a custom store (list, read, commit)"

  /** What `createEngine(options)` throws; fails the test when it returns an engine. */
  function thrownBy(options: unknown): unknown {
    try {
      createEngine(options as never)
    } catch (e) {
      return e
    }
    return expect.unreachable('createEngine must throw')
  }

  it.each([
    ['no options', undefined],
    ['no store', {}],
    ['null', { store: null }],
    ['a string', { store: '/some/project' }],
    ['a number', { store: 1 }],
    ['a function', { store: () => ({ type: 'fs' }) }],
  ])('a store that is not an object (%s) throws a TypeError that names createHost', (_label, options) => {
    const thrown = thrownBy(options)
    expect(thrown).toBeInstanceOf(TypeError)
    expect((thrown as Error).message).toBe(NOT_AN_OBJECT)
  })

  it.each([
    ['an array', []],
    ['an object with no type', { root: fixtureRoot() }],
    ["type 'FS'", { type: 'FS', root: fixtureRoot() }],
    ["type 'fs ' (trailing space)", { type: 'fs ', root: fixtureRoot() }],
    ['type 1', { type: 1, root: fixtureRoot() }],
    ['the three functions of a store port', { list() {}, read() {}, commit() {} }],
  ])('a store of another shape (%s) throws the only-fs TypeError', (_label, store) => {
    const thrown = thrownBy({ store })
    expect(thrown).toBeInstanceOf(TypeError)
    expect((thrown as Error).message).toBe(ONLY_FS)
  })

  it('a `type` getter that throws passes its own error out, and no engine is built', () => {
    const boom = new Error('type is not readable')
    const store = {
      get type(): string {
        throw boom
      },
      root: fixtureRoot(),
    }
    expect(thrownBy({ store })).toBe(boom)
  })

  it("an object with type 'fs' and extra methods is the local folder; the extra methods are never called", async () => {
    const calls: string[] = []
    const record = (name: string) => () => {
      calls.push(name)
      return null
    }
    const store = {
      type: 'fs' as const,
      root: fixture,
      listChanges: record('listChanges'),
      readArtifact: record('readArtifact'),
      list: record('list'),
      read: record('read'),
      commit: record('commit'),
    }
    const engine = createEngine({ store })
    expect(await engine.dispatch(['list', '--json'])).toEqual(cliJson(fixture, ['list', '--json']))
    expect(calls).toEqual([])
  })

  it('each property of the store is read once', () => {
    const reads: string[] = []
    const store = new Proxy(
      { type: 'fs', root: fixture },
      {
        get(target, key, receiver) {
          if (typeof key === 'string') reads.push(key)
          return Reflect.get(target, key, receiver)
        },
      },
    )
    createEngine({ store: store as never })
    expect(reads.sort()).toEqual(['root', 'specDir', 'type'])
  })

  it.each([[1], [null], [{ name: 'Alice' }], [['Alice <alice@example.com>']]])(
    'an actor that is not a string (%j) throws the actor TypeError',
    (actor) => {
      const thrown = thrownBy({ store: { type: 'fs', root: fixture }, actor })
      expect(thrown).toBeInstanceOf(TypeError)
      expect((thrown as Error).message).toBe('createEngine: options.actor must be a string ("Name <email>")')
    },
  )
})
