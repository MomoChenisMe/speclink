import type { Focus } from '../types'

import { COMMAND_HEAD, PREFIX } from './skills'

// 這個 session 正在跑的 speclink 步驟與 change：從你送出的技能指令、模型呼叫的技能，以及 speclink
// CLI 參數裡的 change 名稱找出來。技能一定會用 change 名稱呼叫 CLI，所以就算你沒打名稱也找得到。

// `/speclink-apply add-x` → apply；`/speclink-archive + /speclink-commit` → archive+commit。對話紀錄裡
// 的技能指令寫成 `<command-name>/speclink-apply</command-name>`。不是 speclink 技能（含面板指令）回 null。
export const verbOf = (prompt: string, panelCommand: string) => {
  const text = prompt.trimStart().replace(/^<command-name>/, '')
  const head = text.match(COMMAND_HEAD)?.[0] ?? ''
  const names = [...head.matchAll(/\/(\S+)/g)].map(m => m[1]!).filter(n => n.startsWith(PREFIX) && n !== panelCommand)
  return names.length === 0 ? null : names.map(n => n.slice(PREFIX.length).replace(/<.*$/, '')).join('+')
}

// 文字裡出現的已知 change 名稱，最後一個為準；只比完整的詞，不會把名稱的一部分當成 change。
export const changeIn = (text: string, names: readonly string[]) =>
  text.split(/[^A-Za-z0-9._-]+/).filter(word => names.includes(word)).at(-1) ?? null

// speclink CLI 的指令（`speclink status --change x`），不是就回 null。
export const speclinkCommand = (command: unknown) =>
  typeof command === 'string' && /(^|[\s;&|(])speclink\s/.test(command) ? command : null

// 送出的訊息帶來的變動：speclink 技能指令換成新的步驟，指令裡有 change 名稱就換掉 change，
// 沒有就留著上一個（通常是同一個 change 的下一步）。其他訊息不動。
export const afterPrompt = (focus: Focus, prompt: string, names: readonly string[], panelCommand: string): Focus => {
  const verb = verbOf(prompt, panelCommand)
  return verb === null ? focus : { verb, change: changeIn(prompt, names) ?? focus.change }
}

// 接回的對話（/resume）裡最後的步驟與 change：依序看你送出的技能指令、模型呼叫的技能與 speclink 指令。
export const focusFromHistory = (
  messages: readonly { role: string; text: string; toolUses: readonly { tool: string; input: Record<string, unknown> }[] }[],
  names: readonly string[],
  panelCommand: string,
): Focus => {
  let focus: Focus = { verb: null, change: null }
  for (const message of messages) {
    if (message.role === 'user') {
      focus = afterPrompt(focus, message.text, names, panelCommand)
    }
    for (const use of message.toolUses) {
      const skill = use.tool === 'Skill' ? use.input.skill : undefined
      if (typeof skill === 'string' && skill.startsWith(PREFIX)) {
        focus = { ...focus, verb: skill.slice(PREFIX.length) }
      }
      const command = use.tool === 'Bash' ? speclinkCommand(use.input.command) : null
      const change = command === null ? null : changeIn(command, names)
      if (change !== null) {
        focus = { ...focus, change }
      }
    }
  }
  return focus
}

// session 標題：「apply · add-x」，封存完是「archive · add-x ✓」；兩個都還不知道時回 null，標題維持原樣。
export const titleOf = (focus: Focus) => {
  const title = [focus.verb, focus.change].filter(part => part !== null).join(' · ')
  return title === '' ? null : focus.archived === true ? `${title} ✓` : title
}

// /clear 時標題改回的字樣（Claude Code 預設顯示的）；空字串會被引擎忽略，清不掉。
export const PLAIN_TITLE = 'Claude Code'

// 把這個 mod 設的標題解析回步驟與 change。/resume 接回的對話若被 compact 過，紀錄裡已經沒有 speclink
// 指令，靠標題找回。change 要是現有的 change，或是標了 ✓ 的封存，免得把你自己取的標題當成 change。
export const focusFromTitle = (title: string | undefined, names: readonly string[]): Focus | null => {
  const m = title?.match(/^(?:([a-z][a-z+-]*) · )?([A-Za-z0-9._-]+)( ✓)?$/)
  if (m === undefined || m === null) {
    return null
  }
  const [, verb = null, change = '', done] = m
  if (done !== undefined) {
    return verb === null ? null : { verb, change, archived: true }
  }
  return names.includes(change) ? { verb, change } : null
}
