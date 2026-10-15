// 外觀偏好（desktop-settings-mode design D4）：localStorage 單鍵，null 表跟隨系統。明示淺色／
// 深色以根元素的 data-theme 屬性觸發 theme.css 的對應區塊；server-web 不讀此鍵、不設屬性。

/** null＝跟隨系統的淺色／深色偏好。 */
export type ThemePreference = "light" | "dark" | null;

/** 偏好鍵；desktop 的 index.html 開機腳本讀同一個鍵（首次繪製前先套，避免先閃系統主題）。 */
export const THEME_STORAGE_KEY = "speclink.theme";
const LEGACY_THEME_STORAGE_KEY = "speclink.uiTheme";

/** 讀取偏好；缺鍵、非法值（手改 localStorage）或儲存區不可用一律視為跟隨系統。 */
export function readThemePreference(storage: Storage = localStorage): ThemePreference {
  try {
    const v = storage.getItem(THEME_STORAGE_KEY) ?? storage.getItem(LEGACY_THEME_STORAGE_KEY);
    return v === "light" || v === "dark" ? v : null;
  } catch {
    return null;
  }
}

/** 先保存再由呼叫端套用；失敗向設定頁回報，不顯示未保存的選擇。 */
export function writeThemePreference(pref: ThemePreference, storage: Storage = localStorage): void {
  if (pref === null) {
    storage.removeItem(LEGACY_THEME_STORAGE_KEY);
    storage.removeItem(THEME_STORAGE_KEY);
  } else {
    storage.setItem(THEME_STORAGE_KEY, pref);
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
