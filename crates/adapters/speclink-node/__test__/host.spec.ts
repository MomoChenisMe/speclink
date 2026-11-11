// node-sdk「createHost 與 host.dispatch」and「JS 版的儲存接口」: the host kit's
// Agent-tool door over a JS store port of three functions.
import { mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, describe, expect, it } from 'vitest'

import { createEngine, createHost, StoreConflictError } from '../index.js'
import { rejection } from './helpers'
import { memoryPort, type PortCommit } from './memory-port'

const ALICE = { id: 'u_1', displayName: 'Alice' }
const BOB = { id: 'u_2', displayName: 'Bob' }
const META = 'schema: spec-driven\n'
const PROPOSAL = '## Why\n\nA demo change for the host kit tests, long enough to be a summary.\n'
const OPEN_TASKS = '## 1. Work\n\n- [ ] 1.1 First task\n'
const DONE_TASKS = '## 1. Work\n\n- [x] 1.1 First task\n'
const CONFIG = 'schema: spec-driven\n'

/** A port with change `demo` in scope `s1`. */
function portWithDemo(tasks = OPEN_TASKS, options: Parameters<typeof memoryPort>[0] = {}) {
  const port = memoryPort(options)
  port.seed('s1', {
    'changes/demo/.meta.yaml': META,
    'changes/demo/proposal.md': PROPOSAL,
    'changes/demo/tasks.md': tasks,
    'config.yaml': CONFIG,
  })
  return port
}

function totalCalls(port: ReturnType<typeof memoryPort>) {
  return port.calls.list + port.calls.read + port.calls.commit
}

const isCamel = (key: string) => /^[a-z][a-zA-Z0-9]*$/.test(key)

let cleanup: (() => void)[] = []
afterEach(() => {
  for (const fn of cleanup) fn()
  cleanup = []
})

