import { createContext, useContext, type ReactNode } from "react";
import type { LocalePreference } from "@speclink/ui";

// 語言偏好由 App 持有（它同時決定 I18nProvider 的 locale），header 的切換器需要讀寫它。
// 只有這一組值，不值得引入狀態管理；context 讓「誰在改語言」在原始碼裡一眼可查。
type LocaleState = { pref: LocalePreference; setPref: (pref: LocalePreference) => void };

const LocaleContext = createContext<LocaleState | null>(null);

export function LocaleProvider({ value, children }: { value: LocaleState; children: ReactNode }) {
  return <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>;
}

export function useLocalePreference(): LocaleState {
  const ctx = useContext(LocaleContext);
  if (ctx === null) throw new Error("useLocalePreference 必須在 LocaleProvider 內使用");
  return ctx;
}
