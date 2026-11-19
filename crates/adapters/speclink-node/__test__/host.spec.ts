// node-sdk「createHost 與 host.dispatch」and「JS 版的儲存接口」: the host kit's
// Agent-tool door over a JS store port of three functions.
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { createEngine, createHost, StoreConflictError } from '../index.js'
import { cliJson, rejection, repoRoot } from './helpers'
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

  it('releases a claim for anyone, and show --json names the holder only while the change is held', async () => {
    // change-lifecycle「釋放認領」「認領記錄的讀取」：dispatch 經共用的解析與輸出取得新旗標。
    const port = portWithDemo()
    const host = createHost({ store: port.store })
    const as = (identity: typeof ALICE, argv: string[]) =>
      host.dispatch({ identity, scope: 's1', argv }) as Promise<Record<string, unknown>>

    expect(await as(ALICE, ['claim', 'demo'])).toEqual({ change: 'demo', claimedBy: 'Alice' })
    const held = await as(BOB, ['show', 'demo', '--json'])
    expect(held.claimed).toMatchObject({ by: 'Alice', byId: 'u_1' })
    expect(Number.isNaN(Date.parse((held.claimed as { at: string }).at))).toBe(false)

    // Bob is not the holder: the engine does not restrict who releases.
    expect(await as(BOB, ['claim', 'demo', '--release'])).toEqual({
      change: 'demo',
      released: true,
      previousHolder: 'Alice',
    })
    expect(await as(BOB, ['show', 'demo', '--json'])).not.toHaveProperty('claimed')
    expect(port.doc('s1', 'changes/demo/.meta.yaml')?.content).toBe(META)

    // Nothing to release: a success that writes nothing.
    const commits = port.calls.commit
    expect(await as(BOB, ['claim', 'demo', '--release', '--json'])).toEqual({ change: 'demo', released: false })
    expect(port.calls.commit).toBe(commits)
    expect(await as(BOB, ['claim', 'demo'])).toEqual({ change: 'demo', claimedBy: 'Bob' })
  })

  it('lets beforeCommand keep a release to the holder, under its own command name', async () => {
    // The hook of docs/verb-contract.md: `claim-release` is not `claim`, and
    // the holder is read from `show --json`.
    const port = portWithDemo()
    const asked: { command: string; params: unknown }[] = []
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async ({ identity, command, params }, { read }) => {
          asked.push({ command, params })
          if (command !== 'claim-release') return
          let change: { claimed?: { by: string; byId?: string } }
          try {
            change = (await read(['show', (params as { name: string }).name, '--json'])) as typeof change
          } catch {
            return // The release reports an unknown change or broken data itself.
          }
          const holder = change.claimed
          if (!holder) return
          // A claim with no identifier is judged by the name, as the engine does.
          const isHolder = holder.byId ? holder.byId === identity.id : holder.by === identity.displayName
          if (!isHolder) return { allow: false, reason: '只有認領的人可以釋放' }
        },
      },
    })
    const release = (identity: typeof ALICE) =>
      host.dispatch({ identity, scope: 's1', argv: ['claim', 'demo', '--release'] })
    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['claim', 'demo'] })
    const meta = port.doc('s1', 'changes/demo/.meta.yaml')?.content

    const err = await rejection(release(BOB))

    expect(err.code).toBe('refused')
    expect(err.message).toBe('只有認領的人可以釋放')
    expect(asked.at(-1)).toEqual({ command: 'claim-release', params: { name: 'demo' } })
    expect(port.doc('s1', 'changes/demo/.meta.yaml')?.content).toBe(meta)
    expect(await release(ALICE)).toEqual({ change: 'demo', released: true, previousHolder: 'Alice' })

    // A claim that recorded no identifier: the holder is judged by the name,
    // so the holder is not locked out of the release.
    port.seed('s1', { 'changes/demo/.meta.yaml': `${META}claimed_at: 2026-11-18T10:00:00+00:00\nclaimed_by: Alice\n` })
    expect((await rejection(release(BOB))).code).toBe('refused')
    expect(await release(ALICE)).toEqual({ change: 'demo', released: true, previousHolder: 'Alice' })

    // A change the hook cannot read: the release itself reports it.
    const ghost = await rejection(host.dispatch({ identity: BOB, scope: 's1', argv: ['claim', 'ghost', '--release'] }))
    expect(ghost.code).toBe('not_found')
    expect(ghost.message).toContain('ghost')
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

