import { createContext, useContext, type ReactNode } from "react";
import type { LocalePreference, UiLocale } from "@speclink/ui";

// 語言偏好由 App 持有（它同時決定 I18nProvider 的 locale），帳號選單與登入卡片下方的切換鈕需要讀寫它。
// 只有這一組值，不值得引入狀態管理；context 讓「誰在改語言」在原始碼裡一眼可查。
type LocaleState = { pref: LocalePreference; setPref: (pref: LocalePreference) => void };

const LocaleContext = createContext<LocaleState | null>(null);

/** 介面語言選項（帳號選單與登入卡片下方共用）。名稱維持母語寫法——把「English」翻成「英文」
 *  只會讓看不懂當前語言的人找不到自己的語言；short 給空間窄的切換鈕。 */
export const UI_LOCALES: { value: UiLocale; name: string; short: string }[] = [
  { value: "zh-TW", name: "繁體中文", short: "中文" },
  { value: "en", name: "English", short: "English" },
];

export function LocaleProvider({ value, children }: { value: LocaleState; children: ReactNode }) {
  return <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>;
}

export function useLocalePreference(): LocaleState {
  const ctx = useContext(LocaleContext);
  if (ctx === null) throw new Error("useLocalePreference 必須在 LocaleProvider 內使用");
  return ctx;
}
