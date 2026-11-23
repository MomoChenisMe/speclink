## Why

內嵌引擎的宿主把引擎接到自己的桌面畫面之後，有五樣畫面要的資料只有 HTTP 入口讀得到，動詞（`host.dispatch`）讀不到：已封存變更的清單、變更的來源討論與建立的人、進行中關卡的輪數、關卡工單的全文、討論轉出的變更。宿主因此在程序內另外呼叫入口來補資料，同一個畫面要理解兩條讀取路徑，而且依賴入口回應裡沒有寫進文件的欄位。

目標使用者是內嵌 `@speclink/engine` 的宿主開發者；他們的畫面給 PO、PM 與工程師看一個範圍的總覽。連帶受益的是用 CLI 的開發者與技能：同樣的資料在本機專案與遠端連線也讀得到。對應的 workflow 階段是「讀取狀態」：看板總覽、已封存瀏覽、審查與驗證進行中的檢視。

這是討論 embedded-host-needs-7 五個變更裡的第 1 個（G4 的資料那一半）。入口的權限（第 2 個變更）要用這裡新增的 `list --archived`。

## What Changes

- `speclink list` 新增旗標 `--archived`（必須與 `--json` 並用）：列出已封存的變更，最新的在前。每一項有 `datedName`、`date`、`name`、`reviewOutcome`、`verifyOutcome`，以及有值才出現的 `tasksTotal`、`tasksDone`、`createdBy`、`fromDiscussions`、`deltaCapabilities`、`fields`。不讀 stdin；成功時 exit code 0；`--archived` 不帶 `--json`，或與 `--specs`、`--changes`、`--detail` 並用時，以用法錯誤結束（exit code 非 0，訊息在 stderr）。
- `speclink list --json --detail` 的每個變更增列兩個有值才出現的鍵：`createdBy` 與 `fromDiscussions`。
- `speclink list --json --detail` 與 `speclink show <change> --json --detail` 的進行中關卡（`state` 是 `inProgress`）增列 `rounds`：工單目前的輪數。
- `speclink review show <change> --json` 與 `speclink verify show <change> --json` 新增旗標 `--detail`：輸出增列 `content`，內容是工單全文。不帶 `--json` 的 `--detail` 以用法錯誤結束。
- `speclink discuss list --json` 的每一項與 `speclink discuss show <slug> --json` 的討論資訊增列 `promotedTo`：這份討論轉出或併入的變更名稱，沒有時不出現。
- 線上格式（wire）兩處選填的補充，讓連到遠端的 CLI 不必對每個項目另外發請求：已封存清單的項目增列 `deltaCapabilities`；變更清單與單筆變更回應的關卡物件在 `inProgress` 時帶 `rounds`。
- 已封存變更的文件不新增動詞：既有的 `speclink artifact cat <artifact> --change <名稱或帶日期的名稱>` 已經讀得到。

三種形態下的結果：

| 項目 | 本機 repo | 連到遠端的 checkout | 內嵌宿主 |
| --- | --- | --- | --- |
| `list --archived --json` | 可用 | 可用（用既有的已封存清單端點；舊版 Server 沒有 `deltaCapabilities` 時該鍵不出現） | 可用 |
| `list --json --detail` 的 `createdBy`、`fromDiscussions` | 可用 | 可用（變更清單回應已有這兩個欄位） | 可用 |
| 進行中關卡的 `rounds` | 可用 | 可用（舊版 Server 沒有回報時該鍵不出現） | 可用 |
| `review show`／`verify show` 的 `--json --detail` | 可用 | 可用（工單回應已有 `content`；舊版 Server 沒有時該鍵不出現） | 可用 |
| `discuss list`／`discuss show` 的 `promotedTo` | 可用 | 可用（討論資訊回應已有這個欄位） | 可用 |

相容性影響：

