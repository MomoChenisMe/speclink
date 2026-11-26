## Context

同一個變更的事實，現在由三個出口各自給出，內容不一樣：

- 動詞（`list --json --detail`、`show --json`）沒有階段、開工日、提案 Why 的第一行與建立者的識別；`show --json` 連 `createdBy` 都沒有。
- HTTP 回應（變更清單與單一變更）有 `startedAt`，由 speclink-host 的 `documents::change_extras` 另外讀 metadata 補上。
- Desktop 的本機清單在 desktop-core 的 `list_changes_at` 自己疊上 `startedAt`、`whyExcerpt` 等欄位；`why_excerpt` 這條規則寫在 desktop-core。

內嵌的宿主經呼叫入口只讀動詞，所以三個出口裡它拿到的最少。另外有兩筆資料目前沒有記：建立者的識別（`claim` 記了 `claimed_by_id`，建立時只記顯示名稱，雖然 `ExecutionContext.actor_id` 每次都有值）與任務全部完成的時間。討論沒有修改時間：`Store` 的討論方法只回文字與位置。

限制：

- 不帶 `--detail` 的 `list --json` 承諾逐位元不變（verb-contract「清單的詳細輸出」）。
- metadata 的時間欄位寫完整時戳、輸出只給日期（change-lifecycle「change metadata 時間欄位寫入時戳、對外輸出日期」）。
- `task undone` 是純狀態翻轉，不寫任何其他資料（verb-contract）。
- 三種執行方式對相同的資料輸出相同的鍵與值；連到遠端的 CLI 不得對每個變更另外發一次請求。
- metadata 與討論的 frontmatter 以 `KeyLines` 做文字手術，不重新序列化。

## Goals / Non-Goals

**Goals:**

- 宿主經動詞就拿得到階段、開工日、Why 的第一行、建立者與它的識別、就緒日，以及討論的修改時間。
- 每一條規則只有一份實作，放在 speclink-core；動詞、HTTP 回應與 Desktop 都呼叫它。
- 本機、連到遠端的 CLI、宿主套件三種執行方式輸出相同。
- 既有的輸出與舊資料不受影響。

**Non-Goals:**

- 不做變更的標題欄位。`whyExcerpt` 是說明那一行，不是標題。
- 不改 `summary`、`status`、`created` 的值與格式；不加完整時間的鍵。
- 不做預計完成日。
- 不記開工者的識別（`startedById`）。
- `status --json` 與 `plan --json` 不加鍵。
- `list --archived --json` 不加 `whyExcerpt`。
- `show --json` 不加 `status` 與任務數。
- 不回填舊項目的 `created_by_id` 與 `ready_at`。
- 不改 Desktop 的畫面：就緒日、卡片的最後更新、階段改讀引擎的值、建立者篩選用識別，都屬於下一個變更 `desktop-uses-list-fields`。
- 不新增事件名稱，也不改事件的 payload。
- 匯入的文件保留原本的修改時間，是匯入功能的事，不在這裡處理。

## Decisions

### D1：三條規則是 speclink-core 的純函式，各出口都呼叫它們

- Why 的第一行：把 desktop-core 的 `why_excerpt` 原樣搬到 speclink-core 的 `lifecycle/listing.rs`（輸入提案全文，回第一個 `## Why` 段落裡的第一個非空行；段落缺席或沒有內容時沒有值）。desktop-core 的 `query.rs` 與 `cache.rs` 改成呼叫它，本機專案的畫面不變。
- 階段：`lifecycle/model.rs` 既有的 `stage(store, change)` 拆出不讀儲存的一半：輸入完成數、總數與「有沒有開工標記」，回 `Stage`。`plan` 照舊經 `stage()`，結果不變。
- 就緒日要不要顯示：一個函式，輸入記下的 `ready_at`、完成數與總數；總數大於 0 而且完成數不小於總數時回記下的值，否則沒有值。

替代方案「各出口自己判斷」被否決：現在階段的規則已經有三份，這個變更不再增加一份。

### D2：清單的新鍵只在 `--detail` 出現

`ListChangeJson` 加五個欄位：`whyExcerpt`、`stage`、`startedAt`、`createdById`、`readyAt`。它們和既有的 detail 鍵同一個做法：不帶 `--detail` 時不填、不序列化。`whyExcerpt` 用清單本來就為 `summary` 讀的那一份提案，不多讀。metadata 損壞的變更不帶這五個鍵（和既有的 detail 鍵相同）。`stage` 對每個 metadata 讀得出來的變更都有值；其他四個有值才出現。

