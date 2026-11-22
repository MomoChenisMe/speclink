## MODIFIED Requirements

### Requirement: 宿主的匯出與匯入方法

<!-- BEFORE: importScope 只收 identity、scope、bundle，只能寫進空的範圍，解析為 { scopeVersion, documents } -->

Node SDK 的宿主套件 SHALL 提供 `host.exportScope({ identity, scope })`，回傳 Promise，解析為第二版的包（`formatVersion`、`scopeVersion`、`documents`，每項 `key`、`content`、`digest`）；以及 `host.importScope({ identity, scope, bundle, mode, dryRun })`，回傳 Promise，解析為 `{ scopeVersion, documents, added, changed, removed }`：`documents` 是寫入的份數，`added`、`changed`、`removed` 是鍵的陣列，各依鍵的位元組順序排列，`documents` SHALL 等於 `added` 與 `changed` 的長度相加。`mode` 與 `dryRun` 是選填的：`mode` 是 `'create'`（預設，只寫進空的範圍）或 `'replace'`（讓範圍的內容等於包）；`dryRun` 是布林（預設 `false`），為 `true` 時只回報、不提交。兩種模式與試跑的行為依 host-kit「範圍的匯出與匯入」與「匯入的取代模式與試跑」。不帶 `mode` 與 `dryRun` 的呼叫 SHALL 與引入這兩個欄位之前寫入相同的內容，失敗的 `code` 與 `message` SHALL 逐字相同。`bundle` SHALL 可以是第一版或第二版。兩者 SHALL 經過與 host.dispatch 相同的掛鉤，並 SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。失敗時 SHALL 以帶 code 的 Error 拒絕：包不合格、取代模式的包沒有任何文件為 `invalid_argv`；建立模式的目標範圍不是空的、取代模式的範圍在匯入途中被另一方改動、掛鉤拒絕為 `refused`；儲存層的失敗沿用既有的三個 code。createHost SHALL 接受選填的 `limits.importBytes`（正整數）；不是正整數時 SHALL 同步拋出錯誤。host.handleRequest SHALL 回答匯入與匯出的路由，並套用同一個上限；經 host.handleRequest 的匯入 SHALL 一律是建立模式。

importScope SHALL 在呼叫儲存與掛鉤之前檢查新的欄位，不合格時以 `code` 為 `invalid_argv` 的 Error 拒絕：`mode` 有給但不是 `'create'` 或 `'replace'`（message SHALL 指出收到的是什麼：字串顯示原文，過長時可以截斷；其他型別的值顯示它的型別，SHALL NOT 顯示它的內容）；呼叫的物件有 `dryRun` 鍵但它的值不是布林（字串 `'false'`、數字 `0`、值是 `undefined` 的 `dryRun` 鍵都算不合格；要真的執行時不帶這個鍵）；`mode` 是 `'replace'` 而呼叫的物件有 `identity`、`scope`、`bundle`、`mode`、`dryRun` 以外的鍵（message SHALL 指出那個鍵的名稱）。最後一條只管取代模式：建立模式的呼叫帶其他的鍵時 SHALL 維持既有行為。beforeCommand 收到的 `scope-import` 參數 SHALL 是 `{ documents, bytes, mode, dryRun }`。

#### Scenario: 匯出再匯入

- **WHEN** 以 JS 的記憶體儲存建立 host，在範圍 s1 建立一個變更後 exportScope，再把結果 importScope 到範圍 s2
- **THEN** 對 s2 執行 `['list', '--json']` 得到同一個變更；s2 的 commit 被呼叫正好一次

#### Scenario: 匯入非空的範圍

- **WHEN** 對已有變更的範圍呼叫 importScope
- **THEN** Promise 以 code 為 refused 的 Error 拒絕；commit 沒有被呼叫

#### Scenario: 包不合格

- **WHEN** importScope 的 bundle 有一份文件的 digest 與內容不符
- **THEN** Promise 以 code 為 invalid_argv 的 Error 拒絕，message 含該文件的鍵

#### Scenario: 掛鉤拒絕

