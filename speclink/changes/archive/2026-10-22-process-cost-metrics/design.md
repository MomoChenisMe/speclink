## Context

change metadata（`.openspec.yaml`）的時間欄位都由 `util::today()` 寫成純日期：`created`（lifecycle/newcmd.rs、demo.rs）、`started_at`（lifecycle/inprogress.rs）、`claimed_at`（command/mod.rs 的認領）、`reviewed_at`／`verified_at`（quality/station.rs 的 `write_stamp`）。封存在 change 移入封存目錄之後，才對封存目錄的 metadata 追加純日期的 `archived_at`（與 `archived_by`；現有封存記錄都帶這一行），所以時間只精確到日；`util::now_stamps()` 已能從同一個 `Local::now()` 同時產出日期與 RFC 3339（archive 用它命名目錄並寫 `@trace updated`）。

工單在蓋章的同一次寫入中刪除（`write_stamp` 先移除站別五鍵、再寫新章、同步刪工單），回合數因此消失；`stamp_gate` 已把工單解析成 `Ticket { rounds }`，蓋章當下拿得到回合數。

讀取端：`ChangeMeta`（lifecycle/model.rs）是只讀的 serde 結構，CLI（verbs/query.rs、verbs/checks.rs）、server（api/routes.rs）、desktop core（query.rs、cache.rs）與 drift（speclink-host drift.rs）都直接把 `meta.created`、`meta.started_at` 等字串原樣往外送；`preflight.rs` 以 `%Y-%m-%d` 解析 `created` 算過期天數；client-protocol 已承諾已封存清單的 `created` 是 `YYYY-MM-DD`。

已封存頁：desktop core 的 `cache.rs` 把每個封存變更寫進 SQLite 快取（目前 v6），清單項經 desktop 協定送到 packages/ui 的 `ArchivedList`；遠端工作區的已封存清單來自 server，欄位本來就比本機少。server-web 不使用 `ArchivedList`。

## Goals / Non-Goals

**Goals:**

- 蓋章時留下輪數、封存時留下時戳，讓每個新變更的流程成本可以事後計算。
- 時間欄位精確到秒，但所有既有對外輸出維持日期，既有 golden 與 CLI 測試不變。
- 已封存頁一眼看到整個專案的流程成本中位數與樣本數。

**Non-Goals:**

- 不回填舊記錄：現有 284 筆以上的封存變更不補輪數、不改時間格式。
- 遠端工作區不顯示摘要：server 的已封存清單不加欄位（speclink-protocol 的 `ArchivedItem` 不變）。
- 不加 CLI 指令或 `--json` 輸出流程成本；不放進每個變更的數字磚。
- 不記錄被放棄（discard）的工單回合數。
- 討論記錄的 `created` 與封存日期不在本變更範圍。

## Decisions

### D1：重新蓋章時累加輪數

`reviewed_rounds`／`verified_rounds` = 原值（缺席為 0）＋本次工單回合數。理由：章失效後重跑是真實成本，覆寫會把它藏起來。替代方案「只記最後一次」被否決，因為它與討論要量測的重跑成本相反。`write_stamp` 移除舊章鍵時一併移除舊輪數鍵，再寫入累加後的值，維持「重蓋不留重複鍵、章永遠落在檔尾」的既有作法；原值由 `stamp_gate` 已解析的 metadata 取得。原值不是非負整數時，`ChangeMeta` 的型別化欄位（`Option<u64>`）解析失敗，`check_meta_text` 依既有 fail-closed 路徑拒絕。

### D2：寫入時戳、解析層把對外欄位收斂成日期

所有 change metadata 時間欄位改由新的 `util::now_rfc3339()`（`now_stamps().rfc3339` 的單一出口）寫入。為了不改動每個對外輸出點，`ChangeMeta` 解析時把 `created`、`started_at`、`claimed_at`、`reviewed_at`、`verified_at` 這些既有欄位存成日期部分（值符合 RFC 3339 時取前 10 個字元；純日期與無法辨識的值原樣保留），完整時戳不存進 `ChangeMeta`：只有流程成本需要它，由 `lifecycle/cost.rs` 逐鍵讀 metadata 原文取得（`archived_at` 因此也不是 `ChangeMeta` 的欄位，見 D4）。CLI、server、desktop core、drift 與 preflight 因此不需修改就輸出日期。SDK 的 store bridge（speclink-node 的 `change_from_value`）與 remote drift（speclink-host 的 `RemoteDriftStore`）由已結構化的欄位組 `ChangeMeta`、不經解析層，所以改走 `ChangeMeta::from_fields`，收尾跑同一道收斂；`ChangeMeta` 帶 `#[non_exhaustive]`，core 以外以字面值建構會編譯失敗；`Default` 只當讀檔失敗時的空白後備，不在上面逐欄填值。替代方案「逐一修改每個輸出點」被否決：輸出點散在三個 crate 與 desktop core，漏一處就破壞協定承諾的 `YYYY-MM-DD`。`ChangeMeta` 沒有寫回路徑（寫入一律走 `KeyLines` 文字手術），所以解析層收斂不會把時戳寫丟。

