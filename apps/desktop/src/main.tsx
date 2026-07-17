import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import { createLocalSession } from "./session";
import { createConnectionsAdapter } from "./adapter/connections";
import { createWorkspaceAdapter } from "./adapter/workspace";
// Noto Sans TC 隨 app 打包（離線、未裝字體的機器皆生效）。可變字重版：
// 單一 woff2 檔族涵蓋 100-900 全字重（含 600），且無靜態版的 woff 舊格式死重
// ——WebView2 只取 woff2（design D3）。
import "@fontsource-variable/noto-sans-tc";
import "./index.css";

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App
        createSession={(dir, name) => createLocalSession(dir, { name })}
        workspace={createWorkspaceAdapter()}
        connections={createConnectionsAdapter()}
      />
    </StrictMode>,
  );
}
