> **Roadmap**: 四情境預設 GUI 工具矩陣的插隊刀（B+C+D，先於 ② desktop-acp-agent）。來源：使用者 2026-07-05 拍板；本檔為可攜範圍記錄（取代本機記憶），完整 artifacts 待 /speclink-propose。
> **依賴**: ① desktop-shell-and-browser（已完成 48 任務——Tauri 殼、packages/ui 設計系統、看板/詳情抽屜/互動任務）。「開啟專案」（頂欄）與「設定」（側欄）按鈕已在 UI 佔位。
> **狀態**: 待完整 propose（本檔為範圍骨架）。

## Why

桌面 app（①）目前鎖定啟動時的工作目錄為專案根、設定只能改檔案、UI 硬編繁中。要讓 app 成為可日常使用的工具，需要：換專案不用重啟（含首次開啟自動初始化）、圖形化設定、雙語介面。

## What Changes

- **B 開啟專案（含自動 init）**：頂欄「開啟專案」接資料夾選擇器（Tauri dialog plugin）＋執行期切換專案 root（AppState 可變）；**所選目錄尚無 openspec/ 時自動執行 init 等效流程**（含 CLAUDE.md/AGENTS.md marker 與 skills 生成）；最近開啟清單（選配，propose 時定）。
- **C config 設定頁**：側欄「設定」接結構化表單——`.speclink.yaml`（tools 多選 claude/codex）與 `openspec/config.yaml`（locale/spec_locale 下拉、tdd/audit 開關）；需新增**寫入 config 的 Tauri command**（桌面首次設定寫入，注意 config.yaml 解析失敗即整份政策靜默變預設的既知風險——寫入前後須驗證可解析）。
- **D i18n**：繁中＋英文，預設跟隨系統、設定頁可手動切換。前置事實：packages/ui 與 apps/desktop 的 UI 字串目前全數硬編 zh-TW，需抽 key。

## Non-Goals

- 規格/備忘側欄項的內容頁（仍為佔位）、GUI 自由文字編輯（遞延）、多語言超過 zh-TW/en。

## Capabilities

### New Capabilities

- `desktop-config`: 桌面 app 的專案切換（含自動 init）、設定頁與 i18n。

## Impact

- Affected code: apps/desktop（store/App/設定頁/i18n）、packages/ui（字串抽 key）、apps/desktop/src-tauri 與 apps/desktop/core（open project／write config／init 觸發 command）。
- 消費既有 core API: speclink-core 的 init（workspace init 寫檔）與 config 結構。
- 不影響 CLI 行為。
