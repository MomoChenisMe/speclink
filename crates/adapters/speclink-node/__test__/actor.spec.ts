// 建構期 actor 注入（spec：createEngine 的建構期 actor 注入）。
// fs 形式的回退對照物是 CLI 自己蓋的章——同一 fixture 專案跑兩邊，逐位元比對。
import { beforeAll, describe, expect, it } from 'vitest'
import { execFileSync, execSync } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

import { createEngine } from '../index.js'
import { cliBin, GIT_IDENTITY, GIT_NAME, initGitIdentity, makeFsFixture, projectFiles, rejection, repoRoot } from './helpers'

let fsFixture: string

/** fs 形式的 change metadata 原文。 */
function fsMeta(change: string): string {
  return readFileSync(join(fsFixture, 'speclink', 'changes', change, '.meta.yaml'), 'utf8')
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
  initGitIdentity(fsFixture)
}, 600_000)

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

  // spec「呼叫期無從覆寫身分」：dispatch 沒有身分的參數，argv 也帶不進去。
  it('argv 帶不進身分：--actor 被拒絕，之後的蓋章仍是建構期的 actor', async () => {
    const root = makeFsFixture()
    initGitIdentity(root)
    const engine = createEngine({ store: { type: 'fs', root }, actor: 'Alice <alice@example.com>' })
    const before = projectFiles(root)
    const err = await rejection(engine.dispatch(['new', 'change', 'forged', '--actor', 'Bob <bob@example.com>']))
    expect(err.code).toBe('invalid_argv')
    expect(projectFiles(root)).toEqual(before)

    await engine.dispatch(['new', 'change', 'honest'])
    const meta = readFileSync(join(root, 'speclink', 'changes', 'honest', '.meta.yaml'), 'utf8')
    expect(createdByLine(meta)).toBe('created_by: Alice <alice@example.com>')
  })
})
