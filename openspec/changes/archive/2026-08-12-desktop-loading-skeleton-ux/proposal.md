## Why

desktop-async-commands 解掉了整窗凍結，但其承諾的「資料以載入中狀態呈現」前端從未實作——store 的 loaded 旗標零 UI 消費者。使用者（在多個 workspace 之間切換的 desktop／tray 使用者，即透過 AI 代理跑 SDD 的開發者）切換 workspace 時經歷兩段無回饋等待：A 段（openProject probe 完成前分頁高亮完全不動，tray 上「點了沒反應」感特別明顯）與 B 段（首訪 workspace 顯示空清單，與「真的沒東西」無法區分）；抽屜載文件時部分分頁甚至短暫顯示假的「無文件」空態。本變更對應日常瀏覽與切換情境（看板、tray 面板、詳情抽屜），不屬特定 workflow 階段。

## What Changes

- store 新增「切換中分頁」旗標：activateTab 進入 probe 即設、翻頁或失敗即清；主視窗分頁列與 tray 面板分頁條在目標分頁顯示 spinner。不改「probe 成功才翻 activeKey」的既有順序與 tabErrors 錯誤處理。
- 看板欄與 tray 面板分區在活躍 workspace 快照未載入（loaded 為 false）時以 skeleton 佔位卡呈現；已有舊快取則照現狀顯示舊資料、refresh 完成後靜默更新，不閃 skeleton。
- 詳情抽屜（變更、規格、討論、已封存）統一消費文件三態的 undefined 載入態：載入中渲染文件 skeleton，載入完成且檔案不存在才顯示空態文案——汰換「載入中」文字與「載入中卻顯示無文件」的假空態。
- skeleton 基元（shadcn Skeleton，含 pulse 動畫）新增於 packages/ui 的 shadcn 基元群，desktop 與 server-web 皆可共用。
- TraySnapshot 新增切換中與載入中欄位，tray 面板維持「薄渲染、與主視窗同源」原則呈現上述狀態。

僅影響前端：apps/desktop（TypeScript 面）與 packages/ui。Rust crates 與 apps/desktop/src-tauri 零改動；無 CLI 指令、設定欄位、技能或注入區塊變動；無相容性影響。

## Non-Goals

- 不做樂觀翻頁（先切頁、probe 失敗再彈回）：現有 tabErrors 錯誤歸屬依賴「不翻 activeKey」，回滾設計複雜化不值得。
- 不做「每次 refresh 都閃 skeleton」：watcher 事件與切換共用 refresh，頻繁閃爍反成干擾；有舊快取一律靜默更新，也不加「更新中」指示。
- 不動原生 tray 下拉選單（平台選單項無載入動畫能力）、不做 tray 圖示忙碌變體。
- 不做讀取提速：worktree 觀察面「每次現取、不快取」維持不變，本變更只補視覺回饋。
- server-web 不在本次接線範圍：skeleton 基元落共用套件使其未來可用，web 端消費另案處理。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 新增載入回饋要求——分頁切換中 spinner、看板首訪 skeleton、抽屜文件載入 skeleton 與假空態消除。
- `tray-status-menu`: 新增面板載入回饋要求——分頁條切換中 spinner、分區首訪 skeleton，狀態經 TraySnapshot 與主視窗同源。

## Impact

- Affected specs: desktop-app（修改）、tray-status-menu（修改）
- Affected code:
  - New: packages/ui/src/components/ui/skeleton.tsx、packages/ui/src/components/skeletons.tsx
  - Modified: apps/desktop/src/store.ts、apps/desktop/src/tray.ts、apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/components/ProjectTabs.tsx、apps/desktop/src/App.tsx、apps/desktop/src/i18n/messages.ts、packages/ui/src/index.ts、packages/ui/src/components/KanbanBoard.tsx、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/SpecDrawer.tsx、packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/ArchivedDrawer.tsx、packages/ui/src/i18n.tsx
  - Removed: (none)
