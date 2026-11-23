## Context

領域事件只有一個產生點：`speclink-core` 的 `command/mod.rs` 裡的 `events_of`，它從命令的結果（`CommandOutcome`）推出事件。「一個事件說一件真的發生的寫入」是它既有的原則：沒有改到東西的執行不產生事件。事件的種類是 `DomainEvent`，種類名稱由 `kind()` 給。

宿主套件（`speclink-host`）的 `commit.rs` 有兩個對照：`event_payload`（事件的 camelCase payload）與 `event_resource(kind, payload)`（事件指的資源：類別與識別）。後者是唯一的一張表：示範用 Server 的 SSE 推送用它，`docs/verb-contract` 的 Event stream 表格描述它。推送的規則是一個事件對到一個資源（`invalidate` 的資料只有一個 `scope` 加一個 `resourceId`）。宿主套件另有兩個自己的事件：匯入的 `scope-imported` 與看板順序寫入的 `board-order-updated`。

對照程式碼找到的缺口（都是「提交寫了另一個資源，事件沒有指到它」）：

| 命令 | 順便寫到的資源 | 現在的事件 |
| --- | --- | --- |
| `new change --from-discussion` | 討論記錄（`promoted_to`、狀態） | 只有 `change-created` |
| `archive` | 來源討論被連帶封存（`ArchiveOutcome.archived_discussions`） | 只有 `change-archived` |
| `discuss conclude`（順手封存） | 同一份討論搬進封存（`auto_archived`） | 只有 `discussion-concluded` |
| `discard` | 來源討論被解開（`DiscardOutcome.unlinked_discussions`） | 只有 `change-discarded` |
| `discuss conclude`（重新結論） | 已反映的變更被記上 `restale_from`（`stamp_restale`） | 只有 `discussion-concluded` |
| `discuss link` | 變更的 `from_discussion` | `discussion-linked`，只對到討論 |
| `discuss seal` | 變更的 `restale_from` 被清掉（`clear_restale`） | `discussion-sealed`，只對到討論 |

Speclink Desktop 收到任何推送都整批重讀（`apps/desktop/src-tauri/src/event_manager.rs`），所以受影響的只有依資源分開處理推送的宿主。

## Goals / Non-Goals

**Goals:**

- 每一次提交，凡是被寫到的進行中變更與討論，都有一個指到它的事件。
- 這條規則由測試機械地守住，不靠讀程式碼找。
- 宿主從 SDK 拿得到事件名稱的清單，以及和示範用 Server 相同的「事件對到資源」的答案。

**Non-Goals:**

- 一個事件對到多個資源：推送的資料形狀不變。
- 改變 Speclink Desktop 處理推送的方式。
- 事件帶內容：事件仍然只說哪個資源變了。
- 在 `afterCommit` 直接給寫入的鍵：討論決定事件補齊並有守門測試後不需要第二條路。
- 正式規格（specs）、流程設定、詞彙表、看板順序的逐資源事件：它們維持現在的歸屬（`change-archived` 對到規格；其餘對到 `unknown`）。
- 把領域事件的契約從 experimental 改成穩定。

## Decisions

### 改到幾個資源就送幾個事件，沿用 events_of 單一產生點

仍然只在 `events_of` 產生事件。它需要的事實由命令的結果帶出來，所以要補幾個結果欄位：

- `NewChangeOutcome` 加「這次建立把哪一份討論標成轉出」（沒有 `--from-discussion` 時是 None）。
- `DiscussConcludeOutcome` 加「真的被重寫的變更」清單。既有的 `restale_flagged` 含已經標記、沒有重寫的變更，人眼輸出用它，不能改它的意思。
- `DiscussBindOutcome`（`discuss link` 與 `discuss seal` 共用）加「這次有沒有寫到變更的 metadata」。
- `ArchiveOutcome.archived_discussions`、`DiscardOutcome.unlinked_discussions`、`DiscussConcludeOutcome.auto_archived` 已經有，直接使用。

事件的順序：命令原本的事件在前，新增的事件在後；同一種新增的事件依結果清單的順序。`discuss promote` 維持現在的兩個事件與順序。

