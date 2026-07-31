## ADDED Requirements

### Requirement: 單筆封存的任務完成度守門

speclink archive <change>(單筆路徑)SHALL 於封存前檢查 tasks.md 的任務完成度:任務總數大於零且完成數小於總數、且未帶 --mark-tasks-complete 時 SHALL 拒絕——非零 exit code,stderr 列證據(完成數/總數)與兩條出路(完成任務後再封存、或帶 --mark-tasks-complete),且 SHALL NOT 改動任何檔案(change 目錄、正典 specs、快照與 touched 紀錄逐位元不變)。帶 --mark-tasks-complete 時 SHALL 維持既有語意:先將 tasks.md 全部勾選再封存。任務全數完成、或任務總數為零的 change,單筆封存的人眼與 --json 輸出 SHALL 與守門引入前逐位元一致。此守門 SHALL 於引擎封存流程本體生效,一體適用 CLI 單筆、桌面 app 封存動詞與 server 封存通道——桌面對任務未完成 change 觸發封存時 SHALL 收到引擎拒絕訊息(依既有失敗 toast 語意呈現),SHALL NOT 將該 change 標為已封存。批次封存(--all 或多變更名)的預過濾與跳過回報行為 SHALL 維持不變。本守門屬刻意行為變更:單筆封存對任務未完成 change 由成功改為拒絕。

#### Scenario: 任務未完成的單筆封存被拒

- **WHEN** 對 tasks.md 有 3 個任務、僅 1 個已勾的 change 執行 speclink archive <change>
- **THEN** 非零 exit code;stderr 載明完成數與總數(1/3)並提示完成任務或 --mark-tasks-complete;openspec/ 下任何檔案逐位元不變,changes/archive/ 無新目錄

#### Scenario: --mark-tasks-complete 放行並先全勾

- **WHEN** 對同一 change 執行 speclink archive <change> --mark-tasks-complete
- **THEN** exit code 0;封存後的 tasks.md 全部任務為已勾;change 移入 changes/archive/,stdout 報告與既有成功路徑一致

#### Scenario: 任務全完成的單筆封存逐位元不變

- **WHEN** 對任務全數完成的 change 執行 speclink archive <change>(人眼與 --json 各一次)
- **THEN** 兩種輸出與 exit code 皆與守門引入前完全一致,封存效果(specs 套用、快照、meta 蓋章)不變

#### Scenario: 桌面封存動詞收到引擎拒絕

- **WHEN** 桌面 app 對任務未完成的 change 觸發封存並確認
- **THEN** 封存不發生,app 依既有失敗 toast 語意呈現引擎拒絕訊息,該 change 仍在看板

##### Example: 守門判定

| 任務總數 | 完成數 | 帶 --mark-tasks-complete | 結果 |
| -------- | ------ | ------------------------ | ---- |
| 3        | 1      | 否                       | 拒絕 |
| 3        | 1      | 是                       | 先全勾再封存 |
| 3        | 3      | 否                       | 照常封存 |
| 0        | 0      | 否                       | 照常封存 |
