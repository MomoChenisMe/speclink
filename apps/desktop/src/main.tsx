import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";

import { App } from "./App";
import {
  createLocalSession,
  createRemoteSession,
  normalizeRemoteOpenFailure,
  RemoteOpenError,
  type RemoteOpenInfo,
} from "./session";
import { createConnectionsAdapter } from "./adapter/connections";
import { createWorkspaceAdapter } from "./adapter/workspace";
import { tauriUpdaterAdapter } from "./adapter/updater";
import { tauriCliInstallAdapter } from "./adapter/cliInstall";
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
        openRemote={async (connectionId, target, checkoutRoot) => {
          // handshake fail-closed（remote-data-source 決策 6）：remote_open 成功
          // 才建 session；失敗原樣上拋由開啟表單呈現。
          try {
            const info = await invoke<RemoteOpenInfo>("remote_open", { connectionId, target });
            return createRemoteSession(connectionId, info, checkoutRoot);
          } catch (error) {
            throw new RemoteOpenError(normalizeRemoteOpenFailure(error));
          }
        }}
        workspace={createWorkspaceAdapter()}
        connections={createConnectionsAdapter()}
        updater={tauriUpdaterAdapter()}
        cliInstall={tauriCliInstallAdapter()}
      />
    </StrictMode>,
  );
}
