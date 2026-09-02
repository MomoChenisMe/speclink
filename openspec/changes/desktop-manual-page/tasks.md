## 1. 資料層（desktop core，測試先行）

- [ ] 1.1 於 `apps/desktop/core/src/manual.rs` 先寫 #[cfg(test)] 測試（tempdir 建 `openspec/manual/` 與 `openspec/specs/`）釘住「索引 JSON 的形狀與推導規則」與「過期與未入冊的計算依 manual-pages 契約」：六欄 frontmatter 解析；依 order 升冪、同值以檔名決斷、分區順序為分區內最小 order；缺 title 用檔名、缺 section 歸「其他」、非整數 order 置分區末且輸出 null；不以 `---` 開頭或 YAML 壞掉的頁列入 pages 且進 malformed；stale 依 sources 規格 `@trace updated` 最大值晚於 generated；uncoveredNew 依規格最小 updated 晚於全手冊最大 generated 且不在任何 sources；目錄不存在時 present 為 false；CRLF 內容可解析。驗證：`cargo test -p speclink-desktop-core manual` 先紅。 <!-- speclink-task:tsk_01M1FV0A286V08KE9QA9CVDMJ1 -->
- [ ] 1.2 實作「手冊查詢落在 desktop core 的 manual 模組」：`list_manual_pages_at(root)` 回傳 `{ present, reason, pages[{slug,title,section,order,keywords,sources,generated,stale}], uncoveredNew, malformed }`（欄位 camelCase），`manual_page_at(root, slug)` 回傳去 frontmatter 的內文、不存在回 None；於 `apps/desktop/core/src/lib.rs` 匯出模組；`@trace updated` 以 regex 自 HTML 註解區塊擷取。驗證：1.1 全綠，且 `cargo test -p speclink-desktop-core` 其餘測試不變。 <!-- speclink-task:tsk_01M1FV0A28HR82D4B02SHG9BWC -->
- [ ] 1.3 於 `apps/desktop/src-tauri/src/lib.rs` 新增 `list_manual_pages(root)` 與 `get_manual_page(root, slug)` 兩個 Tauri command，各單行委派至 core 的 manual 模組並註冊到 invoke handler。驗證：`cargo check -p speclink-desktop` 通過，command 名稱與參數在前端 adapter 的呼叫一致（內容審查）。 <!-- speclink-task:tsk_01M1FV0A28ZGAGAZADCN2R0VD1 -->

## 2. 資料源介面

- [ ] 2.1 於 `packages/ui/src/adapter.ts` 的 `SpeclinkDataSource` 新增 `listManualPages(): Promise<ManualIndex>` 與 `getManualPage(slug): Promise<string | null>` 及 `ManualIndex`／`ManualPageItem` 型別；`apps/desktop/src/adapter/tauriDataSource.ts` 以 invoke 實作；`apps/desktop/src/adapter/remoteDataSource.ts` 的 `listManualPages` 回傳 `{ present: false, reason: "remote", pages: [], uncoveredNew: [], malformed: [] }` 且不發任何請求、`getManualPage` 回 null——落實「無手冊與 remote 模式的空狀態」的 remote 半邊。驗證：`npm test -w apps/desktop` 中 `tauriDataSource.test.ts` 斷言兩個 invoke 名稱與參數、`remoteDataSource.test.ts` 斷言零 fetch 與回傳形狀。 <!-- speclink-task:tsk_01M1FV0A28P4KJYDDETKBST03J -->

## 3. 前端元件（packages/ui，測試先行）

