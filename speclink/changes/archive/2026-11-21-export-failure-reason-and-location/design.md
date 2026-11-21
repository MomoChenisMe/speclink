## Context

`engine.exportScope()`（Node SDK）與 Speclink Desktop 的「遷移到 Server」都呼叫 `speclink-host` 的同一個函式 `transfer::local_documents`，把本機專案的文件列成一份清單。這個函式對每一個進行中與已封存的變更，先解析 metadata，再依 metadata 指名的產出流程列出那個變更有哪些文件。

現況有三個問題：

- metadata 的 `schema` 是空字串時，`ChangeMeta` 把它讀成「指名一個叫空字串的產出流程」。只有這個鍵不存在（或是 `~`）時才用預設的 `spec-driven`。
- 產出流程找不到、找到但無法解析、或有列不出的檔案樣式時，`local_documents` 只回傳一句文字。它經 `From<String>` 變成沒有原因代碼的 `LocalExportError`，訊息也沒有變更的名稱。
- metadata 無法解析時有原因代碼 `invalid_change_metadata`，檔案路徑只寫在訊息裡，呼叫端拿不到欄位。

實跑的現況（本機 Node 綁定，進行中與已封存的變更結果相同）：

| `.meta.yaml` 的內容 | `engine.exportScope()` |
| --- | --- |
| `created: ''`、`depends_on: ''`、`archived_at: ''`、`schema: ~` | 成功 |
| `schema: ''`、`schema: nope`、`schema: 123` | 失敗，沒有 `reason`，訊息沒有位置 |
| `reviewed_tasks_total: ''`、`reviewed_rounds: ''`、`reviewed_scope: ''`、重複的鍵、`: [` | 失敗，`reason` 是 `invalid_change_metadata`，路徑只在訊息裡 |

限制：node-sdk「匯出與匯入的失敗帶原因代碼」規定既有代碼的字串與意思不能改，新的代碼只能加在還沒有 `reason` 的失敗上。

## Goals / Non-Goals

**Goals:**

- 空白的 `schema` 對每一個讀取端都等同沒有這個鍵。
- 因為某個變更而失敗的匯出，Error 帶固定的原因代碼，以及那個變更的名稱與 metadata 檔的路徑。
- SDK 文件說清楚三件事：新的代碼與欄位、鍵的對照表是契約與範圍摘要的算法、匯出與匯入的耗時。

**Non-Goals:**

- `host.importScope` 的取代模式與試跑：屬於同一份討論的 cut 2 `import-replace-mode`。
- 匯出與匯入的進度回呼：實測 1,636 份文件、18.8 MB 的規格資料夾，匯出約 3.5 到 3.8 秒，匯入到記憶體儲存約 2.3 到 2.9 秒。
- 回傳範圍摘要的新呼叫：宿主用鍵的前綴與自己的 `commit` 函式就能算。
- 符號連結與列不出內容這兩種失敗的 `path` 屬性：它們的訊息已經有路徑，宿主這一輪沒有遇到。
- 改寫任何 metadata 檔：空白的 `schema` 只在讀取時收斂，檔案原文不動。
- 流程設定（`speclink/config.yaml`）的 `schema` 是空白時的行為：見 D1 的排查說明。
- `host.exportScope`、HTTP 的 `GET /export` 與 `POST /import`、其他動詞在產出流程找不到時的訊息：都不變。

## Decisions

### D1 空白的 schema 在解析 metadata 時收斂成沒有設定

落點是 `speclink-core` 的 `ChangeMeta`：`from_text` 與 `from_fields` 收尾時已經有一道收斂（時間欄位），在同一個收尾步驟把「去掉前後空白之後是空的」`schema` 改成沒有值。之後 `schema_name()` 就回傳 `spec-driven`。

這是領域規則，所以放在 `speclink-core`；`speclink-fs`、`speclink-host`、CLI、Node、Desktop 都經 `ChangeMeta` 讀 metadata，不用各自處理。經儲存接口讀取的路徑（`speclink-host` 的 `documents`）也呼叫 `ChangeMeta::from_text`，所以內嵌宿主與示範用 Server 得到同一個結果。

metadata 的原文不改：這次沒有文字手術，匯出的包裡仍是 `schema: ''`，之後任何一端讀它時再收斂一次。向後相容：既有的檔案全部照舊可讀，只有空白的 `schema` 的解讀改變。

