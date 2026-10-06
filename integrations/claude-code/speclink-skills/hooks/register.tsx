import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register, RenderChildren } from 'claude-code'

import type { Board, Detail, Lang, PanelChange, PanelDiscussion, PanelTab, PanelView, Quality, Skill, Ticket } from '../types'

import { buildBoard, discussionBody, isNoTicket, parseTasks, speclinkArgv, toTicket } from './board'
import type { DiscussJson, DiscussShowJson, ListJson, PlanJson, ShowJson, StatusJson, TicketJson } from './board'
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

const mini = (done: number, total: number) => {
  const filled = Math.round((done / total) * 6)
  return '▰'.repeat(filled) + '▱'.repeat(6 - filled)
}

const pad = (n: number) => String(n).padStart(2, '0')
const hhmm = (d: Date) => `${pad(d.getHours())}:${pad(d.getMinutes())}`

export const register: Register = (on, options) => {
  on('session.start', async ($, e, next) => {
    await setup($, options.language)

    return next(e)
  })

  // `/clear` 換成新的 session，卻不發 session.start；技能清單空著，技能列就整列不畫。
  on('classic.SessionStart', async ($, e, next) => {
    if (e.source === 'clear') {
      await setup($, options.language)
    }

    return next(e)
  })

  // `speclink update` 可能在 session 中途增減技能；這一輪也可能動到面板上的 change。
  on('turn.complete', async ($, e, next) => {
    const done = await next(e)
    await refresh($)
    if (await isPanelOpen($)) {
      await refreshBoard($)
    }

    return done
  })

  // 一輪做到一半的 `speclink task done`（或任何 speclink 呼叫）立刻反映到面板。
  on('tool.call', { tool: 'Bash' }, async ($, e, next) => {
    const ran = await next(e)
    if (/\bspeclink\s/.test(e.command) && (await isPanelOpen($))) {
      await refreshBoard($)
    }

    return ran
  }).catch(($, e, next) => next(e))

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

    return (
      <Box flexDirection="column">
        <Box marginTop={1}>
          <Box flexShrink={0} marginRight={2}>
            <Text dimColor>speclink</Text>
          </Box>
          <Box flexDirection="column" flexGrow={1}>
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
              <Text dimColor> │ </Text>
              <Button key="panel" label={`◧ ${t.panel}`} plain dimColor onPress={() => togglePanel($)} />
            </Box>
            <Box paddingLeft={2}>
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