- **WHEN** beforeCommand 對 scope-import 回傳拒絕，呼叫 importScope
- **THEN** Promise 以 code 為 refused 的 Error 拒絕，message 是宿主的原因

#### Scenario: 上限經 handleRequest 生效

- **WHEN** 以 `limits: { importBytes: 1024 }` 建立 host，經 handleRequest 送出本文為 4096 位元組的匯入請求
- **THEN** 回應的 status 是 413

#### Scenario: 上限不是正整數

- **WHEN** 以 `limits: { importBytes: 0 }` 呼叫 createHost
- **THEN** 同步拋出錯誤

#### Scenario: 取代模式讓範圍等於包

- **WHEN** 範圍 s1 有 `specs/auth/spec.md`（舊內容）與 `discussions/d1.md`，呼叫 `host.importScope({ identity, scope: 's1', bundle, mode: 'replace' })`，bundle 有 `specs/auth/spec.md`（新內容）與 `specs/billing/spec.md`
- **THEN** Promise 解析為 `documents` 是 2、`added` 是 `['specs/billing/spec.md']`、`changed` 是 `['specs/auth/spec.md']`、`removed` 是 `['discussions/d1.md']` 的物件；s1 的 commit 被呼叫正好一次；之後 exportScope 得到的鍵正好是 bundle 的兩個鍵

#### Scenario: 試跑不呼叫 commit

- **WHEN** 對同樣的範圍與 bundle 呼叫 `host.importScope({ identity, scope: 's1', bundle, mode: 'replace', dryRun: true })`
- **THEN** Promise 解析的 `added`、`changed`、`removed` 與真的執行相同；`scopeVersion` 是呼叫前的範圍版本；commit 沒有被呼叫，afterCommit 沒有被呼叫

#### Scenario: 建立模式的回報也有三個清單

- **WHEN** 不帶 `mode`，把含兩份文件的 bundle 匯入空的範圍
- **THEN** Promise 解析的 `documents` 是 2，`added` 是那兩個鍵，`changed` 與 `removed` 都是空陣列

#### Scenario: 新欄位不合格時在呼叫儲存之前拒絕

- **WHEN** 分別以下表的呼叫執行 importScope
- **THEN** 每一個 Promise 都以 code 為 invalid_argv 的 Error 拒絕；儲存的 list、read、commit 與 beforeCommand 都沒有被呼叫

##### Example: 不合格的呼叫

| 呼叫帶的欄位 | message 指出 |
| --- | --- |
| `mode: 'Replace'` | 收到的值 `Replace` |
| `mode: 1` | 收到的型別 `number` |
| `mode: 'replace'`、`dryRun: 'false'` | `dryRun` 不是布林 |
| `mode: 'replace'`、`dryRun: 0` | `dryRun` 不是布林 |
| `mode: 'replace'`、`dryRun: undefined` | `dryRun` 不是布林 |
| `mode: 'replace'`、`dryrun: true` | 未知的鍵 `dryrun` |
| `mode: 'replace'`、`dry_run: true` | 未知的鍵 `dry_run` |

#### Scenario: 掛鉤依模式拒絕

- **WHEN** beforeCommand 在 `params.mode` 是 `replace` 且 `params.dryRun` 是 `false` 時回傳拒絕，其他情況放行；先以取代模式試跑，再以取代模式真的執行
- **THEN** 試跑成功並回報三個清單；真的執行以 code 為 refused 的 Error 拒絕，message 是宿主的原因，commit 沒有被呼叫

### Requirement: 匯出與匯入的失敗帶原因代碼

<!-- BEFORE: 清單有九個代碼；匯入只有建立模式，沒有 empty_bundle 與 scope_changed -->

`engine.exportScope`、`host.exportScope` 與 `host.importScope` 三個呼叫 reject 的 Error，在失敗的原因屬於下表時 SHALL 多一個字串屬性 `reason`，值是表裡的代碼。除了下面寫明的產出流程失敗之外，Error 的 `code` 與 `message` SHALL 與引入本需求之前逐字相同。

