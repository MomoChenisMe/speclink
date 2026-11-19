## Why

內嵌引擎的宿主（自己實作儲存、用 `createHost` 內嵌 Engine 的系統）在第二輪修正後還缺三樣東西。第一，宿主在封存前的掛鉤裡要比對「正式分支上的檔案」和「蓋章時的檔案」是否相同，但讀取動詞拿不到蓋章時的檔案指紋。squash、rebase、cherry-pick 會換掉 commit 編號，所以只比對 commit 不可靠。現在宿主只能直接解析自己儲存裡的 metadata 文件，這依賴引擎的內部格式。第二，宿主在後端下載規格資料夾後呼叫 `exportScope`；資料夾放錯位置時，它會得到零份文件的包，匯入看起來成功，其實什麼都沒搬。第三，SDK 文件沒有寫出「在伺服端匯入」的完整做法，也沒有「只讓認領的人與管理者釋放認領」的範例，而這兩件事是每個宿主都要做的。

使用者是寫宿主程式的整合者，以及在宿主裡透過 Agent 工具操作的開發者。使用情境是封存前的掛鉤（archive 階段，宿主的 `beforeCommand`）、把既有 repo 的規格搬進宿主（導入時），以及認領與釋放認領（apply 前後）。

這個變更來自討論 embedded-host-needs-3。相關的正式規格是 verb-contract、review-station、verify-station、node-sdk、user-documentation。

## What Changes

- **`show` 新增旗標 `--detail`**：`speclink show <change> --json --detail` 的 `review` 與 `verify` 物件在 `state` 為 `stamped` 時多帶 `scope`（陣列，每項 `path` 與 `hash`）。形狀與線上格式單筆變更回應（`GET /changes/{name}`）的 `scope` 相同。
  - 沒有 stdin。成功時 exit code 是 0。
  - `--detail` 不帶 `--json` 是用法錯誤：CLI 以非零 exit code 結束，訊息走 stderr，stdout 沒有輸出；經宿主套件執行時以 `invalid_argv` 拒絕。這和 `list --detail` 的規則相同。
  - 對正式規格（`--item-type spec`，或名稱解析成正式規格）帶 `--detail` 時，輸出不變。
  - `state` 為 `inProgress` 的關卡、沒有記錄的關卡，帶不帶 `--detail` 都一樣。
- **Node 的 `exportScope` 拒絕不是專案的 `root`**：規格資料夾名稱的檢查與符號連結的檢查都通過之後，`root` 底下的規格資料夾不是規格資料夾時（裡面沒有 `changes/`、`specs/`、`discussions/`，也沒有 `config.yaml`），`exportScope` 以 Error 拒絕，`code` 是 `error`，message 以 `no spec folder:` 開頭並以雙引號指出資料夾名稱。規格資料夾存在但沒有文件時照常成功，回傳零份文件。Speclink Desktop 的「遷移到 Server」不變。
- **SDK 文件（中英）補三段**：
  - 「在伺服端匯入」：只下載規格資料夾時 `root` 與 `store.specDir` 怎麼給；`limits.importBytes` 也管程序內的 `importScope`，匯出沒有上限；匯入是一次性的搬遷，不是同步；要重新匯入，宿主先清空自己儲存裡這個範圍的全部文件，這會刪掉匯入後在宿主裡做的全部改動。
  - 釋放認領的掛鉤範例：從 `docs/verb-contract` 搬來，放在封存範例之後，加上由宿主自己判斷的管理者例外；寫明 `reason` 會給誰看。`read` 以 `not_found` 失敗時放行，其他讀取錯誤不放行，因為權限掛鉤不能在判斷不了時放行；範例寫成可以單獨貼上的函式。
  - 蓋章指紋：用 `show --json --detail` 讀到的 `scope` 怎麼比對，算法寫精確（能以 UTF-8 讀的檔先把 CRLF 換成 LF 再算 SHA-256，其他檔取原始位元組），蓋章時已刪除的檔不在 `scope` 裡。
- **動詞參考文件（中英）**：補上 `show` 的 `--detail`；釋放認領一節只留「命令名稱是 `claim-release`」那句並連到 SDK 文件。