### D3：`show --json` 的衍生鍵由輸出層從已有的全文算出

`ShowChange` 加四個記錄值：`created_by`、`created_by_id`、`started_at`、`ready_at`。`stage`、`whyExcerpt` 與「`readyAt` 要不要出現」由 speclink-verbs 的 `show_json` 用 D1 的函式，從 `ShowChange` 已經帶著的 `proposal` 與 `tasks` 全文算出。連到遠端的 CLI 本來就讀這兩份全文，所以本機與遠端走同一段程式，不需要 Server 另外回這三個衍生值。人眼的 `show` 不變。

### D4：建立者的識別在建立時寫入，寫法比照認領

`newcmd::new_change` 與討論的建立函式各多收一個 `actor_id`；`command/mod.rs` 的 `NewChange`、`DiscussNew` 與 `DiscussPromote` 從 `ExecutionContext.actor_id` 傳進去。有值時在 `created_by` 那一行之後寫 `created_by_id`；值的跳脫與 `claimed_by_id` 用同一個函式（`util.rs` 的 YAML 純量跳脫），不手拼。`ChangeMeta` 與 `DiscussionHead` 各加一個 `created_by_id` 欄位，缺席讀作沒有。`created_by` 既有的寫法不動。

宿主套件的 `engine_context` 已經填 `actor_id`；本機 CLI 的 context 沒有，所以本機不寫。不需要改任何命令的參數。

### D5：就緒時間由一個 core 函式寫入，兩個寫入點各呼叫一次

speclink-core 的 `lifecycle/tasks.rs` 加一個函式：讀這個變更目前的任務數，總數大於 0 而且全部勾完時，用 `model::edit_meta` 把 `ready_at` 設成 `util::now_rfc3339()`（已經有就取代，不會留下重複的鍵）；否則什麼都不寫。

- `tasks::complete` 在既有的三步之後呼叫它。順序是：寫回 tasks.md → 記 touched 與 evidence → 蓋開工章 → 寫就緒時間。任一步失敗就停在那裡：任務已經勾了，後面的記錄缺席。這和今天開工章的失敗語意相同，可以接受，因為缺席的 `ready_at` 只讓 `readyAt` 不出現。已經勾過的任務（`already`）不呼叫。
- desktop-core 的 `set_all_tasks_at` 在 `done` 為 true 而且有翻轉時，於既有的側效之後呼叫同一個函式。
- `tasks::uncomplete` 不動 `ready_at`（純狀態翻轉）。過期的值由 D1 的顯示規則擋下：任務不是全部勾完時 `readyAt` 不出現；下次全部勾完時被覆寫。
- `ChangeMeta` 加 `ready_at`，放進收成日期的那一組欄位；輸出只給日期。
- 手動改 tasks.md、整份寫回已勾的任務清單、從別處匯入，都不會有 `ready_at`。

替代方案「在 `task undone` 清掉」違反取消勾選是純狀態翻轉，討論已否決。

### D6：討論的修改時間放在 `DiscussionDoc`，不加 `Store` 方法

討論結論寫的是「`Store` 加一個方法、預設答不出來」。提案時改成在 `store.rs` 的 `DiscussionDoc` 加一個欄位 `updated_at_secs`（0 代表答不出來）：三個回討論文件的儲存實作在組出文件的同一個地方填值——speclink-fs 取那個檔案的修改時間（整秒），speclink-host 的 `port_view` 取那個鍵的修改時間（毫秒除以 1000），core 的測試儲存預設 0、可以由測試設定。

理由：討論的清單與單項讀取本來就回 `DiscussionDoc`，值跟著文件走，不用依 slug 再查一次；已封存的討論同一個 slug 可以有好幾份，依 slug 查會有歧義；也不必替另外兩個只有 `unreachable!()` 的 `Store` 替身補方法。對外的行為和結論相同。

`DiscussionInfo` 加 `modifiedAt` 與 `createdById` 兩個選填欄位。秒數轉成 RFC 3339 的 UTC 時間用 `listing.rs` 既有的 `modified_at` 函式（0 與超出範圍都沒有值），和變更清單同一個格式。

### D7：HTTP 回應帶新欄位，來源仍是同一批讀取

speclink-protocol 的型別加選填欄位（camelCase、`serde(default)`、沒有值就不序列化）：

| 型別 | 新欄位 |
| --- | --- |
| `ChangeSummary` | `whyExcerpt`、`stage`、`createdById`、`readyAt` |
| `ChangeStatus` | `createdById`、`readyAt` |
| `DiscussionInfo` | `modifiedAt`、`createdById` |
| `ArchivedItem` | `createdById` |

