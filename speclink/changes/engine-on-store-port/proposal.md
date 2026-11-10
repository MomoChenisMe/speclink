## Why

示範用的 server 今天執行每個動詞之前，都把整個範圍的文件載入記憶體：橋接層先匯出整個範圍來得知有哪些文件，再逐份讀回。參考量是 165 個已封存變更、約 31 MB，其中 84% 是封存資料，多數動詞用不到。這個做法也讓引擎只能跑在 TeamStore 上，宿主自己實作的儲存接不進來。

討論 embedded-host-needs 定案：引擎改經三個函式的儲存接口讀寫，用到才讀、一次讀多份、寫入收成一次提交並帶事件。這是十六個變更的第二個，建在變更 store-port-contract 的契約上。它同時要回答兩個還沒驗證的問題：三個函式是否足以供應引擎儲存介面的全部操作；用到才讀之後是否還需要快取。

- **目標使用者**：把 speclink 內嵌進自己後端的整合者（之後能把自己的儲存交給引擎），以及連到示範用 server 的工程師、PO、PM（行為不變，讀取量變少）。
- **使用情境**：Server 端執行任何動詞的路徑。不對應單一 workflow 階段或技能。
- **既有規格的對照**：host-runtime 的「engine 動詞經橋接於 TeamStore 上執行」與「Host 承擔 TeamStore 的 UoW 與 event commit」規定今天的橋接與提交路徑，這個變更修改這兩條並新增三條。teamstore-contract 與 store-abstraction 不變。

## What Changes

- `speclink-host` 新增引擎的儲存視圖：它在一個儲存接口與一個範圍上實作引擎內層的儲存介面。
  - 用到才讀：同一次執行裡，同一個前綴最多列出一次，同一個鍵最多讀取一次。
  - 一次讀多份：需要多份文件時（例如全部進行中變更的 metadata）用一次讀取取得。
  - 寫入先暫存，同一次執行裡讀得到自己剛寫的內容；動詞成功後收成一次提交，連同領域事件交給儲存。
  - 每筆寫入的預期版本取自這次執行第一次看到該文件時的版本；沒看過就寫的文件先讀一次。
- `speclink-host` 新增在儲存接口上執行一個引擎命令的入口，回傳結果、領域事件與新的範圍版本號。查詢與沒有寫入的動詞不提交。
- `speclink-host` 新增 TeamStore 的轉接：把一個 TeamStore 範圍包成儲存接口，讓示範用的 server 繼續用現有的三個 driver。這個轉接是過渡用的，在收尾的變更移除。
- 既有的橋接入口改成「TeamStore 轉接＋儲存接口的執行入口」。整個範圍載入記憶體的橋接視圖移除，不留兩套平行的實作。Server 的 drift 查詢也改用新的視圖。
- 新增一個量測測試：把本專案自己的規格資料夾載入記憶體版的儲存接口，對一組代表性的動詞記下對儲存的呼叫次數與讀取的位元組數，並斷言讀取類動詞不讀封存文件的內容。量測結果與「要不要加快取」的結論寫回設計文件。
- **相容性影響**：
  - CLI 指令、人眼輸出、`--json`、線上協定的形狀都不變。
  - 版本衝突（HTTP 409，reason 為 revision_conflict）的 `message` 文字會變：文件的表示法從 TeamStore 的文件種類改成字串鍵。reason 與狀態碼不變。
  - 示範用 server 的事件記錄，時間欄位改用該次提交的時間（原本是每筆事件自己的時間）。事件名稱、內容與順序不變。
  - 沒有設定欄位、技能或 CLI 旗標的變動。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `host-runtime`: 「engine 動詞經橋接於 TeamStore 上執行」改成經儲存接口的執行入口加 TeamStore 轉接；「Host 承擔 TeamStore 的 UoW 與 event commit」改成由 TeamStore 轉接承擔。新增三條：引擎動詞經儲存接口執行、執行時只讀取用到的文件、TeamStore 轉接成儲存接口。

## Impact

- Affected specs: host-runtime（修改兩條、新增三條）
- Affected crates: `speclink-host`（主要）、`speclink-server`（錯誤對映與呼叫點）。依賴變更 store-port-contract 新增的套件 speclink-store-port。`speclink-core`、`speclink-cli`、`speclink-node`、store crates、桌面 app 不改。
- Affected code:
  - New:
    - crates/host/speclink-host/src/port_view.rs
    - crates/host/speclink-host/src/port_exec.rs
    - crates/host/speclink-host/src/teamstore_port.rs
    - crates/host/speclink-host/tests/port_dual_path.rs
    - crates/host/speclink-host/tests/port_cost.rs
  - Modified:
    - crates/host/speclink-host/Cargo.toml
    - crates/host/speclink-host/src/lib.rs
    - crates/host/speclink-host/src/bridge.rs
    - crates/host/speclink-host/src/commit.rs
    - crates/host/speclink-host/src/drift.rs
    - crates/host/speclink-host/tests/bridge_dual_path.rs
    - crates/host/speclink-server/src/api/error.rs
    - crates/host/speclink-server/src/api/verb.rs
    - Cargo.lock
  - Removed: (none)
- 三種形態下的結果：
  - 本機 repo：不適用。本機專案繼續用檔案實作，引擎的儲存介面本身不改。
  - 連到遠端的 checkout：可用。示範用的 server 改經儲存接口執行動詞，用戶端看到的行為不變（例外見相容性影響）。
  - 沒有本機規格檔的內嵌宿主：這次不做，是已知缺口。用 Rust 內嵌的宿主已經能把自己的儲存接口交給執行入口；Node 的宿主還不行（由變更 host-kit-agent-door 補上）。每次呼叫帶身分的入口也還沒有，這次沿用既有的執行情境。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：docs/product-status.zh-TW.md 與 docs/product-status.md 的 Server 效能說明；docs/server-store-drivers.zh-TW.md 與 docs/server-store-drivers.md 提到「每個動詞載入整個範圍」的段落。
