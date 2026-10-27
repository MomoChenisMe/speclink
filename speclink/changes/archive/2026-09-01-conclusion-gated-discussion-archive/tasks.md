## 1. 引擎守門與 conclude 閉環（crates/speclink-core）

- [x] 1.1 測試先行：在 crates/speclink-core/src/archive.rs 測試模組補「未結論來源討論不隨行封存」紅測——來源討論 Conclusion 為佔位註解時，archive 後記錄仍在 live、不在 archived_discussions 清單；並保留既有三個已結論隨行封存測試為綠（輸出逐位元不變）。驗證：cargo test -p speclink-core（新測試紅、既有綠） <!-- speclink-task:tsk_01M1BKXVTRNKNE1KD5HWTD97PK -->
- [x] 1.2 實作連帶封存守門：crates/speclink-core/src/discuss.rs 增 discussion_concluded（包裝 conclusion_text 為 bool，讀取失敗視同未結論），crates/speclink-core/src/archive.rs 的來源討論過濾器加此條件（對應需求：討論以 link 動詞併入既有變更）。驗證：cargo test -p speclink-core 全綠 <!-- speclink-task:tsk_01M1BKXVTRMG8V84QYQ5SEEGQS -->
- [x] 1.3 測試先行：conclude 閉環四情境紅測（crates/speclink-core/src/discuss.rs 測試模組）——promoted_to 非空且無在途引用時順手封存並回報 auto_archived、仍有在途引用不封存、promoted_to 缺席行為不變、封存步失敗保留結論且回錯誤。驗證：cargo test -p speclink-core（新測試紅） <!-- speclink-task:tsk_01M1BKXVTRF6360FKT7JHKT00Q -->
- [x] 1.4 實作 conclude 閉環：conclude 寫入結論後檢查「promoted_to 非空＋無在途變更 from_discussion 引用」，成立則呼叫既有討論封存函式，回傳值攜帶 auto_archived 事實；寫入順序為先結論後封存、封存失敗不回滾結論（對應需求：conclude 於全數轉出變更已封存時順手封存討論）。驗證：cargo test -p speclink-core 全綠 <!-- speclink-task:tsk_01M1BKXVTRZ2BQZ35RZMTN0GQD -->

## 2. CLI conclude 輸出（crates/speclink-cli）

- [x] 2.1 conclude 輸出增量僅於觸發時出現：閉環觸發時 stdout 多一行順手封存告知（--no-color 無 ANSI）、--json 增 autoArchived: true（camelCase）；未觸發時人眼與 --json 輸出與既有基線逐位元一致。先在 crates/speclink-cli/tests/it/ 補整合測試（含 payload 欄位斷言與未觸發基線比對）再實作 crates/speclink-cli/src/verbs/discuss.rs。驗證：cargo test -p speclink-cli --test it <!-- speclink-task:tsk_01M1BKXVTRGVMG373J3JN7TQK9 -->

## 3. protocol 欄位與邊緣組裝

- [x] 3.1 DiscussionInfo 增選填 concluded 欄位（Option 布林、camelCase、serde default、None 省略鍵），round-trip 測試涵蓋 true／false 序列化與舊 payload 缺席容錯（對應需求：討論資訊 payload 增選填 concluded 欄位）。路徑：crates/speclink-protocol/src/query.rs。驗證：cargo test -p speclink-protocol <!-- speclink-task:tsk_01M1BKXVTRAZFHW1M92KSD4MSE -->
- [x] 3.2 server 邊緣組裝與結論端點回填：GET /discussions 每筆恆填 concluded（引擎結論查詢）、單筆查詢失敗以欄位缺席容錯；討論結論端點於閉環觸發時回填 autoArchived: true（未觸發省略鍵）。先補 crates/speclink-server/tests/it/ 的 discussion_routes 與 verb_api 測試再實作 crates/speclink-server/src/routes.rs（對應需求：討論列表回應攜帶 concluded、archive 與工單讀取端點回填完整結果）。驗證：cargo test -p speclink-server --test it，且 crates/speclink-cli/tests/it/remote_verb_parity.rs 維持綠 <!-- speclink-task:tsk_01M1BKXVTRW6WRPKMQ0YRATRP7 -->
- [x] 3.3 本地橋接組裝 concluded：crates/speclink-host/src/bridge.rs 與 apps/desktop/core（speclink-desktop-core）的討論清單查詢各自呼叫同一 core 函式恆填 concluded，本地與 remote 同形。驗證：cargo test -p speclink-host 與 speclink-desktop-core 測試（跑前依既有慣例補 sidecar 與 apps/server-web dist） <!-- speclink-task:tsk_01M1BKXVTRK9229YA30JTAQXC0 -->

