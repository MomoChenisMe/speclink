// spec node-sdk「並發 dispatch 不死結」: twenty dispatches at once, reads and
// writes together, must all settle in bounded time while the event loop keeps
// running timers — on an engine over a local folder, and on a host kit whose
// store port answers with Promises.
import { describe, expect, it } from 'vitest'

import { createHost } from '../index.js'
import { fsEngine } from './helpers'
import { memoryPort } from './memory-port'

const CALLS = 20
const TIME_LIMIT_MS = 30_000
/** Long enough for several turns of a timer. */
const BUSY_MS = 60

/**
 * Run `start()` (which sends every call) next to a timer that counts its
 * turns. Resolves to the results, how long the batch took, and how many turns
 * the timer had when the last call settled.
 */
async function withTicker(start: () => Promise<unknown>[]): Promise<{ results: unknown[]; elapsed: number; ticks: number }> {
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

/**
 * Send `call` again and again, each after the one before settled, until `ms`
 * have passed. It waits on the calls only, never on a timer: when a call
 * settles without a turn of the event loop, no timer runs in all that time.
 */
async function callFor(ms: number, call: () => Promise<unknown>): Promise<void> {
  const until = Date.now() + ms
  while (Date.now() < until) await call()
}

describe('dispatch stress — no deadlock', () => {
  it('twenty concurrent reads and writes on a local folder all settle', async () => {
    const engine = fsEngine()
    const { results, elapsed, ticks } = await withTicker(() => [
      ...Array.from({ length: CALLS }, (_, i) =>
        i % 2 === 0 ? engine.dispatch(['list', '--json']) : engine.dispatch(['new', 'change', `stress-${i}`]),
      ),
      // Twenty calls on a local folder can settle before the first timer turn.
      // This keeps the batch pending for several turns, with dispatches only.
      callFor(BUSY_MS, () => engine.dispatch(['list', '--json'])),
    ])
    expect(results).toHaveLength(CALLS + 1)
    expect(elapsed).toBeLessThan(TIME_LIMIT_MS)
    // The timer ran while calls were pending. A dispatch that settled without
    // a turn of the event loop would leave it at 0.
    expect(ticks).toBeGreaterThan(0)
    // Every write landed.
    const listed = (await engine.dispatch(['list', '--sort', 'name', '--json'])) as { changes: { name: string }[] }
    const written = listed.changes.map((c) => c.name).filter((name) => name.startsWith('stress-'))
    expect(written).toHaveLength(CALLS / 2)
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
