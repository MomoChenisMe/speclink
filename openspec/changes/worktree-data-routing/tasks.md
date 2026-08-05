## 1. 測試先行——凍結度與讀取面紅燈

- [ ] 1.1 [測試先行] 於 apps/desktop/core/src/query.rs 測試模組沿用 verbs.rs 的 attach_worktree fixture，補「變更清單的審查狀態欄位」的 worktree 正反例：worktree 內 scope 檔現值與蓋章雜湊相符、主 checkout 同名檔為舊內容 → 清單項 reviewStatus == "reviewed"（欄位存在、camelCase）；worktree 內再改該檔 → "reviewedStale"。驗證：cargo test -p speclink-desktop-core 兩測試紅燈（現行實作誤判 stale）。 <!-- speclink-task:tsk_01KZ929S2DS9X9VZPBN58DVMZM -->
- [ ] 1.2 [測試先行] 讀取面紅燈：worktree 副本 tasks.md 已勾、metadata 有開工章、主 checkout 皆為舊內容時，document_at 回傳內容 SHALL 含已勾任務行、change_meta_at 的 startedAt 非 null、status_at 反映 worktree 副本。檔案：apps/desktop/core/src/query.rs、apps/desktop/core/src/manage.rs。驗證：cargo test -p speclink-desktop-core 新測試紅燈。 <!-- speclink-task:tsk_01KZ929S2DCZKFWY5WNHE68JMK -->

## 2. 綠燈——per-change 定根與批次解析

- [ ] 2.1 依 design「D1 per-change 入口以重新定根的 worktree context 執行」於 apps/desktop/core/src/lib.rs 新增 crate 內部 helper（每次現取 observed_facts，該 change 有映射即以 worktree 路徑重建 ProjectContext），並將 document_at、status_at、change_capabilities_at（apps/desktop/core/src/query.rs）、change_meta_at（apps/desktop/core/src/manage.rs）、validate_at、analyze_at（apps/desktop/core/src/verbs.rs）切至 helper。行為：「desktop 看板的 worktree 呈現」的抽屜文件、meta、狀態、驗證與分析全數同源 worktree。驗證：1.2 測試轉綠且既有測試不破。 <!-- speclink-task:tsk_01KZ929S2DGFARFSNW674SHCK6 -->
- [ ] 2.2 依 design「D2 批次入口維持 overlay、凍結度指紋逐 change 解析讀檔根」修 apps/desktop/core/src/query.rs 的 list_changes_at：內容指紋的檔案讀取逐 change 解析根目錄（facts 有映射用該 worktree 路徑，否則主 checkout）；apps/desktop/core/src/search.rs 的 search_workspace_at 改以 overlay store 掃描 artifacts。行為：凍結度判定與全文搜尋命中 worktree 現值。驗證：1.1 測試轉綠；新增「搜尋命中僅存在於 worktree 的字串」測試綠燈。 <!-- speclink-task:tsk_01KZ929S2DE8PVZ5DQK790KV5A -->

## 3. 寫入面路由與守門

- [ ] 3.1 [測試先行] 寫入面紅燈：對有 worktree 映射的 change，set_task_done_at 後 worktree 副本 tasks.md 該任務已勾、touched 記錄落於 worktree 副本、主 checkout tasks.md 位元級不變；reorder_card_at 後 rank 寫入 worktree 副本 metadata；delete_change_at 拒絕且錯誤訊息含 worktree-merge 收尾指引。檔案：apps/desktop/core/src/manage.rs。驗證：cargo test -p speclink-desktop-core 新測試紅燈。 <!-- speclink-task:tsk_01KZ929S2DCD3J24BG79NXQG3M -->
- [ ] 3.2 依 design「D1 per-change 入口以重新定根的 worktree context 執行」將 set_task_done_at、set_all_tasks_at、move_task_at、reorder_card_at（apps/desktop/core/src/manage.rs）與 discard_review_at（apps/desktop/core/src/verbs.rs）切至定根 helper。行為：「worktree 掛著時的 desktop 動詞防護」的粒度寫入動詞（勾選、全勾、任務拖排、卡片拖排、放棄審查工單）檔案效果與側效（touched、開工章、git 髒檔歸因）全數落於 worktree 副本。驗證：3.1 寫入測試轉綠。 <!-- speclink-task:tsk_01KZ929S2DGYXN8QKKMWDQ8YY0 -->
- [ ] 3.3 依 design「D3 刪除補守門、動詞防護分兩級」於 delete_change_at（apps/desktop/core/src/manage.rs）前置呼叫 refuse_if_worktree_is_open。行為：worktree 掛著時刪除被拒、主 checkout 與 worktree 的 change 目錄均不變。驗證：3.1 刪除測試轉綠。 <!-- speclink-task:tsk_01KZ929S2DAF721A9TXCZ41JQH -->

## 4. 紅線與回歸收尾

- [ ] 4.1 依 design「D4 facts 每次現取、紅線與缺檔 fallback 不變」補紅線測試：worktree 政策關閉（或 facts 為空）時，上述全部讀取與寫入入口的輸出與檔案落點與無 worktree 時完全相同；worktree 目錄移除後的下一次呼叫回讀主 checkout。檔案：apps/desktop/core/src/query.rs、apps/desktop/core/src/manage.rs。驗證：cargo test -p speclink-desktop-core 全綠。 <!-- speclink-task:tsk_01KZ929S2DBPDZ50W4RYZANGCK -->
- [ ] 4.2 全量回歸確認範圍未越界：cargo test -p speclink-desktop-core 全綠；cargo test -p speclink-core --test it render_golden:: 零變動（CLI 人眼與 --json 非目標）；npm test -w @speclink/ui 全綠（前端視覺不改）。驗證：三組指令全數通過且 git status 無 golden 檔變動。 <!-- speclink-task:tsk_01KZ929S2D69VEQ0VWYA95GVF8 -->
