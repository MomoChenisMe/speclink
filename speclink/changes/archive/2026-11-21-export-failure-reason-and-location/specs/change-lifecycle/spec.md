## ADDED Requirements

### Requirement: change metadata 的 schema 空白時視同沒有設定

change 的 metadata 檔裡 `schema` 的值是空字串，或去掉前後空白之後是空的，引擎 SHALL 把它讀成沒有這個鍵：這個 change 的產出流程 SHALL 是預設的 `spec-driven`。這條規則 SHALL 對進行中與已封存的 change 都成立，並 SHALL 對每一個讀取 metadata 的入口成立——本機專案的 CLI 動詞、Speclink Desktop、Node SDK 的 dispatch 與 exportScope，以及經儲存接口讀取的宿主與 Server。讀取 SHALL NOT 改寫 metadata 檔：檔案 SHALL 逐位元不變，匯出的包裡 SHALL 是原文。`schema` 指名一個非空白的名稱時 SHALL 維持既有行為：名稱找不到時，需要解析產出流程的動詞照舊失敗。這是刻意變更：以前空白的 `schema` 被讀成名為空字串的產出流程，需要解析產出流程的動詞以 `Schema not found: Schema ''` 失敗。

#### Scenario: 空白的 schema 用預設的產出流程

- **WHEN** 本機專案的變更 `old-one` 的 `.meta.yaml` 內容是 `schema: ''` 與 `created: 2026-03-01` 兩行，執行 `speclink status --change old-one --json`
- **THEN** exit code 是 0；stdout 的 JSON 裡 `schemaName` 是 `spec-driven`；`.meta.yaml` 逐位元不變

#### Scenario: show 的輸出與沒有這個鍵時相同

- **WHEN** 同一個變更執行 `speclink show old-one --json`，再把 `.meta.yaml` 的 `schema` 那一行刪掉後執行同一個指令
- **THEN** 兩次的 exit code 都是 0，兩次的 stdout 逐位元相同；JSON 裡 `schema` 與 `created` 兩個欄位都是 `null`

#### Scenario: 已封存變更的空白 schema 不擋匯出

- **WHEN** 本機專案有一個已封存的變更 `2026-03-03-old-one`，它的 `.meta.yaml` 內容是 `schema: ''` 與 `created: ''` 兩行，它有一份 `proposal.md`，以 Node SDK 呼叫 engine.exportScope
- **THEN** 呼叫成功；包裡有鍵 `archive/changes/2026-03-03-old-one/.meta.yaml`（內容是原文的兩行）與 `archive/changes/2026-03-03-old-one/proposal.md`

#### Scenario: 有名稱但找不到的 schema 照舊失敗

- **WHEN** 變更 `old-one` 的 `.meta.yaml` 內容是 `schema: nope`，而沒有名為 `nope` 的產出流程，執行 `speclink status --change old-one --json`
- **THEN** exit code 不是 0；stderr 含 `Schema not found: Schema 'nope'`

##### Example: schema 的各種寫法

| `.meta.yaml` 裡的 `schema` | 這個 change 的產出流程 |
| --- | --- |
| 沒有這個鍵 | `spec-driven` |
| `schema: ~` | `spec-driven` |
| `schema: ''` | `spec-driven` |
| `schema: '   '` | `spec-driven` |
| `schema: no-spec-change` | `no-spec-change` |
| `schema: nope`（沒有這個產出流程） | 找不到，需要解析產出流程的動詞失敗 |
