// 更新日誌的型別面（release-notes spec「更新日誌 JSON 為唯一真相」）：JSON 由 Vite 原生
// 打包，這裡只把 kind 由 string 收窄為三值字面聯集、條目收窄為雙語物件並匯出常數；內容不在
// 這裡手寫。分組標題不在 JSON，由呈現端以 releaseNotes.kind.* 依介面語言決定。
import type { UiLocale } from "@speclink/ui";

import notes from "./release-notes.json";

export type ReleaseNotesKind = "features" | "fixes" | "improvements";

/** 一個條目的兩種語言寫法；鍵與介面語言 UiLocale 同字面，兩種都必填。 */
export type ReleaseNotesItem = Record<UiLocale, string>;

export interface ReleaseNotesSection {
  kind: ReleaseNotesKind;
  items: ReleaseNotesItem[];
}

export interface ReleaseNotesEntry {
  /** 不帶 v，與 tauri.conf.json 的 version 同字面。 */
  version: string;
  /** YYYY-MM-DD */
  date: string;
  sections: ReleaseNotesSection[];
}

/** 全部條目，最新在前。 */
export const RELEASE_NOTES = notes as ReleaseNotesEntry[];

/** 條目在該介面語言的文字（desktop-app spec「更新日誌依介面語言呈現」）。 */
export function entryText(item: ReleaseNotesItem, locale: UiLocale): string {
  return item[locale];
}
