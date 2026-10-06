import type { Stage, TaskGroup, Ticket } from '../types'

// 面板從 CLI `--json` 輸出讀的欄位。
export type ListJson = {
  changes: {
    name: string
    completedTasks: number
    totalTasks: number
    worktree?: { path: string; branch: string }
  }[]
}
export type PlanJson = {
  changes: { name: string; wave: number; stage: Stage; blockedBy: string[] }[]
  next: string | null
  skipped: { change: string }[]
}
export type DiscussJson = {
  discussions: { slug: string; topic: string; rounds: number; status: string }[]
}
export type ShowJson = { tasks: string }
export type StatusJson = { artifacts: { id: string; status: string }[] }
export type DiscussShowJson = { content: string }
export type TicketJson = { lastRound: { index: number; findings: { severity: string; path: string; text: string }[] } }

// 順序與階段取 plan（與 desktop 看板同一條規則），任務數與 worktree 取 list。
export const buildBoard = (list: ListJson, plan: PlanJson, talk: DiscussJson) => {
  const listed = new Map(list.changes.map(c => [c.name, c]))
  return {
    changes: plan.changes.map(p => {
      const c = listed.get(p.name)
      return {
        name: p.name,
        stage: p.stage,
        wave: p.wave,
        blockedBy: p.blockedBy,
        done: c?.completedTasks ?? 0,
        total: c?.totalTasks ?? 0,
        branch: c?.worktree?.branch ?? null,
        cwd: c?.worktree?.path ?? null,
      }
    }),
    discussions: talk.discussions
      .filter(d => d.status !== 'promoted')
      .map(({ slug, topic, rounds, status }) => ({ slug, topic, rounds, status })),
    next: plan.next,
    skipped: plan.skipped.map(s => s.change),
  }
}

// tasks.md 的 `## 標題` 分組與 `- [ ]`／`- [x]` 任務列；`[M]` 緊貼核取框的是手動任務。
// 行尾的 `<!-- speclink-task:… -->` 是引擎的 ID，不顯示。
export const parseTasks = (markdown: string): TaskGroup[] => {
  const groups: TaskGroup[] = []
  for (const line of markdown.split('\n')) {
    const heading = line.match(/^#{2,}\s+(.+?)\s*$/)
    if (heading) {
      groups.push({ title: heading[1] ?? '', tasks: [] })
      continue
    }
    const task = line.match(/^\s*- \[([ xX])\]\s+(\[M\]\s+)?(.*?)\s*(<!--.*-->)?\s*$/)
    if (task) {
      if (groups.length === 0) {
        groups.push({ title: '', tasks: [] })
      }
      groups[groups.length - 1]?.tasks.push({ done: task[1] !== ' ', manual: task[2] !== undefined, label: task[3] ?? '' })
    }
  }
  return groups.filter(g => g.tasks.length > 0)
}

// 工單只看最後一輪：第幾輪、還有哪些發現。
export const toTicket = (json: TicketJson): Ticket => ({
  round: json.lastRound.index,
  findings: json.lastRound.findings,
})

// `review show`／`verify show` 在沒有工單時以這句話失敗，不算錯誤。
export const isNoTicket = (stderr: string) => /no (review|verify) ticket/.test(stderr)

// 討論檔開頭的 YAML front matter 是 metadata，`<!-- -->` 是範本給寫作者的規則；
// 面板只畫內文，Markdown 會把註解原樣印出來。
export const discussionBody = (content: string) =>
  content
    .replace(/\r/g, '')
    .replace(/^---\n[\s\S]*?\n---\n/, '')
    .replace(/<!--[\s\S]*?-->\n?/g, '')
    .replace(/\n{3,}/g, '\n\n')
    .trim()

// `$.process.run` 不經 shell。Windows 上 npm 裝的 `speclink` 是 `.cmd` 殼，只能經
// cmd.exe 執行；桌面版裝的 `speclink.exe` 經 cmd.exe 也一樣找得到。
export const speclinkArgv = (args: string[], isWindows: boolean) =>
  isWindows ? ['cmd.exe', '/d', '/c', 'speclink', ...args] : ['speclink', ...args]