speclink-host 的做法：

- 變更清單：`documents::change_extras` 在既有的那一次批次讀取裡多讀每個變更的 proposal.md，用 D1 的函式算出 `why_excerpt`。`wire::change_summary` 用清單項的任務數與 metadata 的開工標記呼叫 D1 的階段函式，用 D1 的顯示規則決定 `readyAt`，`createdById` 取自 metadata。清單路由不改成跑 `--detail` 的清單：那會讓兩個關卡的工單被讀兩次。
- 單一變更：路由已經讀 metadata，直接帶出 `createdById`；`readyAt` 要先知道任務是不是全部勾完，所以和 metadata 同一批多讀 tasks.md，再用 D1 的顯示規則。
- 討論與已封存清單：由引擎的結果逐欄帶出。

speclink-cli 把回應轉回引擎的清單項與 `ShowChange` 時逐欄對應新欄位；`startedAt` 用回應本來就有的 `startedAt`。舊 Server 不送時欄位是預設值，對應的鍵不出現。Desktop 的遠端清單把 `ChangeSummary` 原樣攤平給前端，所以新欄位（含 `whyExcerpt`）不用改 Tauri 殼就到得了前端。

### D8：文件

docs/verb-contract 的中英兩份：`list --json --detail` 的表、`show --json` 的欄位、`discuss list` 與 `discuss show` 的欄位、`list --archived --json` 的欄位、`GET /changes` 與 `GET /changes/{name}` 的欄位表、討論與已封存的回應，逐處補上新鍵；並在 `list --json` 的段落寫明 `status` 只有 `done` 與 `in-progress` 兩個值、`done` 等於 `stage` 的 `ready`。docs/product-status 的中英兩份在 Node SDK 那一列補一句動詞現在帶出的資料。docs/sdk-node 不用改：`@speclink/engine` 的對外介面沒有變。

## Implementation Contract

**行為**

- `speclink list --json --detail` 的一筆（鍵照字母排序）：

  ```json
  {
    "completedTasks": 3,
    "created": "2026-11-24",
    "createdBy": "Ann <ann@example.com>",
    "createdById": "u_1",
    "modifiedAt": "2026-11-25T14:44:50Z",
    "name": "add-demo",
    "readyAt": "2026-11-25",
    "stage": "ready",
    "startedAt": "2026-11-24",
    "status": "done",
    "summary": "這個變更讓 Desktop Client Credentia…",
    "totalTasks": 3,
    "whyExcerpt": "這個變更讓 Desktop Client Credentials 登入可以在沒有瀏覽器的機器上完成。"
  }
  ```

- 各鍵什麼時候出現：

  | 鍵 | 出現的條件 | 值 |
  | --- | --- | --- |
  | `stage` | metadata 讀得出來 | `proposed`、`in-progress`、`ready`，與 `plan --json` 同一個變更的 `stage` 相同 |
  | `whyExcerpt` | 提案有 `## Why` 段落而且段落裡有非空行 | 那一行的原文（去掉前後空白），不截斷 |
  | `startedAt` | metadata 有 `started_at` | 日期 |
  | `createdById` | metadata 或討論的 frontmatter 有 `created_by_id` | 原樣 |
  | `readyAt` | metadata 有 `ready_at`，而且目前至少一個任務、全部勾完 | 日期 |
  | `createdBy`（`show --json`） | metadata 有 `created_by` | 原樣 |
  | `modifiedAt`（討論） | 儲存答得出這份文件的修改時間 | RFC 3339 的 UTC 時間，整秒 |

- `speclink show add-demo --json` 多出同樣的 `stage`、`whyExcerpt`、`startedAt`、`createdBy`、`createdById`、`readyAt`，規則相同。
- `speclink discuss list --json` 的一筆多出 `"createdById": "u_1"` 與 `"modifiedAt": "2026-11-25T14:50:00Z"`；`discuss show --json` 的 `info` 相同；`list --archived --json` 的一筆多出 `createdById`。
- 經宿主套件（呼叫者的識別是 `u_1`）執行 `new change demo` 之後，`changes/demo/.meta.yaml` 有一行 `created_by_id: u_1`；本機 CLI 建立的變更沒有這一行。`discuss new` 與 `discuss promote` 同理。
- 變更有 3 個任務、勾了 2 個：`task done` 勾第 3 個之後 metadata 有 `ready_at`（RFC 3339、帶時區偏移量）；接著 `task undone` 其中一個，metadata 的 `ready_at` 還在，但 `list --json --detail` 與 `show --json` 沒有 `readyAt`、`stage` 是 `in-progress`；再 `task done` 回來，`ready_at` 被新的時間取代，仍然只有一行。

