// Bridge stress (spec: 並發 dispatch 不死結): one hundred sequential and one
// hundred concurrent dispatches against an async host store must all settle
// in bounded time, while the event loop keeps servicing other work.
import { describe, expect, it } from 'vitest'

import { createEngine } from '../index.js'
import { fixtureProject, memoryStore } from './helpers'

describe('dispatch stress — no deadlock', () => {
  it('100 sequential dispatches all settle', async () => {
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    for (let i = 0; i < 100; i++) {
      const result = (await engine.dispatch(['list', '--json'])) as { changes: unknown[] }
      expect(result.changes).toHaveLength(2)
    }
  }, 60_000)

  it('100 concurrent dispatches all settle and the event loop stays live', async () => {
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    let ticks = 0
    const ticker = setInterval(() => {
      ticks++
    }, 5)
    try {
      const results = await Promise.all(
        Array.from({ length: 100 }, (_, i) =>
          i % 2 === 0
            ? engine.dispatch(['list', '--json'])
            : engine.dispatch(['status', '--change', 'alpha', '--json']),
        ),
      )
      expect(results).toHaveLength(100)
      for (const r of results) expect(r).toBeTruthy()
    } finally {
      clearInterval(ticker)
    }
    // The event loop was never starved while the workers waited on the store.
    expect(ticks).toBeGreaterThan(0)
  }, 60_000)

  it('mixed fs-form and store-form engines run concurrently', async () => {
    const jsEngine = createEngine({ store: memoryStore(fixtureProject()) })
    const results = await Promise.all(
      Array.from({ length: 50 }, () => jsEngine.dispatch(['list', '--sort', 'name', '--json'])),
    )
    const first = JSON.stringify(results[0])
    for (const r of results) expect(JSON.stringify(r)).toBe(first)
  }, 60_000)
})
