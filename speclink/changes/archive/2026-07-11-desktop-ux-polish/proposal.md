## Why

桌面 app 的看板與詳情抽屜在日常使用中暴露七個互動缺陷（出自已結論討論 desktop-ux-polish，2026-07-11）：

1. analyze 結果面板把引擎回傳的 location、recommendation 與維度 status 全數丟棄，只渲染嚴重度加訊息的平鋪清單——使用者看不懂發現項出自哪裡、該怎麼修。
2. 分析結果一旦展開便無法關閉：drawerVerb 狀態只在切換 change 或關閉抽屜時清空，面板無關閉鈕、按鈕無切換行為。
3. validate 通過時只顯示一行「驗證通過」，資訊量近乎零，卻佔一顆獨立按鈕。
4. 已轉出討論藏在互斥檢視的另一面，入口是欄 header 上不明顯的 ↗N 小開關——互斥切換本身就讓另一面永遠不可見。
5. promoted 細列與討論抽屜標題仍以中文 topic 為錨點，slug 不出現也不可複製；配合 --from-discussion 操作時必須離開 app 到編輯器複製檔名。（討論中全卡已於 desktop-card-identity 改為 slug 標題，本次是把例外範圍擴到剩下兩處。）
6. 看板搜尋僅以子字串比對卡片名稱、摘要、主題與 slug，輸入框是無任何輔助的裸輸入——無清除鈕、無命中數、無快捷鍵、無篩選、搜不到 artifacts 內文。
7. 拖曳卡片時封存落點以 140px 的 flex 兄弟插入欄列，觸發全欄 reflow 壓縮卡片寬度；且拖曳不可封存的討論卡時落點也會出現。

## What Changes

- 分析面板改「維度摘要卡＋發現卡」兩層結構：頂列四張繁體中文維度摘要卡（覆蓋度、一致性、模糊度、缺漏；零發現呈成功語意、非零呈警示語意），其下逐條發現卡呈現嚴重度徽章、來源檔、摘要與建議行——補齊引擎已回傳但前端丟棄的欄位。
- 移除獨立「驗證」按鈕：「分析」按鈕一鍵同時執行 validate 與 analyze，分析面板頂部呈「結構驗證」列（通過單列帶過、失敗列出錯誤）。
- 分析面板可關閉：「分析」按鈕改為切換行為（再按一次收合），面板右上加關閉鈕；store 增清空動詞結果的動作。
- 已轉出討論入口改為討論欄欄底常駐收合列「已轉出 N」：點按就地展開 promoted 細列，取消互斥檢視與 header 開關——討論中與已轉出同屏可見。
- slug 識別擴充：promoted 細列改以 slug 為首行（帶複製鈕、topic 降為描述行）、討論抽屜標題改以 slug 為題（帶複製鈕、topic 降為副標）；同步擴充 openspec/LANGUAGE.md 既有例外條目的適用範圍。複製鈕一律緊跟標題文字尾端而非推至列右緣（出自已結論討論 spec-archive-drawer-ux 的卡片複製鈕位置規則，經 ingest 併入）。
- 看板搜尋整包強化：搜尋列元件化（搜尋 icon、清除鈕、聚焦快捷鍵、即時命中數）；卡片名稱與 slug 層加模糊比對（subsequence）；命中字段於卡片上高亮；篩選 chips（建立者、建立時間、來源討論，與搜尋字串取交集）；全文搜尋經 speclink-desktop-core 新增的單一查詢命令比對 artifacts 與討論記錄內文，全文命中的卡片呈現 snippet 行。
- 拖曳封存落點改為絕對定位浮層：疊於看板右緣上方、不佔 flex 空間（欄寬零變動），且僅於拖曳變更卡時浮現（討論卡不可封存、不浮現）。封存確認流程與語意不變。

## Non-Goals

