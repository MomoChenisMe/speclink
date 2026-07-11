## 1. Rust 清單 payload 擴欄位與快取版本（design D4／D5，speclink-desktop-core）

- [x] 1.1 撰寫失敗測試：list_specs_at 每筆規格帶 requirementCount（`### Requirement:` 標題數）、purposeExcerpt（Purpose 首個非空行）、purposeTbd（偵測「TBD - created by archiving」佔位）、traceCount（@trace source 去重數），欄位 camelCase；不可讀／缺席檔案容錯為 0／null 且清單照常回傳——apps/desktop/core/src/query.rs 的 #[cfg(test)]，cargo test -p speclink-desktop-core 預期紅
- [x] 1.2 實作 list_specs_at 欄位計算至綠——「規格與封存卡片收合資訊」的規格卡資料源；Purpose 佔位偵測與封存佔位文案的一致性以同一單元測試釘住（design D4：清單 payload 擴欄位，卡片資訊由 Rust 端計算）
- [x] 1.3 撰寫失敗測試：archived_changes_at 每筆帶 specCount（specs/ 下 capability 目錄數）、createdBy（.openspec.yaml 的 created_by，缺席 null）、fromDiscussions（slug 陣列，缺席空陣列），且 CACHE_VERSION 2→3 後舊版快取整表重建、新欄位入庫——apps/desktop/core/src/cache.rs，預期紅
- [x] 1.4 實作 archived_changes_at 擴欄位至綠（design D5：封存快取版本遞升與重建）
- [x] 1.5 撰寫失敗測試：project_stats_at 回傳 pendingWrapUp＝已就緒（任務全完成）變更數＋concluded 未轉出討論數（promoted 與 open 不計），且不再含 inProgressChanges——apps/desktop/core/src/project.rs，預期紅
- [x] 1.6 實作 pendingWrapUp 派生至綠；cargo test -p speclink-desktop-core 全綠（本組不動 crates/speclink-core 與 crates/speclink-cli，CLI 回歸對照不受影響）

## 2. 頁籤徽章改「待收尾數」（design D6，前端）

- [x] 2.1 撰寫失敗測試：tabs.ts 的 pendingWrapUpCount(changes, discussions) 派生規則（changeStage ready 變更＋status concluded 討論計入；in-progress／promoted／open 不計），tooltip 改待收尾語意文案且 zh-TW 與 en 鍵集合相等——apps/desktop/src/__tests__/tabs.test.ts，npm test -w apps/desktop 預期紅
- [x] 2.2 實作 pendingWrapUpCount 取代 inProgressCount、store.ts 接線（活躍分頁隨看板刷新派生、背景分頁讀 stats 快照的 pendingWrapUp）至綠——涵蓋「專案分頁列存於 app 本機」的徽章語意：2 個已就緒變更＋1 份已結論未轉出討論時徽章顯示 3，全部收尾後歸零（design D6：頁籤徽章改「待收尾數」）
- [x] 2.3 openspec/LANGUAGE.md 收新詞「待收尾」（definition：等使用者執行動詞的卡片＝已就緒變更＋已結論未轉出討論；avoid 與 why 依詞彙表體例），speclink language show 確認條目可讀

## 3. 規格抽屜與規格卡（design D1／D2／D3／D7）

- [x] 3.1 撰寫失敗測試：SpecDrawer 開啟載入正典全文與溯源 footer、文件缺席顯示空狀態、refreshGen 世代重載不清空且 latest-wins 防交錯、寬度樣式與全螢幕切換與變更詳情抽屜一致——packages/ui/src/__tests__/specDrawer.test.tsx，npm test -w packages/ui 預期紅（design D1：唯讀抽屜成對新建，重用既有內容元件；design D3：懶載入與世代重載搬進抽屜）
- [x] 3.2 實作 packages/ui/src/components/SpecDrawer.tsx 至綠（重用 Sheet／Markdown／溯源解析既有元件）
- [x] 3.3 撰寫失敗測試：SpecList 卡片降級——無 chevron 與行內展開、點整列觸發 onOpen、複製鈕位於標題群組內（標題文字後緊跟、hover 顯現）；新欄位渲染——需求數、溯源變更數、相對時間、purposeExcerpt 一行截斷、purposeTbd 時琥珀「Purpose 待補」提示——改寫 packages/ui/src/__tests__/specList.test.tsx，預期紅；涵蓋「桌面 app 呈現 change 與 spec 的清單與內容」的抽屜語意與「規格與封存卡片收合資訊」的規格卡（design D7：卡片版面與資訊欄位）
- [x] 3.4 實作 SpecList 卡片化與 packages/ui/src/adapter.ts 的 SpecItem 欄位擴充至綠

## 4. 封存抽屜與封存卡（design D1／D2／D3／D7）

