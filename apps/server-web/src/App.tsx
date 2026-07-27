import { useState } from "react";
import { BrowserRouter, MemoryRouter } from "react-router-dom";
import {
  I18nProvider,
  readLocalePreference,
  resolveUiLocale,
  writeLocalePreference,
  type LocalePreference,
} from "@speclink/ui";
import type { WebClient } from "./api/client";
import { ClientProvider } from "./app/context";
import { LocaleProvider } from "./i18n/LocaleContext";
import { APP_MESSAGES } from "./i18n/messages";
import { AppRoutes } from "./routes/AppRoutes";

// Server Web Console 應用殼層。client 是唯一 HTTP 入口（測試注入 fake）；
// initialEntries 存在時用 MemoryRouter（測試），否則 BrowserRouter（正式）。
//
// UI 語言：明示偏好優先，未設定則跟隨瀏覽器語言（zh 開頭為 zh-TW，其餘 en）。
// 與 Desktop 同一套機制與同一個 localStorage 鍵。
export function App({
  client,
  initialEntries,
}: {
  client: WebClient;
  initialEntries?: string[];
}) {
  const [pref, setPrefState] = useState<LocalePreference>(() => safeReadPreference());
  // 切換即時生效並持久化（header 的語言三選接這裡）。
  const setPref = (next: LocalePreference) => {
    safeWritePreference(next);
    setPrefState(next);
  };
  const locale = resolveUiLocale(
    pref,
    typeof navigator !== "undefined" ? navigator.language : undefined,
  );

  const tree = (
    <I18nProvider locale={locale} messages={APP_MESSAGES}>
      <LocaleProvider value={{ pref, setPref }}>
        <ClientProvider client={client}>
          <AppRoutes />
        </ClientProvider>
      </LocaleProvider>
    </I18nProvider>
  );
  return initialEntries ? (
    <MemoryRouter initialEntries={initialEntries}>{tree}</MemoryRouter>
  ) : (
    <BrowserRouter>{tree}</BrowserRouter>
  );
}

// localStorage 在隱私模式或被停用時讀寫會丟例外。讀失敗＝視為未設定（跟隨系統），
// 寫失敗＝不持久化，兩者都不該讓整個 app 起不來。
function safeReadPreference(): LocalePreference {
  try {
    return readLocalePreference();
  } catch {
    return null;
  }
}

function safeWritePreference(pref: LocalePreference): void {
  try {
    writeLocalePreference(pref);
  } catch {
    // 不持久化即可，本次切換照常生效。
  }
}