describe('createHost and host.dispatch', () => {
  it('creates and lists a change through the host kit', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })

    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] })
    const listed = (await host.dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'] })) as {
      changes: Record<string, unknown>[]
    }

    expect(listed.changes.map((c) => c.name)).toEqual(['demo'])
    const entry = listed.changes[0]
    expect(Object.keys(entry).every(isCamel)).toBe(true)
    expect(typeof entry.completedTasks).toBe('number')
    expect(typeof entry.totalTasks).toBe('number')
    expect(typeof entry.status).toBe('string')
    expect(port.calls.commit).toBe(1)
    expect(port.commits[0].commit.actor).toEqual({ id: 'u_1', displayName: 'Alice' })
  })

  it('gives the same result as engine.dispatch', async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-host-parity-')))
    cleanup.push(() => rmSync(root, { recursive: true, force: true }))
    const dir = join(root, 'speclink', 'changes', 'demo')
    mkdirSync(dir, { recursive: true })
    writeFileSync(join(dir, '.meta.yaml'), META)
    writeFileSync(join(dir, 'proposal.md'), PROPOSAL)
    writeFileSync(join(dir, 'tasks.md'), OPEN_TASKS)
    const argv = ['status', '--change', 'demo', '--json']

    const viaEngine = await createEngine({ store: { type: 'fs', root } }).dispatch(argv)
    const viaHost = await createHost({ store: portWithDemo().store }).dispatch({ identity: ALICE, scope: 's1', argv })

    expect(viaHost).toEqual(viaEngine)
  })

  it('answers status with no change at all with the notice, as engine.dispatch does', async () => {
    const result = await createHost({ store: memoryPort().store }).dispatch({
      identity: ALICE,
      scope: 's1',
      argv: ['status'],
    })
    expect(result).toEqual({ output: 'No active changes. Create one with: speclink new change <name>\n' })
  })

  it('serves two identities with one host', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'by-alice'] })
    await host.dispatch({ identity: BOB, scope: 's1', argv: ['new', 'change', 'by-bob'] })
    expect(port.doc('s1', 'changes/by-alice/.meta.yaml')?.content).toContain('created_by: Alice\n')
    expect(port.doc('s1', 'changes/by-bob/.meta.yaml')?.content).toContain('created_by: Bob\n')
  })

  it('runs several calls at once', async () => {
    const port = memoryPort({ async: true })
    const host = createHost({ store: port.store })
    await Promise.all(
      ['a', 'b', 'c'].map((name) =>
        host.dispatch({ identity: ALICE, scope: `scope-${name}`, argv: ['new', 'change', `change-${name}`] }),
      ),
    )
    expect(port.calls.commit).toBe(3)
  })

  it('fails at once when a store function is missing', () => {
    const { list, read } = memoryPort().store
    expect(() => createHost({ store: { list, read } as never })).toThrow(/commit/)
  })

  it('rejects with refused when beforeCommand denies', async () => {
    const port = portWithDemo(DONE_TASKS)
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: (info) => (info.command === 'archive' ? { allow: false, reason: '程式碼還沒合併' } : undefined),
      },
    })
    const before = totalCalls(port)
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['archive', 'demo'] }))
    expect(err.code).toBe('refused')
    expect(err.message).toBe('程式碼還沒合併')
    expect(totalCalls(port)).toBe(before)
  })

  it('refuses a hook name it does not know', () => {
    const port = memoryPort()
    expect(() => createHost({ store: port.store, hooks: { beforecommand: () => undefined } as never })).toThrow(
      /unknown hooks: beforecommand/,
    )
  })

  it('waits for an async beforeCommand', async () => {
    const port = memoryPort()
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: () => new Promise((resolve) => setTimeout(() => resolve({ allow: true }), 20)),
      },
    })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] })
    expect(port.calls.commit).toBe(1)
  })

  it('does not allow when beforeCommand throws', async () => {
    const port = memoryPort()
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: () => {
          throw new Error('policy service down')
        },
      },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] }))
    expect(err.code).toBe('internal')
    expect(err.message).toContain('beforeCommand')
    expect(port.calls.commit).toBe(0)
  })

  it('does not allow an answer it does not know', async () => {
    const port = memoryPort()
    const host = createHost({
      store: port.store,
      hooks: { beforeCommand: () => ({ allow: 'yes' }) as never },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] }))
    expect(err.code).toBe('internal')
    expect(err.message).toContain('beforeCommand')
    expect(port.calls.commit).toBe(0)
  })

  it('gives beforeCommand the changed keys of a workflow-config write', async () => {
    const asked: { command: string; changedKeys?: string[]; params: unknown }[] = []
    const host = createHost({
      store: portWithDemo().store,
      hooks: {
        beforeCommand: (info) => {
          asked.push({ command: info.command, changedKeys: info.changedKeys, params: info.params })
        },
      },
    })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['workflow-config', 'set', 'tdd', 'true'] })
    expect(asked).toHaveLength(1)
    expect(asked[0].command).toBe('workflow-config-edit')
    expect(asked[0].changedKeys).toEqual(['tdd'])
  })

  it('calls afterCommit with the events, and its failure changes nothing', async () => {
    const rejections: unknown[] = []
    const onRejection = (reason: unknown) => rejections.push(reason)
    process.on('unhandledRejection', onRejection)
    cleanup.push(() => process.off('unhandledRejection', onRejection))
    const watched: { command: string; scopeVersion: number; events: { name: string; payload: unknown }[] }[] = []
    const port = memoryPort()
    const host = createHost({
      store: port.store,
      hooks: {
        afterCommit: (info) => {
          watched.push({ command: info.command, scopeVersion: info.scopeVersion, events: info.events })
          throw new Error('the notifier is down')
        },
      },
    })

    const created = await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] })
    await new Promise((resolve) => setTimeout(resolve, 50))

    expect(created).toBeDefined()
    expect(watched).toHaveLength(1)
    expect(watched[0].command).toBe('new-change')
    expect(typeof watched[0].scopeVersion).toBe('number')
    expect(watched[0].events.map((e) => e.name)).toEqual(['change-created'])
    expect(rejections).toEqual([])
  })

  it('does not wait for afterCommit, and swallows its rejected Promise', async () => {
    const rejections: unknown[] = []
    const onRejection = (reason: unknown) => rejections.push(reason)
    process.on('unhandledRejection', onRejection)
    cleanup.push(() => process.off('unhandledRejection', onRejection))
    let settled = false
    const host = createHost({
      store: memoryPort().store,
      hooks: {
        afterCommit: () =>
          new Promise((_, reject) =>
            setTimeout(() => {
              settled = true
              reject(new Error('late failure'))
            }, 100),
          ),
      },
    })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] })
    expect(settled).toBe(false)
    await new Promise((resolve) => setTimeout(resolve, 200))
    expect(settled).toBe(true)
    expect(rejections).toEqual([])
  })

  it('reports a store version conflict as revision_conflict', async () => {
    const port = memoryPort()
    const host = createHost({
      store: {
        ...port.store,
        commit: (_scope: string, commit: PortCommit) => {
          throw new StoreConflictError(commit.ops[0].key, 9)
        },
      },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] }))
    expect(err.code).toBe('revision_conflict')
    expect(err.message).toContain('changes/demo/.meta.yaml')
  })

  it('reports an unavailable store as unavailable', async () => {
    const port = portWithDemo()
    const host = createHost({
      store: {
        ...port.store,
        read: () => {
          throw Object.assign(new Error('database offline'), { code: 'unavailable' })
        },
      },
    })
    const err = await rejection(
      host.dispatch({ identity: ALICE, scope: 's1', argv: ['task', 'done', '1', '--change', 'demo'] }),
    )
    expect(err.code).toBe('unavailable')
    expect(port.calls.commit).toBe(0)
  })

  it('refuses a verb that needs a checkout', async () => {
    const port = portWithDemo()
    const host = createHost({ store: port.store })
    const before = totalCalls(port)
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['drift', 'demo'] }))
    expect(err.code).toBe('invalid_argv')
    expect(totalCalls(port)).toBe(before)
  })

  it('refuses an empty identity or scope', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    for (const call of [
      { identity: { id: '', displayName: 'Alice' }, scope: 's1' },
      { identity: { id: 'u_1', displayName: '  ' }, scope: 's1' },
      { identity: ALICE, scope: '' },
      { identity: undefined as never, scope: 's1' },
    ]) {
      const err = await rejection(host.dispatch({ ...call, argv: ['list', '--json'] }))
      expect(err.code, JSON.stringify(call)).toBe('invalid_argv')
    }
    expect(totalCalls(port)).toBe(0)
  })

  it('keeps argv and stdin out of the identity', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo', '--agent', 'Mallory'] })
    expect(port.commits[0].commit.actor).toEqual(ALICE)
    expect(port.doc('s1', 'changes/demo/.meta.yaml')?.content).toContain('created_by: Alice\n')
  })

  it('stamps with the three functions only, in one commit', async () => {
    const port = portWithDemo(DONE_TASKS)
    const host = createHost({ store: port.store })
    await host.dispatch({
      identity: ALICE,
      scope: 's1',
      argv: ['review', 'add-round', 'demo', '--stdin'],
      stdin: '**Scope**: src/lib.rs\n\n- [SUGGESTION] src/lib.rs — rename helper\n',
    })
    const before = port.calls.commit

    await host.dispatch({
      identity: ALICE,
      scope: 's1',
      argv: ['review', 'stamp', 'demo', '--stdin'],
      stdin: JSON.stringify({ scope: [{ path: 'src/lib.rs', hash: '0f9c' }] }),
    })

    expect(port.calls.commit).toBe(before + 1)
    const ops = port.commits.at(-1)!.commit.ops
    expect(ops.some((op) => op.op === 'put' && op.key === 'changes/demo/.meta.yaml')).toBe(true)
    expect(ops.some((op) => op.op === 'delete' && op.key === 'changes/demo/review.md')).toBe(true)
    expect(port.doc('s1', 'changes/demo/.meta.yaml')?.content).toContain('reviewed_by: Alice\n')
  })
})

