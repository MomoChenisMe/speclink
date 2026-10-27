## Context

desktop core（apps/desktop/core，純邏輯、不依賴 Tauri）是看板與抽屜所有 IPC 資料的單一來源。worktree 觀察面（speclink-host 的 observed_facts ＋ WorktreeOverlay）目前只覆蓋 list_changes_at 的任務計數與 watch_targets_at；其餘 per-change 入口（文件、meta、狀態、驗證、分析）與全部寫入動詞仍以主 checkout 的 ProjectContext 讀寫，凍結度內容指紋也直讀主 checkout 檔案。worktree 是 git 的完整 checkout：自帶 openspec/、.git 指標與工作樹，speclink 的任何 Workspace 操作都能直接以 worktree 路徑為根執行。核心引擎的協作函式（如任務完成）把效果分別掛在 store（artifact 讀寫、開工章）與 Workspace（touched 記錄、git 髒檔歸因、head commit）兩個把手上。

## Goals / Non-Goals

**Goals:**

- 有 worktree 映射的 change，desktop 的讀取與寫入落點一致解析到該 worktree 副本
- observed_facts 為空時，所有入口行為與現狀完全相同（位元級不變的既有紅線）
- 刪除動詞補上與封存、退回同級的 worktree 守門
- 以 worktree fixture 回歸測試釘住讀寫落點與審查凍結度正反例

**Non-Goals:**

- 不改 speclink-core 的 review::freshness 純函式與任務協作函式語意
- 不碰 CLI 動詞與其人眼／--json 輸出
- 不動 remote／server 路徑（TeamStore 無 git worktree 概念）
- 不改前端視覺、i18n 文案與 IPC payload 欄位形狀
- 不引入任何持久化快取或新設定欄位

## Decisions

### D1 per-change 入口以重新定根的 worktree context 執行

單一 change 為參數的入口（document_at、status_at、change_capabilities_at、change_meta_at、validate_at、analyze_at、set_task_done_at、set_all_tasks_at、move_task_at、reorder_card_at、discard_review_at）統一經一個 desktop core 內部 helper 解析執行 context：以 observed_facts 查該 change，有映射則以該 worktree 路徑重建 ProjectContext（worktree 是完整 checkout，Workspace 與 store 天然成立），無映射則沿用主 checkout context。讀與寫共用同一機制，因此任務完成的全部側效（tasks.md 勾章、touched 記錄、git 髒檔歸因、head commit、首次完成開工章）自然一致落在 worktree 內——歸因掃的是 worktree 的髒檔，比現狀寫主 checkout 更正確。

替代方案：把 WorktreeOverlay 擴大到寫入面。否決——overlay 只解決 store 層 artifact 讀寫，解不了 Workspace 側的側效（touched、git 歸因都吃 Workspace 根目錄），得為每個動詞逐點傳遞 per-change Workspace，散彈式改動且容易再漏。

### D2 批次入口維持 overlay、凍結度指紋逐 change 解析讀檔根

一次處理全部 change 的批次入口（list_changes_at、search_workspace_at）不逐 change 重建 context：list 維持既有 overlay 組裝；search 的 artifact 掃描改吃同一 overlay store（它只讀 artifacts，overlay 即足夠）。凍結度的內容指紋讀的是 repo 任意程式檔、不是 artifact，overlay 幫不上——list_changes_at 內的檔案讀取閉包改為逐 change 解析根目錄：該 change 在 facts 有映射用其 worktree 路徑，否則用主 checkout 根。

替代方案：批次入口也逐 change 重建 context。否決——list 每次刷新對每個 change 各 spawn 一次 git 代價過高，且 overlay 已存在並被 CLI list 共用（design D6 的同構紅線）。

### D3 刪除補守門、動詞防護分兩級

delete_change_at 加上與封存、退回相同的 worktree 守門（拒絕並提示先執行 worktree-merge 收尾）。防護需求自此分兩級：破壞性生命週期動詞（封存、退回提案中、刪除）守門拒絕——它們的語意是動主 checkout 的 change 目錄存廢，路由進 worktree 沒有意義；粒度寫入動詞（任務勾選、全部勾選、任務拖排、卡片拖排、放棄審查工單）路由至 worktree（D1）——它們的語意是編輯 change 內容，內容的現行所在就是 worktree。

替代方案：粒度寫入也守門擋下。否決——討論 worktree-flow-gaps 裁定顯示與操作一致優先，讀取面修正後唯讀櫥窗體驗劈半。

