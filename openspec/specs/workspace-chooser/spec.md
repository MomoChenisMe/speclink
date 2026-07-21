# workspace-chooser Specification

## Purpose

TBD - created by archiving change 'workspace-chooser-onboarding'. Update Purpose after archive.

## Requirements

### Requirement: 新增 Workspace 的來源分流

Desktop 的所有開啟入口（視窗頂列、空狀態、分頁列加號、伺服器頁籤）SHALL 匯流至單一「新增 Workspace」chooser：第一步 SHALL 分流「本機資料夾」與「Speclink Server」。本機路徑 SHALL 沿用既有資料夾選擇、專案探測與初始化流程且行為不變；伺服器頁籤入口 SHALL 預選該 server 直達 scope 選擇步驟。

#### Scenario: 本機開啟行為凍結

- **WHEN** 經 chooser 選擇本機資料夾開啟既有 speclink 專案
- **THEN** 分頁建立與看板呈現與 chooser 導入前一致；未初始化資料夾仍走既有 init 確認流程


<!-- @trace
source: workspace-chooser-onboarding
updated: 2026-07-20
code:
  - Cargo.lock
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
-->

---
### Requirement: scopes 清單選擇取代文字輸入

server 路徑 SHALL 呈現已登入 connections 供選擇（含就地新增並登入後回流）；選定 server 後 SHALL 以 scopes 端點回應呈現 Project 分組的 Repos 清單供單選——SHALL NOT 要求使用者手動輸入 repo 識別。無任何 membership 時 SHALL 呈現空清單與繁中說明而非錯誤。選定後 SHALL 進入 checkout 分流：略過即以 spec-only 開啟 remote 分頁。

#### Scenario: 清單選擇開出 spec-only 分頁

- **WHEN** 於 chooser 選擇已登入 server，自 scopes 清單選定一個 Project/Repo 並略過 checkout
- **THEN** handshake 成功後 remote 分頁開啟，無任何 repo 識別的手動輸入步驟


<!-- @trace
source: workspace-chooser-onboarding
updated: 2026-07-20
code:
  - Cargo.lock
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
-->

---
### Requirement: checkout 綁定驗證與 marker 寫入

選擇連接 checkout 時 SHALL 依資料夾狀態驗證：含 remote marker（.speclink.yaml 的 remote section）時其 url origin 與 repo SHALL 與所選 scope 一致、不一致即以繁中訊息拒絕並指出 marker 指向；無 marker 時資料夾 SHALL 為 git repo 方可綁定、綁定時 SHALL 寫入 remote marker（與 CLI 的 remote 初始化同構、互通）；非 git repo SHALL 拒絕。綁定成功後 remote locator 的 checkoutRoot SHALL 記錄該資料夾並隨分頁持久化；分頁 SHALL 以最小面（tooltip）呈現已連接的 checkout 路徑。checkout 綁定 SHALL NOT 改變本階段的 capability 可用性（apply、完整 drift、verify 的解鎖屬後續能力）。

#### Scenario: marker 不一致拒絕

- **WHEN** 所選資料夾的 remote marker 指向不同 origin 或不同 repo
- **THEN** 綁定被拒且訊息指出 marker 指向的 origin 與 repo，分頁不建立 checkout 關聯

#### Scenario: 無 marker 的 git repo 綁定後互通

- **WHEN** 對無 marker 的 git repo 完成 checkout 綁定
- **THEN** 該資料夾出現 remote marker 且 CLI 於該資料夾可據以進入 remote 模式


<!-- @trace
source: workspace-chooser-onboarding
updated: 2026-07-20
code:
  - Cargo.lock
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
-->

---
### Requirement: remote marker 資料夾的探測分流

專案探測 SHALL 辨識 remote marker：資料夾僅含 marker 時——對應 connection 已登入即以 handshake 開啟 remote 分頁並以該資料夾為 checkoutRoot；無對應 connection 或未登入即引導至 chooser 的 server 步驟並預填 server 位址。marker 與本地 openspec/ 並存時 SHALL 停下強制選擇，提供三個出口且皆無靜默覆蓋：「繼續本地」（本次以本地開啟、marker 不動）；「以 server 為準」（本地 openspec/ 改名為帶日期備份後，資料夾轉為 checkout 開啟 remote 分頁——不上傳本地內容、不改動 server）；「遷移本地內容」（進入 workspace-migration 能力的遷移流程、目標為空 scope）。對話文案 SHALL 明說「以 server 為準」為備份後棄用本地、非合併。marker YAML 損壞 SHALL 沿 .speclink.yaml 既有 fail-closed 語意呈現錯誤。

#### Scenario: RD 重開 checkout 直達 remote 分頁

- **WHEN** 開啟僅含 remote marker 的資料夾且對應 server 已登入
- **THEN** 不經 chooser 步驟，handshake 後 remote 分頁開啟且 checkoutRoot 為該資料夾

#### Scenario: 並存衝突三出口

- **WHEN** 開啟同時含本地 openspec/ 與 remote marker 的資料夾
- **THEN** 呈現強制選擇對話含三出口：繼續本地以本地開啟；以 server 為準將本地改名備份後轉 checkout 開 remote 分頁且 server 內容未變；遷移本地內容進入遷移流程；無任何自動覆蓋


<!-- @trace
source: local-remote-migration
updated: 2026-07-21
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/migration.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/migrationDialog.test.tsx
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/migration.ts
  - apps/desktop/src/components/MigrationDialog.tsx
  - apps/desktop/src/components/RemoteConflictDialog.tsx
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/import_api.rs
-->
