## ADDED Requirements

### Requirement: 收尾盤點提案中變更的執行順序

技能檔 SHALL 規定：propose 完成、給出下一步建議之前，代理人 SHALL 以 list 動詞的 JSON 輸出列出變更名，並以各變更 metadata（.openspec.yaml）的 started_* 標記缺席判定提案中（未開工）——list 輸出本身分不出已開工與未開工，SHALL NOT 以其 status 或任務數判定。提案中數量 ≥2 時 SHALL 展開執行順序判定——硬信號為 delta capability 重疊（兩個變更的 delta 目錄含同一 capability 即判須依序：delta 重寫同一份正典規格，亂序封存可能觸發合併閘拒絕），軟信號為讀 proposal 與 tasks 推測的程式碼重疊或依賴；僅 1 個時 SHALL 維持既有出邊、SHALL NOT 展開盤點段。有效 worktree 政策（含 SPECLINK_WORKTREE 環境覆寫層）開啟時 SHALL 分「可平行——各開一個 session 以 apply-with-worktree 執行，沿用多 session 配方」與「須依序」兩組呈現；政策關閉時 SHALL 給單一建議順序。盤點為僅建議：SHALL NOT 自動呼叫任何技能，SHALL NOT 依賴引擎新增指令。

#### Scenario: 多提案且 worktree 開啟時分組

- **WHEN** propose 完成、提案中變更 ≥2 且有效 worktree 政策為開啟
- **THEN** 技能檔指示列出全部提案中變更，以 delta capability 重疊與內容推測判定順序，並分「可平行（各開 session 走 apply-with-worktree）」與「須依序」兩組呈現

#### Scenario: 多提案且 worktree 關閉時給單一順序

- **WHEN** propose 完成、提案中變更 ≥2 且有效 worktree 政策為關閉
- **THEN** 技能檔指示給出單一建議執行順序，不出現平行分組

#### Scenario: 單一提案不盤點

- **WHEN** propose 完成且提案中變更僅 1 個
- **THEN** 技能檔維持既有下一步建議，不展開盤點段

##### Example: delta capability 重疊判序

| 變更 A 的 delta 目錄 | 變更 B 的 delta 目錄 | 判定 |
| --- | --- | --- |
| specs/board-card-order/ | specs/board-card-order/ 與 specs/tray-status-menu/ | 須依序（board-card-order 重疊） |
| specs/discuss-skill/ | specs/archive-skill/ | 可平行（無重疊） |
