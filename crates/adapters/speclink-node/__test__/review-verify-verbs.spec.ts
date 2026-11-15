// dispatch 的蓋章動詞（spec：dispatch 的蓋章動詞）。scope 指紋與 missing 清單 argv
// 承載不了，走 stdin 的 JSON（server stamp request body 的 scope／missing 子集；
// accept／agent 走 argv 旗標）。
import { execFileSync } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

import { createEngine } from '../index.js'
import { cliBin, makeFsFixture, projectFiles } from './helpers'

const ACTOR = 'Rev <rev@example.com>'
// beta 的任務全數完成——蓋章守門的前提。
const CHANGE = 'beta'
const SCOPE_FILE = 'crates/b/src/util.rs'
// SUGGESTION 不擋章；CRITICAL 擋。
const CLEAN_ROUND = `**Scope**: ${SCOPE_FILE}\n\n- [SUGGESTION] ${SCOPE_FILE} — rename helper\n`
const BLOCKING_ROUND = `**Scope**: ${SCOPE_FILE}\n\n- [CRITICAL] ${SCOPE_FILE} — unwrap on user input\n`
const STAMP_PAYLOAD = JSON.stringify({ scope: [{ path: SCOPE_FILE, hash: '0f9c' }] })

/** A new fixture project, an engine on it, and readers for the change's metadata and tickets. */
function project(actor?: string) {
  const root = makeFsFixture()
  const store = { type: 'fs' as const, root }
  const engine = actor === undefined ? createEngine({ store }) : createEngine({ store, actor })
  const file = (name: string) => join(root, 'speclink', 'changes', CHANGE, name)
  return {
    root,
    engine,
    meta: () => readFileSync(file('.meta.yaml'), 'utf8'),
    /** The ticket's text, or null when the change has none. */
    ticket: (name = 'review.md') => (existsSync(file(name)) ? readFileSync(file(name), 'utf8') : null),
  }
}

describe('review 蓋章鏈', () => {
  it('add-round 建立工單、stamp 以建構期 actor 落 reviewed_by', async () => {
    const { engine, meta, ticket } = project(ACTOR)
    const added = await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], {
      stdin: CLEAN_ROUND,
    })
    expect(added).toEqual({ change: CHANGE, round: 1 })
    expect(ticket()).toContain('## Round 1')

    const stamped = await engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], {
      stdin: STAMP_PAYLOAD,
    })
    expect(stamped).toEqual({ change: CHANGE })
    expect(meta()).toContain(`reviewed_by: ${ACTOR}\n`)
    // 蓋章是 read-modify-write：既有欄位必須原樣保留，不得整份洗掉。
    expect(meta()).toContain('schema: spec-driven\n')
    expect(meta()).toContain('created_by: tester\n')
    // 蓋章效果的另一半：工單刪除。
    expect(ticket()).toBeNull()
  })
})

describe('verify 蓋章鏈', () => {
  it('同一 actor 落 verified_by', async () => {
    const { engine, meta } = project(ACTOR)
    const added = await engine.dispatch(['verify', 'add-round', CHANGE, '--stdin'], {
      stdin: CLEAN_ROUND,
    })
    expect(added).toEqual({ change: CHANGE, round: 1 })

    await engine.dispatch(['verify', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD })
    expect(meta()).toContain(`verified_by: ${ACTOR}\n`)
  })
})

