// node-sdk「dispatch 與 handleRequest 的 fields」(change host-free-fields): the
// free fields a host attaches to a call, and the field verbs through the SDK.
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, describe, expect, it } from 'vitest'

import { createEngine, createHost, type BeforeCommandInfo } from '../index.js'
import { rejection } from './helpers'
import { memoryPort } from './memory-port'

const API = '/api/speclink/v1'
const ALICE = { id: 'u_1', displayName: 'Alice' }
const META = 'schema: spec-driven\n'
const TASKS = '## 1. Work\n\n- [ ] 1.1 First task\n'
const RECORD =
  '---\ntopic: Board search\nslug: board-search\nstatus: open\ncreated: 2026-11-01\n---\n\n# Discussion: Board search\n\n## Context\n\n## Rounds\n\n## Conclusion\n'

const ONE_REPO = [
  { key: 'alpha', name: 'Alpha', repos: [{ key: 'web', name: 'Web', scope: 's1', canWrite: true, canWritePolicy: true }] },
]

/**
 * Values that are not free fields: an object with a value that is not a
 * string, and anything that is not a plain object. JSON would drop some of
 * these without a word (`undefined`, a function, a symbol key) or turn them
 * into text (a Date), so each one is here by name.
 */
const NOT_FIELDS: unknown[] = [
  { n: 1 },
  { ok: 'x', flag: true },
  { nested: { a: '1' } },
  { source: undefined },
  { source: null },
  { at: new Date(0) },
  { run: () => 'x' },
  { [Symbol('hidden')]: 'x' },
  new Map([['source', 'portal']]),
  new Date(0),
  ['a'],
  'source=portal',
  7,
]

/** A port with change `demo` and discussion `board-search` in scope `s1`. */
function portWithDemo() {
  const port = memoryPort()
  port.seed('s1', {
    'config.yaml': 'schema: spec-driven\n',
    'changes/demo/.meta.yaml': META,
    'changes/demo/tasks.md': TASKS,
    'discussions/board-search.md': RECORD,
  })
  return port
}

function request(method: string, tail: string, body?: unknown) {
  return {
    method,
    path: `${API}/projects/alpha/${tail}`,
    headers: { 'x-speclink-api-version': '1', 'content-type': 'application/json' },
    body: body === undefined ? undefined : new TextEncoder().encode(JSON.stringify(body)),
  }
}

function json(response: { body: Uint8Array } | null): any {
  expect(response).not.toBeNull()
  return JSON.parse(new TextDecoder().decode(response!.body))
}

/** The content the commit writes to `key`. */
function written(port: ReturnType<typeof memoryPort>, index: number, key: string): string | undefined {
  const op = port.commits[index].commit.ops.find((o) => o.op === 'put' && o.key === key)
  return op && op.op === 'put' ? op.content : undefined
}

let cleanup: (() => void)[] = []
afterEach(() => {
  for (const fn of cleanup) fn()
  cleanup = []
})

describe('host.dispatch with fields', () => {
  it('writes the attached fields in the one commit that creates the change, and the hook sees them', async () => {
    // Scenario「dispatch 附上欄位建立變更」。
    const port = memoryPort()
    const asked: BeforeCommandInfo[] = []
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: (info) => {
          asked.push(info)
        },
      },
    })

    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'], fields: { source: 'portal' } })

    expect(port.calls.commit).toBe(1)
    expect(written(port, 0, 'changes/demo/.meta.yaml')).toContain('fields:\n  source: "portal"\n')
    expect(asked).toHaveLength(1)
    expect(asked[0].command).toBe('new-change')
    expect((asked[0].params as { fields: unknown }).fields).toEqual({ source: 'portal' })
  })

  it('wins over a --field of the same key', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    await host.dispatch({
      identity: ALICE,
      scope: 's1',
      argv: ['new', 'change', 'demo', '--field', 'source=model', '--field', 'ticket=PROJ-1'],
      fields: { source: 'portal' },
    })
    const shown = (await host.dispatch({ identity: ALICE, scope: 's1', argv: ['show', 'demo', '--json'] })) as {
      fields: unknown
    }
    expect(shown.fields).toEqual({ source: 'portal', ticket: 'PROJ-1' })
  })

  it('is ignored by a verb that creates nothing', async () => {
    // Scenario「每次呼叫都帶也不影響其他動詞」。
    const port = portWithDemo()
    const host = createHost({ store: port.store })
    const fields = { source: 'portal' }

    const listed = (await host.dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'], fields })) as {
      changes: { name: string }[]
    }
    expect(listed.changes.map((c) => c.name)).toEqual(['demo'])
    expect(port.calls.commit).toBe(0)

    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['task', 'done', '1', '--change', 'demo'], fields })
    expect(port.calls.commit).toBe(1)
    expect(written(port, 0, 'changes/demo/.meta.yaml') ?? '').not.toContain('fields')
  })

  it('rejects a value that is not a string with invalid_argv, before any store call', async () => {
    // Scenario「值不是字串」。
    const port = memoryPort()
    const host = createHost({ store: port.store })
    for (const fields of NOT_FIELDS) {
      const err = await rejection(
        host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'], fields: fields as never }),
      )
      expect(err.code).toBe('invalid_argv')
      expect(err.message).toContain('fields')
    }
    expect(port.calls.commit + port.calls.read + port.calls.list).toBe(0)
  })

  it('rejects attached fields outside the rules with invalid_argv and creates nothing', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    const err = await rejection(
      host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'], fields: { Bad: '1' } }),
    )
    expect(err.code).toBe('invalid_argv')
    expect(port.calls.commit).toBe(0)
  })

  it('treats null and undefined fields as none', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'a'], fields: undefined })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'b'], fields: null as never })
    expect(port.calls.commit).toBe(2)
    expect(written(port, 0, 'changes/a/.meta.yaml')).not.toContain('fields')
  })
})

