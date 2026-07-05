// Parity tests: the fs-form engine must return the same objects the CLI
// prints with --json on the same fixture project (spec: fs 形式與 CLI 行為對等).
import { beforeAll, afterAll, describe, expect, it } from 'vitest'
import { execSync } from 'node:child_process'
import { existsSync, rmSync } from 'node:fs'

import { createEngine } from '../index.js'
import { cliBin, cliJson, makeFsFixture, repoRoot } from './helpers'

let fixture: string

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
