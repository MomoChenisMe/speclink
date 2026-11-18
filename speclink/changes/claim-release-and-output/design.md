## Context

認領由引擎的 `Claim` 命令處理（`speclink-core` 的 `command/mod.rs`，`run_claim`）。它用 `edit_meta` 以文字手術改寫變更的 `.meta.yaml`：寫入 `claimed_at`、`claimed_by`，身分帶識別時再寫 `claimed_by_id`，純量都經 `yaml_scalar` 跳脫。本機的 fs store 不支援認領（`supports_ownership` 為否），命令直接拒絕；CLI 另外在進入命令之前就以固定文案擋掉本機模式。

讀取面目前是這樣：

- 線上：`ChangeSummary`（清單）與 `ChangeStatus`（單筆）有 `claimedBy`，由宿主套件的路由從解析後的 metadata 補上。
- CLI：`claim` 只有人眼輸出（`✓ Claimed change: <name>`）。`show --json` 有 `review`、`verify`、`branch`、`landed`、`fields` 這些選填鍵，沒有認領。
- 遠端的 `show` 由 CLI 用既有的讀取端點組出和本機相同的結果（`speclink-cli` 的 `verbs/query.rs`，`remote_show_outcome`），再交給同一支輸出函式。
- 解析後的 metadata（`ChangeMeta`）有 `claimed_by` 與 `claimed_at`，沒有 `claimed_by_id`。`claimed_at` 在解析時會被截成日期（`collapse_time_fields`）；搜尋 `claimed_at` 的結果顯示，除了 `ChangeMeta` 自己的測試，沒有任何輸出點讀它。

事件 `change-claimed` 在宿主套件的事件歸類（`speclink-host` 的 `commit.rs`）歸到該變更。每個引擎命令在 HTTP 入口都要有歸屬（`http/coverage.rs` 的 `wire_coverage`，沒有萬用分支）。

## Goals / Non-Goals

**Goals:**

- 認領可以釋放，三條路徑（連到遠端的 CLI、宿主套件的 `dispatch`、HTTP 入口）結果相同。
- 只走動詞的使用者讀得到認領的人、識別與時間。
- 沒有被認領的變更，既有輸出逐位元不變。

**Non-Goals:**

- 引擎不限定誰可以釋放（來源討論的裁定）。
- 不改 `Claim` 的規則與訊息。
- 不改 `list --json`；清單的詳細輸出是下一個變更 verb-output-for-embedded 的範圍。
- Speclink Desktop 不加釋放的按鈕。
- 不提供「強制接手」（一步釋放並改成自己認領）；先釋放再認領兩步即可。

## Decisions

### D1 釋放是一個獨立的引擎命令 ClaimRelease

`Command` 新增 `ClaimRelease { name }`，穩定名稱 `claim-release`，參數的形狀與 `Claim` 相同。結果型別 `ClaimReleaseOutcome { name, released, previous_holder }`。

實作在 `speclink-core`，和 `run_claim` 並排，步驟的順序與它一致：

1. 變更存在時先過 metadata 的 fail-closed 守門。
2. store 不支援認領時，以和 `Claim` 相同的文案拒絕。
3. 用 `edit_meta` 在一次改寫裡移除 `claimed_by`、`claimed_at`、`claimed_by_id`（存在哪幾個就移除哪幾個）。移除前先讀出 `claimed_by`（經 `yaml_unscalar`）當作 `previous_holder`。三個都不存在時不寫入。
4. 變更不存在時回 not-found。

寫入只有一筆（metadata 這一份文件），所以沒有半套狀態：不是三個欄位都移除，就是什麼都沒變。只有 `released` 為真時才發事件。

替代方案：在 `Claim` 加一個 `release` 布林。否決：掛鉤靠命令名稱決定放不放行，「認領」與「釋放」是兩種權限，共用一個名稱會逼宿主去看參數；HTTP 入口的對照表也是一個命令對一條路由。

### D2 事件 change-claim-released 歸到該變更

`DomainEvent` 新增 `ChangeClaimReleased { change, occurred_at }`，名稱 `change-claim-released`，內容只有變更名稱（和 `change-claimed` 相同）。宿主套件的事件歸類把它和 `change-claimed` 放在同一組：`scope` 是 `change`，`resourceId` 是變更名稱。用戶端收到後重新讀取該變更，不需要認得新的事件名稱。

