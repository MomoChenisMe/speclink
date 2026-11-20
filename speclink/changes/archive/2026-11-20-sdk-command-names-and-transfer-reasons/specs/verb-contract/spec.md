## ADDED Requirements

### Requirement: --json 輸出裡既有的 snake_case 欄位是凍結契約

動詞的 `--json` 輸出裡，下表的欄位名是 snake_case。它們 SHALL 維持現有的名稱，SHALL NOT 改成 camelCase，也 SHALL NOT 消失；它們和 camelCase 的欄位一樣屬於「動詞 --json 輸出形狀凍結」的契約。本需求不改變任何輸出：它把既有的例外明文列出。

| 動詞 | snake_case 的欄位 |
| --- | --- |
| `task done`、`task undone` | `task_id`、`task_desc` |
| `discuss archive` | `archived_to` |
| `analyze` | `change_id`、`artifacts_analyzed`、`artifacts_missing`、`finding_count`，以及每一條發現的 `summary_msg`、`recommendation_msg` |
| `drift` | `change_id`、`last_commit`、`commits_since_created`、`broken_anchors`、`tasks_maybe_resolved`、`tasks_blocked_external`、`spec_assumptions`、`total_score`、`primary_recommendation`、`contributes_to_total` |

之後新增的 `--json` 欄位 SHALL 是 camelCase。使用文件 SHALL 說明這件事：`docs/verb-contract.md` 與 `docs/verb-contract.zh-TW.md` SHALL 列出上表的動詞與欄位，並寫明它們是凍結的；`docs/sdk-node.md` 與 `docs/sdk-node.zh-TW.md` 說明 dispatch 結果的欄位命名時 SHALL NOT 寫成全部都是 camelCase，SHALL 指向那份清單。

#### Scenario: task done 的欄位名不變

- **WHEN** 變更 demo 有任務 `tsk_01`，執行 `speclink task done tsk_01 --change demo --json`
- **THEN** exit code 是 0；stdout 的 JSON 物件含 `task_id` 與 `task_desc` 兩個鍵，沒有 `taskId` 或 `taskDesc`

#### Scenario: discuss archive 的欄位名不變

- **WHEN** 討論 demo-topic 已有結論，執行 `speclink discuss archive demo-topic --json`
- **THEN** exit code 是 0；stdout 的 JSON 物件含 `archived_to`，沒有 `archivedTo`

#### Scenario: analyze 的欄位名不變

- **WHEN** 執行 `speclink analyze demo --json`
- **THEN** exit code 是 0；stdout 的 JSON 物件含 `change_id`、`artifacts_analyzed`、`artifacts_missing`，每個 dimension 含 `finding_count`；有發現時，每一條發現含 `summary_msg` 與 `recommendation_msg`

#### Scenario: 文件列出全部例外

- **WHEN** 檢視 `docs/verb-contract.md` 與 `docs/verb-contract.zh-TW.md` 說明 `--json` 欄位命名的段落
- **THEN** 兩份文件都列出上表的四組動詞與各自的欄位名，並寫明這些欄位不會改名

#### Scenario: SDK 文件不再寫全部是 camelCase

- **WHEN** 檢視 `docs/sdk-node.md` 與 `docs/sdk-node.zh-TW.md` 說明 dispatch 結果的段落
- **THEN** 兩份文件都寫明結果與 CLI 的 `--json` 輸出相同、欄位大多是 camelCase、少數既有欄位是 snake_case，並連到 verb-contract 文件的清單
