## MODIFIED Requirements

### Requirement: 匯出與匯入的失敗帶原因代碼

<!-- BEFORE: 清單有八個代碼；「變更指定的產出流程找不到」明文列為沒有 reason；Error 只有 code、message 與 reason，沒有變更的名稱與路徑 -->

`engine.exportScope`、`host.exportScope` 與 `host.importScope` 三個呼叫 reject 的 Error，在失敗的原因屬於下表時 SHALL 多一個字串屬性 `reason`，值是表裡的代碼。除了下面寫明的產出流程失敗之外，Error 的 `code` 與 `message` SHALL 與引入本需求之前逐字相同。

| 呼叫 | 失敗的原因 | `code` | `reason` |
| --- | --- | --- | --- |
| `engine.exportScope` | `root` 底下沒有規格資料夾，或那個資料夾沒有規格內容 | `error` | `no_spec_folder` |
| `engine.exportScope` | 規格資料夾的名稱指到專案之外 | `error` | `spec_folder_outside_project` |
| `engine.exportScope` | 規格資料夾、通往它的資料夾、或它底下的檔案或資料夾是符號連結；舊位置的 evidence 資料夾 `.speclink/touched/` 裡有符號連結也算 | `error` | `symlink_in_spec_folder` |
| `engine.exportScope` | 規格資料夾底下有資料夾無法列出內容；舊位置的 evidence 資料夾無法列出內容也算 | `error` | `spec_folder_unreadable` |
| `engine.exportScope` | 進行中或已封存的變更的 metadata 檔無法解析 | `error` | `invalid_change_metadata` |
| `engine.exportScope` | 進行中或已封存的變更指名的產出流程不能用：找不到、定義無法載入，或有引擎列不出的檔案樣式 | `error` | `change_schema_unusable` |
| `host.importScope` | 目標範圍已經有文件 | `refused` | `scope_not_empty` |
| `host.importScope` | 匯入包的內容超過宿主設定的大小上限 | `invalid_argv` | `bundle_too_large` |
| `host.importScope` | 匯入包裡有一份文件不合規則：鍵不合引擎的命名規則、digest 不是內容的 digest、鍵重複，或看板排序的內容超過它的大小上限 | `invalid_argv` | `invalid_bundle_document` |

這份清單 SHALL 是固定的：既有代碼的字串與意思 SHALL NOT 改變，之後新增的代碼只會加在還沒有 `reason` 的失敗上。失敗的原因不在表裡時，Error SHALL NOT 有 `reason` 屬性；這包含儲存的錯誤、beforeCommand 掛鉤的拒絕、匯出時範圍在每次嘗試都改變、匯入包的格式版本不支援或不是匯入包。host.dispatch 與 engine.dispatch 的失敗 SHALL NOT 有 `reason` 屬性。host.handleRequest 的 HTTP 回應（`GET /export`、`POST /import`）SHALL 維持既有的狀態碼與 body。

`reason` 是 `invalid_change_metadata` 或 `change_schema_unusable` 時，Error SHALL 另有兩個字串屬性，指出是哪一個變更：`change` 是那個變更在磁碟上的資料夾名稱（已封存的變更是帶日期的名稱）；`path` 是那個變更的 metadata 檔相對於專案根目錄的路徑，使用正斜線、以規格資料夾的名稱開頭。`path` SHALL 指名實際存在的那個檔案：有 `.meta.yaml` 時是它，只有舊檔名 `.openspec.yaml` 時是舊檔名，兩個都沒有時是 `.meta.yaml` 的位置。其他失敗的 Error SHALL NOT 有 `change` 與 `path` 屬性。`change` 與 `path` SHALL NOT 含絕對路徑，SHALL NOT 含任何檔案內容。

`reason` 是 `change_schema_unusable` 時，`message` SHALL 是 `change '<變更的資料夾名稱>': ` 接上引入這個代碼之前的訊息（刻意變更：以前的訊息沒有變更名稱）。`reason` 是 `invalid_change_metadata` 時，`message` SHALL 與以前逐字相同。

#### Scenario: 沒有規格資料夾的匯出帶 no_spec_folder

- **WHEN** `root` 指向一個沒有規格資料夾的資料夾，呼叫 `createEngine({ store: { type: 'fs', root } }).exportScope()`
- **THEN** 呼叫 reject；Error 的 `code` 是 `error`、`reason` 是 `no_spec_folder`、`message` 以 `no spec folder:` 開頭

#### Scenario: 規格資料夾裡有符號連結

- **WHEN** 專案的 `speclink/specs/linked` 是一個符號連結，呼叫 engine.exportScope
- **THEN** 呼叫 reject；Error 的 `code` 是 `error`、`reason` 是 `symlink_in_spec_folder`、`message` 以 `symbolic link in the spec folder:` 開頭

#### Scenario: 匯入到已有文件的範圍

- **WHEN** 目標範圍已經有一份文件，呼叫 host.importScope 帶一份合法的匯入包
- **THEN** 呼叫 reject；Error 的 `code` 是 `refused`、`reason` 是 `scope_not_empty`、`message` 是 `import (create-new): the target scope already holds documents`；儲存的 commit 函式沒有寫入任何文件