替代方案：讓 `event_resource` 回傳多個資源。否決，因為七種情況裡有五種的 payload 沒有另一個資源的名稱，仍然要補事件；推送的資料形狀也要改。

### 沿用既有事件名稱，只在沒有合適名稱時新增兩個

- `discussion-promoted {slug, change}`：`new change --from-discussion`。語意和 `discuss promote` 相同（討論多一個轉出的變更）。
- `discussion-archived {slug}`：`archive` 連帶封存的每一份討論，以及 `discuss conclude` 的順手封存。語意和 `discuss archive` 相同。
- 新增 `discussion-unlinked {slug, change}`：`discard` 解開的每一份討論。沒有既有名稱表示「討論少了一個轉出的變更」。
- 新增 `change-provenance-changed {change}`：變更的來源討論資訊（`from_discussion` 或 `restale_from`）被別的命令改了。三種情況共用一個名稱：重新結論、`discuss link`、`discuss seal`。

`discuss conclude` 的順手封存這一項是提案時加的：討論沒有逐項列出它，但它符合同一條規則的精神（聽 `discussion-archived` 的宿主現在收不到這次封存），而且結果欄位已經有。

### 事件只在真的寫入時送

沿用 `events_of` 的原則。`change-provenance-changed` 只為 metadata 真的被改寫的變更送：重新結論時已經帶著標記的變更不送；`discuss link` 對已經連結的變更不送；`discuss seal` 沒有標記可清時不送。`discussion-unlinked` 只為真的被解開的討論送（`unlinked_discussions` 本來就略過沒有記錄的 slug）。

### 兩個新事件進唯一的對照表

`commit.rs` 的 `event_payload` 加兩個分支（`{ slug, change }` 與 `{ change }`，沒有其他欄位）；`event_resource` 把 `discussion-unlinked` 歸到討論（`slug`）、`change-provenance-changed` 歸到變更（`change`）。示範用 Server 的 `src/api/events.rs` 的對照測試補兩列。`docs/verb-contract` 的 Event stream 表格同批更新。

### 以守門測試釘住「事件涵蓋被寫到的變更與討論」

新增 `crates/host/speclink-host/tests/event_coverage.rs`。做法：

1. 一張情境表，每一種會寫入的命令至少一個情境（含上面七種跨資源的情境）。
2. 每個情境經宿主套件執行，取得提交的寫入鍵與事件。
3. 從鍵推出被寫到的資源：`changes/<name>/…` 是進行中的變更 `<name>`，`discussions/<slug>.md` 是進行中的討論 `<slug>`（鍵的命名是 store-port-contract 的契約）。其他鍵（封存、規格、流程設定、詞彙表、看板順序）不在這條規則內。
4. 斷言每個被寫到的變更與討論，都是某個事件經 `event_resource` 得到的資源；`change-archived` 的識別是變更名稱，算指到那個變更。
5. 完整性：走訪引擎的命令名稱清單，每個名稱不是唯讀命令就必須在情境表裡，否則測試失敗。這樣新增會寫入的命令時一定要補情境。

例外只有一個：範圍的匯入（`scope-imported`）改的是整個範圍，不逐一檢查。

apply 時的結果：情境表涵蓋 36 個會寫入的引擎命令，守門測試沒有找到這七種以外的缺口，例外清單因此只有範圍的匯入（它不是引擎命令，所以寫在測試檔開頭的說明裡，不是一張清單）。`change rank` 在儲存埠上只寫看板順序文件（不是錨點變更的 metadata），不在這條規則內。

守門測試抓不到的一筆：`discuss conclude` 順手封存時多送的 `discussion-archived`。那次提交只寫到討論 two，而 `discussion-concluded` 已經指到它，所以拿掉 `discussion-archived` 時守門測試仍然通過。這筆事件由 core 的測試 `conclude_that_archives_the_record_also_reports_the_archive` 釘住。其餘六種跨資源情況，拿掉新增的事件都會讓守門測試失敗。

替代方案：在產生提交的程式裡放 debug 斷言。否決，因為用 debug 建置的 Rust 宿主遇到沒料到的缺口時會直接 panic。

### 引擎與 SDK 的事件名稱清單

