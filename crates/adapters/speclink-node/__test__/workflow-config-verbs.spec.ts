// dispatch 的流程設定動詞（spec node-sdk「dispatch 的流程設定動詞」、command-runtime
// 「動詞覆蓋與跨入口一致性」的 workflow-config 情境）。宿主 Store 的寫入走選填方法
// writeWorkflowConfig；沒有它時，會寫入的動詞在動手前被拒絕。
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

import { createEngine } from '../index.js'
import { cliJson, fixtureProject, makeFsFixture, memoryStore, rejection } from './helpers'

const CONFIG = 'schema: spec-driven\nlocale: tw\ntdd: false\n'
const BAD_CONFIG = ': not yaml : [\n'

/** A host store holding `config`, with the names of every method called on it. */
function hostStore(config: string | undefined, options: { writable: boolean }) {
  const project = fixtureProject()
  project.workflowConfig = config
  const calls: string[] = []
  const written: string[] = []
  const base: Record<string, unknown> = { ...memoryStore(project) }
  if (options.writable) {
    base.writeWorkflowConfig = async (content: string) => {
      written.push(content)
      project.workflowConfig = content
    }
  }
  const store = Object.fromEntries(
    Object.entries(base).map(([name, method]) => [
      name,
      (...args: unknown[]) => {
        calls.push(name)
        return (method as (...a: unknown[]) => unknown)(...args)
      },
    ]),
  )
  const writes = () => calls.filter((name) => /^(write|create|delete|archive)/.test(name))
  return { store, project, calls, written, writes }
}

/** An fs fixture project whose workflow config is `config`. */
function fsProject(config: string): string {
  const root = makeFsFixture()
  mkdirSync(join(root, 'speclink'), { recursive: true })
  writeFileSync(join(root, '.speclink.yaml'), 'tools: []\n')
  writeFileSync(join(root, 'speclink', 'config.yaml'), config)
  return root
}

describe('workflow-config show 經命令執行層', () => {
  it('fs 形式的結果等於 CLI 的 --json', async () => {
    const root = fsProject(CONFIG)
    const engine = createEngine({ store: { type: 'fs', root } })
    const shown = await engine.dispatch(['workflow-config', 'show'])
    expect(shown).toEqual(cliJson(root, ['workflow-config', 'show', '--json']))
    expect(shown).toMatchObject({ schema: 'spec-driven', locale: 'tw', tdd: false, specLocale: null })
  })

  it('宿主 Store 形式對同一份內容得到同一個結果', async () => {
    const root = fsProject(CONFIG)
    const { store } = hostStore(CONFIG, { writable: false })
    const engine = createEngine({ store })
    expect(await engine.dispatch(['workflow-config', 'show', '--json'])).toEqual(
      cliJson(root, ['workflow-config', 'show', '--json']),
    )
  })

  it('壞的流程設定以 invalid_config 拒絕', async () => {
    const { store } = hostStore(BAD_CONFIG, { writable: true })
    const err = await rejection(createEngine({ store }).dispatch(['workflow-config', 'show']))
    expect(err.code).toBe('invalid_config')
    expect(err.message).toContain('speclink/config.yaml')
  })
})

describe('宿主 Store 提供 writeWorkflowConfig', () => {
  it('set 呼叫它正好一次，收到的內容裡 tdd 是 true', async () => {
    const { store, written, writes } = hostStore(CONFIG, { writable: true })
    const result = await createEngine({ store }).dispatch(['workflow-config', 'set', 'tdd', 'true'])
    expect(result).toEqual({ output: '✓ tdd = true\n' })
    expect(written).toEqual(['schema: spec-driven\nlocale: tw\ntdd: true\n'])
    expect(writes()).toEqual(['writeWorkflowConfig'])
  })

  it('讀 stdin 的動詞帶 --stdin 寫入內容', async () => {
    const { store, written } = hostStore(CONFIG, { writable: true })
    const engine = createEngine({ store })
    const result = await engine.dispatch(['workflow-config', 'rules', 'tasks', '--stdin'], {
      stdin: 'Use checkbox format\n',
    })
    expect(result).toEqual({ output: '✓ rules.tasks = 1 entries\n' })
    expect(written).toHaveLength(1)
    expect(written[0]).toContain('rules:\n  tasks:\n  - Use checkbox format\n')
    // 少了 --stdin 是 argv 的錯，內容不會被默默丟掉。
    const err = await rejection(engine.dispatch(['workflow-config', 'context']))
    expect(err.code).toBe('invalid_argv')
    expect(err.message).toBe('content is read from stdin — run: speclink workflow-config context --stdin')
  })

  it('寫入方法直接回傳（不是 Promise）也可以', async () => {
    // spec node-sdk：writeWorkflowConfig(content) 回傳值或 Promise 皆可。
    const { store, project } = hostStore(CONFIG, { writable: true })
    const received: string[] = []
    store.writeWorkflowConfig = (content: string) => {
      received.push(content)
      project.workflowConfig = content
    }
    const result = await createEngine({ store }).dispatch(['workflow-config', 'set', 'audit', 'true'])
    expect(result).toEqual({ output: '✓ audit = true\n' })
    expect(received).toHaveLength(1)
    expect(received[0]).toContain('audit: true\n')
  })

  it('帶 --stdin 卻沒有給 stdin 內容時以 invalid_argv 拒絕，不清掉現有的值', async () => {
    // 宿主漏傳 options.stdin 時，內容會被讀成空的；清單會被默默清空。
    const config = `${CONFIG}rules:\n  tasks:\n  - Use checkbox format\n`
    const { store, project, writes } = hostStore(config, { writable: true })
    const engine = createEngine({ store })
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
    expect(writes()).toEqual([])
    expect(project.workflowConfig).toBe(config)
    // 空字串是「刻意清掉」的寫法。
    const cleared = await engine.dispatch(['workflow-config', 'rules', 'tasks', '--stdin'], { stdin: '' })
    expect(cleared).toEqual({ output: '✓ rules.tasks removed\n' })
    // 寫明的 `tdd: false` 也被改寫函式拿掉：false 不落鍵。
    expect(project.workflowConfig).toBe('schema: spec-driven\nlocale: tw\n')
  })

  it('值沒有變時不呼叫寫入方法', async () => {
    const { store, written } = hostStore('schema: spec-driven\ntdd: true\n', { writable: true })
    const result = await createEngine({ store }).dispatch(['workflow-config', 'set', 'tdd', 'true'])
    expect(result).toEqual({ output: '✓ tdd = true\n' })
    expect(written).toEqual([])
  })

  it('不合法的值在寫入前被拒絕', async () => {
    const { store, written } = hostStore(CONFIG, { writable: true })
    const err = await rejection(
      createEngine({ store }).dispatch(['workflow-config', 'set', 'locale', 'fr']),
    )
    expect(err.code).toBe('invalid_config')
    expect(written).toEqual([])
  })

  it('寫入方法失敗時 dispatch 以該方法的錯誤拒絕', async () => {
    const { store } = hostStore(CONFIG, { writable: true })
    store.writeWorkflowConfig = async () => {
      throw Object.assign(new Error('the policy is locked'), { code: 'policy_locked' })
    }
    const err = await rejection(
      createEngine({ store }).dispatch(['workflow-config', 'set', 'tdd', 'true']),
    )
    expect(err.code).toBe('policy_locked')
    expect(err.message).toBe('writeWorkflowConfig: the policy is locked')
  })
})

