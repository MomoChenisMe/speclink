## Context

內嵌宿主的畫面要五樣資料，現在只有 HTTP 入口讀得到，動詞讀不到（討論 embedded-host-needs-7 的 G4）。對照程式碼的現況：

- 已封存變更的清單：沒有動詞。入口的 `GET /archived` 由 `speclink-host` 的 `documents::archived_list` 直接讀儲存接口組出；Speclink Desktop 的本機專案另有一份（`apps/desktop/core/src/cache.rs`，帶快取與桌面專屬欄位）。讀單一文件已經做得到：`artifact cat` 對已封存的變更有效（verb-contract「artifact cat 讀取已封存變更的文件」）。
- 變更的來源討論與建立的人：在變更的 metadata 裡，`list` 已經讀了那份文件，只是 `ListChangeJson` 沒有這兩個欄位。線上格式的變更清單已有 `createdBy` 與 `fromDiscussions`。
- 進行中關卡的輪數：`station::status` 只收「工單在不在」，所以進行中只輸出 `{ "state": "inProgress" }`。入口的 `documents::change_extras` 其實已經把工單讀進來了，只是沒有數輪數。
- 工單全文：引擎的結果有（`ReviewShowOutcome.content`），`--json` 的輸出函式 `ticket_json` 刻意不放。線上格式的工單回應已有 `content`。
- 討論轉出的變更：`DiscussionInfo.head` 是 `serde(skip)`，所以 `promoted_to` 不進 JSON。入口在 `http/wire.rs` 另外組出 `promotedTo`。

限制：

- `--json` 的既有輸出是回歸保護對象，改動要有 golden 與 CLI 測試確認。
- 同一個對外契約落在本機、遠端連線、內嵌宿主三條路時，只能有一份實作。
- 宿主的儲存可能是資料庫，每一次儲存接口的呼叫都有成本。`crates/host/speclink-host/tests/port_cost.rs` 釘住了兩個數字：已封存清單是一次列出加一次讀取；`list --json --detail` 的呼叫次數等於不帶 `--detail` 的清單（verb-output-for-embedded 的 D3）。

## Goals / Non-Goals

**Goals:**

- 五樣資料都能用動詞讀到，三條路（本機專案、連到遠端的 CLI、宿主套件）對相同的資料輸出相同的鍵與值。
- 不帶新旗標時，除了已轉出討論的 `promotedTo` 之外，所有既有輸出逐位元不變。
- 已封存清單在儲存接口上的成本維持一次列出加一次讀取。

**Non-Goals:**

- 入口的權限（八條讀取先問 `beforeCommand`）：這是下一個變更 `door-reads-ask-before-command`。這次 `GET /archived` 仍然不問掛鉤。
- `list --archived` 的人眼輸出：這個旗標必須與 `--json` 並用。它的用途是給程式讀；人要看已封存的變更有 Speclink Desktop 與 `speclink/changes/archive/` 資料夾。
- 合併 Speclink Desktop 本機專案的已封存清單：它有自己的快取與桌面專屬欄位（Why 摘要、流程成本），維持原樣。
- 新增讀已封存文件的動詞：`artifact cat` 已經涵蓋。
- 入口回應的欄位表：討論已決定延後。
- 不帶 `--detail` 就輸出工單全文：審查與驗證技能讀 `review show --json` 取最後一輪，直接加會每次把全文放進模型的上下文。

## Decisions

### 已封存清單的唯一實作放在 speclink-core，入口改用同一份

在 `speclink-core` 的 `lifecycle/listing.rs` 新增一個函式，從 `Store` 介面組出已封存清單的項目（型別 `ListArchivedJson`）。三個呼叫端共用它：

- 引擎命令 `Command::List` 多一個布林欄位 `archived`；為 true 時 `ListOutcome` 只帶 `archived` 清單，不帶 `changes` 與 `specs`。本機 CLI 與宿主套件的 `dispatch` 都走這條。
- 入口的 `documents::archived_list` 改成：以 `PortView` 包住儲存接口後呼叫同一個函式，再把結果轉成線上格式的 `ArchivedItem`（`specCount` 取 `deltaCapabilities` 的個數）。它不經過宿主套件的執行流程，所以這次不會開始問 `beforeCommand`。範圍版本在讀取之前先取，ETag 不會比內容新。
- 連到遠端的 CLI 呼叫既有的 `GET /archived`，把 `ArchivedItem` 轉成 `ListArchivedJson`，交給同一支輸出函式。