describe('the field verbs through host.dispatch', () => {
  it('returns the object the CLI prints with --json', async () => {
    // Scenario「欄位動詞經 dispatch」。
    const port = portWithDemo()
    const host = createHost({ store: port.store })
    const call = (argv: string[]) => host.dispatch({ identity: ALICE, scope: 's1', argv })

    expect(await call(['change', 'field', 'set', 'demo', 'ticket=PROJ-1', '--json'])).toEqual({
      change: 'demo',
      fields: { ticket: 'PROJ-1' },
    })
    expect(port.commits[0].commit.command).toBe('change-fields-edit')
    expect(port.commits[0].commit.events).toEqual([
      { name: 'change-fields-changed', payload: { change: 'demo', keys: ['ticket'] } },
    ])
    // The value is the same with or without --json, as for every verb.
    expect(await call(['change', 'field', 'set', 'demo', 'query=a=b'])).toEqual({
      change: 'demo',
      fields: { query: 'a=b', ticket: 'PROJ-1' },
    })
    expect(await call(['change', 'field', 'unset', 'demo', 'ticket', 'ghost'])).toEqual({
      change: 'demo',
      fields: { query: 'a=b' },
    })
    expect(await call(['discuss', 'field', 'set', 'board-search', 'owner=alice'])).toEqual({
      discussion: 'board-search',
      fields: { owner: 'alice' },
    })
    const shown = (await call(['discuss', 'show', 'board-search', '--json'])) as { info: { fields: unknown } }
    expect(shown.info.fields).toEqual({ owner: 'alice' })
  })

  it('commits nothing when nothing changes, and refuses what breaks the rules', async () => {
    const port = portWithDemo()
    const host = createHost({ store: port.store })
    const call = (argv: string[]) => host.dispatch({ identity: ALICE, scope: 's1', argv })

    expect(await call(['change', 'field', 'unset', 'demo', 'ghost'])).toEqual({ change: 'demo', fields: {} })
    expect(port.calls.commit).toBe(0)
    for (const argv of [
      ['change', 'field', 'set', 'demo', 'Bad=1'],
      ['change', 'field', 'set', 'demo', 'ticket'],
      ['change', 'field', 'set', 'demo', 'a=1', 'a=2'],
    ]) {
      expect((await rejection(call(argv))).code).toBe('invalid_argv')
    }
    expect((await rejection(call(['change', 'field', 'set', 'ghost', 'a=1']))).code).toBe('not_found')
    expect(port.calls.commit).toBe(0)
  })
})

