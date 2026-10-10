import { expect, mock, test } from 'claude-code/testing'
import type { On } from 'claude-code'

import { afterPrompt, changeIn, focusFromHistory, focusFromTitle, speclinkCommand, titleOf, verbOf } from '../hooks/focus'
import { buildBoard, discussionBody, isNoTicket, parseTasks, proposedRows, speclinkArgv, toTicket } from '../hooks/board'
import type { PlanJson } from '../hooks/board'
import { cells } from '../hooks/register'
import { commandHead, groupSkills, withArgument, withCommand } from '../hooks/skills'
import { resolveLang } from '../hooks/text'

const BAND = {
  component: 'AbovePrompt',
  props: {
    hasSurvey: false,
    isWorking: false,
    maxRows: 10,
    bodyColumns: 140,
    scroll: { offset: 0, bodyRows: 9 },
    view: {},
  },
} as const

const PANE = {
  component: 'Pane',
  requestId: 'speclink-panel',
  props: {
    title: 'speclink',
    isFocused: false,
    bodyColumns: 44,
    placement: 'dock',
    scroll: { offset: 0, bodyRows: 30 },
    view: {},
  },
} as const

const TYPED = {
  args: '',
  origin: { kind: 'composer' },
  presentation: { isFullscreen: true, columns: 160 },
} as const

const LIST = {
  changes: [
    {
      name: 'add-auth',
      completedTasks: 5,
      totalTasks: 12,
      worktree: { path: '/repo/.worktrees/add-auth', branch: 'speclink/add-auth' },
    },
    { name: 'refactor-store', completedTasks: 0, totalTasks: 8 },
  ],
}
const PLAN: PlanJson = {
  changes: [
    { name: 'add-auth', wave: 1, stage: 'in-progress', blockedBy: [] },
    { name: 'refactor-store', wave: 2, stage: 'proposed', blockedBy: ['add-auth'] },
  ],
  next: 'add-auth',
  skipped: [],
}
const TALK = {
  discussions: [
    { slug: 'hold-auto-close', topic: 'hold 收尾', rounds: 2, status: 'open' },
    { slug: 'ship-it', topic: '收尾流程', rounds: 4, status: 'concluded' },
    { slug: 'old-idea', topic: '已轉出', rounds: 3, status: 'promoted' },
  ],
}
const TASKS = `## 1. 登入

- [x] 1.1 加上 OAuth 設定 <!-- speclink-task:tsk_01 -->
- [ ] 1.2 接上回呼路由 <!-- speclink-task:tsk_02 -->

## 2. 驗收

- [ ] [M] 2.1 在瀏覽器手動登入一次 <!-- speclink-task:tsk_03 -->
`
const ARTIFACTS = {
  artifacts: [
    { id: 'proposal', status: 'done' },
    { id: 'design', status: 'done' },
    { id: 'specs', status: 'done' },
    { id: 'tasks', status: 'ready' },
  ],
}
const TICKET = {
  change: 'add-auth',
  lastRound: {
    index: 2,
    findings: [
      { severity: 'CRITICAL', path: 'src/auth.ts', text: 'token 沒有檢查過期' },
      { severity: 'WARNING', path: 'src/routes.ts', text: '回呼路由缺少錯誤處理' },
    ],
  },
}

type Run = { argv: readonly string[]; cwd?: string }

// CLI 的替身：依參數回 JSON；沒有工單時照 CLI 的樣子以 exit 1 失敗。
const cli = (argv: readonly string[]) => {
  const [verb, sub, name] = argv.slice(1)
  if (verb === 'list') return LIST
  if (verb === 'plan') return PLAN
  if (verb === 'discuss' && sub === 'list') return TALK
  if (verb === 'discuss' && sub === 'show') return { content: `---\ntopic: ${name}\n---\n\n# 背景\n\n討論 ${name} 的內文。\n`, info: {} }
  if (verb === 'show') return { tasks: TASKS }
  if (verb === 'status') return ARTIFACTS
  if (verb === 'review' && name === 'add-auth') return TICKET
  return null
}