每一項的判定沿用現在入口的規則：任務數來自 `tasks.md`（沒有這份文件就沒有這兩個鍵）；關卡結局用既有的 `ArchivedOutcome::of`（封存的文件含工單是 `notPassed`，否則有章是 `stamped`，否則 `none`）；順序是帶日期的名稱由新到舊。

替代方案：引擎命令在 core 另寫一份、入口維持原樣。否決，因為同一份清單會有兩份判定規則，之後改其中一份就會分岔。

### 以 Store 的 prefetch 提示把讀取合成一次

`Store` 介面新增一個有預設空實作的方法 `prefetch`：呼叫端告訴儲存「接下來會讀這些文件」（進行中變更的文件與已封存變更的文件各一批）。`PortView` 覆寫它，把這些鍵放進既有的 `fetch`，一次讀完；列出結果裡不存在的鍵不讀。其他實作（檔案系統、測試用）用預設的空實作；`speclink-host` 的 worktree 包裝轉給內層。

兩個用途：

- 已封存清單：列出一次之後，對每個已封存變更提示 metadata、`tasks.md` 與兩個關卡的工單，合成一次讀取。成本維持一次列出加一次讀取。
- 進行中關卡的輪數：`list --json --detail` 對每張未結的工單提示一次，全部合成一次讀取。

替代方案一：新增一個回傳「全部已封存變更加指定文件內容」的 Store 方法。否決，因為四個實作都要寫，而且只服務一個呼叫端。替代方案二：不做批次，每份文件各讀一次。否決，因為三百個已封存變更會是上千次儲存呼叫。

這是朝「storage 解耦的規格驅動引擎」靠攏的做法：引擎仍然只認得 `Store` 介面，批次的細節留在儲存接口那一側。

### 清單的詳細輸出增列建立的人與來源討論

`ListChangeJson` 多兩個欄位：`created_by`（輸出鍵 `createdBy`，沒有時省略）與 `from_discussions`（輸出鍵 `fromDiscussions`，空的時候省略）。只有 `detail` 為 true 才填，和既有的五個詳細鍵同一個規則；metadata 損壞的變更不填。值直接取自 `list` 已經讀進來的 metadata，沒有新的讀取。連到遠端的 CLI 用變更清單回應既有的 `createdBy` 與 `fromDiscussions` 組出。

放在 `--detail` 後面而不是直接加：幾乎每個變更都有建立的人，直接加會改到所有既有的 `list --json` 輸出。

### 進行中的關卡帶輪數

`station::status` 的第三個參數從「工單在不在」改成「工單的輪數，沒有工單是 None」。`StationStatus.rounds` 在進行中時填工單的輪數（已蓋章時維持取自 metadata 的 `reviewed_rounds`／`verified_rounds`）。輪數用既有的工單解析函式數，不另寫解析。

輸出的規則：

- `show --json`（不帶 `--detail`）：進行中的關卡維持 `{ "state": "inProgress" }`，不輸出 `rounds`。
- `show --json --detail` 與 `list --json --detail`：進行中的關卡是 `{ "state": "inProgress", "rounds": <整數> }`。
- 線上格式的變更清單與單筆變更回應：關卡物件在 `inProgress` 時帶 `rounds`。入口的 `documents::change_extras` 已經讀了工單，只多一次解析。
- 連到遠端的 CLI：Server 的回應有 `rounds` 就輸出，沒有（舊版 Server）就不輸出這個鍵。
- 工單讀得到但解析失敗時不輸出 `rounds`，清單照常成功。

成本：`list --json --detail` 在有未結工單時，比不帶 `--detail` 多一次儲存讀取（全部未結的工單合在這一次）；沒有未結工單時次數相同。這修訂了 verb-output-for-embedded D3「一次也不多」的數字，`port_cost.rs` 對應的測試跟著改。

### 工單全文放在 review show 與 verify show 的 --detail 後面

