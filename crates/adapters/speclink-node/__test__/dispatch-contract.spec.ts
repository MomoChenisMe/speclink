// dispatch 的輸入輸出契約（spec: 壞工作流設定經 dispatch 拒絕 / CLI 與 dispatch
// 的錯誤分類一致）：錯誤碼出自引擎命令層的封閉註冊表，message 與 CLI 訊息相同。
import { execFileSync } from 'node:child_process'
import { writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

import { createEngine } from '../index.js'
import { cliBin, fixtureProject, makeFsFixture, memoryStore } from './helpers'

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

describe('dispatch change-metadata fail-closed (invalid_config)', () => {
  // 壞 .openspec.yaml 只存在於 fs store（JS host store 交付結構化 meta，
  // 無原始 YAML 文字）——兩案皆以 fs fixture 驗證。
  const brokenFixture = () => {
    const root = makeFsFixture()
    writeFileSync(
      join(root, 'openspec', 'changes', 'alpha', '.openspec.yaml'),
      ': : :\n\t bad yaml [unclosed\n',
    )
    return root
  }

  it('status of a corrupt-meta change rejects with invalid_config and the CLI message text', async () => {
    const root = brokenFixture()
    const engine = createEngine({ store: { type: 'fs', root } })
    const err = await rejection(engine.dispatch(['status', '--change', 'alpha', '--json']))
    expect(err.code).toBe('invalid_config')
    expect(err.message).toContain('invalid openspec/changes/alpha/.openspec.yaml: ')
    // 與 CLI 訊息文字相同：CLI stderr 為 `Error: <message>`。
    let cliStderr = ''
    try {
      execFileSync(cliBin, ['status', '--change', 'alpha'], {
        cwd: root,
        encoding: 'utf8',
        stdio: 'pipe',
      })
      expect.unreachable('CLI must exit non-zero on corrupt meta')
    } catch (e) {
      cliStderr = (e as { stderr?: string }).stderr ?? ''
    }
    expect(cliStderr.trim()).toBe(`Error: ${err.message}`)
  })

  it('list marks the corrupt item with metaError and keeps valid items clean', async () => {
    const engine = createEngine({ store: { type: 'fs', root: brokenFixture() } })
    const list = (await engine.dispatch(['list', '--sort', 'name', '--json'])) as {
      changes: Array<Record<string, unknown>>
    }
    expect(list.changes.map((c) => c.name)).toEqual(['alpha', 'beta'])
    const broken = list.changes.find((c) => c.name === 'alpha')!
    expect(typeof broken.metaError).toBe('string')
    expect((broken.metaError as string).length).toBeGreaterThan(0)
    const good = list.changes.find((c) => c.name === 'beta')!
    expect('metaError' in good).toBe(false)
  })
})
