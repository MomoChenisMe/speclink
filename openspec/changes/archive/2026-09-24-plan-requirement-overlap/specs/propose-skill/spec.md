## MODIFIED Requirements

### Requirement: 收尾盤點提案中變更的執行順序

<!-- BEFORE: 硬信號為 delta capability 重疊交引擎；收尾只寫 depends_on，不碰 rank -->

技能檔 SHALL 規定：propose 完成、給出下一步建議之前，代理人 SHALL 以 list 動詞的 JSON 輸出列出作用中變更名；作用中變更只有本次建立者時 SHALL 維持既有出邊、SHALL NOT 展開盤點段。有兩個以上時 SHALL 只針對本次建立的變更判定軟依賴——讀其他變更的 proposal Impact，判定本變更是否建立在某變更的成果上或動到同一段程式碼；有則 SHALL 執行 `speclink change depends <本變更> --on <前置>...` 落檔，SHALL NOT 只口頭報告。硬信號 SHALL 由引擎計算：requirement 級重疊（同 capability 同 requirement 名被兩個變更動到）由 plan 依該名稱此刻是否在正式規格中排出先後、算成 `archiveAfter`，兩個變更都帶進（新增或改名成）或都拿走（移除或改名掉）同名 requirement 算成 `conflict`；代理人 SHALL NOT 自行判定重疊，但 plan 對本變更回報 `conflict` 時 SHALL 向使用者提出其中一邊要改。落檔 depends_on 之後 SHALL 判定插隊：本變更為提案中且比它前面的提案中變更小（task 數較少且無人依賴它）而急時，SHALL 執行 `speclink change rank <本變更> --before <第一個比它大的提案中變更>`；動詞以「已有 rank」拒絕時 SHALL 口頭建議、SHALL NOT 加 --force；判定不急或不小時 SHALL NOT 執行；插隊判定段落 SHALL 與 ingest 技能收尾的同一段逐字一致。其後 SHALL 執行 `speclink plan --json` 並依有效 worktree 政策（含 SPECLINK_WORKTREE 環境覆寫層）呈現：開啟時 SHALL 列出第 1 波為「可平行——各開一個 session 以 apply-with-worktree 執行，沿用多 session 配方」與後續各波；關閉時 SHALL 依 `changes` 的配置順序給單一建議順序；任一變更的 `archiveAfter` 非空時 SHALL 附一句「封存時 X 要在 Y 之後」。盤點為僅建議：SHALL NOT 自動呼叫任何技能。

#### Scenario: 多提案且 worktree 開啟時分組

- **WHEN** propose 完成、作用中變更 ≥2 且有效 worktree 政策為開啟
- **THEN** 技能檔指示判定本變更的軟依賴並以 change depends 落檔，判定插隊並視情況以 change rank 落檔，再以 plan 輸出列出第 1 波（各開 session 走 apply-with-worktree）與後續波次，並附 archiveAfter 的封存順序句

#### Scenario: 多提案且 worktree 關閉時給單一順序

- **WHEN** propose 完成、作用中變更 ≥2 且有效 worktree 政策為關閉
- **THEN** 技能檔指示以 plan 的 changes 配置順序給出單一建議順序，不出現平行分組

#### Scenario: 單一提案不盤點

- **WHEN** propose 完成且作用中變更僅本次建立者
- **THEN** 技能檔維持既有下一步建議，不執行 change depends、change rank 與 plan

#### Scenario: 小而急的新變更插隊

- **WHEN** propose 完成、本變更有 6 個 task 且無人依賴、前面有一個 30 個 task 的提案中變更 add-big（已有 board_rank，所以排在本變更前面）
- **THEN** 技能檔指示執行 speclink change rank <本變更> --before add-big；動詞拒絕（已有 rank）時只口頭建議、不加 --force

##### Example: 軟依賴落檔後的 plan 呈現

| 作用中變更 | 本次建立 | 代理人判定 | 落檔指令 | plan 波次 |
| --- | --- | --- | --- | --- |
| add-a、add-b | add-b | add-b 建立在 add-a 的新動詞上 | speclink change depends add-b --on add-a | add-a 第 1 波、add-b 第 2 波 |
| add-a、add-c | add-c | 無關 | 不執行 | 兩者同為第 1 波（可平行） |
| add-a、add-d | add-d | 無關，但 add-d 只有 5 個 task、add-a 有 40 個且無人依賴；add-a 已有 board_rank，排在 add-d 前面（兩者都沒有 rank 時小的本就在前，不必插隊） | speclink change rank add-d --before add-a | 兩者同為第 1 波，配置順序 add-d 在前 |