**替代方案：只改 `schema_name()`。** 不採用。有三處直接讀 `meta.schema` 是否有值：依建立日排序的清單、`show` 把 `schema` 與 `created` 成對輸出、HTTP 入口的變更狀態。只改 `schema_name()` 時，這三處仍把空字串當成有值，同一個變更在不同的讀取端有兩種解讀。

**替代方案：匯出時才把空白的 `schema` 當成預設。** 不採用。`show`、`status`、`validate` 對同一個變更仍然失敗，使用者匯入之後在宿主裡打開它還是會遇到同一個錯誤。

**同類位置的排查（任務 1.3 的結果）。** 排查用任務 1.3 寫明的兩個 grep，範圍是 `crates`、`apps/desktop/core` 與 `apps/desktop/src-tauri/src`。

繞過 `ChangeMeta::from_text` 與 `ChangeMeta::from_fields` 的路徑：沒有。

- 反序列化只有一處：`crates/engine/speclink-core/src/lifecycle/model.rs` 的 `from_text` 自己。`ChangeMeta` 帶 `#[non_exhaustive]`，`speclink-core` 以外不能用字面值建構。
- 字面值建構只在 `crates/engine/speclink-core/src/quality/station/tests.rs`（測試，沒有填 `schema`）。不用改。
- `ChangeMeta::default()` 出現在 `crates/engine/speclink-fs/src/lib.rs`、`crates/host/speclink-host/src/port_view.rs` 與 `crates/engine/speclink-core/src/teststore.rs`：metadata 無法解析時的替身，`schema` 本來就沒有值。不用改。
- `ChangeMeta::from_fields` 只有 `crates/host/speclink-host/src/drift.rs` 一個呼叫端，只填 `created`。不用改。
- 其他讀取端全部呼叫 `ChangeMeta::from_text`：`speclink-fs`、`speclink-host` 的 `documents.rs`、`port_view.rs`、`port_exec.rs`、`transfer.rs`，CLI 的 `verbs/plan.rs` 與 `verbs/station.rs`，`apps/desktop/core` 的 `cache.rs` 與 `manage.rs`，以及 `speclink-core` 自己的各個動詞。

直接讀 `meta.schema` 是否有值的位置：四處，全部讀 `from_text` 的結果，所以收斂之後自動一致，都不用改。

| 檔案 | 用途 | 處置 |
| --- | --- | --- |
| `crates/engine/speclink-core/src/command/mod.rs` | `show` 把 `schema` 與 `created` 成對輸出 | 不用改；任務 1.1 的 CLI 測試釘住輸出與沒有這個鍵時相同 |
| `crates/engine/speclink-core/src/lifecycle/listing.rs` | 依建立日排序時判斷 metadata 是否完整 | 不用改 |
| `crates/host/speclink-host/src/http/routes.rs` | HTTP 入口的變更狀態只在有 `schema` 時帶 `created` | 不用改 |
| `apps/desktop/core/src/manage.rs` | Speclink Desktop 讀變更的 metadata 給畫面 | 不用改；空白的 `schema` 現在是 `null` |

不經 `ChangeMeta` 逐鍵讀原文的位置：`crates/engine/speclink-core/src/lifecycle/cost.rs` 只讀時間欄位，不讀 `schema`；`crates/engine/speclink-core/src/lifecycle/change_schema.rs` 用文字手術寫入 `schema`，讀「目前的產出流程」時經 `ChangeMeta::from_text`。兩處都不用改。

沒有一併修正的位置，所以任務 1.1 的測試沒有多加斷言。

同類位置有一處：流程設定 `WorkflowConfig::schema_name()`（`crates/engine/speclink-core/src/workspace/config.rs`）對 `speclink/config.yaml` 的 `schema: ''` 有同樣的解讀。這次不修：宿主沒有遇到，`workflow-config` 的寫入不會寫出空白的 `schema`，而且它的修正屬於 workflow-config 的規格。實測的現況：手改成 `schema: ''` 的流程設定讓 `speclink new change` 在新變更的 metadata 寫出沒有值的 `schema:` 一行；這一行以前就讀成沒有設定，那個變更照 `spec-driven` 執行。

### D2 一個新的原因代碼 change_schema_unusable 涵蓋三種失敗

`speclink-host` 的 `TransferReason` 多一個成員，字串是 `change_schema_unusable`。`local_documents` 在進行中與已封存兩個迴圈裡，下面三種失敗都給這個代碼：