## 4. 前端分區與標示（packages/ui、apps/desktop）

- [x] 4.1 看板討論欄三態分區：packages/ui/src/adapter.ts 的 DiscussionItem 增選填 concluded；packages/ui/src/components/DiscussionColumn.tsx 分區判準改為「promoted 且 concluded === true 才收合」，concluded === false 的 promoted 留上區全卡帶「已轉出・尚無結論」標且無動詞按鈕、計數徽章計上區卡數，欄位缺席退回現行收合。先補 packages/ui/src/__tests__/discussionColumn.test.tsx 的三態測試（false 上區帶標、true 收合、缺席退回）再實作（對應需求：討論於看板第 0 欄兩級呈現）。驗證：npm test -w packages/ui <!-- speclink-task:tsk_01M1BKXVTR4Y0KJYQRY9EFZTNN -->
- [x] 4.2 tray 面板、系統匣與資料來源同判準：apps/desktop/src/panel/TrayPanel.tsx 與 apps/desktop/src/tray.ts 的「已轉出」分區改為 concluded === true 才歸入、缺席退回現行；apps/desktop/src/adapter/tauriDataSource.ts 與 apps/desktop/src/adapter/remoteDataSource.ts 映射 wire 的 concluded、缺席不補值；「已轉出・尚無結論」文案進 i18n 資源。先補 apps/desktop/src/__tests__/ 的 trayPanel、tray 與兩個 dataSource 測試再實作（對應需求：討論列表、capability 驅動停用且不偽造缺口）。驗證：npm test -w apps/desktop <!-- speclink-task:tsk_01M1BKXVTRY1EW1N0K46EB1NRH -->

## 5. 技能措辭與收尾

- [x] 5.1 discuss 與 improve 技能 asset 措辭對齊：crates/speclink-core/assets/skills/discuss.md 與 improve.md 內文「最後一個變更封存時自動封存討論」的敘述補上「且結論已寫入」條件（improve.md 為 fan out 段的同一句），同一次隨動 ASSET_VERSION、再生 golden（crates/speclink-core/tests/golden）、更新 assets.lock，執行 speclink update 再生 claude 與 codex 兩工具的 SKILL.md。驗證：cargo test -p speclink-core（golden 比對綠），內容檢閱 .claude/skills/speclink-discuss/SKILL.md 與 .claude/skills/speclink-improve/SKILL.md 皆含新措辭 <!-- speclink-task:tsk_01M1BKXVTRJQXTG5B37ZKRCQTR -->
- [x] 5.2 收尾：change 橫跨多面，自行加跑一次 npm run test:all；以 git status 盤點收尾 commit 檔集（含 speclink update 再生的 SKILL.md 與 golden），確認無漏帶或多帶。驗證：npm run test:all 全綠、git status 與預期檔集一致 <!-- speclink-task:tsk_01M1BKXVTRH73KV8KHF7QWYKD7 -->
- [x] [M] 5.3 手動驗收：開啟 desktop 看板，確認一筆已轉出但尚無結論的討論呈上區全卡帶「已轉出・尚無結論」標，寫入結論後移入欄底「已轉出」收合列；tray 面板與系統匣分區一致 <!-- speclink-task:tsk_01M1BKXVTRXF8PTWKARNV76BZH -->
