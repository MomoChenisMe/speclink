import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register, RenderChildren } from 'claude-code'

import type { Board, Detail, Focus, Lang, PanelChange, PanelDiscussion, PanelTab, PanelView, Quality, Skill, Ticket } from '../types'

import { buildBoard, discussionBody, isNoTicket, parseTasks, proposedRows, speclinkArgv, toTicket } from './board'
import type { DiscussJson, DiscussShowJson, ListJson, PlanJson, ShowJson, StatusJson, TicketJson } from './board'
import { afterPrompt, changeIn, focusFromHistory, focusFromTitle, PLAIN_TITLE, speclinkCommand, titleOf } from './focus'
import type { GroupId } from './groups'
import { commandHead, groupSkills, PREFIX, withArgument, withCommand } from './skills'
import { resolveLang, TEXT } from './text'

const PANE = 'speclink-panel'
const PANEL_COMMAND = 'speclink-panel'
const skills = atom({ plugin: 'speclink-skills', key: 'skills' } as const, [] as Skill[])
const tab = atom({ plugin: 'speclink-skills', key: 'tab' } as const, '')
const board = atom({ plugin: 'speclink-skills', key: 'board' } as const, null as Board | null)
const lang = atom({ plugin: 'speclink-skills', key: 'lang' } as const, 'en' as Lang)
const view = atom({ plugin: 'speclink-skills', key: 'view' } as const, { tab: 'board', closed: [], open: [] } as PanelView)
const details = atom({ plugin: 'speclink-skills', key: 'details' } as const, {} as Record<string, Detail>)
const quality = atom({ plugin: 'speclink-skills', key: 'quality' } as const, null as Quality | null)
// 這個 session 正在跑的步驟與 change，畫在技能列上，也拿來當 session 標題。
const focus = atom({ plugin: 'speclink-skills', key: 'focus' } as const, { verb: null, change: null } as Focus)

// 已知的 change 名稱與 worktree 資料夾（`speclink list`），用來從指令裡認出 change。
let known: { names: string[]; worktrees: { name: string; path: string }[]; loadedAt: number } = { names: [], worktrees: [], loadedAt: 0 }
// /clear 帶過來的標題是這個 mod 設的：下一次送出時改回「Claude Code」（/clear 當下引擎不讓 mod 改標題）。
let staleTitle = false
// 送出的技能指令換掉焦點之前的那一個；模型開始跑工具時清掉。這一輪在那之前就被 Esc 中斷，焦點回到它。
let undo: Focus | null = null
// 輸入框 `#` 選單用的 change 與階段（`speclink plan`）。每打一個字都會問一次，5 秒內用同一份。
let planned: { changes: PlanJson['changes']; loadedAt: number } = { changes: [], loadedAt: 0 }

// 用 `$` 的函式都留在這個檔：`$` 只能傳進同檔宣告的函式，不能跨 import。

const refresh = async ($: EngineInterface) => {
  const found = (await $.command.list())
    .filter(c => c.name.startsWith(PREFIX) && c.name !== PANEL_COMMAND)
    .map(c => ({ name: c.name, description: c.description }))
  await update($, skills, () => found)
}

// 語言、面板指令與技能清單都跟著 session 走。
const setup = async ($: EngineInterface, language: unknown) => {
  const chosen = resolveLang(language, (await $.settings.read()).language)
  await update($, lang, () => chosen)
  await $.command.register({ name: PANEL_COMMAND, description: TEXT[chosen].panelCommand })
  await refresh($)
  await loadKnown($)
}

// 讀不到（不是 speclink 專案、沒裝 CLI）就當作沒有 change。
const loadKnown = async ($: EngineInterface) => {
  try {
    const list = await speclinkJson<ListJson>($, ['list'])
    known = {
      names: list.changes.map(c => c.name),
      worktrees: list.changes.flatMap(c => (c.worktree === undefined ? [] : [{ name: c.name, path: c.worktree.path }])),
      loadedAt: Date.now(),
    }
  } catch {
    known = { names: [], worktrees: [], loadedAt: Date.now() }
  }
}

// 讀不到時當作沒有 change，選單不出現。
const loadPlanned = async ($: EngineInterface) => {
  if (Date.now() - planned.loadedAt > 5_000) {
    const changes = await speclinkJson<PlanJson>($, ['plan']).then(
      p => p.changes,
      () => [],
    )
    planned = { changes, loadedAt: Date.now() }
  }
  return planned.changes
}

