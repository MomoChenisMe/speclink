// UI 語言偏好（design D8）：localStorage 單鍵，null 表跟隨系統；
// 與 config.yaml 的 locale（AI artifacts 產出語言）互不影響。
import type { UiLocale } from "@speclink/ui";

/** null＝跟隨系統語言。 */
export type LocalePreference = UiLocale | null;

const STORAGE_KEY = "speclink.uiLocale";

/** 系統語言判定：以 zh 開頭 → zh-TW，其餘（含未知）→ en。 */
export function detectSystemLocale(language: string | undefined): UiLocale {
  return language?.toLowerCase().startsWith("zh") ? "zh-TW" : "en";
}

/** 讀取偏好；缺鍵或非法值（手改 localStorage）一律視為未設定。 */
export function readLocalePreference(storage: Storage = localStorage): LocalePreference {
  const v = storage.getItem(STORAGE_KEY);
  return v === "zh-TW" || v === "en" ? v : null;
}

export function writeLocalePreference(pref: LocalePreference, storage: Storage = localStorage): void {
  if (pref === null) storage.removeItem(STORAGE_KEY);
  else storage.setItem(STORAGE_KEY, pref);
}

/** 有效 UI 語言：明示偏好優先，null 跟隨系統。 */
export function resolveUiLocale(pref: LocalePreference, systemLanguage: string | undefined): UiLocale {
  return pref ?? detectSystemLocale(systemLanguage);
}