// 引擎在測試裡的替身：已安裝的技能、設定、輸入框與 CLI 輸出。
// panes：開著的面板（ui.panes 的答案）。
const engine = (on: On, filled: string[], claudeLanguage: string, runs: Run[] = [], panes: () => string[] = () => []) => {
  mock.env(on, {})
  on('settings.read', () => ({ value: { language: claudeLanguage } }))
  on('command.list', () => ({
    value: [
      { name: 'speclink-apply', description: '', source: 'user' },
      { name: 'speclink-propose', description: 'Use when a change needs planning', source: 'user' },
      { name: 'speclink-archive', description: 'Use when a change is finished', source: 'user' },
      { name: 'speclink-commit', description: '', source: 'user' },
      { name: 'commit', description: '', source: 'user' },
    ],
  }))
  on('command.register', ($, e) => ({ value: { command: e.name } }))
  on('session.start', ($, e) => ({ cwd: e.cwd }))
  on('prompt.read', () => ({ value: { text: 'add-x', cursor: 5 } }))
  on('prompt.fill', ($, e) => {
    filled.push(e.text)
    return { isFilled: true }
  })
  on('process.run', ($, e) => {
    runs.push({ argv: e.argv, cwd: e.init?.cwd })
    const out = cli(e.argv)
    const station = e.argv[1] ?? ''
    return {
      value: {
        exitCode: out === null ? 1 : 0,
        stdout: out === null ? '' : JSON.stringify(out),
        stderr: out === null ? `Error: no ${station} ticket for change '${e.argv[3] ?? ''}'` : '',
        isStdoutTruncated: false,
        isStderrTruncated: false,
      },
    }
  })
  on('ui.panes', () => ({ value: panes().map(id => ({ id, title: id })) }) as never)
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('ui.render', { component: 'AbovePrompt' }, ($, e) => {
    const { Box } = $.ui.resolve(e)
    return <Box />
  })
}

test('從送出的指令、技能呼叫與 speclink 指令認出步驟和 change', () => {
  const names = ['add-auth', 'refactor-store']
  expect(verbOf('/speclink-apply add-auth', 'speclink-panel')).toBe('apply')
  expect(verbOf('/speclink-archive + /speclink-commit add-auth', 'speclink-panel')).toBe('archive+commit')
  expect(verbOf('<command-name>/speclink-quality</command-name>\n<command-args>add-auth</command-args>', 'speclink-panel')).toBe('quality')
  expect(verbOf('/speclink-panel', 'speclink-panel')).toBe(null)
  expect(verbOf('/commit', 'speclink-panel')).toBe(null)
  expect(verbOf('幫我看 /speclink-apply 怎麼用', 'speclink-panel')).toBe(null)

  expect(changeIn('speclink status --change "add-auth" --json', names)).toBe('add-auth')
  expect(changeIn('speclink review scope refactor-store --json', names)).toBe('refactor-store')
  expect(changeIn('speclink list --json', names)).toBe(null)
  expect(changeIn('cat add-auth-notes.md', names)).toBe(null)
  expect(speclinkCommand('cd /repo && speclink task done x')).not.toBe(null)
  expect(speclinkCommand('cat speclink.toml')).toBe(null)

  // 新的技能指令換步驟；沒寫名稱時留著上一個 change，其他訊息不動。
  const start = { verb: null, change: null }
  const applying = afterPrompt(start, '/speclink-apply add-auth', names, 'speclink-panel')
  expect(applying).toEqual({ verb: 'apply', change: 'add-auth' })
  expect(afterPrompt(applying, '/speclink-quality', names, 'speclink-panel')).toEqual({ verb: 'quality', change: 'add-auth' })
  expect(afterPrompt(applying, '繼續工作', names, 'speclink-panel')).toBe(applying)

  // 接回的對話：最後的技能呼叫與 speclink 指令為準。
  const history = [
    { role: 'user', text: '/speclink-apply add-auth', toolUses: [] },
    { role: 'assistant', text: '', toolUses: [{ tool: 'Skill', input: { skill: 'speclink-review' } }] },
    { role: 'assistant', text: '', toolUses: [{ tool: 'Bash', input: { command: 'speclink review scope refactor-store --json' } }] },
  ]
  expect(focusFromHistory(history, names, 'speclink-panel')).toEqual({ verb: 'review', change: 'refactor-store' })

  expect(titleOf({ verb: 'apply', change: 'add-auth' })).toBe('apply · add-auth')
  expect(titleOf({ verb: 'archive', change: 'add-auth', archived: true })).toBe('archive · add-auth ✓')

  // 標題解析回焦點：要是現有的 change，或標了 ✓ 的封存；你自己取的標題不算。
  expect(focusFromTitle('apply · add-auth', names)).toEqual({ verb: 'apply', change: 'add-auth' })
  expect(focusFromTitle('refactor-store', names)).toEqual({ verb: null, change: 'refactor-store' })
  expect(focusFromTitle('archive · gone-change ✓', names)).toEqual({ verb: 'archive', change: 'gone-change', archived: true })
  expect(focusFromTitle('apply · gone-change', names)).toBe(null)
  expect(focusFromTitle('整理 statusbar 的按鈕', names)).toBe(null)
  expect(focusFromTitle('Claude Code', names)).toBe(null)
  expect(focusFromTitle(undefined, names)).toBe(null)
  expect(titleOf({ verb: null, change: 'add-auth' })).toBe('add-auth')
  expect(titleOf(start)).toBe(null)
})

