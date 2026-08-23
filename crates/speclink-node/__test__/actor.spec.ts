// 建構期 actor 注入（spec：createEngine 的建構期 actor 注入）。
// fs 形式的回退對照物是 CLI 自己蓋的章——同一 fixture 專案跑兩邊，逐位元比對。
import { beforeAll, afterAll, describe, expect, it } from 'vitest'
import { execFileSync, execSync } from 'node:child_process'
import { existsSync, readFileSync, rmSync } from 'node:fs'
import { join } from 'node:path'

import { createEngine } from '../index.js'
import { cliBin, fixtureProject, makeFsFixture, memoryStore, repoRoot } from './helpers'

// fixture 專案自帶 git 身分——不設就會沿用開發者的 global config，斷言不可重現。
const GIT_NAME = 'Fixture Dev'
const GIT_EMAIL = 'fixture@example.com'
const GIT_IDENTITY = `${GIT_NAME} <${GIT_EMAIL}>`

let fsFixture: string

/** fs 形式的 change metadata 原文。 */
function fsMeta(change: string): string {
  return readFileSync(join(fsFixture, 'openspec', 'changes', change, '.openspec.yaml'), 'utf8')
}

/** metadata 中的 created_by 整行（無則 null）——逐位元比對用。 */
function createdByLine(meta: string): string | null {
  return meta.split('\n').find((l) => l.startsWith('created_by:')) ?? null
}

beforeAll(() => {
  if (!existsSync(cliBin)) {
    execSync('cargo build -p speclink-cli', { cwd: repoRoot, stdio: 'inherit' })
  }
  fsFixture = makeFsFixture()
  execFileSync('git', ['init', '-q'], { cwd: fsFixture })
  execFileSync('git', ['config', 'user.name', GIT_NAME], { cwd: fsFixture })
  execFileSync('git', ['config', 'user.email', GIT_EMAIL], { cwd: fsFixture })
}, 600_000)

afterAll(() => {
  if (fsFixture) rmSync(fsFixture, { recursive: true, force: true })
})

describe('createEngine actor — fs 形式', () => {
  it('明給 actor 時優先於 git identity', async () => {
    const engine = createEngine({
      store: { type: 'fs', root: fsFixture },
      actor: 'Alice <alice@example.com>',
    })
    await engine.dispatch(['new', 'change', 'demo'])
    const meta = fsMeta('demo')
    expect(createdByLine(meta)).toBe('created_by: Alice <alice@example.com>')
    expect(meta).not.toContain(GIT_NAME)
  })

  it('未給 actor 時回退 git identity，與 CLI 蓋章逐位元一致', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fsFixture } })
    await engine.dispatch(['new', 'change', 'demo2'])
    execFileSync(cliBin, ['new', 'change', 'demo2-cli'], { cwd: fsFixture })
    expect(createdByLine(fsMeta('demo2'))).toBe(`created_by: ${GIT_IDENTITY}`)
    expect(createdByLine(fsMeta('demo2'))).toBe(createdByLine(fsMeta('demo2-cli')))
  })

  it('trim 後為空的 actor 視同未給', async () => {
    const engine = createEngine({ store: { type: 'fs', root: fsFixture }, actor: '   ' })
    await engine.dispatch(['new', 'change', 'demo-blank'])
    expect(createdByLine(fsMeta('demo-blank'))).toBe(`created_by: ${GIT_IDENTITY}`)
  })
})

describe('createEngine actor — 宿主 Store 形式', () => {
  it('帶 actor 時落 created_by', async () => {
    const store = memoryStore(fixtureProject())
    const engine = createEngine({ store, actor: 'Bob <bob@example.com>' })
    await engine.dispatch(['new', 'change', 'demo3'])
    const meta = (await store.readArtifact('demo3', '.openspec.yaml')) ?? ''
    expect(createdByLine(meta)).toBe('created_by: Bob <bob@example.com>')
  })

  it('未給 actor 時維持無章', async () => {
    const store = memoryStore(fixtureProject())
    const engine = createEngine({ store })
    await engine.dispatch(['new', 'change', 'demo4'])
    const meta = (await store.readArtifact('demo4', '.openspec.yaml')) ?? ''
    expect(createdByLine(meta)).toBeNull()
  })

  it('trim 後為空的 actor 視同未給——宿主 Store 無回退，維持無章', async () => {
    const store = memoryStore(fixtureProject())
    const engine = createEngine({ store, actor: ' ' })
    await engine.dispatch(['new', 'change', 'demo5'])
    const meta = (await store.readArtifact('demo5', '.openspec.yaml')) ?? ''
    expect(createdByLine(meta)).toBeNull()
  })
})
