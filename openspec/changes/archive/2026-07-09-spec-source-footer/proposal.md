## Why

桌面規格檢視器過濾掉正典 spec.md 內的 @trace 註解（Markdown 元件的 skipHtml 丟棄所有 raw HTML），使用者看不到「這條規格出自哪個變更」。@trace 的 source 欄位是有價值的 SDD 溯源資訊，且該資料已隨 spec.md 全文送達前端，只是未被呈現。

## What Changes

- 規格頁的展開檢視在 spec.md 全文下方新增一行「來源變更」footer，列出該 spec 內各 @trace 區塊的 source 變更名（去重、依文件首次出現順序）。
- 解析在前端進行：讀已載入的 raw markdown、抽出 @trace 區塊的 source 值；不改 core、不新增 IPC。
- footer 僅顯示、不可點擊（跳轉至變更的導航另議、不在此範圍）；不顯示 @trace 的 updated 日（卡頭已呈現檔案 mtime）。
- 原始 @trace 註解維持被 skipHtml 過濾、不以原文直出（現況不變）。

## Non-Goals

見 design.md 的 Goals/Non-Goals。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 「規格頁提供清單、搜尋與展開檢視」需求新增「展開檢視顯示來源變更 footer」的行為與對應 scenario。

## Impact

- Affected specs: desktop-app（modified）
- Affected code:
  - New:
    - packages/ui/src/trace.ts
    - packages/ui/src/__tests__/trace.test.ts
  - Modified:
    - packages/ui/src/components/SpecList.tsx
    - packages/ui/src/i18n.tsx
    - packages/ui/src/__tests__/specList.test.tsx
  - Removed: (none)