**介面與資料形狀**

- 不新增 CLI 指令、旗標、HTTP 路由與事件名稱；`Command` 的參數不變。
- metadata 的新鍵：`created_by_id`（字串）、`ready_at`（RFC 3339 時戳）。討論 frontmatter 的新鍵：`created_by_id`。
- `--json` 的新鍵與 HTTP 回應的新欄位一律 camelCase，沒有值時不出現，不用 null。
- 人眼輸出全部不變。

**失敗模式**

- metadata 損壞的變更：清單照舊列出並帶 `metaError`，不帶任何新鍵；`task done` 照舊在寫入前拒絕。
- 儲存答不出討論的修改時間：`modifiedAt` 不出現，指令不失敗。
- 舊 Server：新欄位缺席，對應的鍵不出現，指令不失敗。
- 寫 `ready_at` 失敗：任務已經勾完，指令回報錯誤；`ready_at` 缺席只讓 `readyAt` 不出現。

**驗收**

- speclink-core：`listing`、`model`、`tasks`、`discuss` 與 `command` 的單元測試涵蓋上面每一列；`cargo test -p speclink-core --test it render_golden::` 不需要更新 golden（技能檔沒有變）。
- speclink-verbs 的 `render/query.rs` 測試：`show --json` 與清單的新鍵；不帶 `--detail` 的清單輸出逐位元不變。
- speclink-host：HTTP 回應的新欄位，以及「回應的每個欄位都有來源」的既有檢查通過。
- speclink-cli：本機與遠端的 `list --json --detail`、`show --json`、`discuss list --json` 輸出相同的鍵。
- desktop-core：`why_excerpt` 改呼叫 core 之後既有測試不變；「全部標完成」寫 `ready_at`。
- scripts 的中英對照、連結與詞彙守門通過。

**範圍**

- 範圍內：D1–D8。
- 範圍外：Non-Goals 列的每一項。

## Risks / Trade-offs

- [回歸對照：`show --json` 與 `discuss list --json` 是既有輸出] → 兩組輸出刻意多鍵，speclink-verbs 與 speclink-cli 釘住它們的測試同批更新；不帶 `--detail` 的 `list --json` 另有一條測試斷言逐位元不變。render 的 golden 只涵蓋技能檔，預期不變。
- [本機與遠端的輸出比對會因修改時間不同而紅] → 討論的 `modifiedAt` 在本機是檔案時間、在 Server 是鍵的修改時間，兩邊的值不同。apply 時先看既有的 parity 測試怎麼處理變更的 `modifiedAt`，討論比照同一種處理。
- [跨平台：檔案修改時間] → 沿用 speclink-fs 讀變更修改時間的同一種寫法（整秒、讀不到就是 0）；Windows、macOS、Linux 的 CI 都跑 speclink-fs 的測試。時戳一律用 chrono 的 RFC 3339 輸出。
- [本機的修改時間不可靠] → checkout 或複製會改到檔案時間；從別處匯入的文件，時間是匯入的時間。文件沿用變更 `modifiedAt` 的同一句提醒：它是提示，不是事實。
- [`ready_at` 過期] → 取消勾選不清它，由顯示規則擋下；手動把任務勾回來時會顯示舊的日期。接受這個誤差：它只發生在繞過動詞的寫入。
- [HTTP 清單多讀提案] → 每個變更多一份文件，但在同一次批次讀取裡，不增加對儲存的呼叫次數。
- [宿主的儲存多一次寫入] → 勾完最後一個任務時多寫一次 metadata，和開工章同一類寫入，在同一個命令的寫入裡完成。
- [識別會被看到] → `createdById` 和 `claimed.byId` 一樣，讀得到專案的人都看得到；文件沿用同一句提醒：宿主不要拿機密的值當識別。
- [storage 解耦] → 新的資料只經 `Store` 與儲存接口既有的讀寫；修改時間取自儲存本來就回報的值，引擎不讀 checkout、不呼叫 git。

## Migration Plan

不需要資料遷移。舊的變更與討論沒有新的鍵，輸出就沒有對應的鍵。Desktop 的快取不用升版：`why_excerpt` 的結果不變。回滾時還原這個變更即可；已經寫進 metadata 的 `created_by_id` 與 `ready_at` 會被舊版略過。

## Open Questions

(none)
