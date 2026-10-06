import type { Lang, PanelTab, Stage } from '../types'

import type { GroupId } from './groups'

type Text = {
  groups: Record<GroupId, string>
  panel: string
  panelCommand: string
  panelOpened: string
  panelClosed: string
  loading: string
  updated: (time: string) => string
  refresh: string
  next: string
  tabs: Record<PanelTab, string>
  rounds: (n: number) => string
  open: string
  concluded: string
  stages: Record<Stage, string>
  waitsFor: (names: string) => string
  noChanges: string
  noDiscussions: string
  artifacts: Record<string, string>
  manual: string
  noTasks: string
  review: string
  verify: string
  round: (n: number) => string
  findings: (n: number) => string
  noFindings: string
  noTicket: string
  noTickets: string
  ticketNote: string
  skipped: (names: string) => string
  cliHint: string
}

// 介面文字。階段、輪數、空狀態、文件、任務與工單沿用 desktop 的正典詞彙（packages/ui 的
// stage.*、common.rounds、board.empty、common.tab*、tasks.*、ticket.*、discussion.status*），
// 中英兩版對等。
export const TEXT: Record<Lang, Text> = {
  en: {
    groups: { plan: 'Plan', build: 'Build', quality: 'Quality', ship: 'Ship', other: 'More' },
    panel: 'Panel',
    panelCommand: 'Toggle the speclink side panel',
    panelOpened: 'speclink panel opened.',
    panelClosed: 'speclink panel closed.',
    loading: 'Loading…',
    updated: time => `Updated ${time}`,
    refresh: '↻ Refresh',
    next: 'Next ▸ ',
    tabs: { board: 'Board', talk: 'Discussions', quality: 'Quality' },
    rounds: n => `${n} rounds`,
    open: 'Open',
    concluded: 'Concluded',
    stages: { proposed: 'Proposed', 'in-progress': 'In progress', ready: 'Ready' },
    waitsFor: names => `Waiting on ${names}`,
    noChanges: 'No active changes',
    noDiscussions: 'No discussions',
    artifacts: { proposal: 'Proposal', design: 'Design', specs: 'Specs', tasks: 'Tasks' },
    manual: 'Manual',
    noTasks: '(no tasks)',
    review: 'Review',
    verify: 'Verify',
    round: n => `Round ${n}`,
    findings: n => `${n} findings`,
    noFindings: 'No findings this round',
    noTicket: 'No ticket',
    noTickets: 'No open review or verify tickets.',
    ticketNote: 'A stamp deletes its ticket, so only open tickets show.',
    skipped: names => `Skipped (bad meta): ${names}`,
    cliHint: 'The panel needs the speclink CLI on PATH.',
  },
  'zh-TW': {
    groups: { plan: '規劃', build: '實作', quality: '品質', ship: '收尾', other: '其他' },
    panel: '面板',
    panelCommand: '開關 speclink 側邊面板',
    panelOpened: 'speclink 面板已開啟。',
    panelClosed: 'speclink 面板已關閉。',
    loading: '讀取中…',
    updated: time => `更新於 ${time}`,
    refresh: '↻ 重新整理',
    next: '下一步 ▸ ',
    tabs: { board: '看板', talk: '討論', quality: '品質' },
    rounds: n => `${n} 輪`,
    open: '討論中',
    concluded: '已結論',
    stages: { proposed: '提案中', 'in-progress': '進行中', ready: '已就緒' },
    waitsFor: names => `等 ${names}`,
    noChanges: '沒有 active change',
    noDiscussions: '沒有討論',
    artifacts: { proposal: '提案', design: '設計', specs: '規格', tasks: '任務' },
    manual: '手動',
    noTasks: '（無任務）',
    review: '審查',
    verify: '驗證',
    round: n => `第 ${n} 輪`,
    findings: n => `${n} 條`,
    noFindings: '本輪無發現',
    noTicket: '沒有工單',
    noTickets: '沒有未結的審查或驗證工單。',
    ticketNote: '蓋章後工單會刪除，這裡只列未結的工單。',
    skipped: names => `略過（meta 有誤）：${names}`,
    cliHint: '面板需要 PATH 上有 speclink CLI。',
  },
}

// `auto` 跟著 Claude Code 自己的 `language` 設定：寫的是中文就用繁中，其餘用英文。
export const resolveLang = (option: unknown, claudeLanguage: unknown): Lang => {
  if (option === 'en' || option === 'zh-TW') {
    return option
  }
  return typeof claudeLanguage === 'string' && /中文|chinese|zh/i.test(claudeLanguage) ? 'zh-TW' : 'en'
}
