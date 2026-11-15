// dispatch 的流程設定動詞（spec node-sdk「dispatch 的流程設定動詞」、command-runtime
// 「動詞覆蓋與跨入口一致性」的 workflow-config 情境）。會寫入的動詞直接寫規格資料夾的
// config.yaml，寫入後的內容和 CLI 在同一個專案執行同一個動詞的結果相同。
import { execFileSync } from 'node:child_process'
import { readFileSync, statSync, utimesSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

import { BAD_CONFIG, cliBin, cliJson, fsEngine, makeFsFixture, projectFiles, rejection } from './helpers'

const CONFIG = 'schema: spec-driven\nlocale: tw\ntdd: false\n'
/** The workflow config, as `projectFiles` names it. */
const CONFIG_FILE = 'speclink/config.yaml'
const configPath = (root: string) => join(root, ...CONFIG_FILE.split('/'))
const configOf = (root: string) => readFileSync(configPath(root), 'utf8')

/** An fs fixture project whose workflow config is `config`. */
function fsProject(config: string): string {
  const root = makeFsFixture()
  writeFileSync(join(root, '.speclink.yaml'), 'tools: []\n')
  writeFileSync(configPath(root), config)
  return root
}

describe('workflow-config show 經命令執行層', () => {
  it.each([[['workflow-config', 'show']], [['workflow-config', 'show', '--json']]])(
    'dispatch(%j) 的結果等於 CLI 的 --json',
    async (argv) => {
      const root = fsProject(CONFIG)
      const shown = await fsEngine(root).dispatch(argv)
      expect(shown).toEqual(cliJson(root, ['workflow-config', 'show', '--json']))
      expect(shown).toMatchObject({ schema: 'spec-driven', locale: 'tw', tdd: false, specLocale: null })
    },
  )

  // spec「壞的流程設定經 dispatch 拒絕」
  it('壞的流程設定以 invalid_config 拒絕', async () => {
    const err = await rejection(fsEngine(fsProject(BAD_CONFIG)).dispatch(['workflow-config', 'show']))
    expect(err.code).toBe('invalid_config')
    expect(err.message).toContain('speclink/config.yaml')
  })
})

describe('會寫入的動詞', () => {
  // spec「經 dispatch 設定政策欄位」。寫明的 `tdd: false` 在改別的欄位時被改寫函式拿掉：
  // false 不落鍵（既有的寫入語意）。
  it.each([
    ['tdd', 'true', 'schema: spec-driven\nlocale: tw\ntdd: true\n'],
    ['audit', 'true', 'schema: spec-driven\n\naudit: true\n\nlocale: tw\n'],
    ['locale', 'ja', 'schema: spec-driven\nlocale: ja\n'],
  ])('set %s %s 只改 config.yaml，內容和 CLI 寫出的相同', async (key, value, written) => {
    const [root, cliRoot] = [fsProject(CONFIG), fsProject(CONFIG)]
    const before = projectFiles(root)
    const result = await fsEngine(root).dispatch(['workflow-config', 'set', key, value])
    expect(result).toEqual({ output: `✓ ${key} = ${value}\n` })
    expect(projectFiles(root)).toEqual({ ...before, [CONFIG_FILE]: written })
    execFileSync(cliBin, ['workflow-config', 'set', key, value], { cwd: cliRoot, encoding: 'utf8' })
    expect(configOf(root)).toBe(configOf(cliRoot))
  })

  // spec「dry-run 不寫入」
  it('帶 --dry-run 時回傳差異，config.yaml 逐位元不變', async () => {
    const root = fsProject(CONFIG)
    const before = projectFiles(root)
    const dry = (await fsEngine(root).dispatch(['workflow-config', 'set', 'tdd', 'true', '--dry-run'])) as {
      output: string
    }
    expect(dry.output).toBe(
      '--- a/speclink/config.yaml\n+++ b/speclink/config.yaml\n@@ -1,3 +1,3 @@\n schema: spec-driven\n locale: tw\n-tdd: false\n+tdd: true\n',
    )
    expect(projectFiles(root)).toEqual(before)
  })

  it('讀 stdin 的動詞帶 --stdin 寫入內容', async () => {
    const root = fsProject(CONFIG)
    const engine = fsEngine(root)
    const result = await engine.dispatch(['workflow-config', 'rules', 'tasks', '--stdin'], {
      stdin: 'Use checkbox format\n',
    })
    expect(result).toEqual({ output: '✓ rules.tasks = 1 entries\n' })
    expect(configOf(root)).toContain('rules:\n  tasks:\n  - Use checkbox format\n')
    // 少了 --stdin 是 argv 的錯，內容不會被默默丟掉。
    const err = await rejection(engine.dispatch(['workflow-config', 'context']))
    expect(err.code).toBe('invalid_argv')
    expect(err.message).toBe('content is read from stdin — run: speclink workflow-config context --stdin')
  })

  // spec「帶 --stdin 卻沒有 stdin 內容時拒絕」
  it('帶 --stdin 卻沒有給 stdin 內容時以 invalid_argv 拒絕，不清掉現有的值', async () => {
    // 呼叫端漏傳 options.stdin 時，內容會被讀成空的；清單會被默默清空。
    const config = `${CONFIG}rules:\n  tasks:\n  - Use checkbox format\n`
    const root = fsProject(config)
    const before = projectFiles(root)
    const engine = fsEngine(root)
    for (const argv of [
      ['workflow-config', 'rules', 'tasks', '--stdin'],
      ['workflow-config', 'context', '--stdin'],
      ['workflow-config', 'review', 'critical', '--stdin', '--dry-run'],
      ['workflow-config', 'subagent-model', 'choices', 'codex', '--stdin'],
    ]) {
      const err = await rejection(engine.dispatch(argv))
      expect(err.code, argv.join(' ')).toBe('invalid_argv')
      expect(err.message, argv.join(' ')).toContain('--stdin but no stdin content')
    }
    expect(projectFiles(root)).toEqual(before)
    // 空字串是「刻意清掉」的寫法。
    const cleared = await engine.dispatch(['workflow-config', 'rules', 'tasks', '--stdin'], { stdin: '' })
    expect(cleared).toEqual({ output: '✓ rules.tasks removed\n' })
    // 寫明的 `tdd: false` 也被改寫函式拿掉：false 不落鍵。
    expect(configOf(root)).toBe('schema: spec-driven\nlocale: tw\n')
  })

  it('值沒有變時不寫檔', async () => {
    const root = fsProject('schema: spec-driven\ntdd: true\n')
    // 一個舊的檔案時間：有寫入的話它會變。
    const old = new Date('2020-01-01T00:00:00Z')
    utimesSync(configPath(root), old, old)
    const result = await fsEngine(root).dispatch(['workflow-config', 'set', 'tdd', 'true'])
    expect(result).toEqual({ output: '✓ tdd = true\n' })
    expect(configOf(root)).toBe('schema: spec-driven\ntdd: true\n')
    expect(statSync(configPath(root)).mtimeMs).toBe(old.getTime())
  })

  it('不合法的值在寫入前被拒絕', async () => {
    const root = fsProject(CONFIG)
    const before = projectFiles(root)
    const err = await rejection(fsEngine(root).dispatch(['workflow-config', 'set', 'locale', 'fr']))
    expect(err.code).toBe('invalid_config')
    expect(projectFiles(root)).toEqual(before)
  })
})

describe('同一個引擎上並行的寫入', () => {
  it('一個接一個執行，每個改動都留下', async () => {
    // 寫入是「讀、改、整份寫回」：兩次寫入要是同時讀到舊內容，後寫的會蓋掉先寫的。
    // 五個寫入同時送出：引擎不讓它們排隊的話，這個測試每次都會少掉幾個鍵。
    const root = fsProject('schema: spec-driven\n')
    const engine = fsEngine(root)
    const writes: Array<[string, string]> = [
      ['tdd', 'true'],
      ['audit', 'true'],
      ['worktree', 'true'],
      ['locale', 'ja'],
      ['spec_locale', 'en'],
    ]
    await Promise.all(writes.map(([key, value]) => engine.dispatch(['workflow-config', 'set', key, value])))
    for (const [key, value] of writes) {
      expect(configOf(root), key).toContain(`${key}: ${value}\n`)
    }
  })
})
