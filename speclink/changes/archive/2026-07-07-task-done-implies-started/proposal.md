## Why

看板「進行中」欄依 change meta 的 started_at 判定，但開工章目前只由 apply 流程的顯式 speclink in-progress add 蓋；CLI task done 與桌面 GUI 勾任務都不蓋章，也只有 CLI 會記 touched-files。實證：desktop-discussion-board 完成 2/14 任務而 meta 無 started_at，卡片停留「提案中」——「任務有進度卻顯示未開工」對所有觀察看板的人都是錯誤資訊。討論「gui-勾任務自動蓋開工章」已結論：完成任務的完整語意應下沉為引擎層單一協作點，並以看板派生涵蓋繞過工具的寫入路徑。

目標使用者與情境：透過 AI 代理跑 SDD 的開發者與旁觀看板的 PO/PM——agent 在 apply 階段跑 speclink task done、人在桌面看板勾任務，兩者都期待卡片即刻且正確地反映開工狀態。

## What Changes

- 引擎（speclink-core 的 tasks 層）新增「任務完成」協作函式，單點完成四件事：勾章、寫回 tasks.md、touched-files 記錄、首次完成時蓋開工章（沿用既有 inprogress 引擎函式——冪等、首章保留、身分「不能歸屬即缺席」）。
- CLI 的 task done 子指令改為該函式的薄呼叫端：人眼輸出、--json 輸出、錯誤訊息與順序、exit code 全部不變；新增的檔案效果僅為首次完成時 change meta 多出 started_* 欄位。
- 桌面 GUI 勾任務（done=true）改走同一協作函式，行為與 CLI task done 對齊（含 touched-files 記錄與蓋章）；取消勾選與拖曳排序維持現行桌面實作、不蓋章、不記 touched。
- 看板欄位派生規則加入「無開工章但任務完成數大於 0 ＝進行中」，涵蓋手改 tasks.md、agent 直接編輯、git pull 拉進他人變更等繞過工具的路徑；詳情抽屜的開工歸屬列維持「meta 有 started_at 才顯示」——派生管顯示、蓋章管歸屬，缺席的歸屬維持缺席。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `change-lifecycle`: 新增（ADDED）需求「任務完成蘊含開工」——經工具路徑（CLI task done／桌面勾選）完成任務 SHALL 單點協作勾章、touched-files 記錄與首次開工蓋章；事後補章 SHALL NOT 發生。
- `desktop-app`: 修改（MODIFIED）需求「看板欄位由生命週期標記驅動」——判定矩陣加入「無 started_at 而完成數大於 0 ＝進行中」；並新增（ADDED）需求「GUI 勾任務與 CLI 完成語意一致」。

## Impact

- Affected specs: change-lifecycle（ADDED 一條）、desktop-app（MODIFIED 一條＋ADDED 一條）
- Affected code:
  - Modified: crates/speclink-core/src/tasks.rs（新增任務完成協作函式與測試）
  - Modified: crates/speclink-cli/src/commands.rs（task done 路徑改薄呼叫端）
  - Modified: apps/desktop/core/src/manage.rs（set_task_done_at 的 done=true 路徑改走協作函式）
  - Modified: packages/ui/src/stage.ts（派生規則加入任務進度）
  - Modified: packages/ui/src/__tests__/stage.test.ts、packages/ui/src/__tests__/kanban.test.tsx（派生矩陣測試同步）
  - New: 無
  - Removed: 無
- 影響的 crate：speclink-core（協作函式）、speclink-cli（薄呼叫端改寫）；另及桌面 app（apps/desktop/core）與前端元件庫（packages/ui）。
- 相容性影響：CLI 人眼與 --json 輸出零變動（蓋章靜默），parity_suite／color_suite／twin 回歸對照不受影響；task done 新增的檔案效果（meta 蓋章）會使自我基線雙沙盒的檔案樹對照出現預期差異，基線需隨本變更刻意更新。in-progress add 指令面（輸出、exit code、冪等、對不存在 change 靜默成功）完全不動。看板欄位判定的變化僅限「無章但有任務進度」的 change 由提案中改列進行中——即修正本變更所針對的錯誤顯示。
