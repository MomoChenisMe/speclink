// node-sdk「合格測試隨套件提供」: runStoreConformance runs the store
// contract's own sixteen checks (the Rust suite) against a JS store port.
import { describe, expect, it } from 'vitest'
import { runStoreConformance } from '../index.js'
import { memoryPort } from './memory-port'

// The check names of speclink-store-port's conformance suite, in report order.
const RUST_CHECKS = [
  'empty-scope',
  'put-then-read',
  'list-prefix-order',
  'read-alignment',
  'update-advances-version',
  'delete-then-recreate',
  'conflict-stale-version',
  'conflict-absent-but-exists',
  'conflict-missing-document',
  'conflict-rejects-whole-commit',
  'events-with-commit',
  'events-only-commit',
  'scope-isolation',
  'concurrent-single-winner',
  'batch-read-atomic',
  'content-roundtrip',
]
// Without recordedEvents only this check cannot decide; events-only-commit
// still checks that an events-only commit moves the scope version.
const READ_BACK_CHECKS = ['events-with-commit']

function scopes() {
  let n = 0
  return () => `scope-${++n}`
}

describe('runStoreConformance', () => {
  it('passes every check for a correct JS store', async () => {
    const port = memoryPort()
    const report = await runStoreConformance({
      store: port.store,
      newScope: scopes(),
      recordedEvents: port.recordedEvents,
    })
    expect(report.checks.map((c) => c.name)).toEqual(RUST_CHECKS)
    expect(report.checks.filter((c) => c.outcome !== 'passed')).toEqual([])
    expect(report.passed).toBe(true)
  })

  it('fails a store that does not compare versions', async () => {
    const port = memoryPort({ ignoreExpected: true })
    const report = await runStoreConformance({
      store: port.store,
      newScope: scopes(),
      recordedEvents: port.recordedEvents,
    })
    expect(report.passed).toBe(false)
    const stale = report.checks.find((c) => c.name === 'conflict-stale-version')!
    expect(stale.outcome).toBe('failed')
    expect(stale.detail).toMatch(/conflict|accepted|succeeded/i)
  })

  it('skips the read-back event check without recordedEvents', async () => {
    const port = memoryPort()
    const report = await runStoreConformance({ store: port.store, newScope: scopes() })
    for (const check of report.checks) {
      if (READ_BACK_CHECKS.includes(check.name)) {
        expect(check.outcome).toBe('skipped')
        expect(check.detail).toContain('recordedEvents')
      } else {
        expect(check.outcome, check.name).toBe('passed')
      }
    }
    expect(report.passed).toBe(true)
  })

  it('completes for a store whose functions return Promises', async () => {
    const port = memoryPort({ async: true })
    const report = await runStoreConformance({
      store: port.store,
      newScope: async () => `async-${Math.random().toString(36).slice(2)}`,
      recordedEvents: async (scope: string) => port.recordedEvents(scope),
    })
    for (const name of ['concurrent-single-winner', 'batch-read-atomic']) {
      expect(report.checks.find((c) => c.name === name)?.outcome, name).toBe('passed')
    }
    expect(report.passed).toBe(true)
  })
})