#### Scenario: 掛鉤拒絕的匯入沒有 reason

- **WHEN** 宿主的 beforeCommand 對 `scope-import` 回傳拒絕，原因是 `not allowed`，呼叫 host.importScope
- **THEN** 呼叫 reject；Error 的 `code` 是 `refused`、`message` 是 `not allowed`，而且 `'reason' in error` 是 `false`

#### Scenario: 一般動詞的失敗沒有 reason

- **WHEN** 經 host.dispatch 執行 `['show', 'no-such-change', '--json']`
- **THEN** 呼叫 reject；Error 的 `code` 是 `not_found`，而且 `'reason' in error` 是 `false`

#### Scenario: 各種失敗對應的代碼

- **WHEN** 分別造出下表每一種失敗並呼叫對應的方法
- **THEN** Error 的 `code` 與 `reason` 如下表；「沒有」表示 Error 沒有 `reason` 屬性

##### Example: 匯出與匯入的失敗

| 呼叫 | 造出的情況 | `code` | `reason` |
| --- | --- | --- | --- |
| engine.exportScope | `store.specDir` 是 `../outside` | `error` | `spec_folder_outside_project` |
| engine.exportScope | `speclink/changes/demo` 是符號連結 | `error` | `symlink_in_spec_folder` |
| engine.exportScope | `speclink/changes/locked` 的權限是 000（非 Windows） | `error` | `spec_folder_unreadable` |
| engine.exportScope | `speclink/changes/demo/.meta.yaml` 的內容是 `: [` | `error` | `invalid_change_metadata` |
| engine.exportScope | `speclink/changes/demo/.meta.yaml` 的內容是 `schema: nope` | `error` | `change_schema_unusable` |
| engine.exportScope | 專案的 `speclink/schemas/broken/schema.yaml` 的內容是 `: [`，`speclink/changes/demo/.meta.yaml` 的內容是 `schema: broken` | `error` | `change_schema_unusable` |
| host.importScope | 匯入包內容 2,000 位元組，`limits.importBytes` 是 1,024 | `invalid_argv` | `bundle_too_large` |
| host.importScope | 一份文件的 `digest` 不是它內容的 digest | `invalid_argv` | `invalid_bundle_document` |
| host.importScope | 一份文件的鍵是 `notes/a.md` | `invalid_argv` | `invalid_bundle_document` |
| host.importScope | 匯入包的 `formatVersion` 是 99 | `invalid_argv` | 沒有 |
| host.exportScope | 儲存的 list 函式丟出 StoreUnavailableError | `unavailable` | 沒有 |

#### Scenario: 產出流程找不到的匯出指出是哪一個變更

- **WHEN** 專案有一個已封存的變更 `2026-03-03-old-one`，它的 `speclink/changes/archive/2026-03-03-old-one/.meta.yaml` 內容是 `schema: nope`，而專案、使用者層設定目錄與內建都沒有名為 `nope` 的產出流程，呼叫 engine.exportScope
- **THEN** 呼叫 reject，沒有回傳任何包；Error 的 `code` 是 `error`、`reason` 是 `change_schema_unusable`、`change` 是 `2026-03-03-old-one`、`path` 是 `speclink/changes/archive/2026-03-03-old-one/.meta.yaml`、`message` 是 `change '2026-03-03-old-one': Schema not found: Schema 'nope' not found in project, user, or built-in locations`

#### Scenario: metadata 無法解析的匯出帶變更的位置

- **WHEN** 專案的 `speclink/changes/demo/.meta.yaml` 內容是 `: [`，呼叫 engine.exportScope
- **THEN** 呼叫 reject；Error 的 `reason` 是 `invalid_change_metadata`、`change` 是 `demo`、`path` 是 `speclink/changes/demo/.meta.yaml`；`message` 以 `invalid speclink/changes/demo/.meta.yaml:` 開頭，與引入位置屬性之前逐字相同

#### Scenario: 與變更無關的失敗沒有位置

- **WHEN** 專案的 `speclink/specs/linked` 是一個符號連結，呼叫 engine.exportScope
- **THEN** 呼叫 reject；Error 的 `reason` 是 `symlink_in_spec_folder`，而且 `'change' in error` 與 `'path' in error` 都是 `false`

##### Example: 位置屬性在哪些失敗出現

| 造出的情況 | `reason` | `change` | `path` |
| --- | --- | --- | --- |
| 進行中的變更 `demo` 的 `.meta.yaml` 是 `schema: nope` | `change_schema_unusable` | `demo` | `speclink/changes/demo/.meta.yaml` |
| 已封存的變更 `2026-03-03-old-one` 只有舊檔名 `.openspec.yaml`，內容是 `schema: nope` | `change_schema_unusable` | `2026-03-03-old-one` | `speclink/changes/archive/2026-03-03-old-one/.openspec.yaml` |
| 進行中的變更 `demo` 的 `.meta.yaml` 是 `reviewed_rounds: ''` | `invalid_change_metadata` | `demo` | `speclink/changes/demo/.meta.yaml` |
| `root` 底下沒有規格資料夾 | `no_spec_folder` | 沒有 | 沒有 |
| `host.importScope` 匯入到已有文件的範圍 | `scope_not_empty` | 沒有 | 沒有 |
