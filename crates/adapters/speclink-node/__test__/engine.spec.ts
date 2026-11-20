// Parity tests: the fs-form engine must return the same objects the CLI
// prints with --json on the same fixture project (spec: fs 形式與 CLI 行為對等).
import { beforeAll, describe, expect, it } from 'vitest'
import { execFileSync, execSync } from 'node:child_process'
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'

import { commandNames, createEngine } from '../index.js'
import { cliBin, cliJson, makeFsFixture, repoRoot } from './helpers'

let fixture: string

beforeAll(() => {
  if (!existsSync(cliBin)) {
    execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
  }
  fixture = makeFsFixture()
}, 600_000)

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

const ONLY_FS = "createEngine: only { type: 'fs', root } is accepted — use createHost for a custom store (list, read, commit)"
const NOT_AN_OBJECT =
  "createEngine: options.store must be an object — pass { type: 'fs', root }, or use createHost for a custom store (list, read, commit)"
// A root for the inputs createEngine refuses before it reads the root.
const ANY_ROOT = '/any/project'

/** What `createEngine(options)` throws; fails the test when it returns an engine. */
function thrownBy(options: unknown): unknown {
  try {
    createEngine(options as never)
  } catch (e) {
    return e
  }
  return expect.unreachable('createEngine must throw')
}

/** `target` behind a Proxy that records the name of every property read. */
function recordReads<T extends object>(target: T): { store: T; reads: string[] } {
  const reads: string[] = []
  const store = new Proxy(target, {
    get(inner, key, receiver) {
      if (typeof key === 'string') reads.push(key)
      return Reflect.get(inner, key, receiver)
    },
  })
  return { store, reads }
}

/** An object of methods that record their calls, as a host's store object has them. */
function recordingMethods(names: string[]): { methods: Record<string, () => null>; calls: string[] } {
  const calls: string[] = []
  const methods = Object.fromEntries(
    names.map((name) => [
      name,
      () => {
        calls.push(name)
        return null
      },
    ]),
  )
  return { methods, calls }
}

// spec node-sdk「createEngine 只接受本機資料夾」
describe('createEngine accepts the local folder only', () => {
  it('an object with methods throws at once, names createHost, and none of its methods runs', () => {
    const { methods, calls } = recordingMethods(['listChanges', 'findChange', 'readArtifact', 'writeArtifact', 'claim'])
    const { store, reads } = recordReads(methods)
    const thrown = thrownBy({ store })
    expect(thrown).toBeInstanceOf(TypeError)
    expect((thrown as Error).message).toBe(ONLY_FS)
    expect(calls).toEqual([])
    expect(reads).toEqual(['type'])
  })

  it("{ type: 'fs' } with no root throws at once and says root must be a string", () => {
    const thrown = thrownBy({ store: { type: 'fs' } })
    expect(thrown).toBeInstanceOf(TypeError)
    expect((thrown as Error).message).toBe("createEngine: fs store requires a string 'root'")
  })
})

// 稽核（sharp edges）：createEngine 的輸入檢查。錯的形狀都在建構當下大聲失敗，
// 訊息指出正確的寫法與 createHost。
describe('createEngine input checks', () => {
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
    ['an object with no type', { root: ANY_ROOT }],
    ["type 'FS'", { type: 'FS', root: ANY_ROOT }],
    ["type 'fs ' (trailing space)", { type: 'fs ', root: ANY_ROOT }],
    ['type 1', { type: 1, root: ANY_ROOT }],
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
      root: ANY_ROOT,
    }
    expect(thrownBy({ store })).toBe(boom)
  })

  it("an object with type 'fs' and extra methods is the local folder; the extra methods are never called", async () => {
    const { methods, calls } = recordingMethods(['listChanges', 'readArtifact', 'list', 'read', 'commit'])
    const engine = createEngine({ store: { type: 'fs', root: fixture, ...methods } })
    expect(await engine.dispatch(['list', '--json'])).toEqual(cliJson(fixture, ['list', '--json']))
    expect(calls).toEqual([])
  })

  it('each property of the store is read once', () => {
    const { store, reads } = recordReads({ type: 'fs' as const, root: fixture })
    createEngine({ store })
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

// node-sdk「SDK 匯出命令名稱清單」
describe('commandNames', () => {
  it('lists each command name once, the host kit’s three included', () => {
    expect(Array.isArray(commandNames)).toBe(true)
    expect(commandNames).toHaveLength(57)
    expect(new Set(commandNames).size).toBe(57)
    for (const name of commandNames) expect(typeof name).toBe('string')
    for (const name of [
      'new-change',
      'claim-release',
      'workflow-config-edit',
      'scope-export',
      'scope-import',
      'put-board-order',
    ]) {
      expect(commandNames).toContain(name)
    }
  })

  // node-sdk「型別定義檔宣告了這個匯出」: strict, and the library file is checked too.
  it('is declared as a read-only string array', () => {
    const require = createRequire(import.meta.url)
    const typescript = dirname(require.resolve('typescript/package.json'))
    const dir = mkdtempSync(join(tmpdir(), 'speclink-command-names-'))
    try {
      writeFileSync(
        join(dir, 'consumer.ts'),
        [
          "import { commandNames } from '@speclink/engine'",
          'export const names: readonly string[] = commandNames',
          '// @ts-expect-error a read-only array has no push',
          "commandNames.push('x')",
          '// @ts-expect-error an item is a string, not a number',
          'export const first: number = commandNames[0]',
          '',
        ].join('\n'),
      )
      writeFileSync(
        join(dir, 'tsconfig.json'),
        JSON.stringify({
          compilerOptions: {
            module: 'nodenext',
            moduleResolution: 'nodenext',
            strict: true,
            noEmit: true,
            skipLibCheck: false,
            types: [],
            paths: { '@speclink/engine': [join(repoRoot, 'crates/adapters/speclink-node/index.d.ts')] },
          },
          files: ['consumer.ts'],
        }),
      )
      const tsc = join(typescript, 'bin', 'tsc')
      // A compile error throws, with the compiler's text in the message.
      execFileSync(process.execPath, [tsc, '-p', dir, '--pretty', 'false'], { encoding: 'utf8', stdio: 'pipe' })
    } finally {
      rmSync(dir, { recursive: true, force: true, maxRetries: 5 })
    }
  }, 120_000)

  it('cannot be changed', () => {
    // A value that is no object is frozen too: the array comes first.
    expect(Array.isArray(commandNames)).toBe(true)
    expect(Object.isFrozen(commandNames)).toBe(true)
  })
})
