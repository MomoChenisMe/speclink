## Why

架構 §10.6 遠端欄的核心承諾——「編輯 server policy，顯示 revision 與權限；409 保留輸入、Reader 唯讀」——是 Phase 3 尚未落地的最後一塊功能面：remote 分頁的專案設定頁還是整頁不可用說明，server 的 /config 只回 schema 名、沒有任何寫入端點，identity 沒有 role 模型可支撐「Reader 看得到但不能編輯」。§3.4 的上游原則（遠端 policy 不可由 client 決定）要求這一切都在 server 端守住。

## What Changes

- GET /config 擴充：回應加入 workflow config 文件原文與 scope revision（與既有 ETag 同值；純加欄位、CLI 既有消費不受影響）。
- 新增 PUT /config：body 帶完整文件原文與 expected revision——revision 不符回 wire 的 revision_conflict（409）且不落盤；寫入前過引擎的 WorkflowConfig fail-closed 驗證，壞 YAML 以 invalid_config 拒收；成功回新 revision。
- identity membership 補最小 role 欄位（editor｜reader，預設 editor；sqlite schema 依既有 migration 機制升版）：admin 設定 membership 時可指定 role 並入 audit；invitation 建立的 membership 預設 editor。
- binding handshake 的 capabilities 加 policy 寫入權布林（role 為 editor 時真）；PUT /config 對 reader 回 403——server 為最終防線，desktop 停用只是 UX。
- desktop 的 remote 專案設定頁由不可用說明改為單一 Workflow 簽：與本地 config.yaml 簽同形的三卡（專案說明、產出規則、產出政策），簽首顯示 policy revision；儲存帶 expected revision；409 時保留使用者輸入、呈現逐欄位對照（server 現值 vs 我的輸入），可選「以 server 版重載」或檢視後以最新 revision 重新提交——不提供未經對照的 force overwrite；reader 表單唯讀、存檔停用附繁中說明。remote 分頁無 .speclink.yaml 簽（tools 屬本機 checkout 概念）。
- desktop-core 的 settings 解析/寫入抽出文字層 seam（from-text 解析、targeted-key 文字改寫），本地路徑行為凍結、remote 路徑複用同一套鍵保留語意（未觸及鍵原樣保留、設回預設移除鍵）。
- CLI 不動。

## Capabilities

### New Capabilities

- `server-policy-write`: server 端 policy 讀寫契約——config 內容與 revision 下發、CAS 寫入與 revision_conflict、引擎 fail-closed 驗證、membership role 最小模型與 capabilities 傳播、reader 403。

### Modified Capabilities

- `desktop-config`: 遠端專案設定頁由整頁不可用改為 Workflow 簽編輯（revision 顯示、expected revision 儲存、409 對照流程、reader 唯讀）。

## Impact

- 相容性影響：GET /config 純加欄位向後相容；identity schema 升版走既有 migration 機制（既有部署升版後 role 一律 editor，行為不變）；desktop 本地設定路徑行為凍結（文字層 seam 為內部重構）。
- Affected specs: `server-policy-write`（新增）、`desktop-config`（修改）
- Affected code:
  - New: crates/speclink-server/tests/policy_write.rs
  - Modified: crates/speclink-server/src/routes.rs、crates/speclink-server/src/app.rs、crates/speclink-server/src/auth.rs、crates/speclink-server/src/identity.rs、crates/speclink-server/src/identity_sqlite.rs、crates/speclink-server/src/admin.rs、crates/speclink-protocol/src/query.rs、crates/speclink-protocol/src/binding.rs、crates/speclink-remote/src/client.rs、apps/desktop/core/src/settings.rs、apps/desktop/src-tauri/src/remote.rs、apps/desktop/src/adapter/workspace.ts、apps/desktop/src/session.ts、apps/desktop/src/views/ProjectSettingsView.tsx、apps/desktop/src/i18n/messages.ts、apps/desktop/src/__tests__/projectSettingsView.test.tsx、Cargo.lock
  - Removed: 無