### D4 facts 每次現取、紅線與缺檔 fallback 不變

per-change helper 每次呼叫現取 observed_facts、不快取：observed_facts 的映射條件已含「worktree 內 change 目錄可讀」，資料夾被移除的下一次呼叫自然回讀主 checkout，無 stale 視窗。三道閘門（非 git 目錄、app config 不可讀、worktree 政策關）任一成立時 facts 為空，全部入口行為與現狀完全相同。凍結度「缺檔即不符 → Stale」維持現行語意（討論裁定保守 fallback 正確）。

## Implementation Contract

- **行為**：對有 worktree 映射的 change——(1) 抽屜任務分頁的勾選狀態與分頁徽章計數同源一致；(2) worktree 內蓋章且其後 scope 檔未再變動時清單項 reviewStatus 為 "reviewed"，scope 檔於 worktree 內真的變動後為 "reviewedStale"；(3) 抽屜各分頁文件、meta 欄位、驗證與分析報告反映 worktree 現值；(4) 全文搜尋能命中僅存在於 worktree 的內容；(5) 勾選、全勾、任務拖排寫入 worktree 的 tasks.md，主 checkout 檔案不動；(6) 卡片拖排的 rank 寫入 worktree 的 change metadata，重新整理後位置保持；(7) 刪除被拒絕，錯誤訊息含 worktree-merge 收尾指引；(8) facts 為空時以上全部退回現狀行為。
- **介面／資料形狀**：desktop core 公開函式簽名、IPC command 名、payload 欄位（camelCase）全部不變；變更僅在函式內部的 context 解析。新增之 helper 為 crate 內部（pub(crate) 以下），不進公開 API。
- **失敗模式**：worktree 目錄不可讀時 observed_facts 略過該條目、入口回讀主 checkout（既有 discovery 慣例，靜默）；守門拒絕沿用封存的錯誤訊息模式（含 change 名與收尾指引）；寫入動詞於定根後的檔案錯誤原樣上拋（與現狀同）。
- **驗收**：cargo test -p speclink-desktop-core 全綠。新增測試沿用 verbs.rs 既有 attach_worktree fixture helper：(a) worktree 內 scope 檔為蓋章時內容、主 checkout 為舊內容 → reviewStatus == "reviewed"；(b) worktree 內 scope 檔蓋章後再改 → "reviewedStale"；(c) document_at 回傳 worktree 版 tasks.md；(d) set_task_done_at 寫 worktree 檔、主 checkout tasks.md 位元不變；(e) delete_change_at 被拒且訊息含收尾指引；(f) facts 空（政策關）時各入口輸出與無 worktree 時一致。npm test -w @speclink/ui 不受影響。
- **範圍邊界**：in——apps/desktop/core 五個檔（query.rs、manage.rs、verbs.rs、search.rs、lib.rs）與其測試、worktree-overlay 與 client-protocol 兩份 delta spec；out——CLI、remote／server、前端元件與文案、speclink-core 引擎、worktree-merge 技能（另一 change）。

## Risks / Trade-offs

- [回歸對照] CLI 零改動、desktop payload 形狀不變，既有 golden（render_golden）與 CLI 測試不應變動；驗收跑 cargo test -p speclink-core --test it render_golden:: 確認未意外波及 → 若有變動即實作越界，回頭修
- [每入口現取 facts 增加 git 呼叫] 抽屜開啟與動詞觸發皆為低頻互動，list 每次刷新已有相同成本先例；不做快取（Non-Goal），效能真成問題時另案 → 以互動延遲觀察，不預先優化
- [跨平台] worktree 路徑於 Windows 為反斜線原生形式、macOS 的 tempdir 有 symlink 正規化差異；測試比對路徑一律經 canonicalize，沿用 attach_worktree fixture 的既有處理 → 三平台 CI 全綠為準
- [GUI 與 agent session 同寫 worktree] 檔案層 last-write-wins，與現狀主 checkout 的併寫語意相同；desktop 端已有 write_guard 序列化自身寫入 → 不新增鎖，維持現狀語意
- [git 併發 spawn] loopback 併行測試曾有 EINVAL 前例（HARNESS_GATE）；新測試沿用既有 fixture 的序列化慣例 → 不引入平行 git fixture

## Migration Plan

無部署面：desktop app 隨版本更新生效，無資料格式變更、無設定遷移。回滾即 revert 對應 commit。

## Open Questions

（無——實作形狀、fallback 語意與守門分級已由討論 worktree-flow-gaps 裁決。）
