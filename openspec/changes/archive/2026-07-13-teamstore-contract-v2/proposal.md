## Why

平台架構藍圖（docs/platform-architecture.zh-TW.md §4.3、§14 Phase 1 第 2 項）要求在任何 Store driver 或 Server 動工前，先固定 TeamStore 的 revision／CAS／Unit of Work／snapshot／outbox／error 契約與 conformance suite——「Store Contract 屬於 Phase 1，因為 archive 原子性、domain event 與遠端 policy 都依賴它；不能等 Server 實作時才補」。現況（重構路線圖 §3.1）的 `speclink_core::store::Store` 是刻意同步、只能替換文件讀寫的 seam：讀取以 Option／Vec／bool 表達，無法區分不存在、無權限、revision 衝突、暫時不可用、損壞與 backend 失敗；以 PathBuf 暴露實體位置作定址；缺 Project／Repo scope、consistent snapshot、CAS、transaction、immutable history、migration 與 outbox。若不先固定契約，Phase 2 的 SQLite／Server FS／PostgreSQL driver 會各自長出不同的 transaction 與 recovery 語意（藍圖 §15.1 P0「Store 能力與 Host 承諾不一致」「Context snapshot 可能混版」「commit 與 event 發布不原子」「遠端規格離開 Git 後沒有完整歷史」四列的根源）。

目標使用者：Phase 1C（host-runtime-binding-policy）與 Phase 2（reference-server）的實作者——包含後續撰寫官方 driver 與自訂 Store 的整合者；他們以本契約與 conformance suite 為唯一正確性基準。本刀是純契約與測試基建，不改任何現行 workflow 階段的使用者可見行為。

## What Changes

- 新增 `speclink-store` crate（Cargo workspace 成員）：TeamStore 契約的唯一 Rust 定義——與 `speclink-core` 零相依，既有 Store trait（Local DocumentStore 層）完全不動。
- 契約介面對齊藍圖 §4.3 概念契約：manifest／health／migrate／snapshot／begin unit of work／commit（帶 event records）／rollback／export／import，外加 outbox 讀取與確認（conformance 的 outbox failure 與 crash recovery 需要可觀察的事件持久化）。
- 讀取全面 typed Result：封閉的 Store 錯誤集合區分不存在、無權限、revision 衝突（帶 expected／actual）、暫時不可用、損壞、backend 失敗；不以 Option／空集合吞錯。
- 文件定址使用 Project／Repo scope 與邏輯 document locator（change metadata、artifact、canonical spec、discussion、workflow config），不暴露 PathBuf 作跨媒介身分。
- 同一 command 的文件寫入、project revision 遞增、immutable history 追加與 outbox 追加，落在同一個 Unit of Work：全部生效或全部不生效；CAS 不符回 revision 衝突。
- 能力宣告與三級能力等級：Local single-writer／Single-node TeamStore／Cluster TeamStore；manifest 宣告 contract version 與 capability 集合，能力不足以支撐宣告等級時 conformance 直接判失敗（Host 啟動驗證屬順位 4 刀，本刀交付可供驗證的宣告面）。
- in-memory reference store：契約的最小參考實作（測試基建，非產品 driver），供 conformance suite 在第一個 SQLite driver 前即可執行。
- conformance suite 以可重用形式輸出（未來 driver 與自訂 Store 跑同一套）：至少涵蓋 CAS race、mixed snapshot、partial commit、outbox failure、crash recovery 與 tenant scope 六類情境（路線圖 §5 Phase 1 gate），含故障注入點。
- versioned export bundle 型別與 import 驗證語意（round-trip 為 conformance 一部分）。

## Non-Goals

- 不實作 SQLite、Server FS、PostgreSQL driver（Phase 2 reference-server 刀）；不動 `@speclink/store-*` N-API facade（Phase 4）。
- 不定案 Client Protocol／Command Query DTO（順位 7 protocol-client-context；路線圖 §4.3 允許的平行草案不在本刀 artifacts 內）。
- 不做 Host 的 ExecutionContext、authorization、binding 與 policy injection（順位 4 host-runtime-binding-policy）；本刀不接線任何現有 command 到 TeamStore。
- 不做事件 transport 與訂閱（SSE／WS 屬 Phase 2）；本刀只保證 outbox 持久化與 cursor 可重讀。
- 不改 `speclink_core::store::Store`、`speclink-fs`、CLI、Node dispatch 與桌面的任何行為——既有 crates 原始碼零改動，人眼與 --json 輸出逐位元不變。
- 不把現有 Store 的 31 個 Node bridge methods 當作 TeamStore API 的基準（路線圖 §6 明列反模式）。
- 不引入 async runtime：契約維持同步、object-safe（與 core 的既有立場一致）；async driver 的需求出現時以契約版本化另議。

## Capabilities

### New Capabilities

- `teamstore-contract`: TeamStore 的 Rust 契約與正確性基準——分層定位（Local DocumentStore 與 TeamStore）、能力等級與 manifest 宣告、typed error 集合、Project／Repo scope 定址、snapshot／UoW／CAS／immutable history／outbox 語意、export/import bundle，以及全部 driver 與自訂 Store 必須通過的 conformance suite。

### Modified Capabilities

（無）——既有的本地 store seam 正典規格（本地指令行為不變、檔案佈局不變、工作區建置含預設儲存實作）完全不受影響。

## Impact

- 影響的 crate：新增 `speclink-store`（契約、in-memory reference、conformance）；根 Cargo workspace 設定與 lock file 隨成員追加而動。`speclink-core`、`speclink-cli`、`speclink-fs`、`speclink-node`、desktop 原始碼零改動。
- 相容性影響：無任何既有指令、輸出或設定行為變更；parity／color／twin 回歸對照不受影響。cargo test --workspace（root 的 npm run test:all 已涵蓋）自動納入新 crate 測試，CI 設定不動。
- Affected specs: `teamstore-contract`（新增）。
- Affected code:
  - New: crates/speclink-store/Cargo.toml、crates/speclink-store/src/lib.rs、crates/speclink-store/src/types.rs、crates/speclink-store/src/error.rs、crates/speclink-store/src/uow.rs、crates/speclink-store/src/memory.rs、crates/speclink-store/src/conformance/mod.rs
  - Modified: Cargo.toml、Cargo.lock（workspace 成員追加）
  - Removed: 無