test('指令放最前面，已經打的字留在後面當參數', () => {
  expect(withCommand('', '/speclink-apply')).toBe('/speclink-apply ')
  expect(withCommand('add-x', '/speclink-apply')).toBe('/speclink-apply add-x')
  expect(withCommand('/speclink-review add-x', '/speclink-verify')).toBe('/speclink-verify add-x')
})

test('archive+commit 一次填入兩個指令，換別的技能時兩個一起換掉', () => {
  const head = commandHead('archive+commit')

  expect(head).toBe('/speclink-archive + /speclink-commit')
  expect(withCommand('add-x', head)).toBe('/speclink-archive + /speclink-commit add-x')
  expect(withCommand('/speclink-archive + /speclink-commit add-x', '/speclink-apply')).toBe('/speclink-apply add-x')
  expect(withArgument('/speclink-archive + /speclink-commit ', 'add-x')).toBe('/speclink-archive + /speclink-commit add-x')
})

test('名稱成為最前面指令的參數；沒有指令時接在已打的字後面', () => {
  expect(withArgument('', 'add-auth')).toBe('add-auth')
  expect(withArgument('/speclink-apply', 'add-auth')).toBe('/speclink-apply add-auth')
  expect(withArgument('/speclink-apply old-one', 'add-auth')).toBe('/speclink-apply add-auth')
  expect(withArgument('look at', 'add-auth')).toBe('look at add-auth')
})

test('技能依工作流程分頁，表上沒有的落到 other，合併鈕要兩個技能都在', () => {
  const groups = groupSkills([
    'speclink-verify',
    'speclink-apply',
    'speclink-archive',
    'speclink-commit',
    'speclink-new-thing',
  ])
  expect(groups.map(g => [g.id, g.skills])).toEqual([
    ['build', ['apply']],
    ['quality', ['verify']],
    ['ship', ['archive', 'archive+commit']],
    ['other', ['commit', 'new-thing']],
  ])

  expect(groupSkills(['speclink-archive']).map(g => [g.id, g.skills])).toEqual([['ship', ['archive']]])
})

