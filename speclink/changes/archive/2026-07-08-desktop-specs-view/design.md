## Context

導覽「規格」項在 App 中無 onClick（純擺設）；資料管道已存在——嵌入引擎的規格清單查詢（apps/desktop/core/src/query.rs）、Tauri 指令 list_specs／spec_document、前端 SpeclinkDataSource 的 listSpecs／getSpecDocument 與 store 的 specs 狀態。SpecItem 型別目前僅含 id，無修改時間。本刀落在 apps/desktop/core、apps/desktop/src-tauri、packages/ui、apps/desktop 四處；speclink-core／speclink-cli 不動。相關者：查閱正典規格的開發者／PO／PM。

## Goals / Non-Goals

**Goals**
- 「規格」導覽進入唯讀規格頁：卡片清單＋名稱搜尋＋展開全文＋修改時間。
- 修改時間由檔案系統 mtime 衍生，經清單查詢一次帶回。

**Non-Goals**
- requirement 計數 badge、全螢幕抽屜（遞延）；全文搜尋；任何規格寫入動詞。

## Decisions

### D1：SpecList 元件進 packages/ui

規格頁清單元件放 packages/ui（沿 ArchivedList 前例），透過 props 注入資料載入函式，不知後端是 Tauri 還是 HTTP——遵守既有「前端元件庫與資料源解耦」需求；App 僅做 wiring。主內容區沿已封存頁模式整頁縱向捲動。
替代案：放 apps/desktop/src/views（SettingsView 模式）——SettingsView 綁 workspace 管理屬 app 殼職責，規格清單是純資料呈現、未來 web 端可重用，放元件庫更對位，否決。

### D2：修改時間走清單查詢單趟帶回

嵌入引擎的規格清單查詢對每個 spec 讀取 spec.md 的檔案系統 mtime，衍生為 YYYY-MM-DD 字串隨清單回傳（serde camelCase 欄位 modifiedAt，Option 型別——mtime 不可得時缺席、前端不顯示該行）；前端以既有相對天級呈現邏輯（今天／昨天／N 天前，RichDetailDrawer 的 relativeDays 抽為共用）顯示。既有需求「清單與狀態資料的欄位與值與 CLI --json 輸出一致」對此呈現層輔助欄位明文豁免（delta 規格處理）。
替代案：展開卡片時另發請求查 mtime——N 張卡 N 趟 IPC 換一個日期，否決；epoch ms 數值——前端還要轉換，YYYY-MM-DD 直接沿 meta.created 慣例餵既有邏輯，否決。

### D3：搜尋僅名稱過濾

搜尋列對 spec 名稱做大小寫不敏感的子字串過濾，純前端、即打即濾；無結果顯示空狀態。
替代案：全文搜尋——需預載全部 spec.md，成本與需求不成比例（YAGNI），否決。

### D4：展開延遲載入

卡片預設全部縮合；點標題展開時才呼叫 getSpecDocument 載入全文（markdown 渲染），載入結果留在元件狀態內、同 session 再展開不重載；再點縮合。refreshGen 世代遞增時清空快取使外部變更可反映。
替代案：進頁即載全部文件——正典規格可能數十份，浪費且拖慢進頁，否決。

## Implementation Contract

**可觀察行為**
1. 點左側導覽「規格」進入規格頁（導覽項呈 active 樣式），顯示全部正典 spec 卡片：名稱、最後修改相對時間（今天／昨天／N 天前；mtime 不可得時該行缺席）、複製名稱鈕、展開箭頭。
2. 搜尋列輸入即過濾（名稱子字串、大小寫不敏感）；清空即還原；無結果與無 spec 專案各顯示空狀態文案。
3. 點卡片標題展開顯示該 spec 的 spec.md 全文 markdown 渲染（首次展開有載入態），再點縮合；展開另一張不影響已展開者。
4. 複製名稱鈕將 spec 名稱寫入剪貼簿並短暫顯示已複製回饋。
5. 外部改動 spec 檔案後（workspace-changed），規格頁清單與已展開內容於世代重載後反映新內容與新修改時間。

**驗收目標**
- Rust：apps/desktop/core 測試斷言規格清單查詢帶 modifiedAt（YYYY-MM-DD、mtime 不可得時缺席），cargo test -p speclink-desktop-core --lib 全綠。
- 前端：npm test -w packages/ui（SpecList 渲染、過濾、展開載入、縮合、複製、空狀態、相對時間）、npm test -w apps/desktop（store boardView、App wiring、dataSource 型別）全綠。
- 真實視窗：進頁、搜尋、展開全文、修改時間呈現逐項截圖確認。

**範圍邊界**
- In scope：apps/desktop/core/src/query.rs、apps/desktop/src-tauri/src/lib.rs、packages/ui 的 SpecList／adapter／index、apps/desktop 的 store／App／tauriDataSource。
- Out of scope：speclink-core／speclink-cli、變更抽屜與討論抽屜、規格寫入、全文搜尋。

## Risks / Trade-offs

- [mtime 語意跨平台差異（複製、git checkout 會刷新 mtime）] → 呈現定位是「最後修改」的近似輔助資訊，天級精度可容忍；mtime 不可得時欄位缺席、UI 不顯示，不阻斷清單。
- [大量 spec 的清單查詢多 N 次 metadata 讀取] → 天級資訊、單趟批次讀取，成本可忽略；不做快取（避免失效複雜度）。
- [回歸對照] → CLI 兩 crate 零接觸，parity／color 對照不受影響；list_specs 為桌面嵌入層 payload，非 CLI --json。
- [跨平台] → mtime 讀取用標準庫跨平台 API，無平台特有分支。

## Migration Plan

無資料遷移。payload 新欄位為 Option 純新增，舊前端忽略即不受影響；回滾即還原 commit 重建。

## Open Questions

（無——範圍已在討論 desktop-reading-and-tasks-ux 定為 Spectra 基準款，進階功能明文遞延）
