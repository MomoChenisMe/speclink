// node-sdk「createHost 與 host.dispatch」and「JS 版的儲存接口」: the host kit's
// Agent-tool door over a JS store port of three functions.
import { mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, describe, expect, it } from 'vitest'

import { createEngine, createHost, StoreConflictError } from '../index.js'
import { cliJson, rejection } from './helpers'
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

  // command-runtime「evidence show 經命令層且各入口一致」、verify-evidence「經 SDK 讀取」
  it('reads the evidence of a change the same way as engine.dispatch, and commits nothing', async () => {
    const evidence = JSON.stringify({
      version: 2,
      change: 'demo',
      entries: [
        { taskId: 'tsk_11', taskDesc: '1.1 First', touchedFiles: ['src/b.rs', 'src/a.rs'], recordedAt: '2026-11-12T03:00:00Z' },
        {
          taskId: 'tsk_12',
          taskDesc: '1.2 Second',
          actor: 'Alice',
          touchedFiles: ['src/b.rs', 'src/c.rs'],
          recordedAt: '2026-11-12T04:00:00Z',
        },
      ],
    })
    const archived = JSON.stringify({
      version: 2,
      change: 'old',
      entries: [{ taskId: 'tsk_o1', taskDesc: '1.1 Old', touchedFiles: ['src/old.rs'], recordedAt: '2026-11-10T00:00:00Z' }],
    })
    const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-host-evidence-')))
    cleanup.push(() => rmSync(root, { recursive: true, force: true }))
    const demoDir = join(root, 'speclink', 'changes', 'demo')
    const oldDir = join(root, 'speclink', 'changes', 'archive', '2026-11-12-old')
    mkdirSync(demoDir, { recursive: true })
    mkdirSync(oldDir, { recursive: true })
    writeFileSync(join(demoDir, '.meta.yaml'), META)
    writeFileSync(join(demoDir, '.evidence.json'), evidence)
    writeFileSync(join(oldDir, '.meta.yaml'), META)
    writeFileSync(join(oldDir, '.evidence.json'), archived)
    const port = memoryPort()
    port.seed('s1', {
      'changes/demo/.meta.yaml': META,
      'changes/demo/.evidence.json': evidence,
      'archive/changes/2026-11-12-old/.meta.yaml': META,
      'archive/changes/2026-11-12-old/.evidence.json': archived,
      'config.yaml': CONFIG,
    })
    const engine = createEngine({ store: { type: 'fs', root } })
    const host = createHost({ store: port.store })
    const viaHost = (argv: string[]) => host.dispatch({ identity: ALICE, scope: 's1', argv })

    const demo = await viaHost(['evidence', 'show', 'demo', '--json'])
    expect(demo).toEqual(await engine.dispatch(['evidence', 'show', 'demo', '--json']))
    // The CLI's own `--json` on the same files is the third entry point.
    expect(demo).toEqual(cliJson(root, ['evidence', 'show', 'demo', '--json']))
    expect(demo).toMatchObject({ change: 'demo', files: ['src/a.rs', 'src/b.rs', 'src/c.rs'] })
    expect(demo).not.toHaveProperty('archivedAs')
    const entries = (demo as { entries: Record<string, unknown>[] }).entries
    expect(entries.map((e) => e.taskId)).toEqual(['tsk_11', 'tsk_12'])
    expect(entries.every((e) => Object.keys(e).every(isCamel))).toBe(true)
    expect(Object.keys(entries[0]).sort()).toEqual(['recordedAt', 'taskDesc', 'taskId', 'touchedFiles'])
    expect(entries[1].actor).toBe('Alice')

    // An archived change, by its own name and by its archived name.
    for (const name of ['old', '2026-11-12-old']) {
      const old = await viaHost(['evidence', 'show', name, '--json'])
      expect(old).toEqual(await engine.dispatch(['evidence', 'show', name, '--json']))
      expect(old).toEqual(cliJson(root, ['evidence', 'show', name, '--json']))
      expect(old).toMatchObject({ change: 'old', archivedAs: '2026-11-12-old', files: ['src/old.rs'] })
    }

    // The text form is the same too.
    expect(await viaHost(['evidence', 'show', 'demo'])).toEqual(await engine.dispatch(['evidence', 'show', 'demo']))
    expect(port.calls.commit).toBe(0)
  })

  it('rejects evidence show of a name that is no change with not_found, as engine.dispatch does', async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-host-evidence-ghost-')))
    cleanup.push(() => rmSync(root, { recursive: true, force: true }))
    mkdirSync(join(root, 'speclink', 'changes'), { recursive: true })
    const argv = ['evidence', 'show', 'ghost', '--json']
    const port = portWithDemo()

    const viaEngine = await rejection(createEngine({ store: { type: 'fs', root } }).dispatch(argv))
    const viaHost = await rejection(createHost({ store: port.store }).dispatch({ identity: ALICE, scope: 's1', argv }))

    expect(viaEngine.code).toBe('not_found')
    expect(viaHost.code).toBe('not_found')
    expect(viaHost.message).toBe(viaEngine.message)
    expect(viaHost.message).toBe("Change 'ghost' not found.")
    expect(port.calls.commit).toBe(0)
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

  it('refuses an operator given at creation', () => {
    const port = memoryPort()
    expect(() => createHost({ store: port.store, actor: 'Alice' } as never)).toThrow(/actor/)
  })

  it('does not allow a null answer', async () => {
    const port = memoryPort()
    const host = createHost({ store: port.store, hooks: { beforeCommand: () => null as never } })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] }))
    expect(err.code).toBe('internal')
    expect(err.message).toContain('beforeCommand')
    expect(port.calls.commit).toBe(0)
  })

  it('does not allow when an async beforeCommand rejects', async () => {
    const port = memoryPort()
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async () => {
          throw new Error('policy service down')
        },
      },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] }))
    expect(err.code).toBe('internal')
    expect(err.message).toContain('beforeCommand')
    expect(port.calls.commit).toBe(0)
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

  it('ignores what afterCommit returns, a cyclic object included', async () => {
    const cyclic: Record<string, unknown> = {}
    cyclic.self = cyclic
    const host = createHost({ store: memoryPort().store, hooks: { afterCommit: () => cyclic as never } })
    const created = await host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] })
    await new Promise((resolve) => setTimeout(resolve, 50))
    expect(created).toBeDefined()
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

  it('keeps stdin out of the identity', async () => {
    const port = portWithDemo()
    const host = createHost({ store: port.store })
    await host.dispatch({
      identity: ALICE,
      scope: 's1',
      argv: ['new', 'artifact', 'design', '--change', 'demo', '--stdin'],
      stdin: '## Context\n\nidentity: { id: "u_evil", displayName: "Mallory" }\n',
    })
    expect(port.commits.at(-1)!.commit.actor).toEqual(ALICE)
  })

  it('judges the owner of a claim by the identifier, not the name', async () => {
    const port = portWithDemo()
    const host = createHost({ store: port.store })
    await host.dispatch({ identity: { id: 'u_1', displayName: 'Alice' }, scope: 's1', argv: ['claim', 'demo'] })
    const commits = port.calls.commit

    const err = await rejection(
      host.dispatch({ identity: { id: 'u_2', displayName: 'Alice' }, scope: 's1', argv: ['claim', 'demo'] }),
    )
    expect(err.code).toBe('refused')
    await host.dispatch({ identity: { id: 'u_1', displayName: 'Alice Chen' }, scope: 's1', argv: ['claim', 'demo'] })

    expect(port.calls.commit).toBe(commits)
    expect(port.doc('s1', 'changes/demo/.meta.yaml')?.content).toContain('claimed_by_id: u_1\n')
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

  it('refuses a tiny fraction instead of rounding it to 0', async () => {
    const host = createHost({
      store: { ...portWithDemo().store, list: () => ({ version: 1e-20, entries: [] }) },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'] }))
    expect(err.code).toBe('internal')
    expect(err.message).toContain('version')
  })

  it('reads -0 as 0, which it equals', async () => {
    const host = createHost({
      store: { ...portWithDemo().store, list: () => ({ version: -0, entries: [] }) },
    })
    const listed = (await host.dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'] })) as {
      changes: unknown[]
    }
    expect(listed.changes).toEqual([])
  })

  it('refuses a listed key outside the prefix or out of byte order', async () => {
    const entry = (key: string) => ({ key, version: 1, modifiedAt: 1 })
    for (const entries of [
      [entry('discussions/a.md')],
      [entry('changes/b/.meta.yaml'), entry('changes/a/.meta.yaml')],
      [entry('changes/a/.meta.yaml'), entry('changes/a/.meta.yaml')],
    ]) {
      const port = portWithDemo()
      const host = createHost({
        store: {
          ...port.store,
          list: (scope: string, prefix: string) =>
            prefix === 'changes/' ? { version: 1, entries } : port.store.list(scope, prefix),
        },
      })
      const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'] }))
      expect(err.code, JSON.stringify(entries)).toBe('internal')
      expect(err.message).toContain('list')
    }
  })

  it('refuses a cyclic value from a store function', async () => {
    const port = portWithDemo()
    const host = createHost({
      store: {
        ...port.store,
        read: async (scope: string, keys: string[]) => {
          const docs = (await port.store.read(scope, keys)) as Record<string, unknown>[]
          return docs.map((doc) => (doc ? Object.assign(doc, { self: doc }) : doc))
        },
      },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'] }))
    expect(err.code).toBe('internal')
    expect(err.message).toContain('read')
  })

  it('refuses a conflict whose actual is not a number or null', async () => {
    for (const actual of ['7', Number.NaN, Number.POSITIVE_INFINITY, 1e-20, { version: 7 }]) {
      const port = memoryPort()
      const host = createHost({
        store: {
          ...port.store,
          commit: (_scope: string, commit: PortCommit) => {
            throw { code: 'conflict', key: commit.ops[0].key, actual }
          },
        },
      })
      const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['new', 'change', 'demo'] }))
      expect(err.code, String(actual)).toBe('internal')
      expect(err.message).toContain('actual')
    }
  })

  it('settles even when the thrown value cannot be read', async () => {
    const host = createHost({
      store: {
        ...memoryPort().store,
        list: () => {
          throw Object.create(null)
        },
      },
    })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['list', '--json'] }))
    expect(err.code).toBe('internal')
  }, 5_000)
})