| 呼叫 | 失敗的原因 | `code` | `reason` |
| --- | --- | --- | --- |
| `engine.exportScope` | `root` 底下沒有規格資料夾，或那個資料夾沒有規格內容 | `error` | `no_spec_folder` |
| `engine.exportScope` | 規格資料夾的名稱指到專案之外 | `error` | `spec_folder_outside_project` |
| `engine.exportScope` | 規格資料夾、通往它的資料夾、或它底下的檔案或資料夾是符號連結；舊位置的 evidence 資料夾 `.speclink/touched/` 裡有符號連結也算 | `error` | `symlink_in_spec_folder` |
| `engine.exportScope` | 規格資料夾底下有資料夾無法列出內容；舊位置的 evidence 資料夾無法列出內容也算 | `error` | `spec_folder_unreadable` |
| `engine.exportScope` | 進行中或已封存的變更的 metadata 檔無法解析 | `error` | `invalid_change_metadata` |
| `engine.exportScope` | 進行中或已封存的變更指名的產出流程不能用：找不到、定義無法載入，或有引擎列不出的檔案樣式 | `error` | `change_schema_unusable` |
| `host.importScope` | 建立模式的目標範圍已經有文件 | `refused` | `scope_not_empty` |
| `host.importScope` | 取代模式的匯入包沒有任何文件 | `invalid_argv` | `empty_bundle` |
| `host.importScope` | 取代模式在列出範圍之後、提交之前，另一方寫入或刪除了這次會寫入或刪除的鍵 | `refused` | `scope_changed` |
| `host.importScope` | 匯入包的內容超過宿主設定的大小上限 | `invalid_argv` | `bundle_too_large` |
| `host.importScope` | 匯入包裡有一份文件不合規則：鍵不合引擎的命名規則、digest 不是內容的 digest、鍵重複，或看板排序的內容超過它的大小上限 | `invalid_argv` | `invalid_bundle_document` |

這份清單 SHALL 是固定的：既有代碼的字串與意思 SHALL NOT 改變，之後新增的代碼只會加在還沒有 `reason` 的失敗上。失敗的原因不在表裡時，Error SHALL NOT 有 `reason` 屬性；這包含儲存的錯誤、beforeCommand 掛鉤的拒絕、匯出時範圍在每次嘗試都改變、匯入包的格式版本不支援或不是匯入包、以及 importScope 的 `mode` 或 `dryRun` 不合格或取代模式的呼叫帶未知的鍵。host.dispatch 與 engine.dispatch 的失敗 SHALL NOT 有 `reason` 屬性。host.handleRequest 的 HTTP 回應（`GET /export`、`POST /import`）SHALL 維持既有的狀態碼與 body。

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
| host.importScope | `mode` 是 `replace`，匯入包的 `documents` 是空陣列 | `invalid_argv` | `empty_bundle` |
| host.importScope | `mode` 是 `replace`，儲存的 commit 函式對這次要更新的鍵丟出 StoreConflictError | `refused` | `scope_changed` |
| host.importScope | `mode` 是 `Replace` | `invalid_argv` | 沒有 |
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

#### Scenario: 取代模式的空匯入包帶 empty_bundle

- **WHEN** 範圍已經有文件，呼叫 `host.importScope({ identity, scope, bundle: { formatVersion: 2, documents: [] }, mode: 'replace' })`
- **THEN** 呼叫 reject；Error 的 `code` 是 `invalid_argv`、`reason` 是 `empty_bundle`、`message` 以 `import (replace): the bundle holds no documents` 開頭；儲存的 commit 函式沒有被呼叫

#### Scenario: 取代途中範圍被改動帶 scope_changed

- **WHEN** 以取代模式匯入，儲存的 commit 函式因為這次要更新的鍵已經被另一方改動而丟出 StoreConflictError
- **THEN** 呼叫 reject；Error 的 `code` 是 `refused`、`reason` 是 `scope_changed`、`message` 是 `import (replace): the target scope changed during the import — run it again`

#### Scenario: 建立模式的空匯入包仍然不是錯誤

- **WHEN** 不帶 `mode`，把 `documents` 是空陣列的匯入包匯入空的範圍
- **THEN** 呼叫成功，`documents` 是 0；Error 沒有產生，儲存的 commit 函式沒有被呼叫
