## Why

桌面 app 的互動控制項仍有原生 HTML 殘留：任務清單與初始化對話框的 checkbox 是原生 input（Windows 上呈 OS chrome 外觀、不吃主題，僅 accent-color 上色）；設定頁兩處多行輸入是裸 textarea；約 26 處 raw button 散落 8 個檔案，各自手寫樣式、focus 與 disabled 態不一致。目標使用者是使用桌面 app 的開發者／PO／PM，使用情境是任務勾核、初始化工具選擇與設定頁編輯的日常操作——控制項外觀應與既有 shadcn 元件體系（Input、Select、Tabs、Sheet 等已採用）一致，跨平台、跨深淺主題統一。源自使用者對任務清單原生 checkbox 外觀的回饋，範圍經確認擴大為全面替換。

## What Changes

- Checkbox 原語升級：packages/ui 的 ui 原語由「原生 input 風格化」升級為 Radix 版 shadcn Checkbox（button 元素、checkbox 角色、主色勾選態、主題邊框），新增 @radix-ui/react-checkbox 依賴；任務清單勾選框與初始化對話框的工具多選改用之，回呼、aria 標籤、disabled 條件不變。
- Textarea 原語新建：新增 shadcn 風格的多行輸入原語（樣式化原生 textarea，無新依賴——border/bg/focus 取自設計 token），設定頁兩處多行輸入改用之。
- 按鈕統一：App 實際渲染路徑上的 raw button（約 26 處、8 檔——任務工具列與拖曳把手、詳情抽屜動作列與 icon 鈕、看板卡片動詞、討論欄動作、已封存展開列、導覽項、專案分頁列、設定頁）改用既有 ui 按鈕原語的變體（ghost／outline＋sm／icon），視覺以現有樣式近似、行為與無障礙名稱不變，focus 可視環與 disabled 態統一。
- 無 Rust 影響：純前端（packages/ui、apps/desktop 的 React 層），嵌入引擎、Tauri 指令、CLI 皆不動。

## Non-Goals

- markdown 內容裡的 GFM 唯讀 checkbox（react-markdown 渲染、非表單控制項）——維持既有 CSS 樣式。
- 未被 App 掛載的舊清單元件（ChangeBoard、ChangeList、ChangeListItem、DocumentTree、DocumentViewer）——死元件不動，清理另議。
- 不重新設計視覺——按鈕替換以「近似現狀」為準，不改 layout 與尺寸節奏。
- 任務互動行為（樂觀更新、批次寫回、拖放）不動——desktop-task-interactions 刀已交付的行為照舊。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 新增控制項呈現需求——表單控制項（勾選框、多行輸入）與按鈕以主題化元件原語呈現，勾選框不得為作業系統原生外觀且保留無障礙語意，深淺主題一致。

## Impact

- Affected specs: desktop-app
- Affected code:
  - New: packages/ui/src/components/ui/textarea.tsx
  - Modified: packages/ui/package.json、package-lock.json、packages/ui/src/components/ui/checkbox.tsx、packages/ui/src/index.ts、packages/ui/src/components/TaskList.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/components/ArchivedList.tsx、apps/desktop/src/App.tsx、apps/desktop/src/components/ProjectTabs.tsx、apps/desktop/src/views/SettingsView.tsx、packages/ui/src/__tests__/taskList.test.tsx、packages/ui/src/__tests__/ui.test.tsx、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/settingsView.test.tsx
  - Removed: （無）
- 新增依賴：@radix-ui/react-checkbox（packages/ui dependencies）