describe('host.handleRequest with fields', () => {
  it('attaches the fields of its third argument to a create request', async () => {
    // Scenario「handleRequest 的第三個參數」。
    const port = memoryPort()
    const host = createHost({ store: port.store, scopes: () => ONE_REPO })

    const created = await host.handleRequest(
      request('POST', 'changes', { name: 'demo', fields: { source: 'client', ticket: 'PROJ-1' } }),
      ALICE,
      { fields: { source: 'portal' } },
    )
    expect(created?.status).toBe(200)
    expect(port.calls.commit).toBe(1)
    const shown = await host.handleRequest(request('GET', 'changes/demo'), ALICE)
    expect(json(shown).fields).toEqual({ source: 'portal', ticket: 'PROJ-1' })
  })

  it('ignores them on a request that creates nothing, and works with no third argument', async () => {
    const port = portWithDemo()
    const host = createHost({ store: port.store, scopes: () => ONE_REPO })
    const listed = await host.handleRequest(request('GET', 'changes'), ALICE, { fields: { source: 'portal' } })
    expect(listed?.status).toBe(200)
    expect(json(listed).changes[0].fields).toBeUndefined()
    const written = await host.handleRequest(request('PUT', 'changes/demo/fields', { set: { ticket: 'PROJ-1' } }), ALICE)
    expect(json(written)).toEqual({ fields: { ticket: 'PROJ-1' }, changed: ['ticket'] })
    const handshake = await host.handleRequest(request('GET', 'binding'), ALICE, {})
    expect(json(handshake).capabilities.fields).toBe(true)
  })

  it('answers 400 when a value is not a string, with nothing written', async () => {
    // Scenario「值不是字串」。
    const port = memoryPort()
    const host = createHost({ store: port.store, scopes: () => ONE_REPO })
    for (const fields of NOT_FIELDS) {
      const response = await host.handleRequest(request('POST', 'changes', { name: 'demo' }), ALICE, {
        fields: fields as never,
      })
      expect(response?.status).toBe(400)
      expect(json(response).reason).toBe('invalid_argument')
    }
    expect(port.calls.commit).toBe(0)
  })

  it('answers 400 when the attached fields break the rules', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store, scopes: () => ONE_REPO })
    const response = await host.handleRequest(request('POST', 'changes', { name: 'demo' }), ALICE, {
      fields: { Bad: '1' },
    })
    expect(response?.status).toBe(400)
    expect(port.calls.commit).toBe(0)
  })
})