test('面板資料：順序與階段取 plan，任務數取 list，已轉出的討論不列', () => {
  const board = buildBoard(LIST, PLAN, TALK)

  expect(board.changes).toEqual([
    {
      name: 'add-auth',
      stage: 'in-progress',
      wave: 1,
      blockedBy: [],
      done: 5,
      total: 12,
      branch: 'speclink/add-auth',
      cwd: '/repo/.worktrees/add-auth',
    },
    {
      name: 'refactor-store',
      stage: 'proposed',
      wave: 2,
      blockedBy: ['add-auth'],
      done: 0,
      total: 8,
      branch: null,
      cwd: null,
    },
  ])
  expect(board.discussions.map(d => d.slug)).toEqual(['hold-auto-close', 'ship-it'])
  expect(board.next).toBe('add-auth')
})

test('輸入框 # 選單：只列提案中的 change，# 後面的字篩選名稱，不分大小寫', () => {
  const changes = [...PLAN.changes, { name: 'Add-Search', wave: 2, stage: 'proposed' as const, blockedBy: [] }]

  expect(proposedRows(changes, '#', '提案中')).toEqual([
    { text: 'refactor-store', description: 'speclink · 提案中' },
    { text: 'Add-Search', description: 'speclink · 提案中' },
  ])
  expect(proposedRows(changes, '#add', '提案中').map(r => r.text)).toEqual(['Add-Search'])
  expect(proposedRows(changes, '#auth', '提案中')).toEqual([])
})

test('Windows 經 cmd.exe 執行 speclink，其他平台直接執行', () => {
  expect(speclinkArgv(['list', '--json'], false)).toEqual(['speclink', 'list', '--json'])
  expect(speclinkArgv(['list', '--json'], true)).toEqual(['cmd.exe', '/d', '/c', 'speclink', 'list', '--json'])
})

test('語言：auto 跟著 Claude Code 的 language 設定，明選的值優先', () => {
  expect(resolveLang('auto', '台灣繁體中文zh-tw')).toBe('zh-TW')
  expect(resolveLang('auto', 'Chinese')).toBe('zh-TW')
  expect(resolveLang('auto', 'English')).toBe('en')
  expect(resolveLang('auto', undefined)).toBe('en')
  expect(resolveLang('en', '台灣繁體中文zh-tw')).toBe('en')
  expect(resolveLang('zh-TW', 'English')).toBe('zh-TW')
})

test('技能列：切分頁、點技能把指令填進輸入框', async ($, on) => {
  const filled: string[] = []
  engine(on, filled, 'English')

  await $.session.start({ cwd: '/tmp', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })

  expect(await ui.find({ type: 'Text', text: ' Plan ' })).toBeDefined()
  expect(await ui.find({ type: 'Button', key: 'tab:plan' })).toBeUndefined()
  expect(await ui.find({ type: 'Button', key: 'propose' })).toBeDefined()
  expect(await ui.find({ type: 'Button', key: 'apply' })).toBeUndefined()

  await ui.press({ key: 'tab:build' })
  await ui.press({ key: 'apply' })
  expect(filled).toEqual(['/speclink-apply add-x'])
})

for (const source of ['clear', 'resume'] as const) {
  test(`/${source} 之後沒有 session.start，技能列仍然補抓清單、畫得出來`, async ($, on) => {
    engine(on, [], 'English')
    on('classic.SessionStart', () => ({}))

    await $.classic.SessionStart({ source })
    const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })

    expect(await ui.find({ type: 'Button', key: 'propose' })).toBeDefined()
    expect(await ui.find({ type: 'Button', key: 'panel', text: '◧ Panel' })).toBeDefined()
  })
}

test('Claude Code 設定是中文時，技能列用繁中標籤', async ($, on) => {
  engine(on, [], '台灣繁體中文zh-tw')

  await $.session.start({ cwd: '/tmp', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })

  expect(await ui.find({ type: 'Text', text: ' 規劃 ' })).toBeDefined()
  expect(await ui.find({ type: 'Button', key: 'tab:build', text: '實作' })).toBeDefined()
  expect(await ui.find({ type: 'Button', key: 'panel', text: '◧ 面板' })).toBeDefined()
})

test('設定選項 language 明選 en 時，不跟 Claude Code 的中文設定', { options: { language: 'en' } }, async ($, on) => {
  engine(on, [], '台灣繁體中文zh-tw')

  await $.session.start({ cwd: '/tmp', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })

  expect(await ui.find({ type: 'Button', key: 'tab:build', text: 'Build' })).toBeDefined()
})