describe('同一個引擎上並行的寫入', () => {
  it('一個接一個執行，兩個改動都留下', async () => {
    // 寫入是「讀、改、整份寫回」，宿主 Store 的介面沒有版本檢查：兩次寫入要是同時
    // 讀到舊內容，後寫的會蓋掉先寫的。
    const { store, project } = hostStore('schema: spec-driven\n', { writable: true })
    const read = store.readWorkflowConfig
    store.readWorkflowConfig = async () => {
      const text = await read()
      await new Promise((resolve) => setTimeout(resolve, 30))
      return text
    }
    const engine = createEngine({ store })
    await Promise.all([
      engine.dispatch(['workflow-config', 'set', 'tdd', 'true']),
      engine.dispatch(['workflow-config', 'set', 'audit', 'true']),
    ])
    expect(project.workflowConfig).toContain('tdd: true\n')
    expect(project.workflowConfig).toContain('audit: true\n')
  })
})

describe('宿主 Store 沒有 writeWorkflowConfig', () => {
  it('寫入動詞以 refused 拒絕，沒有任何寫入方法被呼叫', async () => {
    const { store, project, writes } = hostStore(CONFIG, { writable: false })
    const engine = createEngine({ store })
    for (const [argv, stdin] of [
      [['workflow-config', 'set', 'tdd', 'true'], undefined],
      [['workflow-config', 'context', '--stdin'], 'New context.\n'],
      [['workflow-config', 'review', 'baseline', 'replace'], undefined],
      [['workflow-config', 'subagent-model', 'review', 'claude', 'opus'], undefined],
    ] as Array<[string[], string | undefined]>) {
      const err = await rejection(engine.dispatch(argv, { stdin }))
      expect(err.code, argv.join(' ')).toBe('refused')
      expect(err.message, argv.join(' ')).toContain('writeWorkflowConfig')
    }
    expect(writes()).toEqual([])
    expect(project.workflowConfig).toBe(CONFIG)
  })

  it('讀取與 dry-run 不需要它', async () => {
    const { store, writes } = hostStore(CONFIG, { writable: false })
    const engine = createEngine({ store })
    await expect(engine.dispatch(['workflow-config', 'show'])).resolves.toMatchObject({ tdd: false })
    const dry = (await engine.dispatch(['workflow-config', 'set', 'tdd', 'true', '--dry-run'])) as {
      output: string
    }
    expect(dry.output).toBe(
      '--- a/speclink/config.yaml\n+++ b/speclink/config.yaml\n@@ -1,3 +1,3 @@\n schema: spec-driven\n locale: tw\n-tdd: false\n+tdd: true\n',
    )
    expect(writes()).toEqual([])
  })
})

describe('fs 形式', () => {
  it('直接寫入規格資料夾的 config.yaml', async () => {
    const root = fsProject(CONFIG)
    const engine = createEngine({ store: { type: 'fs', root } })
    const result = await engine.dispatch(['workflow-config', 'set', 'locale', 'ja'])
    expect(result).toEqual({ output: '✓ locale = ja\n' })
    // 寫明的 `tdd: false` 被改寫函式拿掉：false 不落鍵（既有的寫入語意）。
    expect(readFileSync(join(root, 'speclink', 'config.yaml'), 'utf8')).toBe(
      'schema: spec-driven\nlocale: ja\n',
    )
  })
})