1. 變更指名的產出流程在專案、使用者層設定目錄與內建三處都找不到。
2. 產出流程的資料夾找到了，但是它的定義無法載入（解析失敗或繼承成環）。
3. 產出流程有一個不是 `specs/**/*.md` 的萬用字元檔案樣式，引擎列不出它的文件。

三種失敗對呼叫端是同一件事：這個變更指名的產出流程在這台機器上不能用，要請開發者處理那個變更或那個產出流程。細節留在訊息裡。

「同一份文件被列出兩次」維持沒有代碼：它不是某一個變更的 metadata 造成的，實務上也不會發生。

**替代方案：併進 `invalid_change_metadata`。** 不採用。規格規定既有代碼的意思不能改；那個代碼的意思是 metadata 檔無法解析，處理方法是修那個檔案。產出流程找不到時 metadata 檔是好的，常見的處理方法是把自訂的產出流程放到執行匯出的機器上。

**替代方案：三種失敗各一個代碼。** 不採用。後兩種很少見，宿主要多寫兩段說明，而三者的處理方法相同。

### D3 位置由 LocalExportError 帶出，Node 綁定轉成 Error 的 change 與 path

`LocalExportError` 多一個選填的位置：變更的名稱與 metadata 檔的路徑，並提供讀取它的函式。只有 `change_schema_unusable` 與 `invalid_change_metadata` 兩種失敗有位置；其他失敗沒有。

- **`change`**：變更在磁碟上的資料夾名稱。進行中的變更是它的名稱，已封存的變更是帶日期的名稱。
- **`path`**：metadata 檔相對於專案根目錄的路徑，用正斜線，以規格資料夾的名稱開頭。它指名實際存在的那個檔案：有 `.meta.yaml` 時是它；只有舊檔名 `.openspec.yaml` 時是舊檔名；兩個都沒有時是 `.meta.yaml` 的位置。進行中的變更沿用儲存既有的 `change_meta_display`；已封存的變更用同一條規則（現有的已封存分支在兩個檔案都沒有時會給舊檔名，這次一併收斂成同一條規則，由 `local_documents` 裡的一個小函式給兩種失敗共用）。規格資料夾的名稱可以寫成結尾帶斜線（`speclink/`），在 Windows 上也可以帶反斜線（`docs\specs`）；`path` 把這個名稱逐段重組，所以整條路徑只有正斜線，也沒有空的路徑段。訊息裡的路徑維持名稱寫下來的樣子，`invalid_change_metadata` 的訊息才會逐字不變。

`speclink-node` 的 `DispatchError` 多帶這兩個選填的值；`envelope` 只在有值時加上 `change` 與 `path` 兩個鍵；`index.js` 只在信封有這兩個鍵時把它們放到 Error 上。所以沒有位置的失敗，`'change' in error` 與 `'path' in error` 都是 `false`，和 `reason` 的做法一致。

Speclink Desktop 只呼叫這個錯誤的 `to_string()`，不讀位置，所以 `apps/desktop` 不改程式。

**替代方案：把位置寫成一個巢狀物件 `error.location`。** 不採用。`code` 與 `reason` 都是 Error 上的平面屬性，兩個字串維持同一種形狀比較好讀。

**替代方案：`path` 相對於規格資料夾。** 不採用。既有的訊息（metadata、符號連結、列不出的資料夾）全部用相對於專案根目錄的路徑，同一個失敗的訊息與欄位用同一種路徑。

### D4 產出流程失敗的訊息以變更名稱開頭

`local_documents` 把這三種失敗的訊息組成 `change '<名稱>': <原本的訊息>`。原本的訊息逐字保留（例如 `Schema not found: Schema 'nope' not found in project, user, or built-in locations`）。

`speclink-core` 的 `schema::not_found_msg` 不改：CLI 與其他動詞用它，它們的輸出是既有的契約。前綴只在 `local_documents` 加上。

`invalid_change_metadata` 的訊息逐字不變（`invalid <路徑>: <原因>`）：它已經指名檔案。

**替代方案：訊息以 metadata 檔的路徑開頭。** 不採用。討論的結論是以變更名稱開頭；路徑由 `path` 屬性提供。

### D5 SDK 文件的三處補充

