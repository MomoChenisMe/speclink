import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register } from 'claude-code'

import type { Board, Lang, PanelChange, SkillNames } from '../types'

import { buildBoard, speclinkArgv } from './board'
import type { DiscussJson, ListJson, PlanJson } from './board'
import { commandHead, groupSkills, PREFIX, withArgument, withCommand } from './skills'
import { resolveLang, TEXT } from './text'

const PANE = 'speclink-panel'
const PANEL_COMMAND = 'speclink-panel'
const names = atom({ plugin: 'speclink-skills', key: 'names' } as const, [] as SkillNames)
const tab = atom({ plugin: 'speclink-skills', key: 'tab' } as const, '')
const board = atom({ plugin: 'speclink-skills', key: 'board' } as const, null as Board | null)
const lang = atom({ plugin: 'speclink-skills', key: 'lang' } as const, 'en' as Lang)

// 用 `$` 的函式都留在這個檔：`$` 只能傳進同檔宣告的函式，不能跨 import。

const refresh = async ($: EngineInterface) => {
  const found = (await $.command.list())
    .map(c => c.name)
    .filter(n => n.startsWith(PREFIX) && n !== PANEL_COMMAND)
  await update($, names, () => found)
}

const putCommand = async ($: EngineInterface, skill: string) => {
  const { text } = await $.prompt.read()
  await $.prompt.fill({ text: withCommand(text, commandHead(skill)) })
}

const putArgument = async ($: EngineInterface, arg: string) => {
  const { text } = await $.prompt.read()
  await $.prompt.fill({ text: withArgument(text, arg) })
}

const speclinkJson = async <T,>($: EngineInterface, args: string[]): Promise<T> => {
  const isWindows = (await $.env.get('OS')) === 'Windows_NT'
  const ran = await $.process.run(speclinkArgv([...args, '--json'], isWindows))
  if (ran.exitCode !== 0) {
    throw new Error(`speclink ${args.join(' ')}: ${ran.stderr.trim() || `exit ${ran.exitCode}`}`)
  }
  return JSON.parse(ran.stdout) as T
}

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
    const error = err instanceof Error ? err.message : String(err)
    return { changes: [], discussions: [], next: null, skipped: [], ...last, loadedAt, error }
  }
}

const isPanelOpen = async ($: EngineInterface) => (await $.ui.panes()).some(p => p.id === PANE)

const refreshBoard = async ($: EngineInterface) => {
  const loaded = await loadBoard($, await read($, board))
  await update($, board, () => loaded)
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
    const chosen = resolveLang(options.language, (await $.settings.read()).language)
    await update($, lang, () => chosen)
    await $.command.register({ name: PANEL_COMMAND, description: TEXT[chosen].panelCommand })
    await refresh($)

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

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const below = await next(e)
    const groups = groupSkills(await read($, names))
    if (e.props.hasSurvey || groups.length === 0) {
      return below
    }

    const { Box, Button, Text } = $.ui.resolve(e)
    const t = TEXT[await read($, lang)]
    const chosen = await read($, tab)
    const current = groups.find(g => g.id === chosen) ?? groups[0]!

    return (
      <Box flexDirection="column">
        <Box>
          <Text dimColor>speclink </Text>
          <Box flexDirection="column">
            <Box flexWrap="wrap" columnGap={2}>
              {groups.map(g => {
                const look = g.id === current.id ? { variant: 'primary' as const } : { plain: true as const, dimColor: true }
                return (
                  <Button
                    key={`tab:${g.id}`}
                    label={t.groups[g.id]}
                    {...look}
                    onPress={() => update($, tab, () => g.id)}
                  />
                )
              })}
              <Text dimColor>│</Text>
              <Button key="panel" label={t.panel} plain dimColor onPress={() => togglePanel($)} />
            </Box>
            <Box flexWrap="wrap" columnGap={1}>
              {current.skills.flatMap((skill, i) => [
                i > 0 && <Text dimColor>·</Text>,
                <Button key={skill} label={skill} plain onPress={() => putCommand($, skill)} />,
              ])}
            </Box>
          </Box>
        </Box>
        {below}
      </Box>
    )
  })

  on('ui.render', { component: 'Pane', requestId: PANE }, async ($, e) => {
    const { Box, Button, Text } = $.ui.resolve(e)
    const t = TEXT[await read($, lang)]
    const b = await read($, board)
    const width = e.props.bodyColumns

    const changeRow = (c: PanelChange) => {
      const isBlocked = c.blockedBy.length > 0
      const tasks = c.total > 0 ? `${mini(c.done, c.total)} ${c.done}/${c.total}` : ''
      const mark = c.branch === null ? '' : ' ⎇'
      const room = width - 3 - mark.length - tasks.length - 1
      return (
        <Box key={`row:${c.name}`} flexDirection="column">
          <Box>
            <Text dimColor>{`${c.wave}`.padStart(2)} </Text>
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
          {isBlocked && <Text dimColor wrap="truncate-end">{`   ${t.waitsFor(c.blockedBy.join(', '))}`}</Text>}
        </Box>
      )
    }

    const next = b?.next ?? null
    const discussions = b?.discussions ?? []
    const changes = b?.changes ?? []

    return (
      <Box flexDirection="column">
        <Box>
          <Text dimColor>{b === null ? t.loading : t.updated(hhmm(new Date(b.loadedAt)))}</Text>
          <Box flexGrow={1} />
          <Button key="refresh" label={t.refresh} plain dimColor onPress={() => refreshBoard($)} />
        </Box>
        {b?.error && <Text color="error">{b.error}</Text>}
        {b?.error && <Text dimColor>{t.cliHint}</Text>}
        {next !== null && (
          <Box marginTop={1}>
            <Text bold>{t.next}</Text>
            <Button key="next" label={cut(next, width - 9)} plain onPress={() => putArgument($, next)} />
          </Box>
        )}
        {discussions.length > 0 && (
          <Box flexDirection="column" marginTop={1}>
            <Text bold>
              {t.discussions} <Text dimColor>{discussions.length}</Text>
            </Text>
            {discussions.map(d => (
              <Box key={`talk-row:${d.slug}`} flexDirection="column">
                <Box>
                  <Text>{'   '}</Text>
                  <Button
                    key={`talk:${d.slug}`}
                    label={cut(d.slug, width - 10)}
                    plain
                    onPress={() => putArgument($, d.slug)}
                  />
                  <Box flexGrow={1} />
                  <Text dimColor>
                    {d.status === 'open' ? t.rounds(d.rounds) : d.status === 'concluded' ? t.concluded : d.status}
                  </Text>
                </Box>
                <Text dimColor wrap="truncate-end">{`   ${d.topic}`}</Text>
              </Box>
            ))}
          </Box>
        )}
        {STAGES.map(stage => {
          const items = changes.filter(c => c.stage === stage.id)
          return (
            items.length > 0 && (
              <Box key={`stage:${stage.id}`} flexDirection="column" marginTop={1}>
                <Text bold color={stage.color}>
                  {t.stages[stage.id]} <Text dimColor>{items.length}</Text>
                </Text>
                {items.map(changeRow)}
              </Box>
            )
          )
        })}
        {b !== null && b.error === null && discussions.length === 0 && changes.length === 0 && (
          <Text dimColor>{t.empty}</Text>
        )}
        {b !== null && b.skipped.length > 0 && <Text dimColor wrap="wrap">{t.skipped(b.skipped.join(', '))}</Text>}
      </Box>
    )
  })
}
