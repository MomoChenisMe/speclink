## Why

內嵌宿主的管理者按一個按鈕，把 repo 裡的規格資料夾匯入宿主（宿主在程序內呼叫 `engine.exportScope()` 再呼叫 `host.importScope`）。規格資料夾裡有四個舊版工具封存的變更，它們的 `.meta.yaml` 寫的是 `schema: ''`。匯出因此失敗，管理者只看到一句英文：`Schema not found: Schema '' not found in project, user, or built-in locations`。這個失敗沒有原因代碼，訊息也沒有說是哪一個變更；同一個資料夾有 240 個 `.meta.yaml`，開發者要自己找。

目標使用者有兩種：把 `@speclink/engine` 內嵌進自己系統的整合者（他要用自己的語言向管理者說明失敗），以及用 Speclink Desktop「遷移到 Server」的開發者（他看到同一句訊息）。對應的階段是把既有的本機專案搬進遠端專案。

同一輪另外確認了兩件只缺文件的事：宿主以為儲存鍵的命名規則是引擎的內部細節，其實它是正式的契約；宿主想要匯出與匯入的進度，實測引擎的兩步合計只要幾秒。

來源：討論 `embedded-host-needs-6` 的 cut 1。

## What Changes

- **空白的 `schema` 視同沒有設定。** 變更的 metadata 裡 `schema` 的值是空字串或只有空白時，引擎把它當成沒有這個鍵，用預設的 `spec-driven`。這條規則在解析 metadata 的那一步生效，所以每一個讀取端的結果都相同，不只匯出。
- **新的原因代碼 `change_schema_unusable`。** `engine.exportScope()` 因為某個變更的產出流程而失敗時，Error 的 `reason` 是這個代碼，`code` 仍是 `error`。它涵蓋三種失敗：
  - 變更指名的產出流程找不到。
  - 產出流程找到了，但是它的定義無法解析。
  - 產出流程有引擎列不出的檔案樣式。
- **錯誤物件帶位置。** `reason` 是 `change_schema_unusable` 或 `invalid_change_metadata` 時，Error 多兩個字串屬性：
  - `change`：變更在磁碟上的名稱；已封存的變更是帶日期的名稱，例如 `2026-03-03-old-one`。
  - `path`：那個變更的 metadata 檔相對於專案根目錄的路徑，用正斜線。
- **產出流程失敗的訊息以變更名稱開頭。** 形式是 `change '<名稱>': ` 接原本的訊息。Speclink Desktop 的「遷移到 Server」走同一段組裝，所以也顯示新的訊息。
- **SDK 文件（中英兩版）補三段：**
  - 原因代碼的表多一列，並說明 `change` 與 `path`。
  - 鍵的對照表是契約；加一段「範圍的摘要」，說明文件數、最後寫入的人與時間、各階段的變更數各從哪裡取得。
  - 匯出與匯入的耗時量級，以及進度在宿主自己的下載與 `commit` 函式裡。

### 相容性影響

- **`engine.exportScope()` 的失敗**：
  - 產出流程造成的失敗以前沒有 `reason`，現在有。把「沒有 `reason`」當成有效答案的宿主不受影響；它現在可以改顯示自己的說明。
  - 這種失敗的 `message` 多了開頭的變更名稱。比對整句訊息的程式會對不上；SDK 文件一直要求看 `reason`，不要看訊息。
  - `invalid_change_metadata` 的 `code`、`reason` 與 `message` 逐字不變，只多 `change` 與 `path` 兩個屬性。
  - 其他五個代碼的失敗完全不變。
- **`schema` 是空白的變更**（只有舊版工具留下的檔案會這樣；引擎自己不會寫出空白的 `schema`）：
  - 以前：匯出、`show`、`status`、`instructions`、`validate` 這類要解析產出流程的動詞以 `Schema not found: Schema ''` 失敗。
  - 現在：這些動詞照 `spec-driven` 執行。
  - `show --json` 以前對這種變更輸出 `"schema": ""`；現在和沒有這個鍵時相同，`schema` 與 `created` 兩個欄位都是 `null`（這兩個欄位本來就成對出現）。
  - metadata 檔的內容不會被改寫：匯出的包裡仍是原文。
- **Speclink Desktop 的「遷移到 Server」**：含有空白 `schema` 的專案以前會中止，現在可以遷移；產出流程找不到時的錯誤文字多了變更名稱。流程與送出的內容不變。
- 沒有新的 CLI 動詞、旗標或設定欄位；不影響生成的技能。

### 三種形態下的結果

- **本機 repo**：空白 `schema` 的規則對 CLI、Speclink Desktop 與 `createEngine` 的全部讀取生效。`engine.exportScope()` 與 Desktop 的遷移得到新的代碼、位置與訊息。
- **連到遠端的 checkout**：這個 checkout 沒有本機規格檔，`engine.exportScope()` 不適用。空白 `schema` 的規則由 Server 端的引擎在讀取時套用，Server 升級後生效。
- **沒有本機規格檔的內嵌宿主**：`host.exportScope` 只照鍵讀文件、不解析產出流程，所以沒有這三種失敗，行為不變。空白 `schema` 的規則對經儲存接口的讀取（`host.dispatch`、HTTP 入口）同樣生效。

### 既有規格的掃描

`workspace-migration`「本機專案的匯出與遷移產生相同的文件集合」要求兩個入口的中止條件相同並指出同一個檔案，這次的改動在共用的組裝函式裡，兩個入口一起改變，那條需求的字面仍然成立，不用改。`store-port-contract`「鍵的命名規則」已經是契約，這次只讓 SDK 文件說清楚。`workflow-schemas` 規定產出流程的解析順序，沒有規定變更沒有指名時用哪一個，這次不動它。

## Non-Goals (optional)

（design.md 會寫明目標與不做的項目。）

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `node-sdk`: 「匯出與匯入的失敗帶原因代碼」多一個代碼 `change_schema_unusable`，兩個與變更有關的代碼多帶 `change` 與 `path`，產出流程失敗的訊息以變更名稱開頭。
- `change-lifecycle`: 新增一條需求：change metadata 的 `schema` 是空白時視同沒有設定。
- `user-documentation`: 新增一條需求：SDK 文件說明鍵的對照表是契約、範圍的摘要怎麼算，以及匯出與匯入的耗時與進度。

## Impact

- Affected specs: `node-sdk`、`change-lifecycle`、`user-documentation`
- Affected crates: `speclink-core`（metadata 的解析）、`speclink-host`（本機專案的文件組裝與原因代碼）、`speclink-node`（Error 的屬性與型別說明）；`apps/desktop` 不改程式，只是顯示的錯誤文字跟著改變
- Affected code:
  - Modified:
    - crates/engine/speclink-core/src/lifecycle/model.rs
    - crates/host/speclink-host/src/transfer.rs
    - crates/adapters/speclink-cli/tests/it/change_schema.rs
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/export-local.spec.ts
    - crates/adapters/speclink-node/__test__/helpers.ts
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
  - New: (none)
  - Removed: (none)