test('分頁與面板鈕放不下一列時，面板鈕移到 speclink 那一列', async ($, on) => {
  engine(on, [], '台灣繁體中文zh-tw')
  expect(cells('speclink')).toBe(8)
  expect(cells('◧ 面板')).toBe(6)

  await $.session.start({ cwd: '/tmp', surface: 'terminal', isInteractive: true })
  // 規劃、實作、收尾、其他四個分頁加上面板鈕：「分頁 │ 面板」那一列要 31 + 5 + 6 = 42 格。
  const narrow = await $.ui.mount({
    plugin: 'speclink-skills',
    surface: 'terminal',
    ...BAND,
    props: { ...BAND.props, bodyColumns: 41 },
  })
  expect(await narrow.find({ type: 'Text', text: ' 規劃 ' })).toBeDefined()
  expect(await narrow.find({ type: 'Button', key: 'panel', text: '◧ 面板' })).toBeDefined()
  expect(await narrow.find({ type: 'Button', key: 'propose' })).toBeDefined()
  expect(await narrow.find({ type: 'Text', text: ' │ ' })).toBeUndefined()
  expect(await narrow.find({ type: 'Text', text: 'Use when a change needs planning' })).toBeUndefined()
  await narrow.unmount()

  const wide = await $.ui.mount({
    plugin: 'speclink-skills',
    surface: 'terminal',
    ...BAND,
    props: { ...BAND.props, bodyColumns: 42 },
  })
  expect(await wide.find({ type: 'Text', text: ' │ ' })).toBeDefined()
  expect(await wide.find({ type: 'Text', text: 'Use when a change needs planning' })).toBeDefined()
})

test('技能說明：平常藏著、滑鼠移上去才顯示；合併鈕的說明只列兩個指令', async ($, on) => {
  engine(on, [], 'English')

  await $.session.start({ cwd: '/tmp', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })

  expect(await ui.find({ type: 'Text', text: 'Use when a change needs planning' })).toBeDefined()

  await ui.press({ key: 'tab:ship' })
  expect(await ui.find({ type: 'Text', text: 'Use when a change is finished' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: '/speclink-archive → /speclink-commit' })).toBeDefined()
})

test('/speclink-panel 開啟面板，點 change 名稱填進輸入框當參數', async ($, on) => {
  const filled: string[] = []
  engine(on, filled, '台灣繁體中文zh-tw')

  await $.session.start({ cwd: '/tmp', surface: 'terminal', isInteractive: true })
  const ran = await $.command.run({ command: 'speclink-panel', ...TYPED })
  expect(ran.text).toBe('speclink 面板已開啟。')

  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...PANE })

  expect(await ui.find({ type: 'Button', key: 'next', text: 'add-auth' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: /^ 進行中 1$/ })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: '▰▰▰▱▱▱ 5/12' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: /等 add-auth/ })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: ' 看板 ' })).toBeDefined()

  await ui.press({ key: 'change:add-auth' })
  expect(filled).toEqual(['add-x add-auth'])
})

test('任務清單：依 ## 分組，已勾與未勾分開，[M] 是手動任務，行尾 ID 不顯示', () => {
  expect(parseTasks(TASKS)).toEqual([
    {
      title: '1. 登入',
      tasks: [
        { done: true, manual: false, label: '1.1 加上 OAuth 設定' },
        { done: false, manual: false, label: '1.2 接上回呼路由' },
      ],
    },
    { title: '2. 驗收', tasks: [{ done: false, manual: true, label: '2.1 在瀏覽器手動登入一次' }] },
  ])
  expect(parseTasks('- [X] 沒有標題的任務')).toEqual([{ title: '', tasks: [{ done: true, manual: false, label: '沒有標題的任務' }] }])
})

