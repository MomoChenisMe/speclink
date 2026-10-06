import { expect, mock, test } from 'claude-code/testing'
import type { On } from 'claude-code'

import { buildBoard, speclinkArgv } from '../hooks/board'
import type { PlanJson } from '../hooks/board'
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
    { name: 'add-auth', completedTasks: 5, totalTasks: 12, worktree: { branch: 'speclink/add-auth' } },
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
    { slug: 'old-idea', topic: '已轉出', rounds: 3, status: 'promoted' },
  ],
}

// 引擎在測試裡的替身：已安裝的技能、設定、輸入框與 CLI 輸出。
const engine = (on: On, filled: string[], claudeLanguage: string) => {
  const outputs: Record<string, unknown> = { list: LIST, plan: PLAN, discuss: TALK }
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
  on('process.run', ($, e) => ({
    value: {
      exitCode: 0,
      stdout: JSON.stringify(outputs[e.argv[1] ?? '']),
      stderr: '',
      isStdoutTruncated: false,
      isStderrTruncated: false,
    },
  }))
  on('ui.panes', () => ({ value: [] }))
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('ui.render', { component: 'AbovePrompt' }, ($, e) => {
    const { Box } = $.ui.resolve(e)
    return <Box />
  })
}

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
    { name: 'add-auth', stage: 'in-progress', wave: 1, blockedBy: [], done: 5, total: 12, branch: 'speclink/add-auth' },
    { name: 'refactor-store', stage: 'proposed', wave: 2, blockedBy: ['add-auth'], done: 0, total: 8, branch: null },
  ])
  expect(board.discussions.map(d => d.slug)).toEqual(['hold-auto-close'])
  expect(board.next).toBe('add-auth')
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
  expect(await ui.find({ type: 'Text', text: /^進行中 1$/ })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: '▰▰▰▱▱▱ 5/12' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: /等 add-auth/ })).toBeDefined()
  expect(await ui.find({ type: 'Button', key: 'talk:hold-auto-close' })).toBeDefined()
  expect(await ui.find({ type: 'Button', key: 'talk:old-idea' })).toBeUndefined()

  await ui.press({ key: 'change:add-auth' })
  expect(filled).toEqual(['add-x add-auth'])
})
