## Why

外界說 SDD 像瀑布式，核心是流程成本看不見；speclink 自己也一樣——每個變更寫了多少文件、從建立到封存花多久、品質關卡跑了幾輪，都要事後寫腳本翻 git 與對話紀錄才算得出來（2026-10-21 的定位分析就是這樣算的：文件中位 235 行、封存中位 6 小時、每站約 2.5 輪）。其中兩項資料現在根本沒有留下：工單在蓋章時刪除，輪數跟著消失；`.openspec.yaml` 的時間欄位只有日期，算不出時數。本變更是討論 positioning-and-process-cost 分刀計畫的第 2 刀，越早上線、累積的資料越多。

目標使用者與情境：透過 AI agent 開發的開發者（團隊情境下含 PO 與 PM），在桌面 app 的已封存頁回頭看「平常一個變更花多少」；資料在品質關卡蓋章（`review stamp`、`verify stamp`）與封存（`archive`）時寫下。

## What Changes

- 蓋章時把工單輪數寫進 `.openspec.yaml`：審查寫 `reviewed_rounds`、驗證寫 `verified_rounds`，與其他章欄位在同一次原子寫入完成；章失效後重新蓋章時累加，讓重跑的成本也算進去。
- 封存時寫入 `archived_at`，與封存目錄名的日期取自同一個「現在」。
- 新寫入的 change metadata 時間欄位（`created`、`started_at`、`claimed_at`、`reviewed_at`、`verified_at`、`archived_at`）改為帶時區偏移量的 RFC 3339 時戳（例 `2026-10-21T14:05:00+08:00`）；讀取端同時接受純日期與時戳；現有記錄不回改。
- 對外輸出維持日期：CLI 人眼與 `--json`、server 與 desktop 協定裡現在給日期的欄位，一律只給時戳的日期部分，所以既有輸出與既有測試不變。
- 已封存清單每一項增加流程成本欄位：文件行數、建立到封存天數、建立到封存時數（兩端都是時戳時才有）、審查輪數、驗證輪數。
- 桌面 app 已封存頁最上方新增「流程成本」摘要，顯示整個專案的中位數（文件行數另附 P90）與每個數字的樣本數；沒有資料的項目不顯示。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `review-station`: 蓋章時記錄審查輪數（`reviewed_rounds`，重新蓋章累加）。
- `verify-station`: 蓋章時記錄驗證輪數（`verified_rounds`，重新蓋章累加）。
- `change-lifecycle`: change metadata 時間欄位改寫成 RFC 3339 時戳、讀取端兩種格式都接受、對外輸出維持日期；封存寫入 `archived_at`。
- `client-protocol`: desktop 協定的已封存清單項新增流程成本選填欄位（`docLines`、`leadDays`、`leadHours`、`reviewRounds`、`verifyRounds`）；與既有的 `whyExcerpt`、`created` 一樣只由桌面 core 產出，server 的已封存清單（speclink-protocol 的 ArchivedItem）不變。
- `desktop-app`: 已封存頁新增流程成本摘要。

## Impact

- Affected specs: `review-station`, `verify-station`, `change-lifecycle`, `client-protocol`, `desktop-app`
- Affected code:
  - New: crates/engine/speclink-core/src/lifecycle/cost.rs, packages/ui/src/processCost.ts, packages/ui/src/components/ProcessCostSummary.tsx
  - Modified: crates/engine/speclink-core/src/lib.rs, crates/engine/speclink-core/src/util.rs, crates/engine/speclink-core/src/lifecycle/model.rs, crates/engine/speclink-core/src/lifecycle/mod.rs, crates/engine/speclink-core/src/lifecycle/newcmd.rs, crates/engine/speclink-core/src/lifecycle/inprogress.rs, crates/engine/speclink-core/src/lifecycle/archive.rs, crates/engine/speclink-core/src/quality/station.rs, crates/engine/speclink-core/src/command/mod.rs, crates/engine/speclink-core/src/demo.rs, apps/desktop/core/src/cache.rs, packages/ui/src/adapter.ts, packages/ui/src/components/ArchivedList.tsx, packages/ui/src/i18n.tsx, crates/adapters/speclink-node/src/store_bridge.rs
- 影響的 crate／app：speclink-core（寫入與解析、單項成本計算）、apps/desktop/core（已封存快取）、packages/ui（摘要元件與彙總）、speclink-node（SDK store bridge 套用同一道時間收斂）；speclink-cli 與 speclink-server 的程式不需改，靠解析層維持日期輸出。
- 設定欄位：`.openspec.yaml` 新增 `reviewed_rounds`、`verified_rounds`（非負整數）與 `archived_at`（RFC 3339）；`created`、`started_at`、`claimed_at`、`reviewed_at`、`verified_at` 的新寫入值由純日期改為 RFC 3339，舊值照常可讀。`openspec/config.yaml` 與 `.speclink.yaml` 不變。
- 生成的技能：沒有技能檔變動，不升 ASSET_VERSION。
- 相容性影響：CLI 人眼與 `--json` 輸出不變（時間欄位只輸出日期部分），既有 golden 不變；`.openspec.yaml` 的新寫入內容格式改變，直接讀檔的外部工具會看到時戳；舊版 speclink 讀到新格式時，時間欄位是原樣字串，`days_old`／過期判定解析失敗時回到「無建立日期」的既有行為；desktop 已封存快取升版、首次開啟時重建。