describe('the JS store port', () => {
  it('commits a write verb once, with its events', async () => {
    const port = portWithDemo()
    const host = createHost({ store: port.store })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['task', 'done', '1', '--change', 'demo'] })
    expect(port.calls.commit).toBe(1)
    const { commit } = port.commits[0]
    expect(commit.command).toBe('task-done')
    expect(commit.ops.some((op) => op.op === 'put' && op.key === 'changes/demo/tasks.md')).toBe(true)
    expect(commit.events.map((e) => e.name)).toEqual(['task-completed'])
  })

  it('expects null for a new document', async () => {
    const port = memoryPort()
    await createHost({ store: port.store }).dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] })
    const meta = port.commits[0].commit.ops.find((op) => op.key === 'changes/demo/.meta.yaml')
    expect(meta).toMatchObject({ op: 'put', expected: null })
  })

  it('expects the version it read for an existing document', async () => {
    const port = memoryPort()
    port.seed(
      's1',
      { 'changes/demo/.meta.yaml': META, 'changes/demo/proposal.md': PROPOSAL, 'changes/demo/tasks.md': OPEN_TASKS },
      { version: 7 },
    )
    await createHost({ store: port.store }).dispatch({
      identity: ALICE,
      scope: 's1',
      argv: ['task', 'done', '1', '--change', 'demo'],
    })
    const tasks = port.commits[0].commit.ops.find((op) => op.key === 'changes/demo/tasks.md')
    expect(tasks).toMatchObject({ op: 'put', expected: 7 })
  })

  it('does not commit for a query', async () => {
    const port = portWithDemo()
    await createHost({ store: port.store }).dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'] })
    expect(port.calls.commit).toBe(0)
  })

  it('fails when read returns the wrong number of results', async () => {
    const port = portWithDemo()
    const host = createHost({
      store: {
        ...port.store,
        read: async (scope: string, keys: string[]) => ((await port.store.read(scope, keys)) as unknown[]).slice(1),
      },
    })
    const err = await rejection(
      host.dispatch({ identity: ALICE, scope: 's1', argv: ['task', 'done', '1', '--change', 'demo'] }),
    )
    expect(err.code).toBe('internal')
    expect(err.message).toContain('read')
    expect(err.message).toContain('length')
    expect(port.calls.commit).toBe(0)
  })

  it('fails when a version is not an integer', async () => {
    const port = portWithDemo()
    const host = createHost({
      store: { ...port.store, list: () => ({ version: 1.5, entries: [] }) },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'] }))
    expect(err.code).toBe('internal')
    expect(err.message).toContain('list')
    expect(err.message).toContain('version')
  })

  it('refuses a version past 2^53 − 1 instead of rounding it', async () => {
    for (const version of [Number.MAX_SAFE_INTEGER + 1, -1]) {
      const host = createHost({
        store: { ...portWithDemo().store, list: () => ({ version, entries: [] }) },
      })
      const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'] }))
      expect(err.code, String(version)).toBe('internal')
      expect(err.message).toContain('version')
    }
  })

  it('reads a conflict from a plain object', async () => {
    const port = memoryPort()
    const host = createHost({
      store: {
        ...port.store,
        commit: (_scope: string, commit: PortCommit) => {
          throw { code: 'conflict', key: commit.ops[0].key, actual: 9 }
        },
      },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] }))
    expect(err.code).toBe('revision_conflict')
    expect(err.message).toContain('changes/demo/.meta.yaml')
    expect(err.message).toContain('9')
  })

  it('reads a conflict on a key outside the commit as another failure', async () => {
    const port = memoryPort()
    const host = createHost({
      store: {
        ...port.store,
        commit: () => {
          throw { code: 'conflict', key: 'changes/other/.meta.yaml', actual: 1 }
        },
      },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] }))
    expect(err.code).toBe('internal')
  })
})
