// dispatch 的輸入輸出契約（spec: 壞工作流設定經 dispatch 拒絕 / CLI 與 dispatch
// 的錯誤分類一致）：錯誤碼出自引擎命令層的封閉註冊表，message 與 CLI 訊息相同。
import { describe, expect, it } from 'vitest'

import { createEngine } from '../index.js'
import { fixtureProject, memoryStore } from './helpers'

async function rejection(promise: Promise<unknown>): Promise<Error & { code?: string }> {
  let thrown: (Error & { code?: string }) | undefined
  try {
    await promise
  } catch (e) {
    thrown = e as Error & { code?: string }
  }
  expect(thrown).toBeInstanceOf(Error)
  return thrown!
}

describe('dispatch config fail-closed (invalid_config)', () => {
  it('a broken workflow-config text rejects new change with code invalid_config', async () => {
    const project = fixtureProject()
    project.workflowConfig = ': not yaml : [\n'
    const engine = createEngine({ store: memoryStore(project) })
    const err = await rejection(engine.dispatch(['new', 'change', 'demo']))
    expect(err.code).toBe('invalid_config')
    expect(err.message).toContain('openspec/config.yaml')
    expect(err.message).toContain('invalid')
    // Fail-closed: the default-schema path must NOT have created the change.
    const list = (await engine.dispatch(['list', '--sort', 'name', '--json'])) as {
      changes: Array<{ name: string }>
    }
    expect(list.changes.map((c) => c.name)).not.toContain('demo')
  })

  it('an explicit --schema bypasses the workflow-config read and still works', async () => {
    // The config document is only consulted for the DEFAULT schema — an
    // explicit name never reads it, so a broken file cannot block this path.
    const project = fixtureProject()
    project.workflowConfig = ': not yaml : [\n'
    const engine = createEngine({ store: memoryStore(project) })
    const result = (await engine.dispatch([
      'new',
      'change',
      'demo-explicit',
      '--schema',
      'spec-driven',
    ])) as { output: string }
    expect(result.output).toContain('Created change: demo-explicit')
  })
})

describe('dispatch error classification matches the CLI', () => {
  it('status of a missing change rejects with not_found and the CLI message text', async () => {
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    const err = await rejection(engine.dispatch(['status', '--change', 'ghost', '--json']))
    expect(err.code).toBe('not_found')
    expect(err.message).toBe("Change 'ghost' not found.")
  })
})
