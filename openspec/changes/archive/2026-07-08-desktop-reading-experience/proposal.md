## Why

桌面 app 的內容呈現有四個閱讀性缺陷：中文字體 fallback 到系統預設而非 Noto Sans TC（font-family 排序使其永遠輪不到）；markdown 列表符號被 Tailwind preflight 重置吞掉、單換行塌成同段，段落與列表結構在抽屜裡成文字牆；內容字級 14px（任務僅 13px）偏小；討論記錄 scaffold 的 HTML 註解以原文顯示在討論過程分頁。目標使用者是以桌面 app 檢視 SDD 文件的開發者／PO／PM，使用情境是變更抽屜（提案／設計／任務／規格分頁）、討論抽屜與已封存檢視的日常閱讀。源自討論 desktop-reading-and-tasks-ux（七項問題的閱讀體驗刀）。

## What Changes

- 字體打包：新增 @fontsource-variable/noto-sans-tc（可變字重版，單一 woff2 檔族涵蓋全字重）隨 app 打包，body font-family 改為 Noto Sans TC 優先（Segoe UI、system-ui 為後備）——未安裝字體的機器與離線環境皆生效；code 等寬字體維持現況。
- markdown 排版改由 @tailwindcss/typography 接管：渲染容器改用 prose class，手工維護的 .markdown 樣式縮為薄覆寫（teal code chip、GFM 任務清單懸掛縮排與完成刪除線）；prose 色彩變數映射既有 oklch 設計 token，深色模式隨系統偏好自動跟隨，不需 prose-invert。
- 內容字級 16px：markdown 內容 14px→16px（prose 出廠預設即 16px）；任務清單文字 13px→16px 對齊。
- HTML 註解不再呈現：markdown 渲染開 skipHtml——raw HTML（含討論記錄 scaffold 的註解行）不以原文顯示；來源檔案不動。
- 單換行＝換行：markdown 渲染加 remark-breaks——單換行渲染為換行（討論記錄每輪 Focus／Position 各佔一行的格式依賴此行為才有段落感）。
- 無 Rust crate 影響：純前端變更（packages/ui、apps/desktop），speclink-core／speclink-cli 不動，CLI 人眼與 --json 輸出逐位元不變。

## Non-Goals

- 任務工具列與勾選卡頓改善——同討論拆出的 desktop-task-interactions 刀。
- 規格 nav 頁功能——同討論拆出的 desktop-specs-view 刀。
- 不動 CLI 討論記錄 scaffold（註解是 agent／CLI 的功能性錨點，過濾在渲染端做）。
- 不引入 Streamdown（串流場景特化，靜態文件無需求）與 github-markdown-css（自帶配色與設計 token 打架）——討論已否決。
- web 端與 node bridge 不在此刀。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 新增內容呈現需求——介面以打包的 Noto Sans TC 呈現；markdown 內容保留段落／列表／單換行結構、基準字級 16px；raw HTML 不以原文呈現。

## Impact

- Affected specs: desktop-app
- Affected code:
  - Modified: apps/desktop/src/index.css、apps/desktop/src/main.tsx、apps/desktop/package.json、packages/ui/package.json、packages/ui/src/components/Markdown.tsx、packages/ui/src/components/TaskList.tsx、packages/ui/src/__tests__/components.test.tsx、packages/ui/src/__tests__/taskList.test.tsx
  - New: （無）
  - Removed: （無）
- 新增依賴：@fontsource-variable/noto-sans-tc（apps/desktop dependencies）、@tailwindcss/typography（apps/desktop devDependencies）、remark-breaks（packages/ui dependencies）