### D3：`archived_at` 在移動前寫入

archive 守門全數通過、正式規格寫入後，以同一個 `now_stamps()` 的 `rfc3339` 對 change metadata 做 `KeyLines` 取代寫入 `archived_at`，再呼叫 `store.archive_change`。移動失敗時 change 留在進行中並帶著 `archived_at`；它只供流程成本讀取，不影響任何判定，重試時被取代。現行「移動後追加純日期 `archived_at`」的寫法一併移除，`archived_by` 維持移動後追加。替代方案「移動後寫入封存目錄的 metadata」被否決：移動成功而寫入失敗時封存已完成卻少了時戳，且需要新的錯誤回報路徑。

### D4：單項成本在 speclink-core 計算，彙總在 packages/ui

新增 `lifecycle/cost.rs`，提供一個只依賴 `Store` 的函式，對一個封存變更算出 `docLines`、`leadDays`、`leadHours`、`reviewRounds`、`verifyRounds`（皆為 `Option`）；metadata 逐鍵讀原文、不經 `ChangeMeta`，一個鍵缺席、重複或壞掉只讓用到它的欄位缺席；行數只數 proposal.md、design.md、tasks.md 與 `archived_delta_capabilities` 列出的每份 delta spec。desktop core 的 `cache.rs` 呼叫它並把結果存成快取欄位；server 之後要支援時呼叫同一個函式。中位數與 P90 是呈現層的彙總，放在 packages/ui 的純函式 `processCost.ts`，由 `ProcessCostSummary` 元件顯示在 `ArchivedList` 的頁標題區與列表卡之間。替代方案「在 UI 逐項讀文件算行數」被否決：違反 client-protocol「由清單載入一次帶出」。

### D5：desktop 快取升版

`cache.rs` 的已封存表新增五個欄位並把快取版本由 v6 升到 v7；版本不符時沿用既有的整表重建。首次開啟時重建會多讀每個封存變更的 design.md、tasks.md 與 delta spec（proposal.md 已為 `whyExcerpt` 讀過）。

## Implementation Contract

- **寫入（speclink-core）**：`speclink new change`、開工標記、認領、`review stamp`、`verify stamp` 新寫入的時間欄位符合 RFC 3339（帶偏移量、秒精度）；`review stamp`／`verify stamp` 守門通過時同一次寫入含累加後的 `reviewed_rounds`／`verified_rounds`；`speclink archive` 成功後封存目錄 metadata 含一行 `archived_at`。驗收：speclink-core 的單元測試與 `cargo test -p speclink-cli` 的整合測試斷言檔案內容。
- **對外輸出不變**：對純日期 metadata，`speclink show`、`speclink list --json`、drift 報告與 server 回應位元級不變；對時戳 metadata，這些輸出只出現日期部分。驗收：既有 golden 與 CLI 測試不需更新即通過，另加一組時戳 metadata 的 CLI 測試。
- **desktop 協定**：已封存清單項依 client-protocol 新需求帶出五個選填欄位，缺席規則逐欄成立。驗收：`cargo test -p speclink-desktop-core` 的快取測試。
- **畫面**：已封存頁摘要依 desktop-app 新需求顯示、隱藏與計算。驗收：`npm test -w packages/ui` 的 `processCost` 純函式測試與 `ProcessCostSummary` 元件測試。
- **範圍內**：上列 crate 與 app。**範圍外**：server 已封存清單、CLI 成本輸出、舊記錄回填、討論記錄時間。

## Risks / Trade-offs

- [回歸對照：測試以 `util::today()` 比對 metadata 寫入值] → 改用解析後的日期部分或 RFC 3339 格式斷言；golden 的 fixture 都是純日期，輸出不變，任何 golden 需要更新都代表 D2 漏了一個輸出點，要回頭修解析層而不是更新 golden。
- [跨平台：時區偏移格式] → 一律用 chrono 的 `to_rfc3339_opts(Secs, false)`，偏移量寫成 `+08:00` 形式；UTC 環境寫成 `+00:00`。Windows／macOS／Linux 的 CI 都跑 speclink-core 與 CLI 測試。
- [寫入者時區與讀者時區不同] → 日期部分取寫入者時區，與今天純日期的語意相同；跨時區團隊的「日期」本來就以寫入者為準。
- [舊版 speclink 讀新格式] → 舊版把時戳當原樣字串輸出，過期判定解析失敗時回到「沒有建立日期」的行為，不會崩潰；團隊混用版本時輸出會短暫不一致，發版說明提醒升級。
- [快取重建變慢] → 只在升版後第一次開啟發生一次，讀取量與現有 `whyExcerpt` 同級。

## Migration Plan

不需要資料遷移：舊記錄保持純日期，讀取端兩種格式都接受。desktop 快取版本不符時自動重建。回滾時，舊版讀到時戳欄位仍可運作（見 Risks）。

## Open Questions

(none)