describe('蓋章守門與 argv 契約', () => {
  it('末輪未解必修 findings 讓 stamp 以語義化訊息拒絕，且不落任何章', async () => {
    const { engine, meta } = project(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: BLOCKING_ROUND })
    await expect(
      engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD }),
    ).rejects.toThrow(/CRITICAL|unresolved|must-fix/i)
    expect(meta()).not.toContain('reviewed_')
  })

  it('--accept 豁免必修條件後落章', async () => {
    const { engine, meta } = project(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: BLOCKING_ROUND })
    await engine.dispatch(['review', 'stamp', CHANGE, '--accept', '--stdin'], {
      stdin: STAMP_PAYLOAD,
    })
    expect(meta()).toContain(`reviewed_by: ${ACTOR}\n`)
  })

  it('--agent 落 reviewed_with', async () => {
    const { engine, meta } = project(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await engine.dispatch(['review', 'stamp', CHANGE, '--agent', 'claude', '--stdin'], {
      stdin: STAMP_PAYLOAD,
    })
    expect(meta()).toContain('reviewed_with: claude\n')
  })

  it('未給 actor 時 reviewed_by 回退專案的 git 身分', async () => {
    // 與 created_by 同一條解析：有 actor 用它，沒有就用 git 身分。
    const { root, engine, meta } = project()
    execFileSync('git', ['init', '-q'], { cwd: root })
    execFileSync('git', ['config', 'user.name', 'Fixture Dev'], { cwd: root })
    execFileSync('git', ['config', 'user.email', 'fixture@example.com'], { cwd: root })
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD })
    expect(meta()).toContain('reviewed_by: Fixture Dev <fixture@example.com>\n')
  })

  it('同一 engine 的 review 與 verify 並發蓋章不互吃', async () => {
    const { engine, meta } = project(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await engine.dispatch(['verify', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    // stamp 是整份 meta 的 read-modify-write：同實例內序列化，後蓋的章
    // 必須疊在前一個章之上，不得以舊快照覆蓋。
    await Promise.all([
      engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD }),
      engine.dispatch(['verify', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD }),
    ])
    expect(meta()).toContain(`reviewed_by: ${ACTOR}\n`)
    expect(meta()).toContain(`verified_by: ${ACTOR}\n`)
  })

  it('未支援的子動詞以 invalid_argv 拒絕，訊息說明只能在有 checkout 的一端執行', async () => {
    const { engine } = project(ACTOR)
    await expect(engine.dispatch(['review', 'prepare', CHANGE])).rejects.toMatchObject({
      code: 'invalid_argv',
      message: "'review prepare' only runs where a checkout exists — dispatch has none",
    })
  })

  it('stamp 的 stdin JSON 壞掉時以 invalid_argv 拒絕', async () => {
    const { root, engine } = project(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    const before = projectFiles(root)
    await expect(
      engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: 'not json' }),
    ).rejects.toMatchObject({ code: 'invalid_argv' })
    expect(projectFiles(root)).toEqual(before)
  })

  it('stamp 帶了 stdin 卻沒帶 --stdin 時以 invalid_argv 拒絕、不落章', async () => {
    const { engine, meta, ticket } = project(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    // 忘了旗標的 JSON 不能被靜默丟掉、改用空範圍蓋章。
    await expect(
      engine.dispatch(['review', 'stamp', CHANGE], { stdin: STAMP_PAYLOAD }),
    ).rejects.toMatchObject({ code: 'invalid_argv' })
    expect(ticket()).toContain('## Round 1')
    expect(meta()).not.toContain('reviewed_')
  })

  it('stamp 不帶 --stdin 時 scope 與 missing 讀作空清單', async () => {
    const { engine, meta, ticket } = project(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    // 工單聯集非空、scope 為空 → 引擎的分割守門拒絕（拒絕訊息由引擎產生）。
    await expect(engine.dispatch(['review', 'stamp', CHANGE])).rejects.toThrow()
    expect(ticket()).toContain('## Round 1')
    expect(meta()).not.toContain('reviewed_')
  })
})

// spec「讀取與丟棄工單」：show 與 CLI 的 --json 同內容；discard 之後沒有工單。
describe('讀取與丟棄工單', () => {
  for (const station of ['review', 'verify'] as const) {
    it(`${station} show 與 CLI 的 --json 相同，discard 後 show 查無工單`, async () => {
      const root = makeFsFixture()
      const engine = createEngine({ store: { type: 'fs', root }, actor: ACTOR })
      await engine.dispatch([station, 'add-round', CHANGE, '--stdin'], { stdin: BLOCKING_ROUND })
      await engine.dispatch([station, 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })

      const shown = (await engine.dispatch([station, 'show', CHANGE])) as {
        change: string
        rounds: unknown[]
        lastRound: { index: number }
      }
      const cli = execFileSync(cliBin, [station, 'show', CHANGE, '--json'], { cwd: root, encoding: 'utf8' })
      expect(shown).toEqual(JSON.parse(cli))
      expect(shown.change).toBe(CHANGE)
      expect(shown.rounds).toHaveLength(2)
      expect(shown.lastRound.index).toBe(2)

      const discarded = (await engine.dispatch([station, 'discard', CHANGE])) as { output: string }
      expect(discarded.output).toMatch(/^✓ Discarded (review|verification) ticket for change 'beta'\n$/)
      await expect(engine.dispatch([station, 'show', CHANGE])).rejects.toThrow(
        new RegExp(`no ${station} ticket for change '${CHANGE}'`),
      )
    })
  }

  it('show 帶出末輪的 scope，discard 刪掉工單檔', async () => {
    const { engine, ticket } = project(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    expect(await engine.dispatch(['review', 'show', CHANGE])).toMatchObject({
      change: CHANGE,
      lastRound: { index: 1, scope: [SCOPE_FILE] },
    })
    await engine.dispatch(['review', 'discard', CHANGE])
    expect(ticket()).toBeNull()
  })
})
