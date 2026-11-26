## ADDED Requirements

### Requirement: 清單的詳細輸出與 show 帶說明、階段與時間

`speclink list --json --detail` 的 `changes` 每一項，與 `speclink show <change> --json` 的輸出，SHALL 在既有的鍵之外增列下列鍵（鍵名一律 camelCase）：

- `stage`：字串，`proposed`、`in-progress` 或 `ready`。SHALL 與 `speclink plan --json` 對同一個變更回報的 `stage` 相同，且 SHALL NOT 依賴 plan 的計算：宣告的依賴成環、plan 失敗時，這個鍵 SHALL 照樣有值。metadata 讀得出來的變更 SHALL 一定有這個鍵。
- `whyExcerpt`：字串，提案第一個 `## Why` 段落裡第一個有內容的行的原文（去掉前後空白），SHALL NOT 截斷。空行、`#` 開頭的行（例如 `### 子標題`）與 HTML 註解（從 `<!--` 開頭的行起，到第一個含 `-->` 的行為止）SHALL NOT 算內容；`## ` 開頭的行結束段落。提案不存在、沒有 `## Why` 段落或第一個 Why 段落裡沒有內容時 SHALL 不輸出；後面的 `## Why` 段落 SHALL NOT 遞補。
- `startedAt`：字串，開工的日期（`YYYY-MM-DD`）。metadata 沒有開工標記時 SHALL 不輸出。
- `readyAt`：字串，任務全部完成的日期（`YYYY-MM-DD`）。SHALL 只在 metadata 記有就緒時間、而且這個變更目前至少有一個任務並且全部勾完時輸出（見 change-lifecycle「任務全部完成時記下就緒時間」）。

`show --json` SHALL 另外輸出 `createdBy`（字串，metadata 記下的建立者），沒有記錄時 SHALL 不輸出。

既有的 `summary`（含 30 個字元的上限）、`status` 與 `created` SHALL 維持既有的值與格式。`status` 的值 SHALL 只有兩個：至少有一個任務而且全部勾完時是 `done`，其他情況是 `in-progress`；`status` 是 `done` 時 `stage` SHALL 是 `ready`。

不帶 `--detail` 的 `speclink list --json` SHALL 維持既有的位元級輸出，SHALL NOT 含上述任何一個鍵。metadata 損壞的變更在清單裡 SHALL NOT 帶上述任何一個鍵。`speclink status --json`、`speclink plan --json` 與全部的人眼輸出 SHALL NOT 因這條需求而改變。輸出 SHALL 不隨 `locale` 改變。

三種執行方式對相同的資料 SHALL 輸出相同的鍵與值：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的變更清單回應與單一變更回應組出這些鍵，SHALL NOT 對每個變更另外發一次請求。連到不提供這些欄位的舊版 Server 時，清單裡對應的鍵 SHALL 不出現，指令 SHALL NOT 失敗。

#### Scenario: 清單的詳細輸出帶出新的鍵

- **WHEN** 變更 demo 的提案 Why 段落第一行是「這個變更讓 Desktop Client Credentials 登入可以在沒有瀏覽器的機器上完成。」，metadata 有 `started_at: 2026-11-24T17:03:18+08:00` 與 `ready_at: 2026-11-25T09:00:00+08:00`，3 個任務全部勾完，執行 `speclink list --json --detail`
- **THEN** exit code 為 0；demo 這一項的 `stage` 是 `ready`、`whyExcerpt` 是那一行的全文、`startedAt` 是 `2026-11-24`、`readyAt` 是 `2026-11-25`；`summary` 仍是截到 30 個字元的「這個變更讓 Desktop Client Credentia…」，`status` 是 `done`

##### Example: 階段與任務數、開工標記的對應

| 任務（完成／總數） | 有開工標記 | `stage` | `status` |
| --- | --- | --- | --- |
| 0／3 | 否 | `proposed` | `in-progress` |
| 0／3 | 是 | `in-progress` | `in-progress` |
| 1／3 | 否 | `in-progress` | `in-progress` |
| 3／3 | 否 | `ready` | `done` |
| 0／0 | 否 | `proposed` | `in-progress` |

#### Scenario: Why 段落開頭是子標題或註解

