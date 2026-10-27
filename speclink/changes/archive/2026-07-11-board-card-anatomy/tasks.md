## 1. Rust 變更清單 payload：whyExcerpt

- [x] 1.1 撰寫失敗測試：list_changes_at 每筆變更帶 whyExcerpt（proposal.md 的 `## Why` 區段首個非空行原文，camelCase）；無 proposal.md、Why 區段缺席或為空時為 null 且清單照常回傳——apps/desktop/core/src/query.rs 的 #[cfg(test)]，cargo test -p speclink-desktop-core 預期紅
- [x] 1.2 實作 whyExcerpt 取值至綠（design D2：變更卡描述列由 whyExcerpt 帶出）；cargo test -p speclink-desktop-core 全綠（不動 crates/speclink-core 與 crates/speclink-cli，CLI 回歸對照不受影響）

## 2. 變更卡三列骨架（design D1）

- [x] 2.1 撰寫失敗測試：ChangeCard 標題改等寬字型且折行不截斷、複製鈕行內尾隨（緊跟名稱末字元、hover 顯現、點擊寫入名稱且不觸發 onOpen，desktop-ux-polish 已落地此位置——測試釘住不回退）、描述列渲染 whyExcerpt 一行截斷、whyExcerpt 為 null 時描述列缺席、卡上無狀態 chip、進度 meta 列與建立者圓點等既有右端 icons 不變——packages/ui/src/__tests__/kanban.test.tsx，npm test -w packages/ui 預期紅；涵蓋「看板卡片統一解剖學」的變更卡（design D1：三列骨架與識別列規則）
- [x] 2.2 實作 ChangeCard 識別列（等寬標題＋行內尾隨複製鈕）與描述列、packages/ui/src/adapter.ts 的 ChangeItem 新增 whyExcerpt 可選欄位至綠

## 3. 討論卡識別列與 meta 列（design D3）

- [x] 3.1 撰寫失敗測試：討論全卡複製鈕行內尾隨（DOM 上為 slug 文字的行內後隨元素而非右緣定位，點擊寫入 slug 且不開討論抽屜）、建立者僅呈頭像圓點且 hover tooltip 顯示全名、卡面無 createdBy 全名直出文字、createdBy 缺席時圓點缺席、卡底 meta 列並排「N 輪」與建立時間、狀態 chip 與 concluded 卡封存按鈕不變——packages/ui/src/__tests__/discussionColumn.test.tsx，預期紅；涵蓋「討論於看板第 0 欄兩級呈現」的全卡呈現與「看板卡片統一解剖學」的討論卡（design D3：討論卡建立者圓點化與 meta 列）
- [x] 3.2 實作討論全卡識別列（行內尾隨複製鈕）、建立者圓點化與 meta 列至綠；npm test -w packages/ui 全綠

## 4. 收尾驗證與封存順序

- [x] 4.1 全量驗證：cargo test -p speclink-desktop-core、npm test -w packages/ui、npm test -w apps/desktop、npm run build -w apps/desktop 全綠；speclink validate board-card-anatomy 通過
- [x] 4.2 真實視窗手動驗證（CLAUDE.md GUI 備忘，jsdom 測不出行內流位置與 hover）：變更卡與討論卡的複製鈕位置貼合標題文字尾端、hover 顯現與 copied 回饋、長標題截斷、描述列截斷、討論卡圓點 tooltip——驗證結果記錄於 commit 訊息
- [x] 4.3 封存順序確認（design D4：與在途變更的順序邊界）：本變更 archive 前確認 desktop-ux-polish 已封存（speclink list 不含該變更）——本 delta 的「討論於看板第 0 欄兩級呈現」以其落地後正典為基底，順序倒置會回退其內容；並以 speclink drift 檢查 delta 假設仍成立
