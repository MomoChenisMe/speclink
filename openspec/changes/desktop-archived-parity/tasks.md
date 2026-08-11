> design 對應：第 1 組＝D1 落頁修在 store 的 open* 函式，不是各入口；第 2 組＝D2 封存變更卡的描述資料走清單 payload 疊加，不新開 meta 查詢；第 3 組＝D3 封存討論卡雙行為純前端改動；第 4 組＝D4 封存抽屜標頭的層級：標題列＋出身列，僅此兩層。

## 1. 落頁歸位（store 層）

- [x] 1.1 實作需求「變更與討論抽屜開啟時底層落回看板」：apps/desktop/src/store.ts 的 openDetail 與 openDiscussion 於同一次 set 內補 boardView: "board"；openSpec 與封存抽屜開啟路徑不動。先寫測試（apps/desktop/src/__tests__/store.test.ts）：boardView 為 "archived" 時呼叫 openDetail → boardView 變 "board" 且 detailChange 設定；openDiscussion 同款；openSpec 開啟後 boardView 不變 <!-- speclink-task:tsk_01KZNE0N9EY54JG8MERD1R7G4P -->
- [x] 1.2 驗證 detail 抽屜互斥既有測試不回歸：npm test -w apps/desktop -- store.test <!-- speclink-task:tsk_01KZNE0N9EZQPA04XNXD76XDJ8 -->

## 2. 封存清單 payload 疊加欄位（Rust）

- [x] 2.1 實作 client-protocol 需求「已封存清單的呈現輔助欄位」：apps/desktop/core/src/query.rs 的封存清單為每項疊加 whyExcerpt（封存 proposal.md 的 Why 區段首個非空行）與 created（封存目錄 metadata 建立日期）；來源不可讀／缺席時不插 key。先寫 Rust 測試釘住：兩欄位可得、proposal 缺席、無 Why 區段三個 case（比照 list_specs_purpose_tbd_flags_archive_placeholder 的真檔案風格） <!-- speclink-task:tsk_01KZNE0N9ENZE1MDEDDCAN5F08 -->
- [x] 2.2 packages/ui/src/adapter.ts 的 ArchivedItem 增列選填 whyExcerpt 與 created 欄位（含文件註解：缺席＝不顯示） <!-- speclink-task:tsk_01KZNE0N9EBEQW4PQZ218YXJ92 -->
- [x] 2.3 desktop core crate 的 cargo test 通過；欄位缺席時清單照常回傳 <!-- speclink-task:tsk_01KZNE0N9ETS4TZN6TQ5W1Y54K -->

## 3. 封存卡片雙行 anatomy（前端）

- [x] 3.1 實作需求「規格與封存卡片收合資訊」的討論卡雙行：packages/ui/src/components/ArchivedList.tsx 的 ArchivedDiscussionCard 改為 slug 標題（等寬強調＋既有 CopyButton）＋topic 描述列；先寫測試（packages/ui/src/__tests__/archivedList.test.tsx）：slug 以等寬樣式呈現於標題位、topic 於描述列一行截斷、複製鈕仍複製 slug 且點擊不開抽屜 <!-- speclink-task:tsk_01KZNE0N9EMHTEGQV4QVMVH10E -->
- [x] 3.2 同需求的變更卡描述列：ArchivedList.tsx 的 ArchivedCard 於 whyExcerpt 可得時在標題下方一行截斷顯示、缺席時整列缺席（與看板 ChangeCard 描述列同構）；測試補 whyExcerpt 有無兩個 case <!-- speclink-task:tsk_01KZNE0N9EYTHBNDGE2K0D0WWM -->
- [x] 3.3 需求「已封存頁含討論節」的討論節卡片描述文字（日期＋slug 標題＋topic 描述）隨 3.1 落地後，跑 npm test -w packages/ui -- archivedList 通過，既有排序／換頁／搜尋測試不回歸 <!-- speclink-task:tsk_01KZNE0N9EREER8E20VW69AEGH -->

## 4. 封存抽屜標頭補齊（前端）

- [x] 4.1 實作需求「已封存項目以抽屜檢視」的標頭複製鈕：packages/ui/src/components/ArchivedDrawer.tsx 標題後緊跟複製鈕（封存變更複製 datedName、封存討論複製 slug；沿用 useCopied 與 RichDetailDrawer 的按鈕樣式）；先寫測試（packages/ui/src/__tests__/archivedDrawer.test.tsx）：兩種 target 的複製值各自正確、複製後打勾回饋 <!-- speclink-task:tsk_01KZNE0N9E2VQXP14WTQVKARGR -->
- [x] 4.2 同需求的出身列：ArchivedDrawer.tsx 標題下方顯示建立者（首字母圓標＋displayName，完整識別收 tooltip）、建立日期（created）、封存日期（date）；欄位缺席時該欄缺席其餘照常；恆定單行溢出裁切。props 由 App 端自封存清單項帶入（apps/desktop/src/App.tsx 接線）；測試補全欄可得與 created 缺席兩個 case <!-- speclink-task:tsk_01KZNE0N9EJFB45HTK0QP0JFT5 -->
- [x] 4.3 確認封存抽屜無進度條與動詞動作列（既有唯讀語意不變）；npm test -w packages/ui -- archivedDrawer 通過 <!-- speclink-task:tsk_01KZNE0N9EDD01X3ASFGQM3G2F -->
- [x] 4.4 apps/desktop/src/i18n/messages.ts 與 packages/ui/src/i18n.tsx 補新增文案鍵（出身列標籤、複製鈕 aria-label），繁中／英文兩份 <!-- speclink-task:tsk_01KZNE0N9EV82G71XKYWQMZ93M -->

## 5. 收尾驗證

- [x] 5.1 全量前端測試：npm test -w packages/ui && npm test -w apps/desktop 通過 <!-- speclink-task:tsk_01KZNE0N9E2XAMS8FHK5BSAA3N -->
- [ ] 5.2 [M] 桌面 app 實機驗收：已封存頁點卡開抽屜（標頭有複製鈕與出身列）；系統匣在已封存頁開啟某變更 → 底層回看板；封存討論卡雙行（slug 標題＋topic 描述）；封存變更卡描述列顯示 Why 首句 <!-- speclink-task:tsk_01KZNE0N9EFH9PT5GXM2TFZ70J -->