test('工單只取最後一輪；沒有工單的失敗不算錯誤；討論內文去掉 front matter', () => {
  expect(toTicket(TICKET)).toEqual({ round: 2, findings: TICKET.lastRound.findings })
  expect(isNoTicket("Error: no verify ticket for change 'add-auth'")).toBe(true)
  expect(isNoTicket('Error: Change not found')).toBe(false)
  expect(discussionBody('---\ntopic: x\n---\n\n# 背景\r\n內文\n')).toBe('# 背景\n內文')
  expect(discussionBody('# 討論\n\n<!--\n規則\n-->\n\n## 背景\n內文')).toBe('# 討論\n\n## 背景\n內文')
})

test('看板：展開 change 在 worktree 裡讀文件與任務；區塊可以收起', async ($, on) => {
  const runs: Run[] = []
  engine(on, [], '台灣繁體中文zh-tw', runs)

  await $.session.start({ cwd: '/tmp', surface: 'terminal', isInteractive: true })
  await $.command.run({ command: 'speclink-panel', ...TYPED })
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...PANE })

  expect(await ui.find({ type: 'Text', text: /1\.2 接上回呼路由/ })).toBeUndefined()
  await ui.press({ key: 'open:change:add-auth' })

  expect(runs.filter(r => r.argv[1] === 'show' || r.argv[1] === 'status').map(r => [r.argv[1], r.cwd])).toEqual([
    ['show', '/repo/.worktrees/add-auth'],
    ['status', '/repo/.worktrees/add-auth'],
  ])
  expect(await ui.find({ type: 'Text', text: '✓ 提案  ✓ 設計  ✓ 規格  ○ 任務  ' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: '1. 登入' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: '1/2' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: '☑ 1.1 加上 OAuth 設定' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: '☐ [手動] 2.1 在瀏覽器手動登入一次' })).toBeDefined()

  await ui.press({ key: 'open:change:add-auth' })
  expect(await ui.find({ type: 'Text', text: /1\.1 加上 OAuth 設定/ })).toBeUndefined()

  await ui.press({ key: 'section:stage:in-progress' })
  expect(await ui.find({ type: 'Button', key: 'change:add-auth' })).toBeUndefined()
  expect(await ui.find({ type: 'Button', key: 'change:refactor-store' })).toBeDefined()
})

test('討論分頁：討論中與已結論分開，已轉出的不列，展開顯示內文', async ($, on) => {
  engine(on, [], '台灣繁體中文zh-tw')

  await $.session.start({ cwd: '/tmp', surface: 'terminal', isInteractive: true })
  await $.command.run({ command: 'speclink-panel', ...TYPED })
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...PANE })

  expect(await ui.find({ type: 'Button', key: 'talk:hold-auto-close' })).toBeUndefined()
  await ui.press({ key: 'view:talk' })

  expect(await ui.find({ type: 'Text', text: /^ 討論中 1$/ })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: /^ 已結論 1$/ })).toBeDefined()
  expect(await ui.find({ type: 'Button', key: 'talk:hold-auto-close' })).toBeDefined()
  expect(await ui.find({ type: 'Button', key: 'talk:old-idea' })).toBeUndefined()

  await ui.press({ key: 'open:talk:hold-auto-close' })
  expect(await ui.find({ type: 'Markdown', text: '# 背景\n\n討論 hold-auto-close 的內文。' })).toBeDefined()
})

test('品質分頁：只列有工單的 change，顯示輪數與各嚴重度條數，展開看發現', async ($, on) => {
  engine(on, [], '台灣繁體中文zh-tw')

  await $.session.start({ cwd: '/tmp', surface: 'terminal', isInteractive: true })
  await $.command.run({ command: 'speclink-panel', ...TYPED })
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...PANE })

  await ui.press({ key: 'view:quality' })

  expect(await ui.find({ type: 'Button', key: 'ticket:add-auth' })).toBeDefined()
  expect(await ui.find({ type: 'Button', key: 'ticket:refactor-store' })).toBeUndefined()
  expect(await ui.find({ type: 'Text', text: '   審查  第 2 輪 · 1 CRITICAL 1 WARNING ' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: '   驗證  沒有工單' })).toBeDefined()

  await ui.press({ key: 'open:ticket:add-auth' })
  expect(await ui.find({ type: 'Text', text: '審查 · 2 條' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'token 沒有檢查過期' })).toBeDefined()
})

