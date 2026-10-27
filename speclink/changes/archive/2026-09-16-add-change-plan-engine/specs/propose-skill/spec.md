## MODIFIED Requirements

### Requirement: 收尾盤點提案中變更的執行順序

技能檔 SHALL 規定：propose 完成、給出下一步建議之前，代理人 SHALL 以 list 動詞的 JSON 輸出列出作用中變更名；作用中變更只有本次建立者時 SHALL 維持既有出邊、SHALL NOT 展開盤點段。有兩個以上時 SHALL 只針對本次建立的變更判定軟依賴——讀其他變更的 proposal Impact，判定本變更是否建立在某變更的成果上或動到同一段程式碼；有則 SHALL 執行 `speclink change depends <本變更> --on <前置>...` 落檔，SHALL NOT 只口頭報告。硬信號（delta capability 重疊）SHALL 由引擎計算，代理人 SHALL NOT 自行判定。落檔後 SHALL 執行 `speclink plan --json` 並依有效 worktree 政策（含 SPECLINK_WORKTREE 環境覆寫層）呈現：開啟時 SHALL 列出第 1 波為「可平行——各開一個 session 以 apply-with-worktree 執行，沿用多 session 配方」與後續各波；關閉時 SHALL 依 `changes` 的配置順序給單一建議順序。盤點為僅建議：SHALL NOT 自動呼叫任何技能。

#### Scenario: 多提案且 worktree 開啟時分組

- **WHEN** propose 完成、作用中變更 ≥2 且有效 worktree 政策為開啟
- **THEN** 技能檔指示判定本變更的軟依賴並以 change depends 落檔，再以 plan 輸出列出第 1 波（各開 session 走 apply-with-worktree）與後續波次

#### Scenario: 多提案且 worktree 關閉時給單一順序

- **WHEN** propose 完成、作用中變更 ≥2 且有效 worktree 政策為關閉
- **THEN** 技能檔指示以 plan 的 changes 配置順序給出單一建議順序，不出現平行分組

#### Scenario: 單一提案不盤點

- **WHEN** propose 完成且作用中變更僅本次建立者
- **THEN** 技能檔維持既有下一步建議，不執行 change depends 與 plan

##### Example: 軟依賴落檔後的 plan 呈現

| 作用中變更 | 本次建立 | 代理人判定 | 落檔指令 | plan 波次 |
| --- | --- | --- | --- | --- |
| add-a、add-b | add-b | add-b 建立在 add-a 的新動詞上 | speclink change depends add-b --on add-a | add-a 第 1 波、add-b 第 2 波 |
| add-a、add-c | add-c | 無關 | 不執行 | 兩者同為第 1 波（可平行） |
