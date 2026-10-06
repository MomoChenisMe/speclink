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

declare module 'claude-code' {
  interface PluginState {
    'speclink-skills': { skills: Skill[]; tab: string; board: Board | null; lang: Lang }
  }
}
