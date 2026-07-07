## Why

config.yaml 的 context（專案說明）與 rules（產出規則）目前只能手改 YAML——而該檔任何解析錯誤會使整份工作流政策靜默退回預設（既知地雷，如規則條目以反引號開頭即炸檔）。desktop-config-multiproject 的設定頁刻意把這兩個欄位列為 Non-Goal（僅原樣保留），本變更接續補上：讓不熟 YAML 的使用者也能安全編輯，並讓「靜默失效」在 GUI 路徑上結構性不可能。源自討論「config-context-與-rules-gui-編輯」（2026-07-07 結論）。目標使用者：透過桌面 GUI 跑 SDD 的開發者與不熟 CLI/YAML 的 PO/PM；使用情境：workspace 管理（設定頁），影響所有 artifact 產出階段的指令注入。

## What Changes

- 設定頁新增「專案說明」區段：多行文字區編輯 config.yaml 的 context；清空儲存即移除鍵（維持「未設定＝預設」語意）。
- 設定頁新增「產出規則」區段：以活躍 schema 的 artifact id 為固定鍵分節（不提供自由鍵輸入），每節為可新增、編輯、刪除、排序的條目清單（清單順序即指令注入順序）；某節清空即移除該 artifact 鍵，全部清空即移除 rules 鍵。
- 寫入沿用主刀既有機制：僅代換目標鍵、未觸及鍵原樣保留；寫入前後雙重解析驗證、載入解析失敗即警告並停用表單；序列化自動為 YAML 保留起始字元（如反引號）的條目加引號——手改反引號炸檔地雷在 GUI 路徑上不存在。
- GUI 文案採 openspec/LANGUAGE.md 正典詞「專案說明」「產出規則」（2026-07-07 已收錄）。
- 不新增、不變更任何設定欄位與預設值——GUI 僅寫入既有的 context 與 rules 鍵。

## Non-Goals

- .speclink.yaml 的自訂工具描述子與 remote 段的 GUI 編輯（維持主刀 Non-Goal）。
- 遠端 store 情境的設定寫入（桌面接遠端文件時需 store 層寫入介面，web-server-postgres 落地後另議）。
- 規則內容的語意 lint 與指令注入預覽。
- 保留檔內註解（含手寫）——寫入即遺失，沿用主刀 D4 已接受的取捨（討論已確認不保護）。
- CLI 行為與輸出的任何變更——speclink-cli 不動，人眼與 --json 輸出不變，回歸對照不受影響。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-config`: 設定頁需求擴充——「設定頁圖形化讀寫兩層設定」的原樣保留名單縮減（rules、context 改為可編輯），新增「專案說明」與「產出規則」的編輯需求。

## Impact

- Affected specs: `desktop-config`（MODIFIED＋ADDED）。前置相依：desktop-config-multiproject 須先完成封存使 desktop-config 成為正典 spec——本變更的 MODIFIED delta 以其 delta spec 現行措辭為基準，若主刀封存前措辭調整，apply 前先跑 /speclink-drift 校正。
- Affected crates: `speclink-core`（config.rs 的工作流政策更新純函式擴充 context 與 rules 變更集）、`speclink-desktop-core`（apps/desktop/core 的設定讀寫橋接擴充，含活躍 schema artifact id 清單的提供）；`speclink-cli` 不動。
- Affected code:
  - Modified: crates/speclink-core/src/config.rs、apps/desktop/core/src/settings.rs、apps/desktop/src/views/SettingsView.tsx、apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/i18n/messages.ts（主刀 i18n 落地後的新字串 key）
  - New: （無——清單編輯器為 SettingsView 內部元件）
  - Removed: （無）
- 相容性影響：CLI 零變更；config.yaml 經 GUI 寫入後檔內註解遺失（既定取捨，主刀先例一致）。