`review show` 與 `verify show` 新增旗標 `--detail`（必須與 `--json` 並用）。它只影響輸出，不進引擎命令：`ReviewShowOutcome` 已經帶著 `content`，輸出函式在 `--detail` 時多寫一個 `content` 鍵。這和 `show --detail` 是同一種做法（`--detail` 是輸出層的選項）。

輸出的鍵照既有的字母順序排列，所以 `content` 排在 `change` 之後、`lastRound` 之前；值是工單文件的全文，逐字不改。連到遠端的 CLI 用工單回應既有的 `content`；舊版 Server 沒有這個欄位時不輸出這個鍵。

### 討論資訊直接序列化 promotedTo

`DiscussionInfo.head` 從 `serde(skip)` 改成只序列化 `promoted_to` 一個值：輸出鍵 `promotedTo`，空清單時省略，不反序列化（維持取預設，遠端由線上格式回填）。`hold` 與 `board_rank` 仍然不進 JSON。因為 `head` 是結構的最後一個欄位，新鍵出現在每一項的最後，既有鍵的順序不變。

這樣不需要第二個欄位存同一份資料。`discuss list --json` 的每一項與 `discuss show --json` 的 `info` 用的是同一個結構，兩者一起得到這個鍵。`discuss new --json` 也輸出這個結構，但新建的討論沒有轉出的變更，輸出不變。

連到遠端的 CLI 已經把線上格式的 `promotedTo` 回填到 `head`（`verbs/discuss.rs`），所以輸出自動一致；`discuss show` 的遠端路徑要確認同樣有回填。

### list --archived 的參數規則

在 `speclink-verbs` 的 `ListArgs` 加 `--archived`：`requires = "json"`，並與 `--specs`、`--changes`、`--detail` 互斥。`--sort` 不影響已封存清單的順序。違反時是用法錯誤：CLI 以非零 exit code 結束、訊息在 stderr；經宿主套件執行時是 `invalid_argv`。

## Implementation Contract

**行為（使用者看得到的）**

- `speclink list --archived --json` 輸出 `{ "archived": [ … ] }`，沒有 `changes` 與 `specs` 鍵。每一項：
  - 一定有：`datedName`（字串，例如 `2026-11-22-add-two`）、`date`（字串 `YYYY-MM-DD`）、`name`（字串，去掉日期的名稱）、`reviewOutcome` 與 `verifyOutcome`（字串：`none`、`stamped`、`notPassed`）。
  - 有值才有：`tasksTotal` 與 `tasksDone`（整數；封存的文件裡有 `tasks.md` 時兩個都有）、`createdBy`（字串）、`fromDiscussions`（字串陣列，非空）、`deltaCapabilities`（字串陣列，非空，依名稱排序）、`fields`（物件，非空）。
  - 順序：`datedName` 由大到小。沒有已封存的變更時是 `{ "archived": [] }`。
  - 帶日期的名稱沒有日期前綴的舊資料夾：`date` 是空字串，`name` 等於 `datedName`（沿用入口現在的 `split_dated_name`）。
- `speclink list --json --detail`：每個 metadata 可解析的變更，有記錄時多 `createdBy`、`fromDiscussions`；進行中的關卡多 `rounds`。
- `speclink show <change> --json --detail`：進行中的關卡多 `rounds`。
- `speclink review show <change> --json --detail`、`speclink verify show <change> --json --detail`：既有的 `change`、`rounds`、`lastRound` 之外多 `content`（字串，工單全文）。
- `speclink discuss list --json`、`speclink discuss show <slug> --json`：已轉出的討論多 `promotedTo`（字串陣列，順序是轉出的先後）。

**介面與資料形狀**

- 引擎命令：`Command::List` 多 `archived: bool`；`beforeCommand` 收到的參數因此多 `archived`。命令名稱仍是 `list`，`commandNames` 不變。
- 結果：`ListOutcome` 多 `archived: Option<Vec<ListArchivedJson>>`。
- `Store` 介面：多 `prefetch`，預設空實作。
- 線上格式：`ArchivedItem` 多選填的 `deltaCapabilities`（預設空陣列，空的時候不序列化）；關卡物件的 `rounds` 在 `inProgress` 時也會出現。
- 不帶 `--detail` 的 `review show --json`、`verify show --json`、`show --json`、`list --json` 與任何人眼輸出：逐位元不變。

