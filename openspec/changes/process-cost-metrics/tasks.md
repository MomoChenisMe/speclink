## 1. 時戳寫入與解析層（D2）

- [ ] 1.1 先寫失敗測試（需求「change metadata 時間欄位寫入時戳、對外輸出日期」）：crates/engine/speclink-core/src/lifecycle/model.rs 的測試斷言 `ChangeMeta` 解析 `created: 2026-10-21T14:05:00+08:00` 時 `created` 為 `2026-10-21`、完整原值保留在 `created_stamp`；`started_at`／`claimed_at`／`reviewed_at`／`verified_at` 同理；純日期與 `someday` 原樣保留；`reviewed_rounds: many` 與 `verified_rounds: -1` 解析失敗。驗證：`cargo test -p speclink-core model` 這些測試失敗 <!-- speclink-task:tsk_01M5EPVQ95BYGRESRJHQ3KJAQS -->
- [ ] 1.2 實作解析層與時戳出口：crates/engine/speclink-core/src/util.rs 新增 `now_rfc3339()`（取 `now_stamps().rfc3339`）；model.rs 的 `ChangeMeta` 把五個既有時間欄位收斂成日期部分、新增 `created_stamp`、`archived_at`、`reviewed_rounds`、`verified_rounds` 欄位。驗證：`cargo test -p speclink-core model` 通過 <!-- speclink-task:tsk_01M5EPVQ95X7XGJ3PNCQE36ZNS -->
- [ ] 1.3 先寫失敗測試：`speclink new change`、開工標記、認領寫入的 `created`／`started_at`／`claimed_at` 符合 RFC 3339（帶偏移量、秒精度）。驗證：`cargo test -p speclink-core newcmd inprogress claim` 新測試失敗 <!-- speclink-task:tsk_01M5EPVQ95NC12G0DSSFY447Y3 -->
- [ ] 1.4 讓寫入改用時戳：crates/engine/speclink-core/src/lifecycle/newcmd.rs、crates/engine/speclink-core/src/demo.rs、crates/engine/speclink-core/src/lifecycle/inprogress.rs 與 crates/engine/speclink-core/src/command/mod.rs 的認領改呼叫 `now_rfc3339()`；原本以 `util::today()` 比對寫入值的測試改成斷言 RFC 3339 格式與日期部分。驗證：`cargo test -p speclink-core` 通過 <!-- speclink-task:tsk_01M5EPVQ95DAKYBVJ3RDMATQX7 -->
- [ ] 1.5 鎖住對外輸出（需求「change metadata 時間欄位寫入時戳、對外輸出日期」）：crates/adapters/speclink-cli/tests/it 新增整合測試——時戳 metadata 的 `speclink show demo` 人眼輸出為 `Created: 2026-10-21`、`--json` 的 `created` 為 `"2026-10-21"`；`created: someday` 原樣輸出且無過期天數。驗證：`cargo test -p speclink-cli --test it` 通過，且 crates/engine/speclink-core/tests/golden 的既有 golden 未修改即通過（`cargo test -p speclink-core --test it render_golden::`） <!-- speclink-task:tsk_01M5EPVQ95CK27W484AYE82FHY -->

## 2. 蓋章輪數（D1）

- [ ] 2.1 先寫失敗測試（需求「蓋章記錄審查輪數」「蓋章記錄驗證輪數」）：crates/engine/speclink-core/src/quality/station 的測試涵蓋首次蓋章寫入 3、重新蓋章累加成 5 且只有一行鍵、`--accept` 蓋章同樣記錄、守門失敗與放棄工單不改輪數、原值為 `many` 時拒絕且零寫入；兩站各一組。驗證：`cargo test -p speclink-core station` 新測試失敗 <!-- speclink-task:tsk_01M5EPVQ95EZDY6N2W0AXC40A8 -->
- [ ] 2.2 實作：crates/engine/speclink-core/src/quality/station.rs 的 `write_stamp` 在移除舊章鍵時一併移除 `<prefix>_rounds`，以 `stamp_gate` 已解析的原值加上 `ticket.rounds` 的長度寫回；`<prefix>_at` 改寫 `now_rfc3339()`。驗證：`cargo test -p speclink-core station` 通過 <!-- speclink-task:tsk_01M5EPVQ95Y7175JH759QT7W7Y -->

## 3. 封存時戳（D3）

- [ ] 3.1 先寫失敗測試（需求「封存寫入 archived_at」）：crates/engine/speclink-core/src/lifecycle/archive 的測試涵蓋成功封存後封存目錄 metadata 含一行 `archived_at` 且日期與目錄名前綴相同、守門拒絕時不寫、已有 `archived_at` 的 change 重試封存後只留一行新值。驗證：`cargo test -p speclink-core archive` 新測試失敗 <!-- speclink-task:tsk_01M5EPVQ95CTA148R7190B9AW9 -->
- [ ] 3.2 實作：crates/engine/speclink-core/src/lifecycle/archive.rs 在守門與正式規格寫入之後、`store.archive_change` 之前，以同一個 `now_stamps()` 的 `rfc3339` 對 change metadata 做 `KeyLines` 取代寫入 `archived_at`。驗證：`cargo test -p speclink-core archive` 通過 <!-- speclink-task:tsk_01M5EPVQ95SR8SYN7N1GFWSKVD -->

## 4. 單項流程成本與桌面快取（D4、D5）

