import { GROUPS } from './groups'
import type { GroupId } from './groups'

export const PREFIX = 'speclink-'

// 輸入框最前面的斜線指令；合併鈕填入的多個指令以 ` + ` 相連，視為同一段。
export const COMMAND_HEAD = /^\/\S+(?:\s+\+\s+\/\S+)*/

// 已安裝技能（完整名稱）分進各分頁；合併鈕要每個技能都在才出現。
export const groupSkills = (installed: string[]): { id: GroupId; skills: string[] }[] => {
  const short = installed.map(n => n.slice(PREFIX.length))
  const known = new Set(GROUPS.flatMap(g => g.skills.flatMap(s => s.split('+'))))
  const unknown = short.filter(s => !known.has(s))
  return GROUPS.map(g => ({
    id: g.id,
    skills: [
      ...g.skills.filter(s => s.split('+').every(part => short.includes(part))),
      ...(g.id === 'other' ? unknown : []),
    ],
  })).filter(g => g.skills.length > 0)
}

// `archive+commit` → `/speclink-archive + /speclink-commit`
export const commandHead = (skill: string) =>
  skill
    .split('+')
    .map(part => `/${PREFIX}${part}`)
    .join(' + ')

// 指令放最前面，已經打的字留在後面當參數；原本最前面的指令會被換掉。
export const withCommand = (draft: string, head: string) =>
  `${head} ${draft.replace(COMMAND_HEAD, '').trimStart()}`

// 最前面有指令時，`arg` 成為它的參數（換掉原本的參數）；沒有指令時接在已打的字後面。
export const withArgument = (draft: string, arg: string) => {
  const head = draft.match(COMMAND_HEAD)?.[0]
  if (head !== undefined) {
    return `${head} ${arg}`
  }
  return draft.trim() === '' ? arg : `${draft.trimEnd()} ${arg}`
}