// node-sdk「beforeCommand 的讀取函式」: the second parameter of the hook.
describe('the read function of beforeCommand', () => {
  type Read = (argv: string[]) => Promise<unknown>
  type Shown = { name: string; review?: { state: string } }
  const LIST = { identity: ALICE, scope: 's1', argv: ['list', '--json'] }

  it('reads the state of a change and refuses its archive', async () => {
    const port = portWithDemo(DONE_TASKS)
    let asked = 0
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async (info, { read }) => {
          asked += 1
          if (info.command !== 'archive') return
          const shown = (await read(['show', info.params.change as string, '--json'])) as Shown
          if (shown.review?.state !== 'stamped') return { allow: false, reason: '還沒通過審查' }
        },
      },
    })

    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['archive', 'demo'] }))

    expect(err.code).toBe('refused')
    expect(err.message).toBe('還沒通過審查')
    expect(port.calls.commit).toBe(0)
    expect(asked).toBe(1)
  })

  // The archive hook of docs/sdk-node: an archive without a name and an
  // archive of a change that does not exist do not end as `internal`.
  it('lets the archive hook of the docs give a clear answer without a name and for an unknown change', async () => {
    const port = portWithDemo(DONE_TASKS)
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async ({ command, params }, { read }) => {
          if (command !== 'archive') return
          if (typeof params.change !== 'string') return { allow: false, reason: 'Give the name of the change to archive' }
          const shown = (await read(['show', params.change, '--json']).catch((err) => {
            if (err.code === 'not_found') return null
            throw err
          })) as Shown | null
          if (shown === null) return
          if (shown.review?.state !== 'stamped') return { allow: false, reason: 'The change did not pass the review' }
        },
      },
    })
    const archive = (...name: string[]) => rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['archive', ...name] }))

    const unnamed = await archive()
    const unknown = await archive('ghost')
    const unreviewed = await archive('demo')

    expect([unnamed.code, unnamed.message]).toEqual(['refused', 'Give the name of the change to archive'])
    expect(unknown.code).toBe('not_found')
    expect([unreviewed.code, unreviewed.message]).toEqual(['refused', 'The change did not pass the review'])
    expect(port.calls.commit).toBe(0)
  })

  it('gives the value host.dispatch gives for the same argv', async () => {
    const argv = ['show', 'demo', '--json']
    let viaRead: unknown
    const host = createHost({
      store: portWithDemo().store,
      hooks: {
        beforeCommand: async (info, { read }) => {
          if (info.command === 'list') viaRead = await read(argv)
        },
      },
    })

    await host.dispatch(LIST)
    const direct = await host.dispatch({ identity: ALICE, scope: 's1', argv })

    expect(viaRead).toEqual(direct)
    expect(viaRead).toMatchObject({ name: 'demo' })
  })

  it('refuses a verb that writes, before the store gets a call', async () => {
    const port = portWithDemo(DONE_TASKS)
    const refused: { verb: string; code?: string; message: string; storeCalls: number }[] = []
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async (_info, { read }) => {
          // The task of `task done` is checked already: the verb decides, not this result.
          for (const argv of [['new', 'change', 'other'], ['task', 'done', '1', '--change', 'demo'], ['archive', 'demo'], ['claim', 'demo']]) {
            const before = totalCalls(port)
            const err = await rejection(read(argv))
            refused.push({ verb: argv[0], code: err.code, message: err.message, storeCalls: totalCalls(port) - before })
          }
        },
      },
    })

    await host.dispatch(LIST)

    expect(refused.map((r) => r.verb)).toEqual(['new', 'task', 'archive', 'claim'])
    for (const r of refused) expect(r, r.verb).toMatchObject({ code: 'invalid_argv', storeCalls: 0 })
    expect(refused[0].message).toBe("read accepts only verbs that write nothing; 'new change' is not one of them")
    expect(refused[1].message).toContain("'task done'")
    expect(port.doc('s1', 'changes/other/.meta.yaml')).toBeUndefined()
    expect(port.calls.commit).toBe(0)
  })

  it('names a verb that takes content as a verb that writes, not as one that lacks its content', async () => {
    const port = portWithDemo()
    const messages: string[] = []
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async (_info, { read }) => {
          for (const argv of [
            ['workflow-config', 'context', '--stdin'],
            ['new', 'artifact', 'proposal', '--change', 'demo', '--stdin'],
            ['discuss', 'add-round', 'talk'],
            // A stamp's content is JSON: it converts with no placeholder content.
            ['review', 'stamp', 'demo', '--stdin'],
            ['verify', 'stamp', 'demo', '--stdin'],
          ]) {
            const err = await rejection(read(argv))
            messages.push(`${err.code}: ${err.message}`)
          }
        },
      },
    })

    await host.dispatch(LIST)

    const refusal = (verb: string) => `invalid_argv: read accepts only verbs that write nothing; '${verb}' is not one of them`
    expect(messages).toEqual(
      ['workflow-config context', 'new artifact', 'discuss add-round', 'review stamp', 'verify stamp'].map(refusal),
    )
    expect(port.calls.commit).toBe(0)
  })

  it('rejects as host.dispatch does when the change does not exist, and the call goes on', async () => {
    const argv = ['show', 'ghost', '--json']
    let caught: (Error & { code?: string }) | undefined
    const host = createHost({
      store: portWithDemo().store,
      hooks: {
        beforeCommand: async (info, { read }) => {
          if (info.command === 'list') caught = await rejection(read(argv))
        },
      },
    })

    const listed = (await host.dispatch(LIST)) as { changes: { name: string }[] }
    const direct = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv }))

    expect(listed.changes.map((c) => c.name)).toEqual(['demo'])
    expect(caught?.code).toBe('not_found')
    expect(caught?.message).toBe(direct.message)
  })

  it('ends the call as internal when the hook lets a read failure out', async () => {
    const port = portWithDemo()
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async (_info, { read }) => {
          await read(['show', 'ghost', '--json'])
        },
      },
    })

    const err = await rejection(host.dispatch(LIST))

    expect(err.code).toBe('internal')
    expect(err.message).toContain('beforeCommand')
    expect(port.calls.commit).toBe(0)
  })

  it('stops working when the hook has answered', async () => {
    const port = portWithDemo()
    let kept: Read | undefined
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: (_info, { read }) => {
          kept = read
        },
      },
    })

    await host.dispatch(LIST)
    const before = totalCalls(port)
    const err = await rejection(kept!(['list', '--json']))

    expect(err.code).toBe('invalid_argv')
    expect(totalCalls(port)).toBe(before)
  })

  it('reads as the caller and on the scope of the call, whatever the hook does to its info', async () => {
    const port = portWithDemo()
    port.seed('s2', { 'changes/other/.meta.yaml': META })
    let listed: { changes: { name: string }[] } | undefined
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async (info, { read }) => {
          info.scope = 's2'
          info.identity = BOB
          listed = (await read(['list', '--json'])) as typeof listed
        },
      },
    })

    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['show', 'demo', '--json'] })

    expect(listed?.changes.map((c) => c.name)).toEqual(['demo'])
  })

  it('reads in the hook of a workflow-config write too', async () => {
    const port = portWithDemo()
    let shown: unknown
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async (info, { read }) => {
          if (info.command !== 'workflow-config-edit') return
          shown = await read(['workflow-config', 'show', '--json'])
          return { allow: false, reason: 'frozen' }
        },
      },
    })

    const err = await rejection(
      host.dispatch({ identity: ALICE, scope: 's1', argv: ['workflow-config', 'set', 'tdd', 'true'] }),
    )

    expect(err.code).toBe('refused')
    expect(shown).toMatchObject({ schema: 'spec-driven' })
    expect(port.calls.commit).toBe(0)
  })

  // The verbs `read` takes are written out in three texts. A read kind added
  // to `is_read`, or a text that falls behind, fails here.
  it('names the same read verbs in the typings, in both docs and in is_read', () => {
    const between = (file: string, from: string, to: string) => {
      const text = readFileSync(join(repoRoot, file), 'utf8')
      const start = text.indexOf(from)
      const end = text.indexOf(to, start)
      expect(start, `${file}: "${from}"`).toBeGreaterThan(-1)
      expect(end, `${file}: "${to}"`).toBeGreaterThan(start)
      return text.slice(start, end)
    }
    const quoted = (text: string) => [...text.matchAll(/`([^`]+)`/g)].map((match) => match[1]).sort()
    const typings = quoted(between('crates/adapters/speclink-node/index.d.ts', 'The verbs are', 'Any'))
    const english = quoted(between('docs/sdk-node.md', 'It accepts only the verbs that write nothing:', 'All other verbs'))
    const chinese = quoted(between('docs/sdk-node.zh-TW.md', '它只接受不寫入任何文件的動詞：', '其他動詞'))
    // The read kinds of `is_read`, as verb text without spaces and hyphens.
    // `PlanStrict` is the verb `plan` with a flag.
    const kinds = [...between('crates/host/speclink-host/src/read_only.rs', 'match command {', '=> true').matchAll(/Command::(\w+)/g)]
      .map((match) => match[1].toLowerCase())
      .filter((kind) => kind !== 'planstrict')
      .sort()

    expect(typings).toHaveLength(17)
    expect(english).toEqual(typings)
    expect(chinese).toEqual(typings)
    expect(typings.map((verb) => verb.replace(/[ -]/g, '')).sort()).toEqual(kinds)
  })

  it('is not on the host object', () => {
    const host = createHost({ store: memoryPort().store, hooks: { beforeCommand: () => undefined } })
    expect((host as unknown as { read?: unknown }).read).toBeUndefined()
  })

  it('leaves a hook that declares one parameter as it was', async () => {
    const port = memoryPort()
    const argv = ['new', 'change', 'demo']
    const host = createHost({ store: port.store, hooks: { beforeCommand: (_info) => undefined } })

    const created = await host.dispatch({ identity: ALICE, scope: 's1', argv })

    expect(created).toEqual(await createHost({ store: memoryPort().store }).dispatch({ identity: ALICE, scope: 's1', argv }))
    expect(port.doc('s1', 'changes/demo/.meta.yaml')).toBeDefined()
    expect(port.calls.commit).toBe(1)
  })

  it('takes several reads at once, as the caller and on the scope of the call', async () => {
    const port = portWithDemo(OPEN_TASKS, { async: true })
    port.seed('s2', { 'changes/other/.meta.yaml': META })
    let asked = 0
    let results: unknown[] = []
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: async (_info, { read }) => {
          asked += 1
          results = await Promise.all([
            read(['list', '--json']),
            read(['show', 'demo', '--json']),
            read(['status', '--change', 'demo', '--json']),
          ])
        },
      },
    })

    await host.dispatch({ identity: ALICE, scope: 's1', argv: ['show', 'demo', '--json'] })

    const [listed, shown] = results as [{ changes: { name: string }[] }, Shown]
    expect(results).toHaveLength(3)
    expect(listed.changes.map((c) => c.name)).toEqual(['demo'])
    expect(shown.name).toBe('demo')
    expect(asked).toBe(1)
    expect(port.calls.commit).toBe(0)
  })
})

// change-landing「change landed 動詞回報落地的 commit」and the two stations'
// 「蓋章記錄程式碼的 commit」: a host has no checkout, so it names the commit itself.
describe('the landing report and the stamp commit through host.dispatch', () => {
  const SHA = '0123456789abcdef0123456789abcdef01234567'

  it('records a landing in one commit with its event, and the hooks see it', async () => {
    const asked: { command: string; params: unknown }[] = []
    const watched: { command: string; events: { name: string; payload: unknown }[] }[] = []
    const port = portWithDemo()
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: (info) => {
          asked.push({ command: info.command, params: info.params })
        },
        afterCommit: (info) => {
          watched.push({ command: info.command, events: info.events })
        },
      },
    })

    const landed = await host.dispatch({
      identity: ALICE,
      scope: 's1',
      argv: ['change', 'landed', 'demo', '--commit', SHA, '--json'],
    })
    await new Promise((resolve) => setTimeout(resolve, 50))

    expect(landed).toEqual({ change: 'demo', commit: SHA })
    expect(port.calls.commit).toBe(1)
    const { commit } = port.commits[0]
    expect(commit.command).toBe('change-landed')
    expect(commit.ops.map((op) => `${op.op} ${op.key}`)).toEqual(['put changes/demo/.meta.yaml'])
    expect(commit.events).toEqual([{ name: 'change-landed', payload: { change: 'demo', commit: SHA } }])
    expect(asked).toEqual([{ command: 'change-landed', params: { name: 'demo', commit: SHA, branch: null } }])
    expect(watched.map((w) => [w.command, w.events.map((e) => e.name)])).toEqual([['change-landed', ['change-landed']]])
    const meta = port.doc('s1', 'changes/demo/.meta.yaml')?.content ?? ''
    expect(meta).toContain(`landed_commit: ${SHA}\n`)
    expect(meta).toContain('landed_by: Alice\n')
  })

  it('rejects a landing with no --commit as invalid_argv, and commits nothing', async () => {
    const port = portWithDemo()
    const host = createHost({ store: port.store })
    const err = await rejection(host.dispatch({ identity: ALICE, scope: 's1', argv: ['change', 'landed', 'demo'] }))
    expect(err.code).toBe('invalid_argv')
    expect(err.message).toContain('--commit')
    expect(port.calls.commit).toBe(0)
  })

  it('lets beforeCommand refuse a landing', async () => {
    const port = portWithDemo()
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: (info) =>
          info.command === 'change-landed' ? { allow: false, reason: 'the commit is not on main' } : undefined,
      },
    })
    const err = await rejection(
      host.dispatch({ identity: ALICE, scope: 's1', argv: ['change', 'landed', 'demo', '--commit', SHA] }),
    )
    expect(err.code).toBe('refused')
    expect(port.doc('s1', 'changes/demo/.meta.yaml')?.content).toBe(META)
  })

  it('records the headCommit a stamp names on stdin, and show --json reports the records', async () => {
    const port = portWithDemo(DONE_TASKS)
    const host = createHost({ store: port.store })
    const call = (argv: string[], stdin?: string) => host.dispatch({ identity: ALICE, scope: 's1', argv, stdin })
    const keysBefore = Object.keys((await call(['show', 'demo', '--json'])) as object)
    for (const key of ['review', 'verify', 'branch', 'landed']) expect(keysBefore).not.toContain(key)

    await call(['in-progress', 'add', 'demo'])
    await call(['review', 'add-round', 'demo', '--stdin'], '**Scope**: src/lib.rs\n')
    await call(
      ['review', 'stamp', 'demo', '--stdin'],
      JSON.stringify({ scope: [{ path: 'src/lib.rs', hash: '0f9c' }], headCommit: '0123456789ab' }),
    )
    await call(['verify', 'add-round', 'demo', '--stdin'], '**Scope**: src/lib.rs\n')
    await call(['change', 'landed', 'demo', '--commit', SHA, '--branch', 'speclink/demo'])

    expect(port.doc('s1', 'changes/demo/.meta.yaml')?.content).toContain('reviewed_commit: 0123456789ab\n')
    const shown = (await call(['show', 'demo', '--json'])) as Record<string, any>
    expect(shown.review).toMatchObject({ state: 'stamped', commit: '0123456789ab', rounds: 1, tasksTotal: 1, stampedBy: 'Alice' })
    expect(typeof shown.review.stampedAt).toBe('string')
    expect(shown.review.scope).toBeUndefined()
    expect(shown.verify).toEqual({ state: 'inProgress' })
    expect(shown.branch).toBe('speclink/demo')
    expect(shown.landed).toMatchObject({ commit: SHA, by: 'Alice' })
    expect(typeof shown.landed.at).toBe('string')
    for (const key of Object.keys(shown)) expect(isCamel(key)).toBe(true)
  })

  // command-runtime「蓋章事件帶 commit 與流程設定變動的事件」: the store's commit and
  // afterCommit get the commit the stamp recorded, and no commit when it recorded none.
  it('gives the commit function and afterCommit the commit a stamp recorded', async () => {
    const watched: { command: string; events: { name: string; payload: unknown }[] }[] = []
    const port = portWithDemo(DONE_TASKS)
    const host = createHost({
      store: port.store,
      hooks: {
        afterCommit: (info) => {
          watched.push({ command: info.command, events: info.events })
        },
      },
    })
    const call = (argv: string[], stdin?: string) => host.dispatch({ identity: ALICE, scope: 's1', argv, stdin })
    const scope = [{ path: 'src/lib.rs', hash: '0f9c' }]

    await call(['review', 'add-round', 'demo', '--stdin'], '**Scope**: src/lib.rs\n')
    await call(['review', 'stamp', 'demo', '--stdin'], JSON.stringify({ scope, headCommit: SHA }))
    await call(['verify', 'add-round', 'demo', '--stdin'], '**Scope**: src/lib.rs\n')
    await call(['verify', 'stamp', 'demo', '--stdin'], JSON.stringify({ scope }))

    const stamps = port.commits
      .filter(({ commit }) => commit.command.endsWith('-stamp'))
      .map(({ commit }) => [commit.command, commit.events])
    expect(stamps).toEqual([
      ['review-stamp', [{ name: 'review-stamped', payload: { change: 'demo', commit: SHA } }]],
      ['verify-stamp', [{ name: 'verify-stamped', payload: { change: 'demo' } }]],
    ])
    // afterCommit runs after dispatch resolves: wait for it, not for a fixed time.
    await vi.waitFor(() =>
      expect(watched.filter((w) => w.command.endsWith('-stamp')).map((w) => [w.command, w.events])).toEqual(stamps),
    )
  })

  it('stamps without a commit when the stdin JSON names none, and refuses a malformed one', async () => {
    const port = portWithDemo(DONE_TASKS)
    const host = createHost({ store: port.store })
    const call = (argv: string[], stdin?: string) => host.dispatch({ identity: ALICE, scope: 's1', argv, stdin })
    await call(['verify', 'add-round', 'demo', '--stdin'], '**Scope**: src/lib.rs\n')
    const bad = JSON.stringify({ scope: [{ path: 'src/lib.rs', hash: '0f9c' }], headCommit: 'not-a-sha' })
    const err = await rejection(call(['verify', 'stamp', 'demo', '--stdin'], bad))
    expect(err.code).toBe('invalid_argv')
    expect(port.doc('s1', 'changes/demo/verify.md')).toBeDefined()

    await call(['verify', 'stamp', 'demo', '--stdin'], JSON.stringify({ scope: [{ path: 'src/lib.rs', hash: '0f9c' }] }))
    const meta = port.doc('s1', 'changes/demo/.meta.yaml')?.content ?? ''
    expect(meta).toContain('verified_at: ')
    expect(meta).not.toContain('verified_commit')
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

// verb-contract「內嵌宿主的人眼輸出不印路徑行」「清單的詳細輸出」: a host's
// documents are in its store, so the output names no local path, and a list
// with --detail carries what each change has on record.
describe('host output for a store with no local folder', () => {
  const COMMIT = '0123456789abcdef0123456789abcdef01234567'
  const DETAIL_KEYS = ['review', 'verify', 'claimed', 'branch', 'landed']
  const RECORDED_META = [
    'schema: spec-driven',
    'created: 2026-07-01',
    'reviewed_at: 2026-08-01',
    'reviewed_by: Rev <r@example.com>',
    'reviewed_with: claude',
    'reviewed_tasks_total: 1',
    'reviewed_scope:',
    '  - path: src/lib.rs',
    '    hash: 0f9c',
    'branch: speclink/demo',
    `landed_commit: ${COMMIT}`,
    'landed_at: 2026-11-18T11:00:00+00:00',
    '',
  ].join('\n')
  const CLAIM = 'claimed_by: Alice\nclaimed_by_id: u_1\nclaimed_at: 2026-11-18T10:00:00+00:00\n'
  const VERIFY_TICKET = '# Verify — demo\n\n## Round 1\n\n**Scope**: src/lib.rs\n'
  const PLAIN_META = 'schema: spec-driven\ncreated: 2026-07-01\n'

  const dispatcher = (host: ReturnType<typeof createHost>) => (argv: string[], stdin?: string) =>
    host.dispatch({ identity: ALICE, scope: 's1', argv, stdin })

  it('prints no Path line for new change and keeps its other lines', async () => {
    const run = dispatcher(createHost({ store: memoryPort().store }))
    await run(['discuss', 'new', 'Topic', '--slug', 'topic'])

    expect(await run(['new', 'change', 'demo'])).toEqual({ output: '✓ Created change: demo\n  Schema: spec-driven\n' })
    expect(await run(['new', 'change', 'from-topic', '--from-discussion', 'topic'])).toEqual({
      output: '✓ Created change: from-topic\n  Schema: spec-driven\n  From discussion: topic\n',
    })
  })

  // These two verbs have a structured value, and dispatch returns that value
  // with or without --json: there is no text for a Path line to be in. What a
  // host can see is the `path` key, which stays. The text form of the two
  // verbs is pinned where it exists: speclink-verbs' render tests.
  it('returns discuss new and discuss promote as structured values that keep path, with no text output', async () => {
    const run = dispatcher(createHost({ store: memoryPort().store }))

    const created = (await run(['discuss', 'new', 'Topic', '--slug', 'topic'])) as Record<string, unknown>
    expect(created).toMatchObject({ slug: 'topic', topic: 'Topic' })
    expect(created).not.toHaveProperty('output')
    expect(String(created.path)).toContain('topic')
    const viaJson = (await run(['discuss', 'new', 'Other', '--slug', 'other', '--json'])) as Record<string, unknown>
    expect(Object.keys(viaJson).sort()).toEqual(Object.keys(created).sort())
    expect(String(viaJson.path)).toContain('other')

    await run(['discuss', 'conclude', 'topic', '--stdin', '--hold'], '**Decision**: ship it\n')
    const promoted = (await run(['discuss', 'promote', 'topic'])) as Record<string, unknown>
    expect(Object.keys(promoted).sort()).toEqual(['change', 'path', 'slug', 'status'])
    expect(promoted).toMatchObject({ slug: 'topic', change: 'topic', status: 'promoted' })
    expect(String(promoted.path)).toContain('topic')
  })

  it('still prints the Path line for an engine on a local folder', async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-host-path-line-')))
    cleanup.push(() => rmSync(root, { recursive: true, force: true }))
    mkdirSync(join(root, 'speclink', 'changes'), { recursive: true })

    const created = (await createEngine({ store: { type: 'fs', root } }).dispatch(['new', 'change', 'demo'])) as {
      output: string
    }

    expect(created.output.split('\n')).toEqual([
      '✓ Created change: demo',
      `  Path: ${join(root, 'speclink', 'changes', 'demo')}`,
      '  Schema: spec-driven',
      '',
    ])
  })

  it('lists what each change has on record with --detail, and nothing more without it', async () => {
    const port = memoryPort()
    port.seed('s1', {
      'changes/demo/.meta.yaml': RECORDED_META + CLAIM,
      'changes/demo/tasks.md': OPEN_TASKS,
      'changes/demo/verify.md': VERIFY_TICKET,
      'changes/plain/.meta.yaml': PLAIN_META,
      'config.yaml': CONFIG,
    })
    const run = dispatcher(createHost({ store: port.store }))
    type Listed = { changes: Record<string, any>[] }

    const detailed = (await run(['list', '--json', '--detail', '--sort', 'name'])) as Listed
    const [demo, plain] = detailed.changes
    expect(demo.review).toEqual({
      state: 'stamped',
      stampedAt: '2026-08-01',
      stampedBy: 'Rev <r@example.com>',
      stampedWith: 'claude',
      rounds: 0,
      tasksTotal: 1,
    })
    expect(demo.verify).toEqual({ state: 'inProgress' })
    expect(demo.claimed).toEqual({ by: 'Alice', byId: 'u_1', at: '2026-11-18T10:00:00+00:00' })
    expect(demo.branch).toBe('speclink/demo')
    expect(demo.landed).toEqual({ commit: COMMIT })
    expect(Object.keys(demo).every(isCamel)).toBe(true)
    for (const key of DETAIL_KEYS) expect(plain, key).not.toHaveProperty(key)

    const listed = (await run(['list', '--json', '--sort', 'name'])) as Listed
    for (const change of listed.changes) for (const key of DETAIL_KEYS) expect(change, key).not.toHaveProperty(key)
    // The fields that were there before are the same with and without --detail.
    for (const [i, change] of listed.changes.entries()) expect(detailed.changes[i]).toMatchObject(change)
  })

  it('lists the same detail as an engine on a local folder with the same documents', async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-host-detail-')))
    cleanup.push(() => rmSync(root, { recursive: true, force: true }))
    const docs: Record<string, string> = {
      'changes/demo/.meta.yaml': RECORDED_META,
      'changes/demo/tasks.md': OPEN_TASKS,
      'changes/demo/verify.md': VERIFY_TICKET,
      'changes/plain/.meta.yaml': PLAIN_META,
    }
    for (const [key, content] of Object.entries(docs)) {
      mkdirSync(join(root, 'speclink', key, '..'), { recursive: true })
      writeFileSync(join(root, 'speclink', key), content)
    }
    const port = memoryPort()
    port.seed('s1', { ...docs, 'config.yaml': CONFIG })
    const argv = ['list', '--json', '--detail', '--sort', 'name']
    // The last-modified time is the store's own: a file time on one side.
    const changes = (listed: unknown) =>
      (listed as { changes: Record<string, unknown>[] }).changes.map(({ modifiedAt: _, ...rest }) => rest)

    const viaEngine = await createEngine({ store: { type: 'fs', root } }).dispatch(argv)
    const viaHost = await createHost({ store: port.store }).dispatch({ identity: ALICE, scope: 's1', argv })

    expect(changes(viaHost)).toEqual(changes(viaEngine))
    expect(changes(viaHost)[0]).toMatchObject({ branch: 'speclink/demo', landed: { commit: COMMIT } })
  })

  it('refuses --detail without --json or with --specs as invalid_argv, with no store call', async () => {
    const port = portWithDemo()
    const run = dispatcher(createHost({ store: port.store }))
    const before = totalCalls(port)

    const alone = await rejection(run(['list', '--detail']))
    expect(alone.code).toBe('invalid_argv')
    expect(alone.message).toContain('--json')
    const withSpecs = await rejection(run(['list', '--specs', '--json', '--detail']))
    expect(withSpecs.code).toBe('invalid_argv')
    expect(withSpecs.message).toContain('--specs')
    expect(withSpecs.message).toContain('--detail')
    expect(totalCalls(port)).toBe(before)
  })
})

// verb-contract「詞彙文件的寫入動詞」、discussion-docs「討論以 restore 動詞搬回進行中」:
// the two verbs run in a host with no local spec files, and ask beforeCommand first.
describe('language replace and discuss restore through host.dispatch', () => {
  type Asked = { command: string; params: unknown }

  function hostThatRecords(port: ReturnType<typeof memoryPort>, deny: Record<string, string> = {}) {
    const asked: Asked[] = []
    const events: { command: string; names: string[] }[] = []
    const host = createHost({
      store: port.store,
      hooks: {
        beforeCommand: ({ command, params }) => {
          asked.push({ command, params })
          return command in deny ? { allow: false, reason: deny[command] } : undefined
        },
        afterCommit: (info) => {
          events.push({ command: info.command, names: info.events.map((e) => e.name) })
        },
      },
    })
    const run = (argv: string[], stdin?: string) => host.dispatch({ identity: ALICE, scope: 's1', argv, stdin })
    return { run, asked, events }
  }

  it('writes the LANGUAGE document from stdin, and language show reads the same content back', async () => {
    const port = memoryPort()
    const { run, asked, events } = hostThatRecords(port)
    const second = '# Language\n\n## 詞彙\n'

    expect(await run(['language', 'replace', '--stdin'], '# Language\n')).toEqual({ created: true })
    expect(await run(['language', 'show'])).toEqual({ output: '# Language\n' })
    expect(await run(['language', 'replace', '--stdin', '--json'], second)).toEqual({ created: false })
    expect(await run(['language', 'show'])).toEqual({ output: second })
    expect(port.doc('s1', 'LANGUAGE.md')?.content).toBe(second)

    // The hook is asked under the command's own name, and sees the size of the
    // text, never the text.
    expect(asked.filter((a) => a.command === 'language-replace')).toEqual([
      { command: 'language-replace', params: { bytes: 11 } },
      { command: 'language-replace', params: { bytes: 22 } },
    ])
    // afterCommit runs after dispatch resolves: wait for it, not for a fixed time.
    await vi.waitFor(() =>
      expect(events).toEqual([
        { command: 'language-replace', names: ['language-replaced'] },
        { command: 'language-replace', names: ['language-replaced'] },
      ]),
    )
  })

  it('rejects empty content as invalid_argv and leaves the document as it was', async () => {
    const port = memoryPort()
    port.seed('s1', { 'LANGUAGE.md': '# Old\n' })
    const { run } = hostThatRecords(port)
    const commits = port.calls.commit

    for (const stdin of ['', '   \n\n', undefined]) {
      const err = await rejection(run(['language', 'replace', '--stdin'], stdin))
      expect(err.code).toBe('invalid_argv')
      expect(err.message).toContain('empty')
    }
    // Without --stdin the verb is a usage error.
    expect((await rejection(run(['language', 'replace'], '# New\n'))).code).toBe('invalid_argv')

    expect(port.doc('s1', 'LANGUAGE.md')?.content).toBe('# Old\n')
    expect(port.calls.commit).toBe(commits)
  })

  it('restores an archived discussion, and the discussion is in the list again', async () => {
    const port = memoryPort()
    const { run, asked, events } = hostThatRecords(port)
    await run(['discuss', 'new', 'Topic', '--slug', 'topic'])
    await run(['discuss', 'conclude', 'topic', '--stdin', '--hold'], '**Decision**: ship it\n')
    const record = port.doc('s1', 'discussions/topic.md')?.content
    const archived = (await run(['discuss', 'archive', 'topic'])) as { archived_to: string }
    const file = archived.archived_to.replace('discussions/archive/', '')
    expect(port.doc('s1', 'discussions/topic.md')).toBeUndefined()
    const commits = port.calls.commit

    const restored = await run(['discuss', 'restore', 'topic', '--json'])

    expect(restored).toEqual({ slug: 'topic', restoredFrom: file })
    expect(file).toMatch(/^\d{4}-\d{2}-\d{2}-topic\.md$/)
    expect(port.doc('s1', 'discussions/topic.md')?.content).toBe(record)
    expect(port.doc('s1', `archive/discussions/${file}`)).toBeUndefined()
    // One commit carries the two writes.
    expect(port.calls.commit).toBe(commits + 1)
    expect(port.commits.at(-1)!.commit.ops.map((op) => op.op).sort()).toEqual(['delete', 'put'])
    const listed = (await run(['discuss', 'list', '--json'])) as { discussions: { slug: string }[] }
    expect(listed.discussions.map((d) => d.slug)).toEqual(['topic'])
    expect(await run(['discuss', 'list', '--archived', '--json'])).toEqual({ discussions: [] })
    expect(asked.filter((a) => a.command === 'discuss-restore')).toEqual([
      { command: 'discuss-restore', params: { slug: 'topic' } },
    ])
    await vi.waitFor(() =>
      expect(events.at(-1)).toEqual({ command: 'discuss-restore', names: ['discussion-restored'] }),
    )

    // A live record blocks a second restore; a slug with no archived record is not found.
    expect((await rejection(run(['discuss', 'restore', 'topic']))).code).toBe('refused')
    const ghost = await rejection(run(['discuss', 'restore', 'ghost']))
    expect(ghost.code).toBe('not_found')
    expect(ghost.message).toContain('ghost')
  })

  it('lets beforeCommand refuse both verbs under their own command names', async () => {
    const port = memoryPort()
    port.seed('s1', { 'LANGUAGE.md': '# Old\n', 'archive/discussions/2026-11-10-topic.md': 'an archived record\n' })
    const { run, asked } = hostThatRecords(port, {
      'language-replace': '只有維護者可以改詞彙表',
      'discuss-restore': '封存的討論不開放搬回',
    })
    const commits = port.calls.commit

    const language = await rejection(run(['language', 'replace', '--stdin'], '# New\n'))
    const restore = await rejection(run(['discuss', 'restore', 'topic']))

    expect([language.code, language.message]).toEqual(['refused', '只有維護者可以改詞彙表'])
    expect([restore.code, restore.message]).toEqual(['refused', '封存的討論不開放搬回'])
    expect(asked).toEqual([
      { command: 'language-replace', params: { bytes: 6 } },
      { command: 'discuss-restore', params: { slug: 'topic' } },
    ])
    expect(port.doc('s1', 'LANGUAGE.md')?.content).toBe('# Old\n')
    expect(port.doc('s1', 'discussions/topic.md')).toBeUndefined()
    expect(port.calls.commit).toBe(commits)
  })
})