// node-sdk「createHost 的保留欄位前綴」(change reserved-field-prefixes).
describe('createHost with reservedFieldPrefixes', () => {
  const RESERVED = "field 'host.project' is reserved by the host"

  /** A host that keeps `host.` for itself, with hooks that count their calls. */
  function reservingHost() {
    const port = memoryPort()
    const seen = { before: 0, after: 0 }
    const host = createHost({
      store: port.store,
      scopes: () => ONE_REPO,
      reservedFieldPrefixes: ['host.'],
      hooks: {
        beforeCommand: () => {
          seen.before += 1
        },
        afterCommit: () => {
          seen.after += 1
        },
      },
    })
    return { port, host, seen }
  }

  it('keeps the reserved keys for the fields of the host', async () => {
    const { port, host, seen } = reservingHost()
    const call = (argv: string[], fields?: Record<string, string>) =>
      host.dispatch({ identity: ALICE, scope: 's1', argv, fields })

    // Scenario「dispatch 拒絕請求寫保留的鍵」與「beforeCommand 沒有被問」。
    const refused = await rejection(call(['new', 'change', 'spoof', '--field', 'host.project=p2']))
    expect([refused.code, refused.message]).toEqual(['refused', RESERVED])
    expect(port.calls).toEqual({ list: 0, read: 0, commit: 0 })
    expect(seen).toEqual({ before: 0, after: 0 })

    // Scenario「宿主的 fields 寫得了保留的鍵」。
    await call(['new', 'change', 'ok'], { 'host.project': 'p1' })
    const listed = (await call(['list', '--json'])) as { changes: { name: string; fields?: unknown }[] }
    expect(listed.changes.find((c) => c.name === 'ok')?.fields).toEqual({ 'host.project': 'p1' })

    // Scenario「設定與移除都被拒絕」：儲存與兩個掛鉤都沒有被呼叫。
    const calls = { ...port.calls }
    const hooks = { ...seen }
    for (const argv of [
      ['change', 'field', 'set', 'ok', 'host.project=p2'],
      ['change', 'field', 'unset', 'ok', 'host.project'],
    ]) {
      const err = await rejection(call(argv))
      expect([err.code, err.message]).toEqual(['refused', RESERVED])
    }
    expect(port.calls).toEqual(calls)
    expect(seen).toEqual(hooks)

    // Scenario「handleRequest 回 409」。
    const response = await host.handleRequest(
      request('PUT', 'changes/ok/fields', { set: { 'host.project': 'p2' } }),
      ALICE,
    )
    expect(response?.status).toBe(409)
    expect(json(response)).toMatchObject({ reason: 'refused', message: RESERVED })
    expect(port.calls).toEqual(calls)
    expect(seen).toEqual(hooks)
    const shown = (await call(['show', 'ok', '--json'])) as { fields: unknown }
    expect(shown.fields).toEqual({ 'host.project': 'p1' })
  })

  it('refuses the --field=key=value spelling on both verbs that take the flag', async () => {
    // 稽核：`--field=鍵=值` 與分開的兩個參數是同一個欄位。
    const { port, host, seen } = reservingHost()
    for (const argv of [
      ['new', 'change', 'spoof', '--field=host.project=p2'],
      ['discuss', 'new', '主題', '--field=team=web', '--field=host.project=p2'],
    ]) {
      const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv }))
      expect([err.code, err.message]).toEqual(['refused', RESERVED])
    }
    expect(port.calls).toEqual({ list: 0, read: 0, commit: 0 })
    expect(seen).toEqual({ before: 0, after: 0 })
  })

  it('reports the reserved key, not a field outside the rules in the same argv', async () => {
    // 「一次呼叫同時有保留的鍵與不合規則的欄位時，回報保留鍵的拒絕」，經 argv。
    const { port, host, seen } = reservingHost()
    const argv = ['new', 'change', 'demo', '--field', 'Bad=1', '--field', 'host.project=p2']
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv }))
    expect([err.code, err.message]).toEqual(['refused', RESERVED])
    expect(port.calls).toEqual({ list: 0, read: 0, commit: 0 })
    expect(seen).toEqual({ before: 0, after: 0 })
  })

  it('throws at once on a value that is not an array of field keys', () => {
    // Scenario「選項的值不合法」，加上 null 與有空洞的陣列。
    const port = memoryPort()
    // eslint-disable-next-line no-sparse-arrays
    for (const reservedFieldPrefixes of ['host.', ['host.', 7], [''], ['Host.'], null, [, 'host.']]) {
      expect(() => createHost({ store: port.store, reservedFieldPrefixes: reservedFieldPrefixes as never })).toThrow(
        /reservedFieldPrefixes/,
      )
    }
    expect(port.calls).toEqual({ list: 0, read: 0, commit: 0 })
  })

  it('throws on a value JSON would drop, which would leave no key reserved', () => {
    // JSON 會把函式與 symbol 整個丟掉：原生端看起來就是沒有給這個選項。
    const port = memoryPort()
    for (const reservedFieldPrefixes of [() => ['host.'], Symbol('host.')]) {
      expect(() => createHost({ store: port.store, reservedFieldPrefixes: reservedFieldPrefixes as never })).toThrow(
        'createHost: reservedFieldPrefixes must be an array of strings',
      )
    }
  })

  it('is as before when it is left out or empty', async () => {
    // Scenario「省略時行為不變」。
    for (const options of [{}, { reservedFieldPrefixes: [] }]) {
      const port = memoryPort()
      const host = createHost({ store: port.store, ...options })
      const argv = ['new', 'change', 'demo', '--field', 'host.project=p2']
      await host.dispatch({ identity: ALICE, scope: 's1', argv })
      const shown = (await host.dispatch({ identity: ALICE, scope: 's1', argv: ['show', 'demo', '--json'] })) as {
        fields: unknown
      }
      expect(shown.fields).toEqual({ 'host.project': 'p2' })
    }
  })
})

describe('engine.dispatch and the field verbs', () => {
  it('runs them on the fs form', async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-node-fields-')))
    cleanup.push(() => rmSync(root, { recursive: true, force: true }))
    const dir = join(root, 'speclink', 'changes', 'demo')
    mkdirSync(dir, { recursive: true })
    writeFileSync(join(dir, '.meta.yaml'), META)
    const engine = createEngine({ store: { type: 'fs', root } })

    expect(await engine.dispatch(['change', 'field', 'set', 'demo', 'ticket=PROJ-1', '--json'])).toEqual({
      change: 'demo',
      fields: { ticket: 'PROJ-1' },
    })
    expect(readFileSync(join(dir, '.meta.yaml'), 'utf8')).toBe(`${META}fields:\n  ticket: "PROJ-1"\n`)
    await engine.dispatch(['new', 'change', 'fresh', '--field', 'source=portal'])
    const shown = (await engine.dispatch(['show', 'fresh', '--json'])) as { fields: unknown }
    expect(shown.fields).toEqual({ source: 'portal' })
    await engine.dispatch(['discuss', 'new', 'Board search', '--slug', 'board-search', '--field', 'owner=alice'])
    expect(await engine.dispatch(['discuss', 'field', 'unset', 'board-search', 'owner'])).toEqual({
      discussion: 'board-search',
      fields: {},
    })
  })
})
