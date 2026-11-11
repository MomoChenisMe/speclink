// dispatch 的輸入輸出契約（spec: 壞工作流設定經 dispatch 拒絕 / CLI 與 dispatch
// 的錯誤分類一致）：錯誤碼出自引擎命令層的封閉註冊表，message 與 CLI 訊息相同。
import { execFileSync, spawnSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, utimesSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

import { createEngine } from '../index.js'
import { cliBin, fixtureProject, makeFsFixture, memoryStore, rejection, type MemoryProject } from './helpers'

describe('dispatch config fail-closed (invalid_config)', () => {
  it('a broken workflow-config text rejects new change with code invalid_config', async () => {
    const project = fixtureProject()
    project.workflowConfig = ': not yaml : [\n'
    const engine = createEngine({ store: memoryStore(project) })
    const err = await rejection(engine.dispatch(['new', 'change', 'demo']))
    expect(err.code).toBe('invalid_config')
    expect(err.message).toContain('speclink/config.yaml')
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
  // 壞 .meta.yaml 只存在於 fs store（JS host store 交付結構化 meta，
  // 無原始 YAML 文字）——兩案皆以 fs fixture 驗證。
  const brokenFixture = () => {
    const root = makeFsFixture()
    writeFileSync(
      join(root, 'speclink', 'changes', 'alpha', '.meta.yaml'),
      ': : :\n\t bad yaml [unclosed\n',
    )
    return root
  }

  it('status of a corrupt-meta change rejects with invalid_config and the CLI message text', async () => {
    const root = brokenFixture()
    const engine = createEngine({ store: { type: 'fs', root } })
    const err = await rejection(engine.dispatch(['status', '--change', 'alpha', '--json']))
    expect(err.code).toBe('invalid_config')
    expect(err.message).toContain('invalid speclink/changes/alpha/.meta.yaml: ')
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

describe('fs form honors an explicit spec_dir in .speclink.yaml', () => {
  it('reads the folder .speclink.yaml names when specDir is not given', async () => {
    const root = mkdtempSync(join(tmpdir(), 'speclink-node-specdir-'))
    try {
      writeFileSync(join(root, '.speclink.yaml'), 'spec_dir: docs/specs\n')
      const change = join(root, 'docs', 'specs', 'changes', 'gamma')
      mkdirSync(change, { recursive: true })
      writeFileSync(join(change, '.meta.yaml'), 'schema: spec-driven\n')
      const engine = createEngine({ store: { type: 'fs', root } })
      const list = (await engine.dispatch(['list', '--json'])) as { changes: Array<{ name: string }> }
      expect(list.changes.map((c) => c.name)).toEqual(['gamma'])
    } finally {
      rmSync(root, { recursive: true, force: true })
    }
  })
})

// dispatch 的 argv 由與 CLI 共用的動詞解析處理（spec node-sdk「dispatch 的輸入輸出
// 契約」）：接受與拒絕和 CLI 相同，錯誤文字取自 CLI 的 stderr（不含 ANSI 色碼）。
describe('dispatch argv goes through the parsing the CLI uses', () => {
  /** Run the CLI in `cwd`; return its exit code, stdout and stderr. */
  function cli(cwd: string, args: string[]): { code: number; stdout: string; stderr: string } {
    try {
      const stdout = execFileSync(cliBin, args, { cwd, encoding: 'utf8', stdio: 'pipe' })
      return { code: 0, stdout, stderr: '' }
    } catch (e) {
      const err = e as { status?: number; stdout?: string; stderr?: string }
      return { code: err.status ?? -1, stdout: err.stdout ?? '', stderr: err.stderr ?? '' }
    }
  }

  it('an unknown flag rejects with invalid_argv and the CLI error text', async () => {
    const root = makeFsFixture()
    const engine = createEngine({ store: { type: 'fs', root } })
    const err = await rejection(engine.dispatch(['list', '--no-such-flag']))
    expect(err.code).toBe('invalid_argv')
    const run = cli(root, ['list', '--no-such-flag'])
    expect(run.code).not.toBe(0)
    expect(err.message).toBe(run.stderr.trimEnd())
    expect(err.message).not.toContain('\u001b')
  })

  it('extra positionals reject with invalid_argv, as in the CLI', async () => {
    const root = makeFsFixture()
    const engine = createEngine({ store: { type: 'fs', root } })
    const err = await rejection(engine.dispatch(['status', 'extra-one', 'extra-two']))
    expect(err.code).toBe('invalid_argv')
    const run = cli(root, ['status', 'extra-one', 'extra-two'])
    expect(run.code).not.toBe(0)
    expect(err.message).toBe(run.stderr.trimEnd())
  })

  it('--sort=name reads the same as --sort name', async () => {
    const root = makeFsFixture()
    // beta is the newest change, so the default (modified) order differs from name order.
    const later = new Date(Date.now() + 60_000)
    for (const f of ['.meta.yaml', 'proposal.md', 'tasks.md']) {
      utimesSync(join(root, 'speclink', 'changes', 'beta', f), later, later)
    }
    const engine = createEngine({ store: { type: 'fs', root } })
    const names = (v: unknown) => (v as { changes: Array<{ name: string }> }).changes.map((c) => c.name)
    expect(names(await engine.dispatch(['list']))).toEqual(['beta', 'alpha'])
    const spaced = await engine.dispatch(['list', '--sort', 'name'])
    expect(names(spaced)).toEqual(['alpha', 'beta'])
    expect(await engine.dispatch(['list', '--sort=name'])).toEqual(spaced)
  })

  it('--json does not change the returned shape', async () => {
    const engine = createEngine({ store: { type: 'fs', root: makeFsFixture() } })
    expect(await engine.dispatch(['list', '--json'])).toEqual(await engine.dispatch(['list']))
  })

  it('--help rejects with invalid_argv and the verb help text', async () => {
    const root = makeFsFixture()
    const engine = createEngine({ store: { type: 'fs', root } })
    const err = await rejection(engine.dispatch(['list', '--help']))
    expect(err.code).toBe('invalid_argv')
    expect(err.message).toBe(cli(root, ['list', '--help']).stdout.trimEnd())
  })

  it('a verb that needs a checkout says so', async () => {
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    const err = await rejection(engine.dispatch(['drift', 'add-x']))
    expect(err.code).toBe('invalid_argv')
    expect(err.message).toMatch(/drift/)
    expect(err.message).toMatch(/checkout/)
  })

  it('instructions --skill names the CLI, not a checkout, as where it runs', async () => {
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    const err = await rejection(engine.dispatch(['instructions', '--skill', 'tdd']))
    expect(err.code).toBe('invalid_argv')
    expect(err.message).toMatch(/instructions --skill/)
    expect(err.message).toMatch(/built into the CLI/)
    expect(err.message).not.toMatch(/checkout/)
  })

  // spec node-sdk「給了 stdin 卻沒帶 --stdin」
  it('stdin without --stdin rejects a verb that reads stdin only when flagged', async () => {
    const store = memoryStore(fixtureProject())
    const writes: string[] = []
    const write = store.writeArtifact.bind(store)
    store.writeArtifact = async (change: string, artifact: string, content: string) => {
      writes.push(`${change}/${artifact}`)
      return write(change, artifact, content)
    }
    const engine = createEngine({ store })
    const err = await rejection(
      engine.dispatch(['new', 'artifact', 'proposal', '--change', 'beta'], { stdin: '## Why\n' }),
    )
    expect(err.code).toBe('invalid_argv')
    expect(err.message).toMatch(/--stdin/)
    expect(writes).toEqual([])
  })

  // spec node-sdk「蓋章的 --stdin 只有 dispatch 接受」
  it('stamp --stdin is accepted by dispatch and rejected by the CLI', async () => {
    const scopeFile = 'crates/b/src/util.rs'
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    await engine.dispatch(['review', 'add-round', 'beta', '--stdin'], {
      stdin: `**Scope**: ${scopeFile}\n\n- [SUGGESTION] ${scopeFile} — rename helper\n`,
    })
    const stamped = await engine.dispatch(['review', 'stamp', 'beta', '--stdin'], {
      stdin: JSON.stringify({ scope: [{ path: scopeFile, hash: '0f9c' }] }),
    })
    expect(stamped).toEqual({ change: 'beta' })

    const root = makeFsFixture()
    const meta = join(root, 'speclink', 'changes', 'beta', '.meta.yaml')
    const before = readFileSync(meta, 'utf8')
    const run = cli(root, ['review', 'stamp', 'beta', '--stdin'])
    expect(run.code).toBe(2)
    expect(run.stderr).toMatch(/unexpected argument '--stdin'/)
    expect(readFileSync(meta, 'utf8')).toBe(before)
  })

  // spec command-runtime「動詞文字的解析只有一份」的兩個情境，argv 逐字照抄 spec。
  it('speclink list --sort name --json and dispatch list --sort name give the same changes', async () => {
    const root = makeFsFixture()
    const engine = createEngine({ store: { type: 'fs', root } })
    const names = (v: unknown) => (v as { changes: Array<{ name: string }> }).changes.map((c) => c.name)
    const run = cli(root, ['list', '--sort', 'name', '--json'])
    expect(run.code).toBe(0)
    expect(names(await engine.dispatch(['list', '--sort', 'name']))).toEqual(names(JSON.parse(run.stdout)))
  })

  it('status --no-such-flag rejects in both entries with the same text', async () => {
    const root = makeFsFixture()
    const engine = createEngine({ store: { type: 'fs', root } })
    const err = await rejection(engine.dispatch(['status', '--no-such-flag']))
    expect(err.code).toBe('invalid_argv')
    const run = cli(root, ['status', '--no-such-flag'])
    expect(run.code).not.toBe(0)
    expect(err.message).toBe(run.stderr.trimEnd())
    expect(err.message).not.toContain('\u001b')
  })
})

// dispatch 涵蓋全部有引擎命令的動詞（spec node-sdk「dispatch 的輸入輸出契約」、
// command-runtime「動詞的輸出只有一份」）：回傳內容和 CLI 出自同一份輸出定義。
describe('dispatch covers every verb that has an engine command', () => {
  /** Run the CLI in `cwd`; stdout and stderr come back whatever the exit code is. */
  function cliRun(
    cwd: string,
    args: string[],
    input = '',
  ): { code: number; stdout: string; stderr: string } {
    const run = spawnSync(cliBin, args, { cwd, encoding: 'utf8', input })
    return { code: run.status ?? -1, stdout: run.stdout, stderr: run.stderr }
  }

  /** The fs fixture plus a change named `demo`, the name the spec scenarios use. */
  function demoFixture(): string {
    const root = makeFsFixture()
    const demo = join(root, 'speclink', 'changes', 'demo')
    mkdirSync(join(demo, 'specs', 'user-auth'), { recursive: true })
    writeFileSync(join(demo, '.meta.yaml'), 'schema: spec-driven\ncreated: "2026-07-03"\ncreated_by: tester\n')
    writeFileSync(join(demo, 'proposal.md'), '## Why\n\nA demo change for the dispatch tests.\n')
    writeFileSync(join(demo, 'tasks.md'), '## 1. Work\n\n- [ ] 1.1 First task\n- [ ] 1.2 Second task\n')
    writeFileSync(
      join(demo, 'specs', 'user-auth', 'spec.md'),
      '## ADDED Requirements\n\n### Requirement: Logout\nUsers SHALL be able to log out.\n\n#### Scenario: logout\n- **WHEN** the user logs out\n- **THEN** the session ends\n',
    )
    return root
  }

  /**
   * A value two runs can compare: every mention of the fixture root is
   * replaced, and so is every date (a run across midnight writes two days).
   */
  const portable = (value: unknown, root: string): unknown =>
    JSON.parse(
      JSON.stringify(value)
        .split(root)
        .join('<root>')
        .replace(/\d{4}-\d{2}-\d{2}/g, '<date>'),
    )

  const fsEngine = (root: string) => createEngine({ store: { type: 'fs', root } })

  it('show resolves to what speclink show --json prints', async () => {
    const root = demoFixture()
    const sdk = await fsEngine(root).dispatch(['show', 'demo'])
    expect(sdk).toEqual(JSON.parse(cliRun(root, ['show', 'demo', '--json']).stdout))
    expect((sdk as { name: string }).name).toBe('demo')
  })

  it('validate resolves to what speclink validate --json prints', async () => {
    const root = demoFixture()
    const sdk = await fsEngine(root).dispatch(['validate', 'demo'])
    const run = cliRun(root, ['validate', 'demo', '--json'])
    expect(sdk).toEqual(JSON.parse(run.stdout))
    expect(sdk).toEqual([{ change: 'demo', errors: [], valid: true, warnings: [] }])
  })

  it('validate of an invalid change resolves to the same value the CLI prints before it fails', async () => {
    const root = demoFixture()
    // A MODIFIED requirement that drops a scenario of the canonical spec does not validate.
    writeFileSync(
      join(root, 'speclink', 'changes', 'demo', 'specs', 'user-auth', 'spec.md'),
      '## MODIFIED Requirements\n\n### Requirement: Login\nUsers SHALL be able to log in fast.\n\n#### Scenario: fast login\n- **WHEN** credentials are valid\n- **THEN** a session is created at once\n',
    )
    const sdk = (await fsEngine(root).dispatch(['validate', 'demo'])) as Array<{ valid: boolean }>
    const run = cliRun(root, ['validate', 'demo', '--json'])
    expect(run.code).not.toBe(0)
    expect(sdk).toEqual(JSON.parse(run.stdout))
    expect(sdk[0].valid).toBe(false)
  })

  it('artifact cat resolves to { output } with the CLI stdout', async () => {
    const root = demoFixture()
    const sdk = await fsEngine(root).dispatch(['artifact', 'cat', 'proposal', '--change', 'demo'])
    const run = cliRun(root, ['artifact', 'cat', 'proposal', '--change', 'demo', '--no-color'])
    expect(sdk).toEqual({ output: run.stdout })
    expect(run.stdout).toBe('## Why\n\nA demo change for the dispatch tests.\n')
  })

  it('task done resolves to what speclink task done --json prints, and an already-done task rejects', async () => {
    const [sdkRoot, cliRoot] = [demoFixture(), demoFixture()]
    const engine = fsEngine(sdkRoot)
    const sdk = await engine.dispatch(['task', 'done', '1', '--change', 'demo'])
    expect(sdk).toEqual(JSON.parse(cliRun(cliRoot, ['task', 'done', '1', '--change', 'demo', '--json']).stdout))
    expect(sdk).toEqual({ change: 'demo', status: 'done', task_desc: '1.1 First task', task_id: '1' })
    expect(readFileSync(join(sdkRoot, 'speclink', 'changes', 'demo', 'tasks.md'), 'utf8')).toContain('- [x] 1.1')
    const err = await rejection(engine.dispatch(['task', 'done', '1', '--change', 'demo']))
    expect(err.message).toBe('Task 1 is already done')
    expect(err.code).toBe('error')
  })

  // spec「stderr 的提示經 stderr 欄位回傳」：beta 的任務全完成、沒有任務證據。
  it('archive resolves to { output, stderr } with the CLI stdout and stderr', async () => {
    const [sdkRoot, cliRoot] = [demoFixture(), demoFixture()]
    const sdk = (await fsEngine(sdkRoot).dispatch(['archive', 'beta', '-y'])) as {
      output: string
      stderr: string
    }
    const run = cliRun(cliRoot, ['archive', 'beta', '-y', '--no-color'])
    expect(run.code).toBe(0)
    expect(portable(sdk, sdkRoot)).toEqual(portable({ output: run.stdout, stderr: run.stderr }, cliRoot))
    expect(sdk.output).toMatch(/^✓ Archived: beta → \d{4}-\d{2}-\d{2}-beta\n/)
    expect(sdk.stderr).toBe(
      "note: no task evidence recorded for change 'beta' — fine for spec-only changes; otherwise check that tasks went through apply\n",
    )
    expect(sdk.output + sdk.stderr).not.toContain('\u001b')
  })

  // spec「沒有 stderr 文字時不帶 stderr 欄位」
  it('a verb with no stderr text resolves to an object with output only', async () => {
    const created = await createEngine({ store: memoryStore(fixtureProject()) }).dispatch(['new', 'change', 'demo'])
    expect(Object.keys(created as object)).toEqual(['output'])
  })

  // spec「結論已寫入但收尾封存失敗」：記錄已轉出的變更不在進行中，封存資料夾寫不進去。
  it('conclude resolves with closingError when the closing step cannot archive the record', async () => {
    const closingFixture = () => {
      const root = makeFsFixture()
      const discussions = join(root, 'speclink', 'discussions')
      mkdirSync(discussions, { recursive: true })
      writeFileSync(
        join(discussions, 'demo-topic.md'),
        '---\ntopic: Demo topic\nslug: demo-topic\nstatus: promoted\ncreated: 2026-07-01\npromoted_to: gone-change\n---\n\n# Discussion: Demo topic\n\n## Context\n\nx\n\n## Rounds\n\n## Conclusion\n',
      )
      // The archive folder's place is taken by a file, so the record cannot move there.
      writeFileSync(join(discussions, 'archive'), 'not a directory\n')
      return root
    }
    const [sdkRoot, cliRoot] = [closingFixture(), closingFixture()]
    const conclusion = '**Decision**: do A\n'
    const argv = ['discuss', 'conclude', 'demo-topic', '--stdin']

    const sdk = (await fsEngine(sdkRoot).dispatch(argv, { stdin: conclusion })) as {
      slug: string
      status: string
      closingError: string
    }
    const run = cliRun(cliRoot, [...argv, '--json'], conclusion)
    expect(run.code).not.toBe(0)

    // The CLI says the same two things: the result on stdout, the reason in its error line.
    const { closingError, ...value } = sdk
    expect(value).toEqual(JSON.parse(run.stdout))
    expect(value).toEqual({ slug: 'demo-topic', status: 'concluded' })
    expect(closingError.length).toBeGreaterThan(0)
    expect(run.stderr.trim()).toBe(
      `Error: conclude closing step failed to archive the record: ${closingError}`,
    )
    // The conclusion landed and the record is still live.
    expect(readFileSync(join(sdkRoot, 'speclink', 'discussions', 'demo-topic.md'), 'utf8')).toContain(
      '**Decision**: do A',
    )
  })

  it('the discuss verbs resolve to what the CLI prints with --json', async () => {
    const [sdkRoot, cliRoot] = [demoFixture(), demoFixture()]
    const engine = fsEngine(sdkRoot)
    const round = '**Focus**: which way\n\n**Position**: way A\n\n**Ruled out**: way B\n\n**Open**: none\n'
    const conclusion =
      '**Decision**: do A\n\n**Rationale**: simple\n\n**Rejected alternatives**: B\n\n**Deferred**: none\n\n**Capture to**: a change\n\n**Next**: propose\n'
    const steps: Array<{ argv: string[]; stdin?: string }> = [
      { argv: ['discuss', 'new', 'Rate limits'] },
      { argv: ['discuss', 'add-round', 'rate-limits', '--stdin'], stdin: round },
      { argv: ['discuss', 'list'] },
      { argv: ['discuss', 'show', 'rate-limits'] },
      { argv: ['discuss', 'conclude', 'rate-limits', '--stdin'], stdin: conclusion },
    ]
    for (const { argv, stdin } of steps) {
      const sdk = await engine.dispatch(argv, stdin === undefined ? undefined : { stdin })
      const out = execFileSync(cliBin, [...argv, '--json'], { cwd: cliRoot, encoding: 'utf8', input: stdin ?? '' })
      expect(portable(sdk, sdkRoot), argv.join(' ')).toEqual(portable(JSON.parse(out), cliRoot))
    }
    expect(await engine.dispatch(['discuss', 'list'])).toMatchObject({
      discussions: [{ slug: 'rate-limits', status: 'concluded', rounds: 1 }],
    })
  })

  // spec node-sdk「dispatch 執行 change rank」：fs 形式寫各變更的 metadata，plan 跟著改順序。
  it('change rank runs and the plan follows it', async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), 'speclink-node-rank-')))
    for (const [name, meta] of [
      ['a', 'schema: spec-driven\ncreated: 2026-09-01\nboard_rank: n\n'],
      ['c', 'schema: spec-driven\ncreated: 2026-09-02\n'],
    ]) {
      mkdirSync(join(root, 'speclink', 'changes', name), { recursive: true })
      writeFileSync(join(root, 'speclink', 'changes', name, '.meta.yaml'), meta)
    }
    try {
      const engine = createEngine({ store: { type: 'fs', root } })
      expect(await engine.dispatch(['change', 'rank', 'c', '--before', 'a', '--json'])).toEqual({
        change: 'c',
        anchor: 'a',
        position: 'before',
      })
      const plan = (await engine.dispatch(['plan', '--json'])) as { changes: { name: string }[] }
      expect(plan.changes.map((c) => c.name)).toEqual(['c', 'a'])
    } finally {
      rmSync(root, { recursive: true, force: true })
    }
  })

  it('change rank on a store without the change-metadata pair is refused before it runs', async () => {
    const store = memoryStore(fixtureProject()) as Record<string, unknown>
    delete store.readChangeMeta
    delete store.writeChangeMeta
    const engine = createEngine({ store: store as never })
    const err = await rejection(engine.dispatch(['change', 'rank', 'beta', '--before', 'alpha']))
    expect(err.code).toBe('refused')
    expect(err.message).toMatch(/readChangeMeta, writeChangeMeta/)
  })

  it('change rank names only the change-metadata methods it is missing', async () => {
    const store = memoryStore(fixtureProject()) as Record<string, unknown>
    delete store.readChangeMeta
    delete store.deleteArtifact
    const engine = createEngine({ store: store as never })
    const err = await rejection(engine.dispatch(['change', 'rank', 'beta', '--before', 'alpha']))
    expect(err.code).toBe('refused')
    expect(err.message).toMatch(/readChangeMeta/)
    expect(err.message).not.toMatch(/writeChangeMeta|deleteArtifact/)
  })

  // change-plan「Node SDK 既有宿主 Store 沿用 metadata」：31 方法宿主 Store 的順序鍵寫進 metadata。
  it('change rank on a host store writes the metadata and the plan follows it', async () => {
    const meta = (created: string, rank?: string) =>
      `schema: spec-driven\ncreated: ${created}\n${rank ? `board_rank: ${rank}\n` : ''}`
    const project: MemoryProject = {
      changes: {
        a: { meta: { schema: 'spec-driven', created: '2026-09-01' }, artifacts: { '.openspec.yaml': meta('2026-09-01', 'n') } },
        c: { meta: { schema: 'spec-driven', created: '2026-09-02' }, artifacts: { '.openspec.yaml': meta('2026-09-02') } },
      },
      specs: {},
    }
    const base = memoryStore(project)
    // listChanges reports the board rank the raw metadata carries, so a write is seen.
    const rankOf = (name: string) => /^board_rank: (\S+)$/m.exec(project.changes[name]?.artifacts['.openspec.yaml'] ?? '')?.[1]
    const withRank = <T extends { name: string; meta: object } | null>(c: T) =>
      c && { ...c, meta: { ...c.meta, boardRank: rankOf(c.name) } }
    const store = {
      ...base,
      async listChanges() {
        return (await base.listChanges()).map(withRank)
      },
      async findChange(name: string) {
        return withRank(await base.findChange(name))
      },
    }
    const engine = createEngine({ store: store as never })
    expect(await engine.dispatch(['change', 'rank', 'c', '--before', 'a', '--json'])).toEqual({
      change: 'c',
      anchor: 'a',
      position: 'before',
    })
    const written = rankOf('c')
    expect(written && written < 'n', `c's rank ${written}`).toBe(true)
    expect(project.changes.a.artifacts['.openspec.yaml']).toBe(meta('2026-09-01', 'n'))
    const plan = (await engine.dispatch(['plan', '--json'])) as { changes: { name: string }[] }
    expect(plan.changes.map((c) => c.name)).toEqual(['c', 'a'])
  })

  it('the verbs that need a checkout still reject with invalid_argv', async () => {
    const engine = createEngine({ store: memoryStore(fixtureProject()) })
    for (const argv of [
      ['drift', 'alpha'],
      ['review', 'prepare', 'beta'],
      ['review', 'scope', 'beta'],
      ['verify', 'scope', 'beta'],
    ]) {
      const err = await rejection(engine.dispatch(argv))
      expect(err.code, argv.join(' ')).toBe('invalid_argv')
      expect(err.message, argv.join(' ')).toMatch(/only runs where a checkout exists/)
    }
  })
})

// 原本 9 個動詞的回傳值全文（spec node-sdk：回傳形狀維持不變）。宿主 Store 的
// 路徑是相對字串，值不隨機器改變。
describe('the nine verbs dispatch served before keep their return values', () => {
  const SCOPE_FILE = 'crates/b/src/util.rs'
  const ROUND = `**Scope**: ${SCOPE_FILE}\n\n- [SUGGESTION] ${SCOPE_FILE} — rename helper\n`
  const STAMP = JSON.stringify({ scope: [{ path: SCOPE_FILE, hash: '0f9c' }] })
  const engine = () => createEngine({ store: memoryStore(fixtureProject()) })

  it('list', async () => {
    expect(await engine().dispatch(['list', '--sort', 'name'])).toMatchInlineSnapshot(`
      {
        "changes": [
          {
            "completedTasks": 1,
            "deltaCapabilities": [
              "api-quota",
            ],
            "name": "alpha",
            "status": "in-progress",
            "summary": "Add rate limiting to the publi…",
            "totalTasks": 3,
          },
          {
            "completedTasks": 1,
            "name": "beta",
            "status": "done",
            "summary": "Small cleanup change with ever…",
            "totalTasks": 1,
          },
        ],
      }
    `)
    expect(await engine().dispatch(['list', '--specs', '--changes', '--sort', 'name'])).toMatchInlineSnapshot(`
      {
        "changes": [
          {
            "completedTasks": 1,
            "deltaCapabilities": [
              "api-quota",
            ],
            "name": "alpha",
            "status": "in-progress",
            "summary": "Add rate limiting to the publi…",
            "totalTasks": 3,
          },
          {
            "completedTasks": 1,
            "name": "beta",
            "status": "done",
            "summary": "Small cleanup change with ever…",
            "totalTasks": 1,
          },
        ],
        "specs": [
          {
            "id": "user-auth",
            "path": "specs/user-auth",
          },
        ],
      }
    `)
    expect(await engine().dispatch(['list', '--specs'])).toMatchInlineSnapshot(`
      {
        "specs": [
          {
            "id": "user-auth",
            "path": "specs/user-auth",
          },
        ],
      }
    `)
  })

  it('status', async () => {
    expect(await engine().dispatch(['status', '--change', 'alpha'])).toMatchInlineSnapshot(`
      {
        "applyRequires": [
          "tasks",
        ],
        "artifacts": [
          {
            "id": "proposal",
            "outputPath": "proposal.md",
            "status": "done",
          },
          {
            "id": "design",
            "outputPath": "design.md",
            "status": "ready",
          },
          {
            "id": "specs",
            "outputPath": "specs/**/*.md",
            "status": "done",
          },
          {
            "id": "tasks",
            "outputPath": "tasks.md",
            "status": "done",
          },
        ],
        "changeName": "alpha",
        "isComplete": false,
        "schemaName": "spec-driven",
      }
    `)
  })

  // 沒有任何變更時 status 和 CLI 一樣只印提示：同一句、同一個尾端換行。
  it('status with no change at all', async () => {
    const empty = createEngine({ store: memoryStore({ changes: {}, specs: {} }) })
    expect(await empty.dispatch(['status'])).toEqual({
      output: 'No active changes. Create one with: speclink new change <name>\n',
    })
  })

  it('status with no change at all matches the CLI stdout', async () => {
    const root = mkdtempSync(join(tmpdir(), 'speclink-node-empty-'))
    try {
      mkdirSync(join(root, 'speclink', 'changes'), { recursive: true })
      const sdk = await createEngine({ store: { type: 'fs', root } }).dispatch(['status'])
      const cli = execFileSync(cliBin, ['status', '--no-color'], { cwd: root, encoding: 'utf8' })
      expect(sdk).toEqual({ output: cli })
    } finally {
      rmSync(root, { recursive: true, force: true })
    }
  })

  it('new change', async () => {
    expect(await engine().dispatch(['new', 'change', 'demo'])).toMatchInlineSnapshot(`
      {
        "output": "✓ Created change: demo
        Path: changes/demo
        Schema: spec-driven
      ",
      }
    `)
  })

  it('new artifact', async () => {
    expect(
      await engine().dispatch(['new', 'artifact', 'design', '--change', 'alpha', '--stdin'], {
        stdin: '## Context\n\nA design.\n',
      }),
    ).toMatchInlineSnapshot(`
      {
        "artifact": "design",
        "change": "alpha",
        "path": "changes/alpha/design.md",
        "status": "created",
        "validated": true,
        "warnings": [],
      }
    `)
    expect(await engine().dispatch(['new', 'artifact', 'spec', 'user-auth', '--change', 'beta'])).toMatchInlineSnapshot(`
      {
        "artifact": "spec",
        "change": "beta",
        "path": "changes/beta/specs/user-auth/spec.md",
        "status": "created",
        "validated": false,
        "warnings": [],
      }
    `)
  })

  it('claim', async () => {
    const store = memoryStore(fixtureProject()) as ReturnType<typeof memoryStore> & {
      claim?: (name: string) => Promise<unknown>
    }
    store.claim = async (name: string) => ({ claimed: true, claimedBy: 'you', name })
    expect(await createEngine({ store }).dispatch(['claim', 'alpha'])).toMatchInlineSnapshot(`
      {
        "claimed": true,
        "claimedBy": "you",
        "name": "alpha",
      }
    `)
  })

  it('review add-round and review stamp', async () => {
    const e = engine()
    expect(await e.dispatch(['review', 'add-round', 'beta', '--stdin'], { stdin: ROUND })).toMatchInlineSnapshot(`
      {
        "change": "beta",
        "round": 1,
      }
    `)
    expect(await e.dispatch(['review', 'stamp', 'beta', '--stdin'], { stdin: STAMP })).toMatchInlineSnapshot(`
      {
        "change": "beta",
      }
    `)
  })

  it('verify add-round and verify stamp', async () => {
    const e = engine()
    expect(await e.dispatch(['verify', 'add-round', 'beta', '--stdin'], { stdin: ROUND })).toMatchInlineSnapshot(`
      {
        "change": "beta",
        "round": 1,
      }
    `)
    expect(await e.dispatch(['verify', 'stamp', 'beta', '--stdin'], { stdin: STAMP })).toMatchInlineSnapshot(`
      {
        "change": "beta",
      }
    `)
  })
})

// 新開放的寫入動詞沿用引擎的守門，fs 形式留下的記錄和 CLI 相同（sharp-edges 稽核）。
describe('the write verbs dispatch now serves keep the gates and records of the CLI', () => {
  const git = (cwd: string, ...args: string[]) =>
    execFileSync('git', args, { cwd, encoding: 'utf8', stdio: 'pipe' })

  /** The fs fixture as a git repo with one uncommitted source edit. */
  function dirtyRepoFixture(): string {
    const root = makeFsFixture()
    mkdirSync(join(root, 'src'))
    writeFileSync(join(root, 'src', 'lib.rs'), 'fn a() {}\n')
    git(root, 'init', '-q')
    git(root, 'config', 'user.name', 'Fixture User')
    git(root, 'config', 'user.email', 'fixture@example.com')
    git(root, 'add', '-A')
    git(root, 'commit', '-q', '-m', 'init')
    writeFileSync(join(root, 'src', 'lib.rs'), 'fn a() {}\nfn b() {}\n')
    return root
  }

  it('task done on the fs form records the same evidence fields the CLI records', async () => {
    const [sdkRoot, cliRoot] = [dirtyRepoFixture(), dirtyRepoFixture()]
    await createEngine({ store: { type: 'fs', root: sdkRoot } }).dispatch(['task', 'done', '2', '--change', 'alpha'])
    execFileSync(cliBin, ['task', 'done', '2', '--change', 'alpha'], { cwd: cliRoot, encoding: 'utf8' })
    const entry = (root: string) => {
      const record = JSON.parse(
        readFileSync(join(root, 'speclink', 'changes', 'alpha', '.evidence.json'), 'utf8'),
      ) as { entries: Array<Record<string, unknown>> }
      expect(record.entries).toHaveLength(1)
      return record.entries[0]
    }
    const [sdk, cli] = [entry(sdkRoot), entry(cliRoot)]
    expect(Object.keys(sdk).sort()).toEqual(Object.keys(cli).sort())
    for (const field of ['taskDesc', 'actor', 'repo', 'touchedFiles']) {
      expect(sdk[field], field).toEqual(cli[field])
    }
    expect(sdk.repo).toBe('main')
    expect(sdk.touchedFiles).toEqual(['src/lib.rs'])
  })

  it('archive of a change with open tasks is refused with the CLI message', async () => {
    const root = makeFsFixture()
    const err = await rejection(createEngine({ store: { type: 'fs', root } }).dispatch(['archive', 'alpha', '-y']))
    expect(err.code).toBe('refused')
    let cliStderr = ''
    try {
      execFileSync(cliBin, ['archive', 'alpha', '-y'], { cwd: root, encoding: 'utf8', stdio: 'pipe' })
      expect.unreachable('the CLI must refuse too')
    } catch (e) {
      cliStderr = (e as { stderr?: string }).stderr ?? ''
    }
    expect(cliStderr.trim()).toBe(`Error: ${err.message}`)
    expect(err.message).toMatch(/1\/3 tasks complete/)
  })

  it('several changes or --all in one archive call reject with invalid_argv and archive nothing', async () => {
    const store = memoryStore(fixtureProject())
    const engine = createEngine({ store })
    for (const argv of [['archive', '--all', '-y'], ['archive', 'alpha', 'beta', '-y']]) {
      const err = await rejection(engine.dispatch(argv))
      expect(err.code, argv.join(' ')).toBe('invalid_argv')
    }
    expect(await store.changeExists('beta')).toBe(true)
  })

  it('a stamp on a change with open tasks is refused before any write', async () => {
    const store = memoryStore(fixtureProject())
    const engine = createEngine({ store, actor: 'Rev <rev@example.com>' })
    await engine.dispatch(['review', 'add-round', 'alpha', '--stdin'], { stdin: '**Scope**: src/lib.rs\n' })
    const err = await rejection(engine.dispatch(['review', 'stamp', 'alpha']))
    expect(err.code).toBe('refused')
    expect(err.message).toMatch(/review stamp requires every code task done/)
    expect(await store.readArtifact('alpha', 'review.md')).toContain('## Round 1')
    expect(await store.readArtifact('alpha', '.openspec.yaml')).not.toContain('reviewed_')
  })

  it('{ output } carries no color codes, whatever the process environment says', async () => {
    const saved = { force: process.env.CLICOLOR_FORCE, no: process.env.NO_COLOR }
    process.env.CLICOLOR_FORCE = '1'
    delete process.env.NO_COLOR
    try {
      const engine = createEngine({ store: memoryStore(fixtureProject()) })
      const created = (await engine.dispatch(['new', 'change', 'demo'])) as { output: string }
      expect(created.output).toBe('✓ Created change: demo\n  Path: changes/demo\n  Schema: spec-driven\n')
      const removed = (await engine.dispatch(['in-progress', 'remove', 'demo'])) as { output: string }
      expect(removed.output).toBe("Change 'demo' has no in-progress marker — already proposed\n")
      for (const text of [created.output, removed.output]) {
        expect(text).not.toContain('\u001b')
      }
    } finally {
      if (saved.force === undefined) delete process.env.CLICOLOR_FORCE
      else process.env.CLICOLOR_FORCE = saved.force
      if (saved.no !== undefined) process.env.NO_COLOR = saved.no
    }
  })
})