// 還不知道焦點時（剛開 session、/resume 接回）：在某個 change 的 worktree 裡開的 session 就是那個
// change；否則從對話紀錄找最後的步驟與 change，再找不到就看 session 標題是不是這個 mod 設的。
const recall = async ($: EngineInterface, title?: string) => {
  const current = await read($, focus)
  if (current.verb !== null || current.change !== null) {
    return current
  }
  const cwd = await $.session.cwd()
  const here = known.worktrees.find(w => cwd === w.path || cwd.startsWith(`${w.path}/`))
  const history = here !== undefined ? { verb: null, change: here.name } : focusFromHistory(await $.session.messages(), known.names, PANEL_COMMAND)
  const found = history.verb !== null || history.change !== null ? history : (focusFromTitle(title, known.names) ?? history)
  await update($, focus, () => found)
  return found
}

// speclink 指令跑完後認 change；認不出來時可能是剛建的 change，重讀一次清單（最多每 10 秒一次）。
// 封存指令跑完也重讀：change 從清單消失就是封存好了，標上 ✓。名稱要用封存前的清單認。
const noteCommand = async ($: EngineInterface, command: string) => {
  let change = changeIn(command, known.names)
  const archiving = /(^|[\s;&|(])speclink\s+archive\b/.test(command)
  if (archiving || (change === null && Date.now() - known.loadedAt > 10_000)) {
    await loadKnown($)
    change ??= changeIn(command, known.names)
  }
  if (change === null) {
    return
  }
  const archived = archiving && !known.names.includes(change)
  await update($, focus, f =>
    f.change === change && (f.archived === true) === archived ? f : archived ? { ...f, change, archived: true as const } : { verb: f.verb, change },
  )
}

const putCommand = async ($: EngineInterface, skill: string) => {
  const { text } = await $.prompt.read()
  await $.prompt.fill({ text: withCommand(text, commandHead(skill)) })
}

const putArgument = async ($: EngineInterface, arg: string) => {
  const { text } = await $.prompt.read()
  await $.prompt.fill({ text: withArgument(text, arg) })
}

// worktree 裡的 change 要在它的資料夾跑，才讀得到它那份 tasks.md。
const runSpeclink = async ($: EngineInterface, args: string[], cwd: string | null = null) => {
  const isWindows = (await $.env.get('OS')) === 'Windows_NT'
  return $.process.run(speclinkArgv([...args, '--json'], isWindows), cwd === null ? undefined : { cwd })
}

const failure = (args: string[], ran: { exitCode: number; stderr: string }) =>
  new Error(`speclink ${args.join(' ')}: ${ran.stderr.trim() || `exit ${ran.exitCode}`}`)

const speclinkJson = async <T,>($: EngineInterface, args: string[], cwd: string | null = null): Promise<T> => {
  const ran = await runSpeclink($, args, cwd)
  if (ran.exitCode !== 0) {
    throw failure(args, ran)
  }
  return JSON.parse(ran.stdout) as T
}

const errorText = (err: unknown) => (err instanceof Error ? err.message : String(err))

// 讀取失敗時保留上一份資料，並記下原因。
const loadBoard = async ($: EngineInterface, last: Board | null): Promise<Board> => {
  const loadedAt = new Date().toISOString()
  try {
    const [list, plan, talk] = await Promise.all([
      speclinkJson<ListJson>($, ['list']),
      speclinkJson<PlanJson>($, ['plan']),
      speclinkJson<DiscussJson>($, ['discuss', 'list']),
    ])
    return { ...buildBoard(list, plan, talk), loadedAt, error: null }
  } catch (err) {
    return { changes: [], discussions: [], next: null, skipped: [], ...last, loadedAt, error: errorText(err) }
  }
}

// 展開一列時才讀：change 讀文件狀態與任務，討論讀內文。重讀時先留著舊內容，畫面不閃。
const loadDetail = async ($: EngineInterface, id: string) => {
  await update($, details, d => (d[id] === undefined ? { ...d, [id]: { kind: 'loading' as const } } : d))
  let detail: Detail
  try {
    if (id.startsWith('change:')) {
      const name = id.slice('change:'.length)
      const cwd = (await read($, board))?.changes.find(c => c.name === name)?.cwd ?? null
      const [shown, status] = await Promise.all([
        speclinkJson<ShowJson>($, ['show', name], cwd),
        speclinkJson<StatusJson>($, ['status', '--change', name], cwd),
      ])
      detail = {
        kind: 'change',
        artifacts: status.artifacts.map(a => ({ id: a.id, done: a.status === 'done' })),
        groups: parseTasks(shown.tasks),
      }
    } else {
      const shown = await speclinkJson<DiscussShowJson>($, ['discuss', 'show', id.slice('talk:'.length)])
      detail = { kind: 'talk', body: discussionBody(shown.content) }
    }
  } catch (err) {
    detail = { kind: 'error', message: errorText(err) }
  }
  await update($, details, d => ({ ...d, [id]: detail }))
}

// 沒有工單時 CLI 以失敗回報，算「沒有」；其他失敗才是錯誤。
const ticketOf = async ($: EngineInterface, station: 'review' | 'verify', change: string, cwd: string | null) => {
  const args = [station, 'show', change]
  const ran = await runSpeclink($, args, cwd)
  if (ran.exitCode === 0) {
    return toTicket(JSON.parse(ran.stdout) as TicketJson)
  }
  if (isNoTicket(ran.stderr)) {
    return null
  }
  throw failure(args, ran)
}

// 每個 change 各問一次 review 與 verify，所以只在品質分頁開著時讀。
const loadQuality = async ($: EngineInterface) => {
  const changes = (await read($, board))?.changes ?? []
  let loaded: Quality
  try {
    const rows = await Promise.all(
      changes.map(async c => {
        const [review, verify] = await Promise.all([
          ticketOf($, 'review', c.name, c.cwd),
          ticketOf($, 'verify', c.name, c.cwd),
        ])
        return { change: c.name, review, verify }
      }),
    )
    loaded = { rows: rows.filter(r => r.review !== null || r.verify !== null), error: null }
  } catch (err) {
    loaded = { rows: (await read($, quality))?.rows ?? [], error: errorText(err) }
  }
  await update($, quality, () => loaded)
}

// 按鈕不等這些讀完：讀取跑超過 10 秒，按鈕會被引擎判逾時。
const toggleRow = async ($: EngineInterface, id: string) => {
  const opening = !(await read($, view)).open.includes(id)
  await update($, view, v => ({ ...v, open: opening ? [...v.open, id] : v.open.filter(o => o !== id) }))
  // ticket 列的發現已在品質資料裡，不用另外讀。
  if (opening && !id.startsWith('ticket:')) {
    await loadDetail($, id)
  }
}

const toggleSection = ($: EngineInterface, id: string) =>
  update($, view, v => ({ ...v, closed: v.closed.includes(id) ? v.closed.filter(c => c !== id) : [...v.closed, id] }))

const chooseTab = async ($: EngineInterface, tab: PanelTab) => {
  await update($, view, v => ({ ...v, tab }))
  if (tab === 'quality') {
    await loadQuality($)
  }
}

const isPanelOpen = async ($: EngineInterface) => (await $.ui.panes()).some(p => p.id === PANE)

// 展開中的列與品質分頁跟著重讀；不等它們，免得拖住呼叫的 hook。
const refreshBoard = async ($: EngineInterface) => {
  const loaded = await loadBoard($, await read($, board))
  await update($, board, () => loaded)
  const { open, tab } = await read($, view)
  const alive = new Set([
    ...loaded.changes.map(c => `change:${c.name}`),
    ...loaded.discussions.map(d => `talk:${d.slug}`),
  ])
  for (const id of open.filter(o => alive.has(o))) {
    void loadDetail($, id)
  }
  if (tab === 'quality') {
    void loadQuality($)
  }
}

// 回傳切換後面板是否開著。
const togglePanel = async ($: EngineInterface) => {
  if (await isPanelOpen($)) {
    await $.ui.close({ id: PANE })
    return false
  }
  await $.ui.open({ id: PANE, title: 'speclink' })
  await refreshBoard($)
  return true
}

// 分頁顏色用主題色鍵，跟著使用者的深淺主題走。
const GROUP_COLOR: Record<GroupId, string> = {
  plan: 'merged',
  build: 'ide',
  quality: 'claude',
  ship: 'success',
  other: 'inactive',
}

const TABS: readonly PanelTab[] = ['board', 'talk', 'quality']

const SEVERITIES = ['CRITICAL', 'WARNING', 'SUGGESTION'] as const
const SEVERITY_COLOR: Record<string, string> = { CRITICAL: 'error', WARNING: 'warning', SUGGESTION: 'suggestion' }

const STAGES = [
  { id: 'proposed', color: undefined },
  { id: 'in-progress', color: 'suggestion' },
  { id: 'ready', color: 'success' },
] as const

const cut = (text: string, room: number) =>
  text.length <= room ? text : `${text.slice(0, Math.max(1, room - 1))}…`

// 文字在終端機佔幾格：中日韓文字（U+2E80 起）一個字兩格。
export const cells = (text: string) => [...text].reduce((n, ch) => n + (ch.codePointAt(0)! >= 0x2e80 ? 2 : 1), 0)

const mini = (done: number, total: number) => {
  const filled = Math.round((done / total) * 6)
  return '▰'.repeat(filled) + '▱'.repeat(6 - filled)
}

const pad = (n: number) => String(n).padStart(2, '0')
const hhmm = (d: Date) => `${pad(d.getHours())}:${pad(d.getMinutes())}`

export const register: Register = (on, options) => {
  on('session.start', async ($, e, next) => {
    await setup($, options.language)
    await recall($)

    return next(e)
  })

  // `/clear`、`/resume` 換成另一個 session，卻不發 session.start；技能清單空著，技能列就整列不畫。
  // 面板留在畫面上，資料卻是新 session 的（空的），開著就重讀，否則會一直停在「讀取中」。
  // /clear：新 session 沒有焦點；Claude Code 會把舊標題帶過來，是這個 mod 設的就在下一次送出時改回
  // 「Claude Code」，你自己取的不動。/resume：接回的 session 帶著它的標題，從標題找回步驟與 change。
  on('classic.SessionStart', async ($, e, next) => {
    if (e.source !== 'clear' && e.source !== 'resume') {
      return next(e)
    }
    await setup($, options.language)
    if (await isPanelOpen($)) {
      await refreshBoard($)
    }
    const fromTitle = focusFromTitle(e.session_title, known.names)
    if (e.source === 'clear') {
      staleTitle = fromTitle !== null
    } else if (fromTitle !== null) {
      await update($, focus, f => (f.verb === null && f.change === null ? fromTitle : f))
    }

    return next(e)
  })

  // `speclink update` 可能在 session 中途增減技能；這一輪也可能動到面板上的 change。送出技能指令後
  // 還沒跑任何工具就按了 Esc：那一步沒有開始，技能列回到原本的焦點（標題等下一次送出才改得回來）。
  on('turn.complete', async ($, e, next) => {
    const done = await next(e)
    if (e.agentId === undefined) {
      const back = e.isAborted ? undo : null
      undo = null
      if (back !== null) {
        await update($, focus, () => back)
      }
    }
    await refresh($)
    if (await isPanelOpen($)) {
      await refreshBoard($)
    }

    return done
  })

  // 一輪做到一半的 `speclink task done`（或任何 speclink 呼叫）立刻反映到面板；指令裡的 change 名稱
  // 就是這個 session 正在處理的 change。subagent 的指令（帶 agentId）不算。
  on('tool.call', { tool: 'Bash' }, async ($, e, next) => {
    const ran = await next(e)
    const command = speclinkCommand(e.command)
    if (command !== null) {
      if (e.agentId === undefined) {
        await noteCommand($, command)
      }
      if (await isPanelOpen($)) {
        await refreshBoard($)
      }
    }

    return ran
  }).catch(($, e, next) => next(e))

  // 主對話開始跑工具，送出的那一步就算開始了，之後被中斷也不回到上一步。
  on('tool.call', async ($, e, next) => {
    if (e.agentId === undefined) {
      undo = null
    }

    return next(e)
  })

  // 模型自己呼叫的 speclink 技能（例如 quality 裡接著跑 review）：換成那一步。
  on('tool.call', { tool: 'Skill' }, async ($, e, next) => {
    if (e.agentId === undefined && e.skill.startsWith(PREFIX)) {
      await update($, focus, f => ({ ...f, verb: e.skill.slice(PREFIX.length) }))
    }

    return next(e)
  }).catch(($, e, next) => next(e))

  // 你送出訊息時：speclink 技能指令換成新的步驟；然後把「步驟 · change」設成 session 標題，Warp
  // 側欄的分頁名稱就是它。標題只能在送出訊息與開 session 時改，所以技能中途才認出的 change，
  // 要到下一次送出才會出現在標題上。
  on('classic.UserPromptSubmit', async ($, e, next) => {
    const done = await next(e)
    const before = staleTitle ? await read($, focus) : await recall($, e.session_title)
    const after = afterPrompt(before, e.prompt, known.names, PANEL_COMMAND)
    undo = after !== before ? before : null
    if (after !== before) {
      await update($, focus, () => after)
    }
    const title = titleOf(after) ?? (staleTitle ? PLAIN_TITLE : null)
    staleTitle = false

    return title === null ? done : { ...done, sessionTitle: title }
  }).catch(($, e, next) => next(e))

  // 輸入框打 `#`：下拉選單列出提案中的 change，選了就填入名稱，例如 `/speclink-apply #` 再選一個。
  on('prompt.autocomplete', { token: /^#/ }, async ($, e, next) => {
    const below = await next(e)
    const t = TEXT[await read($, lang)]

    return { suggestions: [...below.suggestions, ...proposedRows(await loadPlanned($), e.token, t.stages.proposed)] }
  })

  on('command.run', { command: PANEL_COMMAND }, async $ => {
    const t = TEXT[await read($, lang)]
    const isOpen = await togglePanel($)

    return { text: isOpen ? t.panelOpened : t.panelClosed }
  })

  // 上方空一列，和 Claude Code 自己的訊息分開。每個分頁前面一顆同色圓點，選中的
  // 分頁是有底色的膠囊；沒選中的左右各留一格，切換分頁時位置不會跳動。滑鼠移到技能上，
  // 技能和技能列右邊的說明同屬一個 hover scope，一起亮起來；說明平常藏著，不佔位置。
  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const below = await next(e)
    const installed = await read($, skills)
    const groups = groupSkills(installed.map(s => s.name))
    if (e.props.hasSurvey || groups.length === 0) {
      return below
    }

    const { Box, Button, Text } = $.ui.resolve(e)
    const t = TEXT[await read($, lang)]
    const chosen = await read($, tab)
    const current = groups.find(g => g.id === chosen) ?? groups[0]!
    const color = GROUP_COLOR[current.id]
    // 合併鈕沒有自己的 description，只列指令，不分語言。
    const about = (skill: string) =>
      skill.includes('+')
        ? skill.split('+').map(commandHead).join(' → ')
        : (installed.find(s => s.name === PREFIX + skill)?.description ?? '')

    const tabs = (
      <Box flexWrap="wrap" columnGap={1}>
        {groups.map(g => (
          <Box>
            <Text color={GROUP_COLOR[g.id]}>●</Text>
            {g.id === current.id ? (
              <Text backgroundColor={color} color="inverseText" bold>{` ${t.groups[g.id]} `}</Text>
            ) : (
              <Box paddingX={1}>
                <Button
                  key={`tab:${g.id}`}
                  label={t.groups[g.id]}
                  plain
                  dimColor
                  onPress={() => update($, tab, () => g.id)}
                />
              </Box>
            )}
          </Box>
        ))}
      </Box>
    )
    const panelButton = <Button key="panel" label={`◧ ${t.panel}`} plain dimColor onPress={() => togglePanel($)} />
    // 這個 session 正在跑的步驟與 change：「▸ apply · add-x」，步驟用它所在分頁的顏色。
    const now = await read($, focus)
    const nowGroup = groups.find(g => now.verb !== null && g.skills.includes(now.verb))
    const nowLine = (now.verb !== null || now.change !== null) && (
      <Box>
        <Text dimColor>▸ </Text>
        {now.verb !== null && (
          <Text color={nowGroup === undefined ? undefined : GROUP_COLOR[nowGroup.id]} bold>
            {now.verb}
          </Text>
        )}
        {now.verb !== null && now.change !== null && <Text dimColor> · </Text>}
        {now.change !== null && <Text wrap="truncate-end">{now.change}</Text>}
        {now.archived === true && <Text color="success"> ✓</Text>}
      </Box>
    )
    const skillButtons = (
      <Box flexWrap="wrap" columnGap={2} flexShrink={1}>
        {current.skills.map(skill => (
          <Button
            key={skill}
            label={skill}
            plain
            hover={{ scope: `skill:${skill}`, color }}
            onPress={() => putCommand($, skill)}
          />
        ))}
      </Box>
    )

    // 第一列是「speclink」與目前在處理的步驟、change，下面一列分頁與面板鈕，再下面一列技能。技能列的
    // 寬度（終端機扣掉引擎右邊放 [-] 的五格）放不下「分頁 │ 面板」時改窄版：面板鈕移到第一列最右邊，
    // 分頁不會從中間拆開。窄版右邊沒有地方放技能說明，所以不顯示。
    const title = (
      <Box columnGap={2} flexShrink={1}>
        <Text dimColor>speclink</Text>
        {nowLine}
      </Box>
    )
    const tabsWidth = groups.reduce((n, g) => n + cells(t.groups[g.id]) + 3, groups.length - 1)
    const wideWidth = tabsWidth + 5 + cells(`◧ ${t.panel}`)
    if (e.props.bodyColumns < wideWidth) {
      return (
        <Box flexDirection="column">
          <Box marginTop={1} flexDirection="column" alignSelf="flex-start">
            <Box justifyContent="space-between" columnGap={2}>
              {title}
              {panelButton}
            </Box>
            {tabs}
            <Box paddingLeft={2}>{skillButtons}</Box>
          </Box>
          {below}
        </Box>
      )
    }

    return (
      <Box flexDirection="column">
        <Box marginTop={1} flexDirection="column">
          {title}
          <Box flexWrap="wrap" columnGap={1}>
            {tabs}
            <Text dimColor> │ </Text>
            {panelButton}
          </Box>
          <Box paddingLeft={2}>
            {skillButtons}
            <Box width={0} flexGrow={1} marginLeft={3} flexDirection="column" overflow="hidden">
              {current.skills.map(skill => (
                <Box display="none" hover={{ scope: `skill:${skill}`, display: 'flex' }}>
                  <Text dimColor wrap="truncate-end">
                    {about(skill)}
                  </Text>
                </Box>
              ))}
            </Box>
          </Box>
        </Box>
        {below}
      </Box>
    )
  })

  on('ui.render', { component: 'Pane', requestId: PANE }, async ($, e) => {
    const { Box, Button, Markdown, Text } = $.ui.resolve(e)
    const t = TEXT[await read($, lang)]
    const b = await read($, board)
    const v = await read($, view)
    const shown = await read($, details)
    const q = await read($, quality)
    const width = e.props.bodyColumns
    const isOpen = (id: string) => v.open.includes(id)

    const toggle = (id: string) => (
      <Button key={`open:${id}`} label={isOpen(id) ? '▾' : '▸'} plain dimColor onPress={() => void toggleRow($, id)} />
    )

    // 可收合的區塊：`▾ 提案中 8`，點 ▾ 收起、點 ▸ 打開。
    const section = (id: string, title: string, color: string | undefined, rows: RenderChildren[]) => {
      const closed = v.closed.includes(id)
      return (
        <Box key={`section:${id}`} flexDirection="column" marginTop={1}>
          <Box>
            <Button key={`section:${id}`} label={closed ? '▸' : '▾'} plain dimColor onPress={() => void toggleSection($, id)} />
            <Text bold color={color}>
              {` ${title} `}
              <Text dimColor>{rows.length}</Text>
            </Text>
          </Box>
          {!closed && rows}
        </Box>
      )
    }

    const pending = (detail: Detail | undefined) =>
      detail === undefined || detail.kind === 'loading' ? (
        <Text dimColor>{t.loading}</Text>
      ) : detail.kind === 'error' ? (
        <Text color="error" wrap="truncate-end">
          {detail.message}
        </Text>
      ) : null

    // 展開的 change：文件打勾，再列任務（已勾的畫暗，手動任務標出來）。
    const changeDetail = (c: PanelChange) => {
      const detail = shown[`change:${c.name}`]
      return (
        <Box flexDirection="column" paddingLeft={5}>
          {pending(detail)}
          {detail?.kind === 'change' && (
            <Text wrap="truncate-end">
              {detail.artifacts.map(a => (
                <Text color={a.done ? 'success' : undefined} dimColor={!a.done}>
                  {`${a.done ? '✓' : '○'} ${t.artifacts[a.id] ?? a.id}  `}
                </Text>
              ))}
            </Text>
          )}
          {detail?.kind === 'change' && detail.groups.length === 0 && <Text dimColor>{t.noTasks}</Text>}
          {detail?.kind === 'change' &&
            detail.groups.map(g => (
              <Box flexDirection="column">
                {g.title !== '' && (
                  <Box>
                    <Box flexShrink={1}>
                      <Text dimColor wrap="truncate-end">
                        {g.title}
                      </Text>
                    </Box>
                    <Box flexShrink={0} marginLeft={2}>
                      <Text dimColor>{`${g.tasks.filter(task => task.done).length}/${g.tasks.length}`}</Text>
                    </Box>
                  </Box>
                )}
                {g.tasks.map(task => (
                  <Text wrap="truncate-end" dimColor={task.done}>
                    {task.done ? '☑ ' : '☐ '}
                    {task.manual && <Text color="warning">{`[${t.manual}] `}</Text>}
                    {task.label}
                  </Text>
                ))}
              </Box>
            ))}
        </Box>
      )
    }

    const changeRow = (c: PanelChange) => {
      const isBlocked = c.blockedBy.length > 0
      const tasks = c.total > 0 ? `${mini(c.done, c.total)} ${c.done}/${c.total}` : ''
      const mark = c.branch === null ? '' : ' ⎇'
      const room = width - 5 - mark.length - tasks.length - 1
      return (
        <Box key={`row:${c.name}`} flexDirection="column">
          <Box>
            {toggle(`change:${c.name}`)}
            <Text dimColor>{`${c.wave}`.padStart(3)} </Text>
            <Button
              key={`change:${c.name}`}
              label={cut(c.name, room)}
              plain
              dimColor={isBlocked}
              onPress={() => putArgument($, c.name)}
            />
            {mark !== '' && <Text color="suggestion">{mark}</Text>}
            <Box flexGrow={1} />
            <Text dimColor={isBlocked}>{tasks}</Text>
          </Box>
          {isBlocked && <Text dimColor wrap="truncate-end">{`     ${t.waitsFor(c.blockedBy.join(', '))}`}</Text>}
          {isOpen(`change:${c.name}`) && changeDetail(c)}
        </Box>
      )
    }

    const talkRow = (d: PanelDiscussion) => {
      const detail = shown[`talk:${d.slug}`]
      return (
        <Box key={`talk-row:${d.slug}`} flexDirection="column">
          <Box>
            {toggle(`talk:${d.slug}`)}
            <Text> </Text>
            <Button key={`talk:${d.slug}`} label={cut(d.slug, width - 12)} plain onPress={() => putArgument($, d.slug)} />
            <Box flexGrow={1} />
            <Text dimColor>
              {d.status === 'open' ? t.rounds(d.rounds) : d.status === 'concluded' ? t.concluded : d.status}
            </Text>
          </Box>
          <Text dimColor wrap="truncate-end">{`  ${d.topic}`}</Text>
          {isOpen(`talk:${d.slug}`) && (
            <Box flexDirection="column" paddingLeft={2} marginBottom={1}>
              {pending(detail)}
              {detail?.kind === 'talk' && <Markdown text={detail.body} />}
            </Box>
          )}
        </Box>
      )
    }

    // `審查 第 2 輪 · 1 CRITICAL 2 WARNING`
    const ticketLine = (label: string, ticket: Ticket | null) => (
      <Text wrap="truncate-end">
        <Text dimColor>{`   ${label}  `}</Text>
        {ticket === null && <Text dimColor>{t.noTicket}</Text>}
        {ticket !== null && `${t.round(ticket.round)} · `}
        {ticket !== null && ticket.findings.length === 0 && <Text color="success">{t.noFindings}</Text>}
        {ticket !== null &&
          SEVERITIES.map(severity => {
            const n = ticket.findings.filter(f => f.severity === severity).length
            return n > 0 && <Text color={SEVERITY_COLOR[severity]}>{`${n} ${severity} `}</Text>
          })}
      </Text>
    )

    const findings = (label: string, ticket: Ticket | null) =>
      ticket !== null &&
      ticket.findings.length > 0 && (
        <Box flexDirection="column" marginTop={1}>
          <Text bold>{`${label} · ${t.findings(ticket.findings.length)}`}</Text>
          {ticket.findings.map(f => (
            <Box flexDirection="column">
              <Text wrap="truncate-end">
                <Text color={SEVERITY_COLOR[f.severity]}>{f.severity}</Text>
                <Text dimColor>{` ${f.path}`}</Text>
              </Text>
              <Text wrap="wrap">{f.text}</Text>
            </Box>
          ))}
        </Box>
      )

    const next = b?.next ?? null
    const discussions = b?.discussions ?? []
    const changes = b?.changes ?? []

    const boardTab = (
      <Box flexDirection="column">
        {next !== null && (
          <Box marginTop={1}>
            <Text bold>{t.next}</Text>
            <Button key="next" label={cut(next, width - 9)} plain onPress={() => putArgument($, next)} />
          </Box>
        )}
        {STAGES.map(stage => {
          const items = changes.filter(c => c.stage === stage.id)
          return items.length > 0 && section(`stage:${stage.id}`, t.stages[stage.id], stage.color, items.map(changeRow))
        })}
        {b !== null && b.error === null && changes.length === 0 && (
          <Box marginTop={1}>
            <Text dimColor>{t.noChanges}</Text>
          </Box>
        )}
        {b !== null && b.skipped.length > 0 && <Text dimColor wrap="wrap">{t.skipped(b.skipped.join(', '))}</Text>}
      </Box>
    )

    const open = discussions.filter(d => d.status === 'open')
    const rest = discussions.filter(d => d.status !== 'open')
    const talkTab = (
      <Box flexDirection="column">
        {open.length > 0 && section('talks-open', t.open, 'suggestion', open.map(talkRow))}
        {rest.length > 0 && section('talks-rest', t.concluded, 'success', rest.map(talkRow))}
        {b !== null && b.error === null && discussions.length === 0 && (
          <Box marginTop={1}>
            <Text dimColor>{t.noDiscussions}</Text>
          </Box>
        )}
      </Box>
    )

    const qualityTab = (
      <Box flexDirection="column" marginTop={1}>
        <Text dimColor wrap="wrap">
          {t.ticketNote}
        </Text>
        {q === null && <Text dimColor>{t.loading}</Text>}
        {q?.error && <Text color="error">{q.error}</Text>}
        {q !== null && q.error === null && q.rows.length === 0 && <Text dimColor>{t.noTickets}</Text>}
        {q?.rows.map(r => (
          <Box key={`ticket-row:${r.change}`} flexDirection="column" marginTop={1}>
            <Box>
              {toggle(`ticket:${r.change}`)}
              <Text> </Text>
              <Button
                key={`ticket:${r.change}`}
                label={cut(r.change, width - 3)}
                plain
                onPress={() => putArgument($, r.change)}
              />
            </Box>
            {ticketLine(t.review, r.review)}
            {ticketLine(t.verify, r.verify)}
            {isOpen(`ticket:${r.change}`) && (
              <Box flexDirection="column" paddingLeft={3}>
                {findings(t.review, r.review)}
                {findings(t.verify, r.verify)}
              </Box>
            )}
          </Box>
        ))}
      </Box>
    )

    return (
      <Box flexDirection="column">
        <Box>
          {TABS.map(id =>
            id === v.tab ? (
              <Text backgroundColor="suggestion" color="inverseText" bold>{` ${t.tabs[id]} `}</Text>
            ) : (
              <Box paddingX={1}>
                <Button key={`view:${id}`} label={t.tabs[id]} plain dimColor onPress={() => void chooseTab($, id)} />
              </Box>
            ),
          )}
          <Box flexGrow={1} />
          <Button key="refresh" label={t.refresh} plain dimColor onPress={() => void refreshBoard($)} />
        </Box>
        <Text dimColor>{b === null ? t.loading : t.updated(hhmm(new Date(b.loadedAt)))}</Text>
        {b?.error && <Text color="error">{b.error}</Text>}
        {b?.error && <Text dimColor>{t.cliHint}</Text>}
        {v.tab === 'board' && boardTab}
        {v.tab === 'talk' && talkTab}
        {v.tab === 'quality' && qualityTab}
      </Box>
    )
  })
}