### 三種形態下的結果

| 項目 | 本機 repo | 連到遠端的 checkout | 沒有本機規格檔的內嵌宿主 |
| --- | --- | --- | --- |
| `show --json --detail` | 可用：引擎從 metadata 讀出 `scope` | 可用：CLI 從既有的 `GET /changes/{name}` 回應取 `scope`，不多發請求；Server 回應沒有關卡欄位時，對應的鍵不出現 | 可用：`host.dispatch` 與掛鉤的 `read` 都接受這個 argv |
| `exportScope` 拒絕非專案的 `root` | 適用：這是 `createEngine` 指向本機資料夾時的方法 | 不適用：遠端範圍的匯出是 `host.exportScope` 與 `GET /export`，讀的是儲存，不是資料夾 | 適用：宿主在後端以 `createEngine` 讀下載下來的資料夾 |
| SDK 文件三段 | 不適用：本機 repo 不經宿主套件 | 不適用：同上 | 適用：文件的讀者就是宿主的整合者 |

CLI 仍然沒有匯入與匯出的動詞（已知缺口不變）。

### 相容性影響

- 不帶 `--detail` 的 `show --json` 逐位元不變；`show` 的人眼輸出不變；`list --json` 與 `list --json --detail` 不變（`list --detail` 仍不帶 `scope`）。
- `exportScope` 對沒有規格資料夾的 `root` 從「回傳零份文件」改成「拒絕」。`exportScope` 在 v0.12.0 之後才加入、還沒發版，沒有已發布的使用者依賴舊行為。宿主要匯出空的資料夾時，先在裡面建立規格資料夾。
- 回歸對照：既有的 `show --json` 對照不需要改；新增的對照只涵蓋帶 `--detail` 的輸出。
- 沒有設定欄位的變動；`store.specDir` 是既有的選項。沒有技能的變動，Agent 工具的說明由 CLI 的用法自動產生，不需要手改。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verb-contract`：新增「單筆變更的詳細輸出」——`show --json --detail` 帶出蓋章時的 `scope`，旗標規則與 `list --detail` 相同，三種執行方式輸出相同。
- `review-station`：「蓋章記錄程式碼的 commit」補上帶 `--detail` 時 `review` 物件多帶 `scope`，不帶時不含。
- `verify-station`：「驗證蓋章記錄程式碼的 commit」補上帶 `--detail` 時 `verify` 物件多帶 `scope`，不帶時不含。
- `node-sdk`：「本機引擎的 exportScope 方法」改成 `root` 底下沒有規格資料夾時拒絕；規格資料夾存在但沒有文件時仍回傳零份文件。
- `user-documentation`：新增「SDK 文件說明伺服端匯入、釋放認領的掛鉤與蓋章指紋」。

## Impact

- Affected specs: verb-contract、review-station、verify-station、node-sdk、user-documentation
- Affected crates: speclink-verbs（`show` 的旗標、用法檢查與輸出）、speclink-core（關卡狀態帶 `scope` 的輸出形狀）、speclink-cli（本機與遠端的 `show` 把旗標交給輸出）、speclink-node（`exportScope` 的檢查與型別說明）；speclink-host 只補一個測試（transfer.rs，釘住 Desktop「遷移到 Server」的空清單），speclink-protocol、speclink-remote、apps/desktop 的程式碼不改
- Affected code:
  - New:
    - crates/adapters/speclink-cli/tests/it/show_detail.rs
  - Modified:
    - crates/adapters/speclink-verbs/src/families/query.rs
    - crates/adapters/speclink-verbs/src/parse.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-verbs/src/render/query.rs
    - crates/engine/speclink-core/src/quality/station.rs
    - crates/engine/speclink-core/src/quality/station/tests.rs
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/adapters/speclink-cli/src/verbs/query.rs
    - crates/adapters/speclink-cli/tests/it/main.rs
    - crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs
    - crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs
    - crates/host/speclink-host/src/transfer.rs
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/export-local.spec.ts
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - Removed: (none)
