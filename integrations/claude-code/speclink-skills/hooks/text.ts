import type { Lang, Stage } from '../types'

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
  discussions: string
  rounds: (n: number) => string
  concluded: string
  stages: Record<Stage, string>
  waitsFor: (names: string) => string
  empty: string
  skipped: (names: string) => string
  cliHint: string
}

// 介面文字。階段、輪數、空狀態沿用 desktop 的正典詞彙（packages/ui 的 stage.*、
// common.rounds、board.empty），中英兩版對等。
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
    discussions: 'Discussions',
    rounds: n => `${n} rounds`,
    concluded: 'Concluded',
    stages: { proposed: 'Proposed', 'in-progress': 'In progress', ready: 'Ready' },
    waitsFor: names => `Waiting on ${names}`,
    empty: 'No active changes or discussions.',
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
    discussions: '討論',
    rounds: n => `${n} 輪`,
    concluded: '已結論',
    stages: { proposed: '提案中', 'in-progress': '進行中', ready: '已就緒' },
    waitsFor: names => `等 ${names}`,
    empty: '沒有 active change 或討論。',
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