- 會改到既有輸出的只有一處：已轉出的討論，在 `discuss list --json` 與 `discuss show --json` 多一個 `promotedTo` 鍵，放在該項的最後。沒有轉出的討論輸出逐位元不變。只認得舊鍵的程式不受影響；golden 同批更新。這推翻了討論 remote-remaining-gaps 的決定（引擎不輸出、由 Server 端組裝）：當時的理由是 CLI 的 JSON 逐位元不變，之後 `fields`、`createdBy`、`kind` 都採用「沒有值就不出現」而成為慣例；而且內嵌宿主直接讀動詞的輸出，沒有 Server 那一層可以補。
- 正式規格 verb-contract 的「工單原文 SHALL NOT 出現在任何 --json 輸出」改成「不帶 `--detail` 時不出現」。不帶 `--detail` 的 `review show --json`、`verify show --json` 逐位元不變，所以審查與驗證技能讀到的內容不變。
- 其餘四項都在新旗標或 `--detail` 後面：不帶這些旗標的輸出逐位元不變，人眼輸出不變。
- Node SDK 的 `dispatch` 對這些動詞一律回傳結構化的值，所以新鍵會直接出現在回傳的物件裡（`promotedTo`），或在 argv 帶了新旗標時出現。
- 線上格式的兩處都是選填：舊版用戶端照常解析新回應，新版用戶端解析舊回應時把它們讀作沒有資料。
- 不涉及設定欄位、生成的技能與 Agent 指令。

掃描過的相關規格（沒有列進下面清單的）：review-station「審查工單的讀取」與 verify-station「驗證工單的讀取」規定兩個 show 動詞既有的 JSON 欄位，這次不改它們，新旗標寫在 verb-contract；server-read-api「archived 瀏覽三端點」的路由不變；node-sdk「dispatch 的輸入輸出契約」的一般規則已涵蓋新旗標；change-lifecycle、free-fields 不受影響。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `verb-contract`: 新增「已封存變更的清單」與「關卡工單的詳細輸出」兩條需求；修改「清單的詳細輸出」（增列建立的人、來源討論、進行中的輪數）、「單筆變更的詳細輸出」（進行中的關卡帶輪數）、「動詞 --json 輸出形狀凍結」（工單原文只在 `--detail` 出現；記下 `promotedTo` 這次刻意的契約更新）
- `discussion-docs`: 新增「討論的 JSON 輸出帶出轉出的變更」
- `client-protocol`: 修改「討論資訊 payload 增選填 promotedTo 欄位」（拿掉「引擎側結構不得因此改動」）與「線上變更回應的關卡欄位」（`inProgress` 帶 `rounds`）；新增「已封存清單的 delta capability 欄位」

## Impact

- Affected specs: `verb-contract`、`discussion-docs`、`client-protocol`
- Affected code:
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/typed.rs
    - crates/engine/speclink-core/src/command/fields.rs
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/lifecycle/listing.rs
    - crates/engine/speclink-core/src/lifecycle/discuss.rs
    - crates/engine/speclink-core/src/quality/station.rs
    - crates/engine/speclink-core/src/quality/station/tests.rs
    - crates/engine/speclink-core/src/store.rs
    - crates/engine/speclink-core/tests/it/command_describe.rs
    - crates/engine/speclink-core/tests/it/render_golden.rs
    - crates/adapters/speclink-verbs/src/families/query.rs
    - crates/adapters/speclink-verbs/src/families/station.rs
    - crates/adapters/speclink-verbs/src/render/query.rs
    - crates/adapters/speclink-verbs/src/render/station.rs
    - crates/adapters/speclink-verbs/src/render/discuss.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-cli/src/verbs/query.rs
    - crates/adapters/speclink-cli/src/verbs/station.rs
    - crates/adapters/speclink-cli/src/verbs/discuss.rs
    - crates/adapters/speclink-cli/tests/it/review_verbs.rs
    - crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs
    - crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/src/port_view.rs
    - crates/host/speclink-host/src/worktree.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/src/http/wire.rs
    - crates/host/speclink-host/src/port_exec.rs
    - crates/host/speclink-host/tests/documents.rs
    - crates/host/speclink-host/tests/station_wire.rs
    - crates/host/speclink-host/tests/port_cost.rs
    - crates/host/speclink-host/tests/port_dual_path.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - crates/adapters/speclink-node/__test__/review-verify-verbs.spec.ts
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
  - New: （無）
  - Removed: （無）
