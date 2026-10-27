## ADDED Requirements

### Requirement: change metadata 時間欄位寫入時戳、對外輸出日期

系統新寫入 change metadata（`.openspec.yaml`）的 `created`、`started_at`、`claimed_at`、`reviewed_at`、`verified_at` 與 `archived_at` SHALL 為帶本機時區偏移量、精確到秒的 RFC 3339 時戳（例 `2026-10-21T14:05:00+08:00`）。讀取 SHALL 同時接受純日期（`YYYY-MM-DD`）與 RFC 3339 時戳；既有的純日期記錄 SHALL NOT 被改寫。凡目前輸出這些欄位的地方——CLI 人眼輸出與 `--json`（例如 `speclink show`、`speclink list --json`、drift 報告）、server 回應與 desktop 協定——SHALL 只輸出日期部分，也就是時戳的前 10 個字元（寫入者時區的日期），所以同一天寫入的純日期記錄與時戳記錄產生位元級相同的輸出。依建立日期計算的天數（變更過期判定）SHALL 取日期部分。既不是純日期也不是 RFC 3339 時戳的值 SHALL 維持現行行為：原樣輸出，且不計算過期天數。

#### Scenario: 新變更寫入時戳

- **WHEN** 執行 `speclink new change demo`
- **THEN** exit code 為 0，`demo` 的 `.openspec.yaml` 中 `created:` 的值符合 RFC 3339、帶時區偏移量、精確到秒

#### Scenario: 對外輸出只給日期部分

- **WHEN** change `demo` 的 `.openspec.yaml` 含 `created: 2026-10-21T14:05:00+08:00`，執行 `speclink show demo` 與 `speclink show demo --json`
- **THEN** 兩者 exit code 為 0；人眼輸出為 `Created: 2026-10-21`，`--json` 的 `created` 為字串 `"2026-10-21"`

#### Scenario: 純日期記錄的輸出不變

- **WHEN** change 的 `.openspec.yaml` 含 `created: 2026-07-29`，執行 `speclink show <change> --json` 與 `speclink list --json`
- **THEN** 輸出與本變更之前的版本位元級一致，exit code 為 0，沒有警告

#### Scenario: 無法辨識的時間值維持現行行為

- **WHEN** change 的 `.openspec.yaml` 含 `created: someday`，執行 `speclink show <change>`
- **THEN** exit code 為 0，人眼輸出為 `Created: someday`，該 change 不計算過期天數

### Requirement: 封存寫入 archived_at

`speclink archive <change>` 成功且該 change 帶 metadata（`.openspec.yaml`）時，封存目錄的 `.openspec.yaml` SHALL 含 `archived_at`，值為 RFC 3339 時戳，且與封存目錄名的日期前綴、正式規格 `@trace` 的 `updated` 取自同一個「現在」。寫入順序 SHALL 為：守門全數通過後、把 change 移入封存目錄之前，先把 `archived_at` 寫入該 change 的 metadata（已有該鍵時取代，檔內只留一行）；移動失敗時，該 change 留在進行中並帶著 `archived_at`，之後重試封存成功時 SHALL 以新的「現在」取代。封存守門拒絕時 SHALL NOT 寫入 `archived_at`。沒有 metadata 的舊式 change 照常封存，SHALL NOT 為了 `archived_at` 新建 metadata。

#### Scenario: 成功封存帶封存時戳

- **WHEN** 在 2026-10-21 對守門全數通過的 change `demo` 執行 `speclink archive demo`
- **THEN** exit code 為 0，封存目錄名以 `2026-10-21-` 開頭，其 `.openspec.yaml` 含一行 `archived_at`，值為 `2026-10-21T` 開頭、帶時區偏移量的 RFC 3339 時戳

#### Scenario: 封存守門拒絕不寫封存時戳

- **WHEN** change 的審查章已失效，執行 `speclink archive <change>`
- **THEN** exit code 非零，change 留在進行中，其 `.openspec.yaml` 沒有 `archived_at`

#### Scenario: 重試封存只留一行封存時戳

- **WHEN** change 的 `.openspec.yaml` 已帶著前一次失敗封存留下的 `archived_at`，再次執行 `speclink archive <change>` 且成功
- **THEN** exit code 為 0，封存目錄的 `.openspec.yaml` 只有一行 `archived_at`，值為這次封存的時戳
