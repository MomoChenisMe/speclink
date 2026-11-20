## ADDED Requirements

### Requirement: SDK 匯出命令名稱清單

`@speclink/engine` SHALL 匯出 `commandNames`：一個凍結的字串陣列，含引擎每一種命令的名稱，以及宿主套件自己的三個命令名稱 `scope-export`、`scope-import` 與 `put-board-order`（host.handleRequest 寫看板排序時用的名稱），每個名稱正好一次。宿主的 beforeCommand 掛鉤與儲存的 commit 函式收到的 `command` SHALL 一定是這個陣列裡的一個名稱。陣列裡名稱的順序 SHALL NOT 是契約的一部分。`commandNames` SHALL NOT 帶命令的其他性質（例如會不會寫入）。型別定義檔 SHALL 把它宣告成唯讀的字串陣列。

#### Scenario: 匯入得到全部名稱

- **WHEN** 以 `const { commandNames } = require('@speclink/engine')` 取得清單
- **THEN** 它是 57 個字串的陣列，沒有重複；含 `new-change`、`claim-release`、`workflow-config-edit`、`scope-export`、`scope-import` 與 `put-board-order`

#### Scenario: 清單不能被改動

- **WHEN** 檢查 `Object.isFrozen(commandNames)`
- **THEN** 結果是 `true`

#### Scenario: 掛鉤收到的名稱都在清單裡

- **WHEN** 宿主的 beforeCommand 記下每次收到的 `command`，之後依序經 host.dispatch 執行 `new change demo`、`claim demo`、`claim demo --release`，再呼叫 host.exportScope 與 host.importScope
- **THEN** 記下的名稱依序是 `new-change`、`claim`、`claim-release`、`scope-export`、`scope-import`，每一個都在 `commandNames` 裡

#### Scenario: 看板排序的寫入也用清單裡的名稱

- **WHEN** 宿主的 beforeCommand 記下收到的 `command`，之後經 host.handleRequest 送一個合法的 `PUT /board-order`
- **THEN** 記下的名稱是 `put-board-order`，它在 `commandNames` 裡；儲存的 commit 函式收到的 `command` 也是 `put-board-order`

#### Scenario: 型別定義檔宣告了這個匯出

- **WHEN** 一支 TypeScript 檔以 `import { commandNames } from '@speclink/engine'` 匯入，並把 `commandNames` 指派給型別為 `readonly string[]` 的變數，以 `strict`、不開 `skipLibCheck` 編譯
- **THEN** 編譯沒有錯誤

### Requirement: 匯出與匯入的失敗帶原因代碼

`engine.exportScope`、`host.exportScope` 與 `host.importScope` 三個呼叫 reject 的 Error，在失敗的原因屬於下表時 SHALL 多一個字串屬性 `reason`，值是表裡的代碼。Error 的 `code` 與 `message` SHALL 與引入本需求之前逐字相同。

| 呼叫 | 失敗的原因 | `code` | `reason` |
| --- | --- | --- | --- |
| `engine.exportScope` | `root` 底下沒有規格資料夾，或那個資料夾沒有規格內容 | `error` | `no_spec_folder` |
| `engine.exportScope` | 規格資料夾的名稱指到專案之外 | `error` | `spec_folder_outside_project` |
| `engine.exportScope` | 規格資料夾、通往它的資料夾、或它底下的檔案或資料夾是符號連結；舊位置的 evidence 資料夾 `.speclink/touched/` 裡有符號連結也算 | `error` | `symlink_in_spec_folder` |
| `engine.exportScope` | 規格資料夾底下有資料夾無法列出內容；舊位置的 evidence 資料夾無法列出內容也算 | `error` | `spec_folder_unreadable` |
| `engine.exportScope` | 進行中或已封存的變更的 metadata 檔無法解析 | `error` | `invalid_change_metadata` |
| `host.importScope` | 目標範圍已經有文件 | `refused` | `scope_not_empty` |
| `host.importScope` | 匯入包的內容超過宿主設定的大小上限 | `invalid_argv` | `bundle_too_large` |
| `host.importScope` | 匯入包裡有一份文件不合規則：鍵不合引擎的命名規則、digest 不是內容的 digest、鍵重複，或看板排序的內容超過它的大小上限 | `invalid_argv` | `invalid_bundle_document` |

這份清單 SHALL 是固定的：既有代碼的字串與意思 SHALL NOT 改變，之後新增的代碼只會加在還沒有 `reason` 的失敗上。失敗的原因不在表裡時，Error SHALL NOT 有 `reason` 屬性；這包含儲存的錯誤、beforeCommand 掛鉤的拒絕、匯出時範圍在每次嘗試都改變、匯入包的格式版本不支援或不是匯入包、以及變更指定的產出流程找不到。host.dispatch 與 engine.dispatch 的失敗 SHALL NOT 有 `reason` 屬性。host.handleRequest 的 HTTP 回應（`GET /export`、`POST /import`）SHALL 維持既有的狀態碼與 body。

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
| host.importScope | 匯入包內容 2,000 位元組，`limits.importBytes` 是 1,024 | `invalid_argv` | `bundle_too_large` |
| host.importScope | 一份文件的 `digest` 不是它內容的 digest | `invalid_argv` | `invalid_bundle_document` |
| host.importScope | 一份文件的鍵是 `notes/a.md` | `invalid_argv` | `invalid_bundle_document` |
| host.importScope | 匯入包的 `formatVersion` 是 99 | `invalid_argv` | 沒有 |
| host.exportScope | 儲存的 list 函式丟出 StoreUnavailableError | `unavailable` | 沒有 |