- **WHEN** 變更 demo 的提案是 `## Why` 之後依序有空行、`### 背景`、`<!-- 佔位`、`還沒寫 -->`，接著是「這次要補上說明。」，執行 `speclink list --json --detail` 與 `speclink show demo --json`
- **THEN** 兩個輸出的 `whyExcerpt` 都是「這次要補上說明。」

#### Scenario: 第一個 Why 段落沒有內容

- **WHEN** 變更 demo 的提案是 `## Why` 之後只有一個註解行，接著 `## Impact`，再接著第二個 `## Why` 與「後面的說明。」，執行 `speclink list --json --detail`
- **THEN** exit code 為 0；demo 這一項沒有 `whyExcerpt`

#### Scenario: show 帶出同樣的鍵與建立者

- **WHEN** 對同一個變更 demo（metadata 另有 `created_by: Ann <ann@example.com>`）執行 `speclink show demo --json`
- **THEN** exit code 為 0；輸出含 `stage`（`ready`）、`whyExcerpt`、`startedAt`（`2026-11-24`）、`readyAt`（`2026-11-25`）與 `createdBy`（`Ann <ann@example.com>`），值與清單的詳細輸出相同；既有的 `proposal`、`tasks` 等鍵不變

#### Scenario: 依賴成環時清單照樣有階段

- **WHEN** 變更 a 與 b 的 metadata 互相宣告為前置，`speclink plan --json` 因成環而失敗，執行 `speclink list --json --detail`
- **THEN** exit code 為 0；a 與 b 兩項都有 `stage`

#### Scenario: 取消勾選後就緒日不出現

- **WHEN** 變更 demo 的 3 個任務全部勾完、metadata 記有就緒時間，之後取消勾選其中一個任務，執行 `speclink list --json --detail` 與 `speclink show demo --json`
- **THEN** 兩個輸出都沒有 `readyAt`，`stage` 都是 `in-progress`

#### Scenario: 不帶 detail 的清單不變

- **WHEN** 對同一個專案執行不帶 `--detail` 的 `speclink list --json`
- **THEN** 輸出與這條需求加入之前逐位元相同，沒有 `stage`、`whyExcerpt`、`startedAt`、`readyAt`

#### Scenario: 連到舊版 Server 時鍵缺席

- **WHEN** CLI 連到一台回應裡沒有 `stage`、`whyExcerpt` 與 `readyAt` 的舊版 Server，執行 `speclink list --json --detail`
- **THEN** exit code 為 0；每一項都沒有這三個鍵，其他鍵照常輸出

### Requirement: 建立者識別的輸出

metadata 或討論記錄記有建立者的識別時，下列輸出 SHALL 以 `createdById`（字串，原樣）帶出它：`speclink list --json --detail` 的每個變更、`speclink show <change> --json`、`speclink discuss list --json`（含 `--archived`）的每一筆、`speclink discuss show <slug> --json` 的 `info`，以及 `speclink list --archived --json` 的每一筆；`speclink discuss search --json` 的命中項是討論清單的項目加上 `matches`，所以每一筆命中 SHALL 帶出與清單相同的 `createdById`。沒有記錄時 SHALL 不輸出這個鍵，SHALL NOT 輸出 null 或空字串。`createdBy` 既有的輸出 SHALL 不變。三種執行方式（本機專案、連到遠端的 CLI、宿主套件）對相同的資料 SHALL 輸出相同的值；連到不提供這個欄位的舊版 Server 時鍵 SHALL 不出現，指令 SHALL NOT 失敗。

#### Scenario: 五個輸出都帶出識別

- **WHEN** 變更 demo 與討論 two 的記錄各有建立者的識別 `u_1`，另有一個已封存的變更 old 的 metadata 也記有 `u_1`，依序執行 `speclink list --json --detail`、`speclink show demo --json`、`speclink discuss list --json`、`speclink discuss show two --json`、`speclink list --archived --json`
- **THEN** 五個指令的 exit code 都是 0；demo、two 與 old 對應的項目（`discuss show` 是 `info`）都有 `"createdById": "u_1"`

#### Scenario: 沒有識別時不輸出

- **WHEN** 變更 plain 的 metadata 只有 `created_by`、沒有識別，執行 `speclink list --json --detail` 與 `speclink show plain --json`
- **THEN** 兩個輸出都有 `createdBy`、都沒有 `createdById` 這個鍵
