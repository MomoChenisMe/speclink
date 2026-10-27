## Why

remote workspace 目前只能從伺服器頁籤的臨時對話以文字輸入 repo 識別開啟——/scopes 端點已就位，architecture §10.5 的正典 onboarding（新增 Workspace → 來源分流 → 登入 → 選 Project/Repo → 連接 checkout）是 Phase 3 第 3 項的核心 UX，也是 PM/RD 第一次接觸 remote 模式的門面。同時 workspace-session 正典仍寫著 remote 變體「無任何建構路徑」——刀 3 落地後未同步修正的規格債，本刀一併清償。

## What Changes

- 新增「新增 Workspace」chooser（多步對話）取代現行三個開啟入口（視窗右上開啟專案、空狀態開啟、分頁列加號）：第一步來源分流「本機資料夾｜Speclink Server」。
- 本機路徑：沿用既有資料夾選擇與 PROBE（project／uninitialized／init 流程不變）。
- Server 路徑：選擇既有已登入 server 或就地新增並登入（整合連線 registry 與 device/PAT 登入）→ 消費 /scopes 以清單選擇 Project 與 Repo（取代文字輸入）→「連接本機 checkout？」分流——略過即開 remote spec-only 分頁；選資料夾則驗證後綁定，locator 的 checkoutRoot 首次落地。
- checkout 驗證（誠實形狀）：資料夾含 remote marker（.speclink.yaml 的 remote section：url＋repo）時其 origin 與 repo 必須與所選 scope 一致、不一致即拒絕；無 marker 時資料夾須為 git repo，綁定時寫入 remote section（與 CLI 的 init_remote 同構）。registry 尚無 workspacePath／git remote 欄位，深度驗證屬後續 server 刀。
- 開資料夾的 PROBE 擴充：偵測 remote marker——僅 remote marker 時解析 origin 對應 connection、取 Keychain 憑證 handshake、以該資料夾為 checkoutRoot 開 remote 分頁（未登入則引導至 chooser 的 server 步驟）；本地 openspec/ 與 remote marker 並存時停下強制選擇——可繼續本地，remote 路徑明示待正式 migration（後續刀），不得任一方靜默覆蓋。
- 伺服器頁籤的「開啟 workspace」臨時對話退役，按鈕改開 chooser 並預選該 server。
- checkoutRoot 僅持久化與最小呈現（分頁 tooltip 顯示已連接 checkout）；apply／完整 drift／verify 的功能解鎖屬後續 RD 刀。

## Capabilities

### New Capabilities

- `workspace-chooser`: 新增 Workspace 的 onboarding 行為保證——來源分流、scopes 清單選擇、checkout 綁定驗證與 marker 寫入、remote marker 資料夾的 PROBE 分流、並存衝突強制選擇。

### Modified Capabilities

- `workspace-session`: remote locator 建構路徑措辭修正（刀 3 遺留規格債）——remote 變體經 handshake 成功路徑建構、checkoutRoot 由 checkout 綁定流程寫入。
- `remote-workspace-data`: handshake 需求的開啟入口措辭由「輸入 repo 識別」更新為 chooser 的 scope 清單選擇。

## Impact

- 相容性影響：本機開啟流程行為不變（入口位置改為 chooser 內）；伺服器頁籤臨時對話移除屬刀 3 已知過渡形狀的退役；.speclink.yaml remote section 讀寫與 CLI 完全同構、互通。
- Affected specs: `workspace-chooser`（新增）、`workspace-session`（修改）、`remote-workspace-data`（修改）
- Affected code:
  - New: apps/desktop/src/components/WorkspaceChooser.tsx、apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - Modified: apps/desktop/src/App.tsx、apps/desktop/src/components/ProjectTabs.tsx、apps/desktop/src/components/ServersPanel.tsx、apps/desktop/src/store.ts、apps/desktop/src/session.ts、apps/desktop/src/adapter/workspace.ts、apps/desktop/src/adapter/connections.ts、apps/desktop/src/i18n/messages.ts、apps/desktop/core/src/project.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/src/remote.rs、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/store.test.ts、apps/desktop/src/__tests__/workspace.test.ts、apps/desktop/src/__tests__/serversPanel.test.tsx、apps/desktop/src/__tests__/remoteOpen.test.ts
  - Removed: 無（臨時對話為元件內程式碼移除，非整檔刪除）
