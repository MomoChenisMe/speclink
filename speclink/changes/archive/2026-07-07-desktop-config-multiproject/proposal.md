## Why

桌面 app（① desktop-shell-and-browser 產出）已能瀏覽與管理單一專案的 change 與 spec，但仍鎖定啟動時工作目錄為專案根、設定只能手改 YAML 檔、UI 硬編 zh-TW。要讓 app 成為日常可用的 SDD 入口——目標使用者是透過桌面 GUI 跑 SDD 的開發者與不熟 CLI 的 PO/PM，情境涵蓋 workflow 各階段的瀏覽與 workspace 管理（初始化、設定）——需要：換專案不必重啟（含首次開啟自動初始化）、圖形化設定、雙語介面。使用者 2026-07-05 拍板 B+C+D 併為一刀，插隊先於 ② desktop-acp-agent。

## What Changes

- **B 開啟專案（含自動 init）**：
  - 頂欄「開啟專案」接原生資料夾選擇器；執行期切換專案 root，所有既有查詢與管理操作跟隨新 root，無須重啟。
  - 所選目錄尚無 speclink 工作區時，經使用者於確認對話框同意（含 AI 工具多選，預設 claude）後，執行與 speclink init 等效的初始化（openspec 骨架、.speclink.yaml、工具指令檔 marker 與 skills 生成）。
  - 專案分頁列（2026-07-06 討論「專案選擇對齊-spectra」定案，UI 形態對齊 Spectra 桌面 app）：頂欄以持久化分頁列呈現開啟過的專案——跨啟動還原、點分頁即切換、分頁徽章顯示該專案進行中變更數（背景分頁為最後已知值）、「＋」掛分頁列尾端與右上「開啟專案」雙入口、Ctrl+Tab／Ctrl+1..9 鍵盤切換。分頁列即最近開啟清單（上限 10、成功開啟去重、關閉分頁即自清單移除），屬 app 本機狀態，不寫入專案檔案。零分頁時顯示「開啟專案」空狀態引導頁；分頁指向已消失路徑時以錯誤態呈現、點擊可自分頁移除。
- **C config 設定頁**：側欄「設定」開結構化表單：
  - .speclink.yaml：tools 多選（claude／codex；自訂工具描述子項目原樣保留、不可經 GUI 編輯）。寫入後同步技能檔（update 等效）。
  - openspec/config.yaml：locale 與 spec_locale 下拉、tdd 與 audit 開關。
  - 不新增任何設定欄位、不改任何預設值；GUI 僅寫入上述既有欄位，未觸及的鍵（rules、context、remote、spec_dir 等）原樣保留。
  - 寫入前後驗證可解析——config.yaml 解析失敗會使整份工作流政策靜默退回預設（既知風險），寫入失敗須明確回報且不得留下不可解析的檔案。
- **D i18n**：UI 介面語言支援 zh-TW 與 en，預設跟隨系統語言、設定頁可手動切換；語言偏好存 app 本機，與 config.yaml 的 locale（AI artifacts 產出語言）為兩件事、互不影響。packages/ui 與 apps/desktop 目前全數硬編 zh-TW 的 UI 字串抽 key。

## Non-Goals

- 側欄「規格」「備忘」的內容頁（仍為佔位）與 GUI 自由文字編輯 artifacts（遞延後續變更）。
- zh-TW 與 en 以外的語言。
- CLI 行為與輸出的任何變更——GUI 僅消費既有 core API，speclink-cli 不動，回歸對照不受影響。
- 經 GUI 編輯 config.yaml 的 rules 與 context、.speclink.yaml 的自訂工具描述子與 remote 段（僅原樣保留）。
- 多視窗與多專案 root 同時活躍——分頁列僅為切換器，同時僅一個活躍專案 root；背景分頁不掛監看、不持續刷新（徽章為最後已知值）。

## Capabilities

### New Capabilities

- `desktop-config`: 桌面 app 的專案切換（含未初始化目錄的確認後自動 init 與最近開啟清單）、設定頁（.speclink.yaml tools 與 openspec/config.yaml 政策欄位的圖形化讀寫）、UI 介面 i18n（zh-TW／en）。

### Modified Capabilities

（無——desktop-app 既有需求（直嵌引擎、啟動語境與空狀態）不變；workflow-config 與 workspace-tools 的解析與驗證規則不變，本變更僅新增其圖形化寫入者。）

## Impact

- Affected specs: 新增 `desktop-config`；`desktop-app`、`workflow-config`、`workspace-tools` 為行為參照但需求不變。
- Affected crates:
  - `speclink-core`：新增設定回寫 API（目前僅 remote 段有寫回先例，tools 與政策欄位只能整檔範本寫入）。
  - `speclink-desktop-core`（apps/desktop/core）：開專案、init 觸發、設定讀寫的橋接函式。
  - `speclink-desktop`（apps/desktop/src-tauri）：專案 root 由啟動時固定改為執行期可變、新增對應 Tauri command、引入資料夾選擇對話框能力。
  - `speclink-cli` 不動。
- Affected code:
  - Modified: apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/Cargo.toml、apps/desktop/src-tauri/capabilities/default.json、apps/desktop/core/src/lib.rs、apps/desktop/src/App.tsx、apps/desktop/src/store.ts、apps/desktop/src/adapter/tauriDataSource.ts、packages/ui/src/index.ts、crates/speclink-core/src/config.rs，以及 packages/ui/src/components 下所有含硬編 UI 字串的元件與其測試。
  - New: apps/desktop/core/src/project.rs、apps/desktop/core/src/settings.rs、apps/desktop/src/views/SettingsView.tsx、apps/desktop/src/components/ProjectTabs.tsx、apps/desktop/src/i18n/messages.ts、packages/ui/src/i18n.tsx
  - Removed: （無）
- 相容性影響：無——CLI 人眼與 --json 輸出皆不變。
- 技能與注入區塊：自動 init 與 tools 變更後的同步會為所選工具（claude、codex）生成或更新 skills 及 CLAUDE.md／AGENTS.md 的 SPECLINK marker 區塊——全數消費既有 init 與 update 邏輯，生成內容本身不變。