中英兩版的 SDK 文件（`docs/sdk-node.md`、`docs/sdk-node.zh-TW.md`）結構要對等，由 `scripts/docs` 的對照測試守門。

- 「匯出本機專案」一節：原因代碼的表從五列變六列；說明 `change` 與 `path` 在哪兩個代碼出現；把「其他失敗都沒有 `reason`，例如變更指定的產出流程找不到」那句的例子換掉。
- 「實作三個儲存函式」一節：把「你的儲存不需要認得文件的種類」那句改寫成「儲存照鍵存取就夠；這張表是契約，既有的鍵不會改，你可以用它在自己的儲存裡統計」。之後加一小節「範圍的摘要」，說明三樣資料的來源：
  - 文件數：用鍵的前綴在自己的儲存裡數（`specs/`、`changes/<名稱>/`、`archive/changes/<日期-名稱>/`、`discussions/`、`archive/discussions/`）。
  - 最後寫入的人與時間：`commit` 收到的 `actor` 與 `at`，在同一次交易裡記下來。
  - 各階段的變更數：每個範圍一次 `host.dispatch` 的 `list --json`，它會經過 `beforeCommand`。
- 「在伺服端匯入」一節：加一點說明耗時的量級與進度的位置：引擎的兩步各只要幾秒；匯入只呼叫宿主的 `list` 一次與 `commit` 一次，寫入的時間在宿主自己的 `commit` 函式裡；下載是宿主自己的工作。

`index.d.ts` 的型別說明跟著改：`exportScope` 的說明列出六個代碼與兩個新屬性。

任務 2.5 的稽核另外加了一句（文件兩版與 `index.d.ts` 都有）：`change` 與 `path` 是磁碟上的名稱，原樣給宿主，顯示它們與訊息時要當成純文字。資料夾名稱可以含 `<`、換行或終端機控制字元，引擎不跳脫也不截斷。

## Implementation Contract

**行為：**

- 變更的 metadata 的 `schema` 是空字串或只有空白時，每一個讀取端的結果與沒有這個鍵時相同。`show --json` 對這種變更輸出的 `schema` 與 `created` 都是 `null`。metadata 檔的位元組不變。
- `engine.exportScope()` 與 Speclink Desktop 的遷移，對含有空白 `schema` 的變更的專案成功，那個變更的文件照 `spec-driven` 列出。
- `engine.exportScope()` 因為產出流程失敗時，Promise 以 Error 拒絕：`code` 是 `error`，`reason` 是 `change_schema_unusable`，`change` 與 `path` 是字串，`message` 以 `change '<名稱>': ` 開頭。
- `engine.exportScope()` 因為 metadata 無法解析而失敗時：`code`、`reason`、`message` 與現在逐字相同，多 `change` 與 `path`。
- 其他失敗（沒有規格資料夾、規格資料夾在專案外、符號連結、列不出的資料夾）：Error 與現在逐字相同，沒有 `change` 與 `path` 屬性。

**資料的形狀：**

```
Error {
  message: string
  code: 'error'
  reason?: 'change_schema_unusable' | 'invalid_change_metadata' | …既有的四個代碼
  change?: string   // 只在上面兩個代碼出現，例如 '2026-03-03-old-one'
  path?: string     // 只在上面兩個代碼出現，例如 'speclink/changes/archive/2026-03-03-old-one/.meta.yaml'
}
```

Rust 端：`TransferReason` 多一個成員，`as_str` 回傳 `change_schema_unusable`；`LocalExportError` 多一個讀取位置的函式 `location()`，回傳選填的 `ChangeLocation`。它有 `change` 與 `path` 兩個具名的字串欄位，不用 tuple，兩個字串就不會被悄悄對調（任務 2.5 的稽核結論）。

**例子：**

| 專案的內容 | 結果 |
| --- | --- |
| `speclink/changes/archive/2026-03-03-old-one/.meta.yaml` 是 `schema: ''`、`created: ''` | 匯出成功 |
| 同一個檔案是 `schema: nope` | `reason` 是 `change_schema_unusable`；`change` 是 `2026-03-03-old-one`；`path` 是 `speclink/changes/archive/2026-03-03-old-one/.meta.yaml`；`message` 是 `change '2026-03-03-old-one': Schema not found: Schema 'nope' not found in project, user, or built-in locations` |
| `speclink/changes/demo/.meta.yaml` 是 `: [` | `reason` 是 `invalid_change_metadata`；`change` 是 `demo`；`path` 是 `speclink/changes/demo/.meta.yaml`；`message` 以 `invalid speclink/changes/demo/.meta.yaml:` 開頭 |
| `speclink/specs/linked` 是符號連結 | `reason` 是 `symlink_in_spec_folder`；沒有 `change` 與 `path` |

