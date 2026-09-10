// dispatch 的蓋章動詞（spec：dispatch 的蓋章動詞）。宿主 Store 形式沒有工作樹，
// 指紋是宿主自己的真相——scope／missing 走 stdin 的 JSON（server stamp request
// body 的 scope／missing 子集；accept／agent 走 argv 旗標）。
import { describe, expect, it } from 'vitest'

import { createEngine } from '../index.js'
import { fixtureProject, memoryStore } from './helpers'

const ACTOR = 'Rev <rev@example.com>'
// beta 的任務全數完成——蓋章守門的前提。
const CHANGE = 'beta'
const SCOPE_FILE = 'crates/b/src/util.rs'
// SUGGESTION 不擋章；CRITICAL 擋。
const CLEAN_ROUND = `**Scope**: ${SCOPE_FILE}\n\n- [SUGGESTION] ${SCOPE_FILE} — rename helper\n`
const BLOCKING_ROUND = `**Scope**: ${SCOPE_FILE}\n\n- [CRITICAL] ${SCOPE_FILE} — unwrap on user input\n`
const STAMP_PAYLOAD = JSON.stringify({ scope: [{ path: SCOPE_FILE, hash: '0f9c' }] })

function engineWithActor(actor?: string) {
  const store = memoryStore(fixtureProject())
  const engine = actor === undefined ? createEngine({ store }) : createEngine({ store, actor })
  return { store, engine }
}

const metaOf = (store: ReturnType<typeof memoryStore>) =>
  store.readArtifact(CHANGE, '.openspec.yaml').then((m) => m ?? '')

describe('review 蓋章鏈', () => {
  it('add-round 建立工單、stamp 以建構期 actor 落 reviewed_by', async () => {
    const { store, engine } = engineWithActor(ACTOR)
    const added = await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], {
      stdin: CLEAN_ROUND,
    })
    expect(added).toEqual({ change: CHANGE, round: 1 })
    expect(await store.readArtifact(CHANGE, 'review.md')).toContain('## Round 1')

    const stamped = await engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], {
      stdin: STAMP_PAYLOAD,
    })
    expect(stamped).toEqual({ change: CHANGE })
    const meta = await metaOf(store)
    expect(meta).toContain(`reviewed_by: ${ACTOR}\n`)
    // 蓋章是 read-modify-write：既有欄位必須原樣保留，不得整份洗掉。
    expect(meta).toContain('schema: spec-driven\n')
    expect(meta).toContain('created_by: tester\n')
    // 蓋章效果的另一半：工單刪除。
    expect(await store.readArtifact(CHANGE, 'review.md')).toBeNull()
  })
})

describe('verify 蓋章鏈', () => {
  it('同一 actor 落 verified_by', async () => {
    const { store, engine } = engineWithActor(ACTOR)
    const added = await engine.dispatch(['verify', 'add-round', CHANGE, '--stdin'], {
      stdin: CLEAN_ROUND,
    })
    expect(added).toEqual({ change: CHANGE, round: 1 })

    await engine.dispatch(['verify', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD })
    expect(await metaOf(store)).toContain(`verified_by: ${ACTOR}\n`)
  })
})

