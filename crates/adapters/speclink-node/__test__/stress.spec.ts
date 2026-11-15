// spec node-sdk「並發 dispatch 不死結」: twenty dispatches at once, reads and
// writes together, must all settle in bounded time while the event loop keeps
// running timers — on an engine over a local folder, and on a host kit whose
// store port answers with Promises.
import { rmSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

import { createEngine, createHost } from '../index.js'
import { makeFsFixture } from './helpers'
import { memoryPort } from './memory-port'

const CALLS = 20
const TIME_LIMIT_MS = 30_000

/**
 * Run `start()` (which sends every call) next to a timer that counts its
 * turns. Resolves to the results, how long the batch took, and the count.
 */
async function withTicker<T>(start: () => Promise<T>[]): Promise<{ results: T[]; elapsed: number; ticks: number }> {
  let ticks = 0
  const ticker = setInterval(() => {
    ticks++
  }, 0)
  const started = Date.now()
  try {
    const results = await Promise.all(start())
    return { results, elapsed: Date.now() - started, ticks }
  } finally {
    clearInterval(ticker)
  }
}

describe('dispatch stress — no deadlock', () => {
  it('twenty concurrent reads and writes on a local folder all settle', async () => {
    const root = makeFsFixture()
    try {
      const engine = createEngine({ store: { type: 'fs', root } })
      const { results, elapsed, ticks } = await withTicker(() =>
        Array.from({ length: CALLS }, (_, i) =>
          i % 2 === 0 ? engine.dispatch(['list', '--json']) : engine.dispatch(['new', 'change', `stress-${i}`]),
        ),
      )
      expect(results).toHaveLength(CALLS)
      expect(elapsed).toBeLessThan(TIME_LIMIT_MS)
      // The event loop was not starved while the workers ran. A batch that
      // finishes inside a few timer turns can see zero ticks on a fast
      // machine, so the timer is only checked when the batch was long enough.
      if (elapsed >= 50) expect(ticks).toBeGreaterThan(0)
      // Every write landed.
      const listed = (await engine.dispatch(['list', '--sort', 'name', '--json'])) as { changes: { name: string }[] }
      const written = listed.changes.map((c) => c.name).filter((name) => name.startsWith('stress-'))
      expect(written).toHaveLength(CALLS / 2)
    } finally {
      rmSync(root, { recursive: true, force: true, maxRetries: 5 })
    }
  }, 60_000)

  it('twenty concurrent reads and writes on a host kit with an async store port all settle', async () => {
    const port = memoryPort({ async: true })
    port.seed('shared', { 'changes/demo/.meta.yaml': 'schema: spec-driven\n' })
    const host = createHost({ store: port.store })
    const identity = { id: 'u_1', displayName: 'Alice' }
    const { results, elapsed, ticks } = await withTicker(() =>
      Array.from({ length: CALLS }, (_, i) =>
        i % 2 === 0
          ? host.dispatch({ identity, scope: 'shared', argv: ['list', '--json'] })
          : // Each write has its own scope, so no two commits meet on a version.
            host.dispatch({ identity, scope: `scope-${i}`, argv: ['new', 'change', `stress-${i}`] }),
      ),
    )
    expect(results).toHaveLength(CALLS)
    expect(elapsed).toBeLessThan(TIME_LIMIT_MS)
    // Every store call settles on a timer of its own, and the ticker was set
    // first: it ran at least once before any dispatch could finish.
    expect(ticks).toBeGreaterThan(0)
    expect(port.calls.commit).toBe(CALLS / 2)
    for (let i = 1; i < CALLS; i += 2) {
      expect(port.doc(`scope-${i}`, `changes/stress-${i}/.meta.yaml`)).toBeDefined()
    }
  }, 60_000)
})