`speclink-core` 的 `command/describe.rs` 加 `EVENT_NAMES`（和 `COMMAND_NAMES` 放在一起），列出 `DomainEvent` 每一種的名稱；以測試釘住它和 `kind()` 一致（新增事件種類而沒有加進清單時測試失敗）。`speclink-host` 的 `kit.rs` 加 `event_names()`：引擎的清單加宿主套件自己的 `scope-imported` 與 `board-order-updated`。Node 把它凍結成 `eventNames`，做法和 `commandNames` 相同。這次之後是 39 個：引擎 37 個（現有 35 個加兩個新的）加宿主套件 2 個。

### SDK 的 eventResource 直接呼叫 Rust 的對照表

Node 新增 `eventResource(name, payload)`，經 napi 呼叫 `speclink_host::commit::event_resource`，回傳 `{ scope, resourceId }`。`scope` 是字串：`change`、`discussion`、`spec`，或對照表沒有的名稱時的 `unknown`（此時 `resourceId` 是空字串）。payload 裡沒有對應的字串鍵時，`resourceId` 是空字串、`scope` 維持該事件的類別（和 Rust 的行為相同）。`name` 不是字串時拋出 `TypeError`；payload 是任何 JSON 值都接受。

## Implementation Contract

**行為**（每一列是一次成功的呼叫與它提交的事件，依序）

| 呼叫 | 事件 |
| --- | --- |
| `new change add-two --from-discussion two` | `change-created {change: add-two}`、`discussion-promoted {slug: two, change: add-two}` |
| `new change solo` | `change-created {change: solo}` |
| `archive add-two`，連帶封存討論 two | `change-archived {change: add-two, datedName}`、`discussion-archived {slug: two}` |
| `archive add-two`，討論還有別的變更在途 | `change-archived` |
| `discard add-two`，解開討論 two | `change-discarded {change: add-two}`、`discussion-unlinked {slug: two, change: add-two}` |
| `discuss conclude two`，順手封存 | `discussion-concluded {slug: two}`、`discussion-archived {slug: two}` |
| `discuss conclude two`（重新結論），add-two 已反映、新記上標記 | `discussion-concluded {slug: two}`、`change-provenance-changed {change: add-two}` |
| 同上再執行一次，add-two 已經帶著標記 | `discussion-concluded {slug: two}` |
| `discuss link two add-three`，第一次 | `discussion-linked {slug: two, change: add-three}`、`change-provenance-changed {change: add-three}` |
| `discuss seal two add-two`，清掉標記 | `discussion-sealed {slug: two, change: add-two}`、`change-provenance-changed {change: add-two}` |
| `discuss seal two add-two`，沒有標記可清 | `discussion-sealed {slug: two, change: add-two}` |
| `discuss promote two --name add-two` | `discussion-promoted`、`change-created`（不變） |

- `eventResource('discussion-unlinked', { slug: 'two', change: 'add-two' })` 回 `{ scope: 'discussion', resourceId: 'two' }`；`eventResource('change-provenance-changed', { change: 'add-two' })` 回 `{ scope: 'change', resourceId: 'add-two' }`；`eventResource('change-archived', { change: 'add-two', datedName: '…' })` 回 `{ scope: 'spec', resourceId: 'add-two' }`；`eventResource('workflow-config-changed', { keys: ['tdd'] })` 與任何不認得的名稱回 `{ scope: 'unknown', resourceId: '' }`。
- `eventNames` 是 39 個不重複的字串，凍結，含兩個新名稱與 `scope-imported`、`board-order-updated`；宿主的 `afterCommit` 與儲存的 `commit` 收到的每個事件名稱都在裡面。

**介面與資料形狀**

- Rust：`DomainEvent` 多 `DiscussionUnlinked { slug, change, occurred_at }` 與 `ChangeProvenanceChanged { change, occurred_at }`；`EVENT_NAMES`；`kit::event_names()`；三個結果型別的新欄位。
- Node：`eventNames: readonly string[]`；`eventResource(name: string, payload: unknown): { scope: 'change' | 'discussion' | 'spec' | 'unknown'; resourceId: string }`。
- 新事件的 payload 只有名稱，沒有內容、沒有值。

**失敗模式**

