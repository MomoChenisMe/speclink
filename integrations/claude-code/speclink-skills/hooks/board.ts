import type { Stage } from '../types'

// 面板從 CLI `--json` 輸出讀的欄位。
export type ListJson = {
  changes: { name: string; completedTasks: number; totalTasks: number; worktree?: { branch: string } }[]
}
export type PlanJson = {
  changes: { name: string; wave: number; stage: Stage; blockedBy: string[] }[]
  next: string | null
  skipped: { change: string }[]
}
export type DiscussJson = {
  discussions: { slug: string; topic: string; rounds: number; status: string }[]
}

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
      }
    }),
    discussions: talk.discussions
      .filter(d => d.status !== 'promoted')
      .map(({ slug, topic, rounds, status }) => ({ slug, topic, rounds, status })),
    next: plan.next,
    skipped: plan.skipped.map(s => s.change),
  }
}

// `$.process.run` 不經 shell。Windows 上 npm 裝的 `speclink` 是 `.cmd` 殼，只能經
// cmd.exe 執行；桌面版裝的 `speclink.exe` 經 cmd.exe 也一樣找得到。
export const speclinkArgv = (args: string[], isWindows: boolean) =>
  isWindows ? ['cmd.exe', '/d', '/c', 'speclink', ...args] : ['speclink', ...args]
