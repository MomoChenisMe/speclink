// 更新日誌的型別面（release-notes spec「更新日誌 JSON 為唯一真相」）：JSON 由 Vite 原生
// 打包，這裡只把 title 由 string 收窄為三值字面聯集並匯出常數；內容不在這裡手寫。
import notes from "./release-notes.json";

export type ReleaseNotesSectionTitle = "新功能" | "修正" | "改善";

export interface ReleaseNotesSection {
  title: ReleaseNotesSectionTitle;
  items: string[];
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
