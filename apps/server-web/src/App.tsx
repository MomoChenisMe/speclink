import { BrowserRouter, MemoryRouter } from "react-router-dom";
import { I18nProvider } from "@speclink/ui";
import type { WebClient } from "./api/client";
import { ClientProvider } from "./app/context";
import { AppRoutes } from "./routes/AppRoutes";

// Server Web Console 應用殼層。client 是唯一 HTTP 入口（測試注入 fake）；
// initialEntries 存在時用 MemoryRouter（測試），否則 BrowserRouter（正式）。
export function App({
  client,
  initialEntries,
}: {
  client: WebClient;
  initialEntries?: string[];
}) {
  const tree = (
    <I18nProvider locale="zh-TW">
      <ClientProvider client={client}>
        <AppRoutes />
      </ClientProvider>
    </I18nProvider>
  );
  return initialEntries ? (
    <MemoryRouter initialEntries={initialEntries}>{tree}</MemoryRouter>
  ) : (
    <BrowserRouter>{tree}</BrowserRouter>
  );
}
