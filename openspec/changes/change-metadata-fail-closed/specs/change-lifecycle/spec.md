## ADDED Requirements

### Requirement: 壞 metadata 使生命週期寫入 fail closed

change 的 `.openspec.yaml` 存在但 YAML 解析失敗時，讀寫該 change 生命週期狀態的動詞——in-progress add、claim、task done、task undone、new artifact、archive 與 discard——SHALL 以帶檔案位置與解析原因的錯誤拒絕，且 SHALL NOT 寫入、移動或刪除任何檔案。壞 metadata SHALL NOT 被解讀為未開工、預設 schema 或無來源討論。「檔案不存在」與「欄位缺席」SHALL 維持既有預設行為（「meta 新欄位向後相容」需求不變，其約束對象是缺欄位而非壞檔）。

#### Scenario: in-progress add 對壞 metadata 拒絕且不疊寫

- **WHEN** 對 `.openspec.yaml` 為壞 YAML 的 change 執行 speclink in-progress add 該 change
- **THEN** 以非零 exit code 結束；該 `.openspec.yaml` 逐位元不變（未被文字手術追加或代換 started_* 行）

#### Scenario: task done 因蘊含開工標記而拒絕

- **WHEN** 對壞 metadata 的 change 執行 task done 勾選任一任務
- **THEN** 以非零 exit code 結束；tasks.md 與 `.openspec.yaml` 皆逐位元不變

#### Scenario: discard 不得把壞 metadata 當未開工

- **WHEN** 對壞 metadata 的 change 執行 speclink discard 該 change（未帶 --force）
- **THEN** 以非零 exit code 結束且 change 目錄完整保留；stderr 指出 metadata 損壞，而非以「未開工」放行刪除

#### Scenario: discard 帶 --force 仍拒絕

- **WHEN** 對壞 metadata 的 change 執行 speclink discard 該 change --force
- **THEN** 以非零 exit code 結束且 change 目錄完整保留（使用者修復 metadata 後方可廢棄）

#### Scenario: archive 對壞 metadata 拒絕

- **WHEN** 對壞 metadata 的 change 執行 speclink archive 該 change
- **THEN** 以非零 exit code 結束；正典規格未被併入、change 目錄未被移動