**驗收：**

- `speclink-core` 的單元測試涵蓋空白 `schema` 的四種寫法（`''`、只有空白、`~`、沒有鍵）與一個有名稱的對照。
- `speclink-host` 的 `transfer` 測試涵蓋三種產出流程失敗（進行中與已封存各至少一個）、metadata 失敗的位置、沒有位置的失敗、空白 `schema` 的匯出，以及代碼字串的清單。
- `crates/adapters/speclink-node/__test__/export-local.spec.ts` 從 JS 端斷言 `code`、`reason`、`change`、`path` 與 `message`，並斷言沒有位置的失敗沒有這兩個屬性。
- 既有的 golden（`cargo test -p speclink-core --test it render_golden::`）不變。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過（文件的中英對照、連結與詞彙守門）。

**範圍：**

- 在範圍內：`ChangeMeta` 的收斂、`local_documents` 的三種失敗與位置、Node 綁定的 Error 屬性與型別說明、SDK 文件。
- 不在範圍內：`apps/desktop` 的程式、HTTP 入口的回應、`host.exportScope` 與 `host.importScope` 的失敗、CLI 的訊息、流程設定的 `schema`。

## Risks / Trade-offs

- [既有測試釘住「產出流程找不到時沒有代碼」] → `speclink-host` 的 `transfer` 測試有一個以此命名的測試，要改成新的行為；同批更新，不留兩套說法。
- [`show --json` 對空白 `schema` 的變更，`schema` 與 `created` 兩個欄位變成 `null`] → 只有舊版工具留下的檔案會這樣；結果與沒有這個鍵的變更相同。golden 沒有這種夾具，跑 `render_golden::` 確認沒有意外的變動。
- [Windows 的路徑分隔] → `path` 一律用正斜線：規格資料夾名稱之後的路徑段用 `/` 組字串（進行中的變更沿用既有的 `change_meta_file`，已封存的變更用同一種組法）；規格資料夾的名稱本身逐段重組成正斜線，因為它可以帶反斜線或結尾的斜線（品質關卡第 1 輪發現原樣接上會讓 `path` 出現 `\` 或 `//`）。Node 的測試在 Windows CI 也跑，斷言用正斜線的字面值，其中一列用 `path.join` 組出的巢狀名稱。
- [變更名稱來自磁碟上的資料夾名稱，會進到訊息與 `change` 屬性] → 這和既有的 metadata 訊息帶路徑是同一個邊界；任務 2.5 用 sharp-edges 檢查表確認名稱不會影響原因代碼，也不會帶出專案之外的路徑。
- [宿主比對整句訊息] → 產出流程失敗的訊息多了前綴。SDK 文件一直要求看 `reason`；提案的相容性影響已經寫明。
- [已封存變更在兩個 metadata 檔都沒有時，路徑從舊檔名變成 `.meta.yaml`] → 只影響 `path` 屬性與產出流程失敗；metadata 失敗一定有檔案，所以它的訊息不變。
- [metadata 的 `schema` 值含 `..` 路徑段時，匯出會去讀產出流程資料夾之外的 `schema.yaml`（任務 2.5 的稽核實測到；這次改動之前就是這樣）] → 引擎找產出流程時把名稱直接接在資料夾後面，沒有檢查名稱的形狀，每一個動詞都一樣。專案有 `schemas/` 資料夾時，`schema: ../../../outside/evil` 會讀到專案外的檔案；那個檔案無法解析時，訊息會引用它裡面的一個值。這次不修：修正點在 `speclink-core` 找產出流程的函式，影響全部動詞，屬於另一條需求。稽核說明（`transfer.rs` 的 `TransferReason` 上方）記下了這個邊界。

## Migration Plan

沒有資料要搬。宿主升級 `@speclink/engine` 之後就得到新的代碼與屬性；要顯示自己的說明時，在原因代碼的對照裡加一項 `change_schema_unusable`。還沒有加的宿主照舊原樣顯示訊息，訊息現在會指出是哪一個變更。

## Open Questions

沒有。