### D3 動詞的解析與輸出落在共用的 speclink-verbs

`claim` 的參數加兩個旗標：`--release` 與 `--json`。解析出 `Claim` 或 `ClaimRelease`。CLI 與 Node SDK 共用這份解析，所以 `host.dispatch(['claim', 'demo', '--release'])` 不用另外寫。Agent 工具的動詞清單由同一份定義產生。

輸出函式在 `render/lifecycle.rs`：

- 認領的人眼輸出不變。`--json` 是 `{ "change": <名稱>, "claimedBy": <持有人> }`。
- 釋放的人眼輸出：`✓ Released claim on change: <name> (was claimed by <holder>)`，沒有 `previous_holder` 時省略括號那一段；`released` 為假時是 `Change '<name>' is not claimed — nothing to release`（不帶勾號）。`--json` 是 `{ "change", "released" }`，有 `previous_holder` 時加 `previousHolder`。

`--json` 的鍵名比照 `change landed --json`（用 `change` 當變更名稱的鍵）。

### D4 線上的釋放端點是 DELETE /changes/{name}/claim

- `wire_coverage` 新增 `ClaimRelease` → `DELETE /changes/{name}/claim`。
- 路由經既有的命令閘道執行（editor 限定、revision 前置條件、掛鉤、事件都沿用），回應型別 `ClaimReleaseResponse { released: bool, previous_holder: Option<String> }`（camelCase，選填鍵沒有值時不序列化）。
- `speclink-remote` 的用戶端新增 `release_claim(name)`。
- CLI 的遠端分支：`claim --release` 呼叫它，把回應轉成 `ClaimReleaseOutcome` 交給 D3 的輸出函式；本機模式沿用 `claim` 既有的固定拒絕文案。

用 DELETE 而不是 `POST /changes/{name}/unclaim`，因為它移除的正是 POST 建立的那個資源，路徑不必多一個。

### D5 認領記錄的三個欄位一路帶到輸出

- `ChangeMeta` 新增 `claimed_by_id`（選填，缺席讀作沒有）。`claimed_at` 從 `collapse_time_fields` 的清單移除，讓它保留寫入時的完整時間；搜尋確認沒有輸出點依賴它被截成日期，受影響的只有 `ChangeMeta` 自己的一個測試，同批更新。
- `ShowChange` 新增 `claimed: Option<ClaimRecord>`，`ClaimRecord { by, by_id: Option, at: Option }`，只在有 `claimed_by` 時建立。
- `show_json` 比照 `landed` 的寫法：有記錄才加 `claimed` 鍵，內容是 `by`，以及有值才出現的 `byId` 與 `at`。
- 線上：`ChangeSummary` 與 `ChangeStatus` 新增選填的 `claimed_by_id` 與 `claimed_at`（序列化成 `claimedById`、`claimedAt`，沒有值時不序列化、缺席時反序列化成沒有），由補 `claimedBy` 的同一處補上。
- 遠端的 `show`：`remote_show_outcome` 用單筆回應的三個欄位組出同一個 `ClaimRecord`。連到舊版 Server 時只有 `by`。

唯一的輸出實作是 `speclink-verbs` 的 `show_json`；本機、遠端與宿主套件都經過它。

替代方案：在 `show --json` 只加一個字串 `claimedBy`。否決：識別與時間就沒有地方放，而且和 `landed` 這個前例的形狀不一致。

### D6 使用者文件

- 動詞契約文件（中英）：動詞表的 `claim` 一列補上 `--release` 與 `--json`；端點表新增 DELETE 一列；事件對照表的第一列加入 `change-claim-released`；回應範例新增釋放的回應；欄位說明新增 `claimedById` 與 `claimedAt`；`show --json` 的欄位說明新增 `claimed`。
- 專案能力狀態（中英）：對應的能力列提到釋放認領；已知缺口新增一列「Speclink Desktop 沒有釋放認領的按鈕」，替代做法是用 CLI 的 `speclink claim <change> --release`。

## Implementation Contract

**行為**

