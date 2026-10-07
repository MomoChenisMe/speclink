// 已安裝的 speclink 技能：完整名稱與輸入框提示用的那行說明。
export type Skill = { name: string; description: string }

export type Lang = 'en' | 'zh-TW'

export type Stage = 'proposed' | 'in-progress' | 'ready'

export type PanelChange = {
  name: string
  stage: Stage
  wave: number
  blockedBy: string[]
  done: number
  total: number
  branch: string | null
  // worktree 的資料夾；change 在 worktree 裡時，show／status 要在這裡跑才讀得到它那份。
  cwd: string | null
}

export type PanelDiscussion = { slug: string; topic: string; rounds: number; status: string }

export type Board = {
  changes: PanelChange[]
  discussions: PanelDiscussion[]
  next: string | null
  skipped: string[]
  loadedAt: string
  error: string | null
}

export type PanelTab = 'board' | 'talk' | 'quality'

// 面板的分頁、收起的區塊、展開的列（`change:<name>`、`talk:<slug>`、`ticket:<name>`）。
export type PanelView = { tab: PanelTab; closed: string[]; open: string[] }

export type TaskItem = { done: boolean; manual: boolean; label: string }

export type TaskGroup = { title: string; tasks: TaskItem[] }

export type Finding = { severity: string; path: string; text: string }

export type Ticket = { round: number; findings: Finding[] }

// 展開一列時才讀的內容。
export type Detail =
  | { kind: 'loading' }
  | { kind: 'error'; message: string }
  | { kind: 'change'; artifacts: { id: string; done: boolean }[]; groups: TaskGroup[] }
  | { kind: 'talk'; body: string }

export type QualityRow = { change: string; review: Ticket | null; verify: Ticket | null }

// 品質分頁：切過去才讀；null 是還沒讀完。
export type Quality = { rows: QualityRow[]; error: string | null }

// 這個 session 正在跑的 speclink 步驟（技能名稱去掉 speclink-，例如 apply）與 change；還不知道是 null。
// archived：這個 change 已經在這個 session 封存。
export type Focus = { verb: string | null; change: string | null; archived?: true }

declare module 'claude-code' {
  interface PluginState {
    'speclink-skills': {
      skills: Skill[]
      tab: string
      board: Board | null
      lang: Lang
      view: PanelView
      details: Record<string, Detail>
      quality: Quality | null
      focus: Focus
    }
  }
}