- [x] 4.1 撰寫失敗測試：ArchivedDrawer 以 discriminated target 承載兩型——封存變更四分頁唯讀（提案／設計／任務／規格，任務核取方塊 disabled 且無批次工具列）、封存討論「背景」「討論過程」「結論」區段、缺件文件空狀態、無任何寫入動詞——packages/ui/src/__tests__/archivedDrawer.test.tsx，預期紅；涵蓋「已封存項目以抽屜檢視」
- [x] 4.2 實作 packages/ui/src/components/ArchivedDrawer.tsx 至綠（重用 SectionedDoc／TaskList readOnly／DeltaSpecView／splitDiscussionSections＋RoundsView／ConclusionView）
- [x] 4.3 撰寫失敗測試：ArchivedList 兩節卡片化、行內展開全數移除（正典「已封存變更可展開檢視」requirement 依 delta 移除後由抽屜承接）；封存變更卡欄位——任務徽章未全完成警示樣式與全完成可辨、觸及規格數、createdBy 頭像圓點 tooltip、來源討論標記缺席不顯示；封存討論卡——複製 slug 鈕寫入剪貼簿且不開抽屜、輪數與衍生變更數（自既有 promoted_to 長度派生）——改寫 ArchivedList 既有測試，預期紅；涵蓋「已封存頁含討論節」與「規格與封存卡片收合資訊」的封存卡
- [x] 4.4 實作 ArchivedList 卡片化與 packages/ui/src/adapter.ts 的 ArchivedItem 欄位擴充至綠

## 5. store 與宿主接線（design D2）

- [x] 5.1 撰寫失敗測試：store 新增 detailSpec 與 detailArchived（change／discussion discriminated union）開閉 action 比照 detailChange；App 掛載 SpecDrawer 與 ArchivedDrawer 並接線；tauriDataSource.ts 與 workspace.ts 原樣傳遞新 payload 欄位——apps/desktop/src/__tests__/store.test.ts、App.test.tsx，npm test -w apps/desktop 預期紅（design D2：抽屜狀態比照 detailChange 接進 store）
- [x] 5.2 實作 store.ts、App.tsx、apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/adapter/workspace.ts 接線至綠；packages/ui/src/index.ts 匯出新抽屜、i18n.tsx 補齊兩語系新鍵

## 6. 收尾驗證

- [x] 6.1 全量驗證：cargo test -p speclink-desktop-core、npm test -w packages/ui、npm test -w apps/desktop、npm run build -w apps/desktop 全綠；speclink validate spec-archive-drawer 通過
- [x] 6.2 真實視窗手動驗證（CLAUDE.md GUI 備忘，jsdom 測不出抽屜與點擊互動）：點規格卡與封存卡開抽屜、全螢幕切換與還原、外部改檔後開啟中的抽屜內容更新、封存一個已就緒變更後分頁徽章即時遞減、封存變更抽屜點來源討論 chip 跳轉——驗證結果記錄於 commit 訊息
- [x] 6.3 以 speclink drift 檢查與在途變更 desktop-ux-polish 的檔案交集（apps/desktop/src/App.tsx、store.ts），有交集落地順序衝突時由後落地者 rebase 並重跑 6.1

## 7. 驗證回饋修正（2026-07-11 真實視窗驗證）

- [x] 7.1 撰寫失敗測試：ArchivedDrawer 封存變更 target 的來源討論 chips——sourceDiscussions 帶值時標題下方顯示各討論 topic 的可點 chips、點擊以 slug 呼叫 onOpenDiscussion、sourceDiscussions 缺席或空陣列時區塊不渲染、封存討論 target 不顯示 chips——packages/ui/src/__tests__/archivedDrawer.test.tsx，npm test -w packages/ui 預期紅（design D1 增補：header 同源連結）
- [x] 7.2 實作 ArchivedDrawer 的 sourceDiscussions／onOpenDiscussion props 與 App.tsx 接線（自 store.archived 以 datedName 查 fromDiscussions、topic 自 discussions 兩節以 slug 解析且缺席退回 slug、點擊 openArchived({ kind: "discussion", slug }) 於同一抽屜切換）至綠——涵蓋 spec Scenario「自封存變更抽屜跳轉來源討論」；App.test.tsx 補接線測試（點 chip 後抽屜呈現討論區段）
- [x] 7.3 撰寫失敗測試：卡片計數 meta 統一「裸 icon＋數字」——規格卡需求數元素不再帶 pill 底色類（rounded-full／bg-muted 缺席）、封存討論卡衍生變更數為 icon＋數字（非 Badge 圓圈）；任務數徽章維持 pill 與琥珀警示不變——改寫 specList.test.tsx 與 archivedList.test.tsx 對應斷言，預期紅（design D7 增補：計數 meta 文法統一）
- [x] 7.4 實作 SpecList／ArchivedList 計數 meta 樣式統一至綠（tooltip 與 aria-label 在地化全文保留）
- [x] 7.5 重跑全量驗證：npm test -w packages/ui、npm test -w apps/desktop、npm run build -w apps/desktop 全綠（Rust 端無涉不重跑）；speclink validate spec-archive-drawer 通過