**失敗模式**

- `list --archived` 不帶 `--json`，或與 `--specs`、`--changes`、`--detail` 並用：用法錯誤（CLI 非零 exit code、訊息在 stderr、stdout 沒有輸出；宿主套件 `invalid_argv`）。
- `review show --detail`、`verify show --detail` 不帶 `--json`：同樣的用法錯誤。
- 沒有工單時 `review show --json --detail` 與不帶 `--detail` 時相同地失敗。
- 連到舊版 Server：缺的資料對應的鍵不出現，指令不失敗。離線或認證失效時與不帶新旗標的同一個動詞相同地失敗。
- 已封存變更的 metadata 無法解析：該項照常列出，只有名稱、日期、任務數與 delta capability；`createdBy`、`fromDiscussions`、`fields` 不出現，兩個結局以「沒有章」判定（沿用入口現在的行為）。

**驗收條件**

- `cargo test -p speclink-core --test it render_golden::` 通過；只有已轉出討論的 golden 有差異，而且差異只是多出 `promotedTo`。
- `crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs` 對五項各有一個對照：本機與遠端的 `--json` 輸出相等。
- `crates/host/speclink-host/tests/port_dual_path.rs`（或同層的測試）確認宿主套件與本機對 `list --archived --json` 的結果深度相等。
- `crates/host/speclink-host/tests/port_cost.rs`：已封存清單經共用函式仍是一次列出加一次讀取；`list --json --detail` 在有未結工單時比不帶時最多多一次讀取、沒有時次數相同。
- `GET /archived` 的回應除了多出 `deltaCapabilities` 之外，和這個變更之前對同一份資料的回應相同。
- Node 的測試經 `host.dispatch` 讀到五項資料。

**範圍界線**

- 在範圍內：上面五項的動詞輸出、兩處線上格式的選填補充、`GET /archived` 改用共用函式、`docs/verb-contract` 與 `docs/sdk-node` 的中英文說明、golden。
- 不在範圍內：入口問不問掛鉤、Speclink Desktop 的畫面與它本機的已封存清單、`archive` 動詞的行為、其他動詞的輸出。

## Risks / Trade-offs

- [`promotedTo` 改到既有的 `discuss list --json` 輸出] → 只有已轉出的討論多一個鍵，放在最後；golden 同批更新；提案的相容性影響寫明。技能讀這份輸出時只取認得的鍵，不受影響。
- [`DiscussionInfo` 的序列化改變波及其他把它整個序列化的地方] → apply 時搜尋全部序列化這個結構的地方（含 `apps/desktop`）。Speclink Desktop 的討論清單自己組 JSON 並明確寫入 `promotedTo`，確認不會出現重複的鍵。
- [入口的 `GET /archived` 改走 `PortView` 後成本變高] → `prefetch` 把讀取合成一次，`port_cost.rs` 的測試釘住一次列出加一次讀取。
- [`list --json --detail` 多一次讀取，推翻先前的設計數字] → 只在有未結工單時發生，而且全部工單合成一次；測試改成釘住這個新的上限。
- [給 `Command::List` 加欄位會漏改手寫這個命令的地方] → 任務列出全部位置：`speclink-host` 的 `http/routes.rs`、`port_exec.rs` 與 `tests/` 底下的檔，`speclink-core` 的 `command/fields.rs`、`command/tests.rs`、`tests/it/command_describe.rs`。
- [給 `Store` 加方法會漏掉測試對照] → `port_view.rs` 測試的 `COVERAGE` 表要補一個檢查函式；預設空實作讓其他實作不必改。
- [回歸對照] → 不帶新旗標的輸出由既有的 golden 與 CLI 測試保護；新旗標各有新的測試。
- [跨平台] → 這次沒有新的路徑處理與 git 呼叫。已封存清單的 `datedName` 來自儲存的鍵（一律正斜線）或資料夾名稱，不含路徑分隔字元。Windows、macOS、Linux 的行為相同。
- [舊版 Server] → 兩處線上格式的補充都是選填並帶預設值；新版用戶端對舊回應把它們讀作沒有資料，測試涵蓋這個情況。