- 命令失敗時沒有事件（不變）。
- `discard` 與 `archive` 的後段步驟失敗而前段已寫入時，事件照實際寫入的資源送（結果清單本來就只列成功的）。

**驗收條件**

- `cargo test -p speclink-core command::tests` 涵蓋上表每一列。
- `cargo test -p speclink-host --test event_coverage` 通過；把新增的事件拿掉時它失敗（`discuss conclude` 順手封存的 `discussion-archived` 除外，它由 core 的測試釘住，見「以守門測試釘住…」一節）。
- `cargo test -p speclink-host commit` 與 `cargo test -p speclink-server` 的事件對照測試含兩個新名稱。
- Node 的測試確認 `eventNames` 的個數與內容、`eventResource` 的結果、`afterCommit` 收到補齊的事件。
- 人眼輸出與 `--json` 輸出的 golden 不變。

**範圍界線**

- 在範圍內：core 的事件種類、結果欄位與 `events_of`；宿主套件的兩個對照與名稱清單；示範用 Server 的對照測試；Node 的兩個匯出；`docs/verb-contract` 與 `docs/sdk-node` 的中英文說明；守門測試。
- 不在範圍內：`speclink-protocol` 的 `invalidate` 型別、`speclink-remote`、Speclink Desktop、CLI 的輸出、各命令寫入哪些文件。

### Sharp-edges 稽核的結論（apply 時）

對 `eventNames`、`eventResource` 與兩個新事件套用 `speclink instructions --skill audit` 的清單：

- **濫用**：`eventResource` 不讀寫儲存、沒有設定；回傳的 `resourceId` 只是 payload 裡的字串，和示範用 Server 推送的相同，用戶端照舊經有權限的路由重讀。`eventNames` 凍結，改不掉。
- **預設安全**：不認得的名稱回 `unknown`，宿主照文件重讀整個範圍，不會漏。
- **用錯時大聲失敗**：參數對調、把整個事件物件當 `name` 傳，都因 `name` 不是字串而同步拋出 `TypeError`；`scope` 的型別是四個字面值的聯集。
- **不回傳錯誤的識別**：payload 不是物件、缺鍵、鍵值不是字串時，`resourceId` 是空字串，不會把數字或物件轉成字串當識別。
- **已修**：漏傳 payload（`undefined`）原本由 napi 拋出看不懂的錯誤；index.js 改成把它當 `null`，型別也標成可省略。
- **不修**：payload 含 BigInt、函式或 Symbol 這些不是 JSON 的值時，napi 拋出一般的 `Error`。契約只承諾 JSON 值，`afterCommit` 與儲存的 `commit` 給的 payload 一定是 JSON；為它們另寫轉換是為不會發生的輸入加防護。
- **新事件只帶名稱**：`discussion-unlinked` 是 `{ slug, change }`，`change-provenance-changed` 是 `{ change }`，沒有內容與值。

## Risks / Trade-offs

- [嚴格比對事件名稱或個數的宿主測試會壞] → 提案的相容性影響寫明；`eventNames` 讓宿主在升級時由測試發現新名稱；不認得的名稱照文件當成整個範圍變了，行為仍然正確。
- [守門測試找到這七種以外的缺口] → 這正是它的用途。apply 時逐一處理：補事件，或在測試的例外清單寫明理由並記進 design。
- [改結果型別波及人眼輸出] → 只加欄位，不改 `restale_flagged` 等既有欄位的意思；`cargo test -p speclink-core --test it render_golden::` 與 CLI 測試確認輸出不變。
- [示範用 Server 多推幾筆 `invalidate`] → 每筆只是提示重讀；Speclink Desktop 整批重讀，同一次提交的多筆推送由既有的去抖動處理。任務裡以既有的 Desktop 事件測試確認沒有回歸。
- [事件個數 39 算錯] → 個數由測試斷言，並以 `EVENT_NAMES` 對 `kind()` 的一致性測試為準；規格的數字在 apply 時若不符，先修規格再實作。
- [回歸對照] → 既有事件的名稱、payload 與順序不變，既有的事件測試不改就要通過。
- [跨平台] → 沒有路徑處理、檔案系統或 git 的呼叫；守門測試用記憶體儲存。Windows、macOS、Linux 的行為相同。