- [ ] 3.1 於 `packages/ui/src/__tests__/markdownAlerts.test.tsx` 先寫測試再於 `packages/ui/src/components/Markdown.tsx` 實作「GitHub Alert 以共用 Markdown 元件的內建轉換呈現」（不新增依賴的 remark 轉換）：`> [!NOTE]`／`[!TIP]`／`[!WARNING]`／`[!CAUTION]` 開頭的 blockquote 渲染為帶類型 class 與類型標籤的提示框、標記文字消失、其餘內容保留；首段不以四種標記開頭的 blockquote 輸出與變更前逐位元一致；四型配色取介面狀態語意色 token——滿足「Markdown 的 GitHub Alert 提示框」。驗證：`npm test -w packages/ui` 該檔綠。 <!-- speclink-task:tsk_01M1FV0A28MKN7939AMWD3GGFB -->
- [ ] 3.2 於 `packages/ui/src/__tests__/manualPage.test.tsx` 先寫測試釘住「側欄樹、搜尋與上下頁在前端由索引推導」：給定索引（含 stale 頁、缺欄頁、malformed 頁）斷言分區與列序、上一頁／下一頁對應、搜尋以大小寫不敏感比對 title 與 keywords 且無命中顯示無結果文案、stale 列帶「可能過期」標記、uncoveredNew 非空時底部顯示計數提示、出處 capability 點擊觸發 `onOpenSpec(capability)` 而正典不存在者為純文字、內文載入失敗顯示失敗文案、`present: false` 顯示尚無手冊空狀態、`reason: "remote"` 顯示 remote 空狀態——對應「手冊頁的側欄樹與閱讀序」「手冊頁的搜尋列」「內頁渲染與出處跳規格」「可能過期與未入冊的標示」「無手冊與 remote 模式的空狀態」。驗證：`npm test -w packages/ui` 該檔先紅。 <!-- speclink-task:tsk_01M1FV0A286QRHV07SYGEBN32S -->
- [ ] 3.3 實作 `packages/ui/src/components/ManualPage.tsx`（props：索引、載入內文函式、`onOpenSpec`、正典 capability 清單），內文以共用 Markdown 與閱讀欄渲染、載入中 skeleton；並於 `packages/ui/src/i18n.tsx` 加入手冊頁文案鍵（zh-TW 與 en，用詞遵循 LANGUAGE.md「手冊」「可能過期」）。驗證：3.2 全綠；i18n 鍵集合相等的既有測試綠。 <!-- speclink-task:tsk_01M1FV0A28RWAZXCA85T40HTTM -->

## 4. App 接線

- [ ] 4.1 落實「側欄第六項與零分頁行為與既有五項同型」：於 `apps/desktop/src/App.tsx` 與 `apps/desktop/src/store.ts` 加入「手冊」導覽項（規格之後、已封存之前，無障礙標籤「手冊」）、手冊視圖與切頁高亮、零分頁時的空狀態引導頁；`apps/desktop/src/i18n/messages.ts` 加入 zh-TW 與 en 鍵——滿足 desktop-app 的「側欄導覽結構」。驗證：`apps/desktop/src/__tests__/App.test.tsx` 斷言側欄六項順序、點手冊切頁且高亮、零分頁空狀態，`npm test -w apps/desktop` 綠。 <!-- speclink-task:tsk_01M1FV0A2813G9F47Y5XS14P4J -->
- [ ] 4.2 接通「內頁渲染與出處跳規格」的 App 半邊：`onOpenSpec(capability)` 切至規格頁並以既有規格卡的展開路徑展開該 capability（規格頁滾至該卡）。驗證：`App.test.tsx` 新增案例——點出處後規格項高亮且該卡展開。 <!-- speclink-task:tsk_01M1FV0A2801DBB40K7T1WGB4A -->
- [ ] 4.3 落實「手冊頁的外部變更重載沿用既有 watcher 事件」：store 收到帶 root 的檔案變更事件且手冊視圖活躍時重取索引並重載目前頁內文，交錯回應以最新為準；不新增監看目標——滿足「手冊頁隨外部變更即時更新」。驗證：`apps/desktop/src/__tests__/store.test.ts` 以假事件斷言重取次數與最新回應勝出。 <!-- speclink-task:tsk_01M1FV0A28A7PBEBBSC0F7PQ6A -->

## 5. 手動驗收

- [ ] [M] 5.1 於本 repo 以 manual 技能生成手冊（或手放含兩個分區、一頁 stale 的 fixture）後啟動 desktop：確認側欄六項順序、手冊頁分區樹與上一頁／下一頁、搜尋過濾、可能過期標記、點出處切至規格頁並展開、`> [!NOTE]` 呈現為提示框。 <!-- speclink-task:tsk_01M1FV0A28THN7TWPY14MR6XEV -->
- [ ] [M] 5.2 desktop 開著手冊頁時，於外部修改一頁內文並新增一頁（order 落於既有兩頁之間）：數秒內內容與側欄更新、順序正確；刪除整個 `openspec/manual/` 後手冊頁轉為尚無手冊空狀態。 <!-- speclink-task:tsk_01M1FV0A28BMDC63486Q3W45G7 -->
- [ ] [M] 5.3 以 dev-harness 開 remote 分頁進入手冊頁：顯示 remote 模式尚不支援手冊的空狀態，開發者工具網路面板無手冊相關請求。 <!-- speclink-task:tsk_01M1FV0A282GZATQRFA59TQXRK -->