describe('蓋章守門與 argv 契約', () => {
  it('末輪未解必修 findings 讓 stamp 以語義化訊息拒絕，且不落任何章', async () => {
    const { store, engine } = engineWithActor(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: BLOCKING_ROUND })
    await expect(
      engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD }),
    ).rejects.toThrow(/CRITICAL|unresolved|must-fix/i)
    expect(await metaOf(store)).not.toContain('reviewed_by:')
  })

  it('--accept 豁免必修條件後落章', async () => {
    const { store, engine } = engineWithActor(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: BLOCKING_ROUND })
    await engine.dispatch(['review', 'stamp', CHANGE, '--accept', '--stdin'], {
      stdin: STAMP_PAYLOAD,
    })
    expect(await metaOf(store)).toContain(`reviewed_by: ${ACTOR}\n`)
  })

  it('--agent 落 reviewed_with', async () => {
    const { store, engine } = engineWithActor(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await engine.dispatch(['review', 'stamp', CHANGE, '--agent', 'claude', '--stdin'], {
      stdin: STAMP_PAYLOAD,
    })
    expect(await metaOf(store)).toContain('reviewed_with: claude\n')
  })

  it('未給 actor 時蓋章無 _by 欄位（宿主 Store 無回退）', async () => {
    const { store, engine } = engineWithActor()
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD })
    const meta = await metaOf(store)
    expect(meta).toContain('reviewed_at:')
    expect(meta).not.toContain('reviewed_by:')
  })

  it('宿主 Store 未實作 deleteArtifact 時蓋章在動手前拒絕、工單不動', async () => {
    const store = memoryStore(fixtureProject())
    // deleteArtifact 是蓋章前置的選配方法——拿掉它，其餘動詞照走。
    delete (store as Partial<typeof store>).deleteArtifact
    const engine = createEngine({ store, actor: ACTOR })
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await expect(
      engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD }),
    ).rejects.toThrow(/deleteArtifact/)
    // 前置檢查擋在任何寫入之前：工單還在、章沒落。
    expect(await store.readArtifact(CHANGE, 'review.md')).toContain('## Round 1')
    expect(await metaOf(store)).not.toContain('reviewed_by:')
    await expect(engine.dispatch(['list', '--json'])).resolves.toBeTruthy()
  })

  it('宿主 Store 未實作 writeChangeMeta 時蓋章在刪工單前拒絕', async () => {
    const store = memoryStore(fixtureProject())
    // 蓋章順序是先刪工單再寫 meta——缺 writeChangeMeta 若不擋在前面，
    // 工單會先被刪掉、章卻落不下來（全部輪次永久遺失）。
    delete (store as Partial<typeof store>).writeChangeMeta
    const engine = createEngine({ store, actor: ACTOR })
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await expect(
      engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD }),
    ).rejects.toThrow(/writeChangeMeta/)
    expect(await store.readArtifact(CHANGE, 'review.md')).toContain('## Round 1')
  })

  it('宿主 Store 未實作 readChangeMeta 時蓋章拒絕，不得靜默洗掉既有 meta', async () => {
    const store = memoryStore(fixtureProject())
    // 缺 readChangeMeta 時橋接層讀作「無 metadata」，蓋章寫回會把
    // schema／created_by 整份丟掉——必須拒絕，不能靜默資料流失。
    delete (store as Partial<typeof store>).readChangeMeta
    const engine = createEngine({ store, actor: ACTOR })
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await expect(
      engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD }),
    ).rejects.toThrow(/readChangeMeta/)
    expect(await metaOf(store)).toContain('created_by: tester\n')
  })

  it('同一 engine 的 review 與 verify 並發蓋章不互吃', async () => {
    const { store, engine } = engineWithActor(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await engine.dispatch(['verify', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    // stamp 是整份 meta 的 read-modify-write：同實例內序列化，後蓋的章
    // 必須疊在前一個章之上，不得以舊快照覆蓋。
    await Promise.all([
      engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD }),
      engine.dispatch(['verify', 'stamp', CHANGE, '--stdin'], { stdin: STAMP_PAYLOAD }),
    ])
    const meta = await metaOf(store)
    expect(meta).toContain(`reviewed_by: ${ACTOR}\n`)
    expect(meta).toContain(`verified_by: ${ACTOR}\n`)
  })

  it('未支援的子動詞以 invalid_argv 拒絕', async () => {
    const { engine } = engineWithActor(ACTOR)
    await expect(engine.dispatch(['review', 'show', CHANGE])).rejects.toMatchObject({
      code: 'invalid_argv',
    })
    await expect(engine.dispatch(['review', 'show', CHANGE])).rejects.toThrow(/add-round.*stamp/)
  })

  it('stamp 的 stdin JSON 壞掉時以 invalid_argv 拒絕', async () => {
    const { engine } = engineWithActor(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    await expect(
      engine.dispatch(['review', 'stamp', CHANGE, '--stdin'], { stdin: 'not json' }),
    ).rejects.toMatchObject({ code: 'invalid_argv' })
  })

  it('stamp 不帶 --stdin 時 scope 與 missing 讀作空清單', async () => {
    const { engine } = engineWithActor(ACTOR)
    await engine.dispatch(['review', 'add-round', CHANGE, '--stdin'], { stdin: CLEAN_ROUND })
    // 工單聯集非空、scope 為空 → 引擎的分割守門拒絕（拒絕訊息由引擎產生）。
    await expect(engine.dispatch(['review', 'stamp', CHANGE])).rejects.toThrow()
  })
})