- `speclink claim <change> --release`：遠端專案上移除認領並回報釋放前的持有人；沒有認領時成功且不寫入；本機專案拒絕。
- `speclink claim <change> --json` 與 `--release --json`：輸出規格 delta 定義的物件。
- `speclink show <change> --json`：被認領的變更多一個 `claimed` 物件。
- 三條路徑（遠端的 CLI、`host.dispatch`、HTTP 入口）對同一個狀態給出相同的結果。

**介面與資料形狀**

- 引擎命令 `claim-release`，參數含變更名稱；事件 `change-claim-released`，內容 `{ change }`。
- CLI：`speclink claim <NAME> [--release] [--json] [--no-color]`。
- `claim --json`：`{ "change": string, "claimedBy": string }`。
- `claim --release --json`：`{ "change": string, "released": boolean, "previousHolder"?: string }`。
- `show --json` 的 `claimed`：`{ "by": string, "byId"?: string, "at"?: string }`。
- 線上：`DELETE /api/speclink/v1/projects/{key}/changes/{name}/claim` → `{ "released": boolean, "previousHolder"?: string }`；`ChangeSummary` 與 `ChangeStatus` 的 `claimedById?: string`、`claimedAt?: string`。

**失敗模式**

- 本機專案：非零結束，stderr 是 `claim` 既有的拒絕文案。
- 變更不存在：not-found（HTTP 404）。
- metadata 壞掉：既有的 fail-closed 拒絕，訊息點名檔案。
- reader：HTTP 403。掛鉤拒絕：`refused` 與宿主給的原因。
- revision 衝突、離線、認證失效：和其他寫入動詞相同。
- 認領被拒絕時 `--json` 不輸出任何 JSON。

**驗收**

- `cargo test -p speclink-core`：釋放的單元測試（三種起始狀態、fs store 拒絕、not-found、壞 metadata、事件）；命令名稱與參數的 golden 已更新。
- `cargo test -p speclink-verbs`：解析與輸出的測試；`show_json` 對沒有認領的變更輸出不變。
- `cargo test -p speclink-protocol`：新欄位的序列化與缺席時的反序列化。
- `cargo test -p speclink-host`：端點的四個情境、兩個讀取回應的新欄位、事件歸類、對照表涵蓋新命令。
- `cargo test -p speclink-cli --test it`：遠端的 `claim --release`、`claim --json`、`show --json` 與 `remote_verb_parity` 的對照。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。

**範圍**

- 範圍內：上面列的命令、事件、輸出、線上型別與端點、遠端 CLI 分支、文件。
- 範圍外：`list --json`、Speclink Desktop 的畫面、認領規則本身、宿主的權限規則、Node SDK 的程式碼（它經共用的解析與輸出自動取得新動詞；只需要重新建置）。

## Risks / Trade-offs

- [回歸對照：`show --json` 與 `claim` 的既有輸出] → 沒有認領的變更逐位元不變，由 `speclink-verbs` 既有的輸出測試與 `speclink-cli` 的 `remote_verb_parity` 固定；命令名稱與參數的 golden 是這次刻意更新的唯一 golden。
- [引擎新增命令會讓沒有萬用分支的對照表編譯不過] → 這是預期的提醒：`wire_coverage` 在這個變更裡補上。變更 hook-read-function 會新增另一張這樣的對照（讀取類命令的判定）；兩個變更有先後時，後合併的那一個要在對方的對照裡補上 `ClaimRelease`（歸為寫入類），所以把這個變更排在 hook-read-function 之後。
- [`claimed_at` 不再截成日期] → 搜尋確認沒有輸出點讀它；`ChangeMeta` 的時間欄位測試同批更新，並新增一個測試固定「`claimed_at` 保留原樣」。
- [以文字手術改寫 YAML] → 釋放只移除整行，不寫入新的純量；移除沿用 `edit_meta` 既有的移除操作，讀出 `previous_holder` 時經 `yaml_unscalar`。
- [舊版 Server 沒有 DELETE 端點] → 新版 CLI 得到 404；訊息沿用既有的錯誤翻譯。套件尚未發版，官方的示範用 server 與 CLI 會一起更新。
- [跨平台] → 沒有檔案路徑或 git 的行為；輸出的勾號與色碼沿用既有的輸出函式，`--no-color` 的行為由既有機制處理。
- [任何 editor 都能釋放別人的認領] → 這是來源討論的裁定；文件寫明，並在範例說明宿主如何用掛鉤限定。
