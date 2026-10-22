## ADDED Requirements

### Requirement: 蓋章記錄審查輪數

`speclink review stamp <change>` 守門通過時，系統 SHALL 在寫入其他章欄位的同一次原子寫入中，把 `reviewed_rounds` 寫入該 change 的 `.openspec.yaml`：值為原有 `reviewed_rounds`（缺席視為 0）加上本次蓋章的審查工單回合數，檔內 SHALL 只有一行 `reviewed_rounds`。帶 `--accept` 的蓋章 SHALL 同樣記錄。守門失敗時 SHALL NOT 改動 `reviewed_rounds`；放棄審查（只刪工單、未蓋章）SHALL NOT 改動 `reviewed_rounds`。原有 `reviewed_rounds` 不是非負整數時，metadata 視為無法解析，蓋章 SHALL 以非零 exit code 拒絕且零寫入。

#### Scenario: 首次蓋章記錄回合數

- **WHEN** change 的審查工單有 3 個回合、寫碼任務全數完成、末輪零必修 findings，執行 `speclink review stamp <change>`
- **THEN** exit code 為 0，`.openspec.yaml` 含 `reviewed_rounds: 3` 與 `reviewed_at` 等章欄位，`review.md` 已刪除

#### Scenario: 章失效後重新蓋章累加輪數

- **WHEN** change 的 `.openspec.yaml` 已有 `reviewed_rounds: 3`，章失效後重開的審查工單有 2 個回合，再次執行 `speclink review stamp <change>` 且守門通過
- **THEN** exit code 為 0，`.openspec.yaml` 只有一行 `reviewed_rounds`，值為 5

#### Scenario: 守門失敗不改輪數

- **WHEN** 審查工單末輪有一條未解的 WARNING finding，執行 `speclink review stamp <change>` 且未帶 `--accept`
- **THEN** exit code 非零，`.openspec.yaml` 的 `reviewed_rounds` 與其他欄位皆不變，`review.md` 仍存在

#### Scenario: 原有輪數不是整數時拒絕

- **WHEN** change 的 `.openspec.yaml` 含 `reviewed_rounds: many`，執行 `speclink review stamp <change>`
- **THEN** exit code 非零，stderr 說明該 change 的 metadata 無法解析，`.openspec.yaml` 與 `review.md` 皆不變
