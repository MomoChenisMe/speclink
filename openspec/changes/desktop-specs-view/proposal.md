## Why

左側導覽的「規格」頁簽目前沒有任何功能——點擊無反應，使用者無法在桌面 app 瀏覽正典規格；而資料管道（規格清單與正典內容讀取）在嵌入引擎與前端介面皆已存在，缺的只是頁面本身。目標使用者是想確認系統現行行為的開發者／PO／PM；使用情境是實作或討論前查閱正典規格（對照 Spectra 的規格頁：搜尋＋卡片展開全文）。源自討論 desktop-reading-and-tasks-ux（七項問題的規格頁刀）。

## What Changes

- 規格頁（Spectra 基準款）：左側導覽「規格」進入專屬頁——正典 spec 卡片清單（名稱、最後修改相對時間、複製名稱、展開／縮合箭頭）＋搜尋列名稱過濾；點卡片展開該 spec 的 spec.md 全文 markdown 渲染，內容展開時才載入；無 spec 專案與搜尋無結果各有空狀態。
- 修改時間資料：嵌入引擎的規格清單查詢補上各 spec 的最後修改日期（自檔案系統 mtime 衍生），前端 SpecItem 型別同步擴充；此為呈現層輔助欄位，既有需求的 CLI --json 對齊範圍明文豁免之。
- 導覽狀態：主視圖新增 specs 一態（與看板、已封存、設定並列），「規格」導覽項接上切換與 active 樣式。
- 無 CLI 影響：speclink-core／speclink-cli 不動，CLI 人眼與 --json 輸出逐位元不變；變更落在 apps/desktop/core（嵌入引擎包裝）、apps/desktop/src-tauri（payload）、packages/ui 與 apps/desktop 前端。

## Non-Goals

- requirement 計數 badge、全螢幕抽屜等 Spectra 進階功能——討論記為遞延，需要再加。
- 規格全文搜尋——僅名稱過濾（全文搜尋需預載全部文件，本刀不做）。
- 規格的編輯或任何寫入動詞——規格頁純唯讀。
- markdown 渲染樣式與字體（desktop-reading-experience 刀）；任務互動（desktop-task-interactions 刀）。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 新增規格頁需求——清單、搜尋、展開檢視與修改時間呈現；既有「呈現 change 與 spec 的清單與內容」需求補呈現層輔助欄位（檔案系統衍生的修改時間）的對齊豁免。

## Impact

- Affected specs: desktop-app
- Affected code:
  - New: packages/ui/src/components/SpecList.tsx、packages/ui/src/__tests__/specList.test.tsx
  - Modified: apps/desktop/core/src/query.rs、apps/desktop/src-tauri/src/lib.rs、packages/ui/src/adapter.ts、packages/ui/src/index.ts、apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/store.ts、apps/desktop/src/App.tsx、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/store.test.ts、apps/desktop/src/__tests__/tauriDataSource.test.ts
  - Removed: （無）
- 新增依賴：（無）
