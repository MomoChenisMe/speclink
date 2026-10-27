## Why

worktree 平行開發流程中，desktop core 對同一個 change 的資料路徑劈成兩半：看板清單的任務計數經 WorktreeOverlay 讀 worktree 副本，但抽屜文件原文、meta 欄位、狀態報告、分析、全文搜尋與審查凍結度的內容指紋全部直讀主 checkout；寫入面（任務勾選、任務拖排、卡片拖排）則寫進主 checkout，與 worktree 分支直接分岔。可觀察症狀：抽屜任務計數正確但勾選全空、worktree 內蓋章的 change 被誤標「已審查·其後有變動」、看板拖排 worktree 卡片彈回原位、「分析」報告整份失真；更危險的是勾選寫錯位置會在 merge 收尾時撞出衝突，而「刪除」無守門會留下幽靈卡。目標使用者是以 desktop 看板監看與操作 worktree 平行開發（apply-with-worktree → worktree-merge 流程）的開發者；來源討論 worktree-flow-gaps 含逐入口掃雷清單與裁決過程。

## What Changes

- desktop core（apps/desktop/core）對有 worktree 映射的 change，資料路徑一律解析到該 change 的 worktree 副本；observed_facts 為空（政策關、非主 checkout、git 不可用）時行為與現狀完全相同——這是既有紅線。
- 讀取面 8 個入口改為解析到 worktree：變更清單的凍結度內容指紋（list_changes_at 內的檔案讀取）、抽屜文件原文（document_at）、狀態報告（status_at）、規格分頁清單（change_capabilities_at）、meta 欄位（change_meta_at）、驗證與分析（validate_at、analyze_at）、看板全文搜尋（search_workspace_at）。
- 寫入面 5 個入口路由至 worktree 副本：任務勾選（set_task_done_at）、全部勾選（set_all_tasks_at）、任務拖排（move_task_at）、卡片拖排（reorder_card_at）、放棄審查工單（discard_review_at）。側效（touched 記錄、首次完成開工章、git 髒檔歸因）隨定根一併落在 worktree 內，歸因反而比現狀正確。
- delete_change_at 補 worktree 守門（比照封存與退回提案中的既有拒絕語意）——刪除屬破壞性生命週期動詞，worktree 掛著時拒絕並提示先收尾，不路由。
- 補 worktree 情境的回歸測試：worktree 內容與主 checkout 相異時各入口讀寫落點的斷言（含審查凍結度 reviewed／reviewedStale 正反例）。

相容性影響：desktop IPC 的人眼呈現與 payload 欄位形狀（camelCase 契約）皆不變，變的只是 worktree 映射存在時欄位的「值」（讀到 worktree 現值）；CLI 的人眼輸出與 --json 完全不動；不涉及設定欄位與技能注入區塊。

## Non-Goals

- 不改 review::freshness 雙錨純函式語意（core 行為正確，錯在呼叫端資料源）
- 不碰 CLI 動詞（list 已走 overlay；show／status 等維持現狀）
- 不改看板視覺呈現與 i18n 文案
- 不動 remote／server 路徑（TeamStore 後端無 git worktree 概念）
- 凍結度「缺檔即 Stale」fallback 維持現行語意
- worktree-merge 技能的 rebase-first 改動歸同討論扇出的另一 change（worktree-merge-rebase-first）

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `worktree-overlay`: 「desktop 看板的 worktree 呈現」自卡片標示與抽屜分支資訊，擴充為全讀取面（抽屜各分頁文件、meta 欄位、狀態報告、驗證／分析、全文搜尋）解析到 worktree 副本；「worktree 掛著時的 desktop 動詞防護」重述為兩級——破壞性生命週期動詞（封存、退回提案中、刪除）守門拒絕，粒度寫入動詞（任務勾選、全部勾選、任務拖排、卡片拖排、放棄審查工單）路由至 worktree 副本。
- `client-protocol`: 「變更清單的審查狀態欄位」的凍結度重算，內容指紋於有 worktree 映射時 SHALL 讀該 worktree 的檔案現值（新增 worktree 情境 Example）。

## Impact

- Affected specs: worktree-overlay、client-protocol
- Affected code:
  - Modified: apps/desktop/core/src/query.rs、apps/desktop/core/src/manage.rs、apps/desktop/core/src/verbs.rs、apps/desktop/core/src/search.rs、apps/desktop/core/src/lib.rs、apps/desktop/core/src/testfixture.rs
  - New: (none)
  - Removed: (none)