- [ ] 4.1 先寫失敗測試（需求「已封存清單的流程成本欄位」）：新檔 crates/engine/speclink-core/src/lifecycle/cost.rs 的測試以記憶體 Store 重現三個情境——新格式項得到 `docLines` 250、`leadDays` 1、`leadHours` 30.5、`reviewRounds` 2、`verifyRounds` 3；舊格式項只有 `docLines` 與 `leadDays` 2；沒有 metadata 與文件時五欄皆為缺席。驗證：`cargo test -p speclink-core cost` 失敗 <!-- speclink-task:tsk_01M5EPVQ95TEYJRS89YH2R58JA -->
- [ ] 4.2 實作 cost.rs：行數只數 proposal.md、design.md、tasks.md 與 `archived_delta_capabilities` 列出的 delta spec，最後一行無換行也計一行；`leadDays` 取封存目錄名前綴減 `created` 日期部分（最小 0）；`leadHours` 只在 `created_stamp` 與 `archived_at` 都是時戳時計算並四捨五入到一位小數。驗證：`cargo test -p speclink-core cost` 通過 <!-- speclink-task:tsk_01M5EPVQ95NPGVPQ00CK1B5XGD -->
- [ ] 4.3 先寫失敗測試：apps/desktop/core/src/cache.rs 的已封存清單測試斷言清單項 JSON 帶出 `docLines`、`leadDays`、`leadHours`、`reviewRounds`、`verifyRounds`，來源缺席時不出現對應鍵（不以 0 或 null 佔位）；舊版 v6 快取開啟時重建。驗證：`cargo test -p speclink-desktop-core cache` 失敗 <!-- speclink-task:tsk_01M5EPVQ95CWFNMEJ0PJ4GXCPK -->
- [ ] 4.4 實作：cache.rs 的已封存表新增五欄、`CACHE_VERSION` 由 6 升到 7，建表時呼叫 cost.rs 的函式，讀出時缺席欄位不插 key。驗證：`cargo test -p speclink-desktop-core cache` 通過 <!-- speclink-task:tsk_01M5EPVQ95MGZWAANTGMBRKM50 -->

## 5. 已封存頁的流程成本摘要

- [ ] 5.1 先寫失敗測試（需求「已封存頁的流程成本摘要」）：packages/ui/src/__tests__/processCost.test.ts 斷言中位數（奇數取中間、偶數取中間兩值平均）、P90（第 ⌈0.9 × n⌉ 個）、樣本數、樣本為 0 時不產生該數字、四項皆空時回傳空摘要，並重現規格情境的數字（200／400／3 筆、0.5 天、18.3 小時）。驗證：`npm test -w packages/ui -- processCost` 失敗 <!-- speclink-task:tsk_01M5EPVQ950XPMCPGP1GYWTWHT -->
- [ ] 5.2 實作 packages/ui/src/processCost.ts 的純函式彙總，數字最多一位小數、整數不帶小數點。驗證：`npm test -w packages/ui -- processCost` 通過 <!-- speclink-task:tsk_01M5EPVQ95DB9PS2517Y9GDDKD -->
- [ ] 5.3 先寫失敗測試：packages/ui/src/__tests__ 的 `ProcessCostSummary` 與 `ArchivedList` 測試斷言摘要位於頁標題區與列表卡之間、繁中與英文文案、沒有資料時整個摘要不出現、輸入搜尋字串後列表變少而摘要數字不變。驗證：`npm test -w packages/ui -- ProcessCostSummary ArchivedList` 失敗 <!-- speclink-task:tsk_01M5EPVQ95YM41DKT1NPPGP5DH -->
- [ ] 5.4 實作：新檔 packages/ui/src/components/ProcessCostSummary.tsx；packages/ui/src/components/ArchivedList.tsx 以全部封存變更（不受搜尋過濾）計算並放置摘要；packages/ui/src/adapter.ts 的 `ArchivedItem` 新增五個選填欄位；packages/ui/src/i18n.tsx 新增繁中與英文文案。驗證：`npm test -w packages/ui` 通過 <!-- speclink-task:tsk_01M5EPVQ95WQC0W05VS3DN2AN0 -->

## 6. 收尾

- [ ] 6.1 對新增的 metadata 欄位與解析套用 sharp-edges 稽核清單（`speclink instructions --skill audit`）：檢查輪數的負數、非整數、超大值與時戳的非法偏移量都走 fail-closed 或原樣保留的既定路徑，並把結論記在本任務完成註記。驗證：稽核清單逐項有結論，發現的問題已補測試 <!-- speclink-task:tsk_01M5EPVQ95DG4J8CZS0J20C28B -->
- [ ] [M] 6.2 在桌面 app 打開本 repo 的已封存頁，確認流程成本摘要出現在頁標題區與列表卡之間：舊記錄為主時顯示文件行數、任務數與建立到封存天數，品質關卡輪數因沒有樣本而不顯示；切到英文介面時文案為英文 <!-- speclink-task:tsk_01M5EPVQ951ZG5R49CNW4PB8AY -->
- [ ] 6.3 受影響面測試與文件守門全部通過。驗證：`cargo test -p speclink-core`、`cargo test -p speclink-cli --test it`、`cargo test -p speclink-desktop-core`、`npm test -w packages/ui`、`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 皆通過，且 `speclink validate process-cost-metrics` 通過 <!-- speclink-task:tsk_01M5EPVQ952SFX806RAQ4BHM7Z -->