- 不拆刀：兩刀（UI 微調＋搜尋整包）與三刀（再獨立全文 IPC）方案已於討論中否決——單次提案、驗證、封存循環的流程成本優於拆刀省下的元件重複改動。
- 不改引擎 CLI：speclink validate 與 speclink analyze 的輸出與語意不動，本變更僅改桌面前端呈現與桌面 core 查詢層。
- 全文層不做模糊比對：全文命中維持子字串——全文 fuzzy 成本高且命中難解釋；模糊比對僅限卡片名稱與 slug 層。
- 不做任務進度篩選維度（討論中使用者未選）。
- 封存落點的封存語意不變：確認流程、僅變更卡可封存、跨欄拖曳不改階段（board-card-order 規格）皆維持——本變更僅動呈現層。
- 討論中全卡的 slug 標題與複製鈕（desktop-card-identity 已實作）不重做。
- 搜尋不涉及已封存頁：已封存頁維持既有搜尋，僅看板搜尋強化。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 修訂四條 requirement——動詞操作面（分析面板兩層結構、validate 併入、可關閉）、討論於看板第 0 欄兩級呈現（欄底收合列取代互斥檢視、細列 slug 化）、討論抽屜檢視與轉出變更（標題 slug 化）、看板搜尋過濾卡片（搜尋列 UI、模糊比對、高亮、篩選、全文搜尋）；另新增一條拖曳封存落點浮層呈現的 requirement。

## Impact

- Affected specs: desktop-app（4 條 MODIFIED＋1 條 ADDED）
- Affected code:
  - New:
    - apps/desktop/core/src/search.rs（workspace 全文查詢：遍歷 active changes 的 artifacts 與討論記錄、子字串比對、回傳命中卡片與 snippet）
    - packages/ui/src/components/BoardSearchBar.tsx（搜尋列元件：icon、清除鈕、命中數、篩選 chips）
  - Modified:
    - packages/ui/src/components/AnalyzePanel.tsx（兩層結構＋結構驗證列＋繁中維度名）
    - packages/ui/src/components/RichDetailDrawer.tsx（驗證鈕移除、分析 toggle、面板關閉）
    - packages/ui/src/components/DiscussionColumn.tsx（欄底收合列、細列 slug 化）
    - packages/ui/src/components/DiscussionDrawer.tsx（標題 slug 化＋複製鈕）
    - packages/ui/src/components/KanbanBoard.tsx（搜尋列元件接入、篩選過濾、浮層落點、高亮與 snippet 呈現）
    - packages/ui/src/search.ts（模糊比對與交集過濾規則）
    - packages/ui/src/adapter.ts（AnalyzePanel 驗證結果型別、全文查詢方法、ChangeItem 補 created 欄位）
    - packages/ui/src/i18n.tsx（新增文案）
    - packages/ui/src/index.ts(元件匯出)
    - apps/desktop/src/App.tsx（搜尋與篩選狀態接線、動詞結果清空）
    - apps/desktop/src/store.ts（drawerVerb 清空動作、分析一鍵雙動詞、全文查詢與篩選狀態）
    - apps/desktop/src/adapter/tauriDataSource.ts（全文查詢 IPC 呼叫）
    - apps/desktop/src/i18n/messages.ts（新增文案）
    - apps/desktop/core/src/lib.rs（search 模組掛載）
    - apps/desktop/core/src/query.rs（change 清單補 created 日期）
    - apps/desktop/src-tauri/src/lib.rs（search_workspace 命令委派）
    - openspec/LANGUAGE.md（slug 例外條目範圍擴充）
    - packages/ui/src/__tests__/analyzePanel.test.tsx
    - packages/ui/src/__tests__/richDrawer.test.tsx
    - packages/ui/src/__tests__/discussionColumn.test.tsx
    - packages/ui/src/__tests__/discussionDrawer.test.tsx
    - packages/ui/src/__tests__/kanban.test.tsx
    - apps/desktop/src/__tests__/App.test.tsx
    - apps/desktop/src/__tests__/store.test.ts
  - Removed: (none)