test('送出 speclink 技能指令後，技能列標出步驟與 change，session 標題也換成它；speclink 指令裡的 change 跟著更新', async ($, on) => {
  engine(on, [], 'English')
  on('session.cwd', () => ({ value: '/repo' }))
  on('session.messages', () => ({ value: [] }))
  on('classic.UserPromptSubmit', () => ({}))
  on('tool.call', () => ({ result: { stdout: '', stderr: '', interrupted: false } }) as never)

  await $.session.start({ cwd: '/repo', surface: 'terminal', isInteractive: true })
  const result = await $.classic.UserPromptSubmit({ prompt: '/speclink-apply add-auth' })
  expect(result.sessionTitle).toBe('apply · add-auth')

  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })
  expect(await ui.find({ type: 'Text', text: 'apply' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'add-auth' })).toBeDefined()

  // 技能中途用另一個 change 呼叫 CLI：技能列立刻換，標題等下一次送出。
  await $.tool.call({ tool: 'Bash', command: 'speclink status --change "refactor-store" --json' } as never)
  expect(await ui.find({ type: 'Text', text: 'refactor-store' })).toBeDefined()
  const next = await $.classic.UserPromptSubmit({ prompt: '繼續工作' })
  expect(next.sessionTitle).toBe('apply · refactor-store')
})

test('還沒碰過 speclink 的 session 不畫焦點，也不改標題', async ($, on) => {
  engine(on, [], 'English')
  on('session.cwd', () => ({ value: '/repo' }))
  on('session.messages', () => ({ value: [] }))
  on('classic.UserPromptSubmit', () => ({}))

  await $.session.start({ cwd: '/repo', surface: 'terminal', isInteractive: true })
  const result = await $.classic.UserPromptSubmit({ prompt: '幫我看一下 README' })
  expect(result.sessionTitle).toBeUndefined()
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })
  expect(await ui.find({ type: 'Text', text: '▸ ' })).toBeUndefined()
})

test('在某個 change 的 worktree 裡開的 session，一開始就標出那個 change', async ($, on) => {
  engine(on, [], 'English')
  on('session.cwd', () => ({ value: '/repo/.worktrees/add-auth' }))
  on('session.messages', () => ({ value: [] }))

  await $.session.start({ cwd: '/repo/.worktrees/add-auth', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })
  expect(await ui.find({ type: 'Text', text: 'add-auth' })).toBeDefined()
})

// 焦點測試的引擎替身：技能、CLI、對話紀錄與 classic 事件的底層。
const focusEngine = (on: On, cwd = '/repo') => {
  engine(on, [], 'English')
  on('session.cwd', () => ({ value: cwd }))
  on('session.messages', () => ({ value: [] }))
  on('classic.UserPromptSubmit', () => ({}))
  on('classic.SessionStart', () => ({}))
  on('tool.call', () => ({ result: { stdout: '', stderr: '', interrupted: false } }) as never)
}

test('/clear：技能列清空；標題是這個 mod 設的，下一次送出時改回 Claude Code，你自己取的不動', async ($, on) => {
  focusEngine(on)
  await $.session.start({ cwd: '/repo', surface: 'terminal', isInteractive: true })

  // /clear 當下引擎不讓 mod 改標題，等下一次送出。
  const cleared = await $.classic.SessionStart({ source: 'clear', session_title: 'apply · add-auth' })
  expect(cleared.sessionTitle).toBeUndefined()
  const first = await $.classic.UserPromptSubmit({ prompt: '幫我看 README', session_title: 'apply · add-auth' })
  expect(first.sessionTitle).toBe('Claude Code')
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })
  expect(await ui.find({ type: 'Text', text: '▸ ' })).toBeUndefined()

  await $.classic.SessionStart({ source: 'clear', session_title: '我的工作' })
  const mine = await $.classic.UserPromptSubmit({ prompt: '幫我看 README', session_title: '我的工作' })
  expect(mine.sessionTitle).toBeUndefined()
})

