// 外觀偏好（desktop-settings-mode design D4）：localStorage 單鍵，null 表跟隨系統。明示淺色／
// 深色以根元素的 data-theme 屬性觸發 theme.css 的對應區塊；server-web 不讀此鍵、不設屬性。

/** null＝跟隨系統的淺色／深色偏好。 */
export type ThemePreference = "light" | "dark" | null;

/** 偏好鍵；desktop 的 index.html 開機腳本讀同一個鍵（首次繪製前先套，避免先閃系統主題）。 */
export const THEME_STORAGE_KEY = "speclink.theme";

/** 讀取偏好；缺鍵、非法值（手改 localStorage）或儲存區不可用一律視為跟隨系統。 */
export function readThemePreference(storage: Storage = localStorage): ThemePreference {
  try {
    const v = storage.getItem(THEME_STORAGE_KEY);
    return v === "light" || v === "dark" ? v : null;
  } catch {
    return null;
  }
}

/** 寫入偏好；儲存區不可用（停用、配額滿）時靜默略過——本次照常套用，只是不跨啟動保留。 */
export function writeThemePreference(pref: ThemePreference, storage: Storage = localStorage): void {
  try {
    if (pref === null) storage.removeItem(THEME_STORAGE_KEY);
    else storage.setItem(THEME_STORAGE_KEY, pref);
  } catch {
    /* 見上 */
  }
}

/** 套用偏好：明示值設 data-theme，跟隨系統移除屬性（交回 prefers-color-scheme）。 */
export function applyThemePreference(
  pref: ThemePreference,
  root: HTMLElement = document.documentElement,
): void {
  if (pref === null) root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", pref);
}