test('/resume：從接回 session 的標題找回步驟與 change，封存過的帶 ✓', async ($, on) => {
  focusEngine(on)
  await $.classic.SessionStart({ source: 'resume', session_title: 'review · refactor-store' })
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })
  expect(await ui.find({ type: 'Text', text: 'review' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'refactor-store' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: ' ✓' })).toBeUndefined()
})

test('封存指令跑完、change 從清單消失後，技能列與標題標上 ✓', async ($, on) => {
  focusEngine(on)
  await $.session.start({ cwd: '/repo', surface: 'terminal', isInteractive: true })
  await $.classic.UserPromptSubmit({ prompt: '/speclink-archive add-auth' })

  const before = LIST.changes
  LIST.changes = LIST.changes.filter(c => c.name !== 'add-auth')
  try {
    await $.tool.call({ tool: 'Bash', command: 'speclink archive add-auth --yes' } as never)
    const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })
    expect(await ui.find({ type: 'Text', text: 'add-auth' })).toBeDefined()
    expect(await ui.find({ type: 'Text', text: ' ✓' })).toBeDefined()
    const next = await $.classic.UserPromptSubmit({ prompt: '好' })
    expect(next.sessionTitle).toBe('archive · add-auth ✓')
  } finally {
    LIST.changes = before
  }
})

test('送出技能指令後馬上按 Esc（還沒跑任何工具）：技能列回到原本的步驟；工具開始跑之後才中斷就留著', async ($, on) => {
  focusEngine(on)
  on('turn.complete', () => ({ text: '' }))
  const turn = $.turn as unknown as { complete: (e: unknown) => Promise<unknown> }
  const finish = (isAborted: boolean) =>
    turn.complete({ turnId: 't', durationMs: 1000, answer: '', isAborted, reason: isAborted ? 'aborted' : 'answer' })
  await $.session.start({ cwd: '/repo', surface: 'terminal', isInteractive: true })
  await $.classic.UserPromptSubmit({ prompt: '/speclink-apply add-auth' })
  await finish(false)
  const ui = await $.ui.mount({ plugin: 'speclink-skills', surface: 'terminal', ...BAND })

  await $.classic.UserPromptSubmit({ prompt: '/speclink-quality add-auth' })
  expect(await ui.find({ type: 'Text', text: 'quality' })).toBeDefined()
  await finish(true)
  expect(await ui.find({ type: 'Text', text: 'apply' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'quality' })).toBeUndefined()
  // 標題只能在送出時改：下一次送出就回到原本的步驟。
  const next = await $.classic.UserPromptSubmit({ prompt: '繼續工作' })
  expect(next.sessionTitle).toBe('apply · add-auth')
  await finish(false)

  await $.classic.UserPromptSubmit({ prompt: '/speclink-quality add-auth' })
  await $.tool.call({ tool: 'Bash', command: 'git status' } as never)
  await finish(true)
  expect(await ui.find({ type: 'Text', text: 'quality' })).toBeDefined()
})

test('/clear、/resume 換成新 session 時，開著的面板重新讀取，不會一直停在讀取中', async ($, on) => {
  const runs: Run[] = []
  let isOpen = false
  engine(on, [], 'English', runs, () => (isOpen ? ['speclink-panel'] : []))
  on('session.cwd', () => ({ value: '/repo' }))
  on('session.messages', () => ({ value: [] }))
  on('classic.SessionStart', () => ({}))
  const readsBoard = () => runs.some(r => r.argv.includes('plan'))

  for (const source of ['clear', 'resume'] as const) {
    isOpen = false
    runs.length = 0
    await $.classic.SessionStart({ source })
    expect(readsBoard()).toBe(false)

    isOpen = true
    runs.length = 0
    await $.classic.SessionStart({ source })
    expect(readsBoard()).toBe(true)
  }
})
