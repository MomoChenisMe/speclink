## Why

review 與 verify 兩個品質站的「章」（蓋章後寫進 change meta 的 `reviewed_*`／`verified_*` 欄位）有兩份知識：寫入端靠 `Station.meta_prefix` 字串拼行（`station.rs` 的 `write_stamp`），讀取端靠 `ChangeMeta` 的十個具名欄位。`Station` 常數因此讀不回自己寫的章，每個要判斷「章還新不新鮮」的地方都得手工把 `meta.reviewed_at`／`reviewed_tasks_total`／`reviewed_scope` 塞進 `StampAnchors`——這份對映今天有**四份**手寫副本：`review.rs` 的 `freshness`、`verify.rs` 的 `freshness`、`archive.rs` 的 `guard_stale_stamps` 兩臂。

`review.rs`（1013 行）與 `verify.rs`（820 行）兩個「站別門面」存在的理由就是補這條縫：各自 6 支公開函式全是一行轉發到 `station.rs`，唯一有內容的就是那份 anchors 對映。門面當初（verify-station-parity design D1）是遷移期的回歸網——「review 的公開函式簽名不變，兩個前置 change 的測試不改一字」——那次遷移 2026-08-06 已封存，回歸網已證明 `station.rs` 正確；今天門面只剩讀者每次要多跳一層的稅，而第三個站要來時得再複製 100 行門面＋900 行測試。

本 change 是討論 improve-lifecycle-layer 結論刀三的前半（候選 4）。結論授權 propose 期依 review 面決定是否把刀三拆回兩刀：候選 4 光是測試搬家就約 1600 行機械改動，候選 3 又改三個封存入口的守門順序，合在一刀會讓守門順序的 review 淹在測試搬家裡，因此拆開、候選 4 先做（它動 `station.rs` 的測試搬家，先落地讓候選 3 的 `archive.rs` diff 乾淨）。候選 3（守門鏈收進 `archive()`）在本 change 封存後立案。零行為變化。

**目標使用者與情境**：透過 AI 代理跑 SDD 的開發者。受影響的 workflow 階段是品質站——`/speclink-review`、`/speclink-verify`、`/speclink-quality` 的工單動詞與蓋章、`/speclink-archive` 的章失效守門、desktop 看板的審查／驗證狀態卡。

## What Changes

- **`ChangeMeta::anchors(&self, st: &Station) -> StampAnchors<'_>`**（crates/speclink-core/src/model.rs）：依站別選 `reviewed_*` 或 `verified_*` 三欄，四份手寫對映收成這一份。`Station` 加一個 `id: StationId`（兩值列舉 Review／Verify），`anchors` 對它做窮舉 match——第三站來時編譯器逼著補臂，不會靜默漏掉。
- **`station::freshness` 改吃 `(st, meta, counts, read_file)`**：直接從 meta 取 anchors；`is_stamped`／`stale_reason` 維持吃 `StampAnchors`（`guard_stale_stamps` 用 `meta.anchors(st)` 餵）。
- **站別常數搬進 `station.rs`**：`station::REVIEW`／`station::VERIFY`（原 `review::STATION`／`verify::STATION`）與 `station::REVIEW_DOC`／`station::VERIFY_DOC`。`review.rs`／`verify.rs` 兩檔刪除，`lib.rs` 的兩個 `pub mod` 移除；原本經門面 `pub use` 轉出的 `content_fingerprint`、`NotFound`、`Freshness` 等型別本來就住在 `station`。
- **呼叫端改直接對 `station`**：`command/mod.rs` 的 8 個 Review／Verify 臂（`crate::review::add_round(store, …)` → `crate::station::add_round(&station::REVIEW, store, …)`）與 `classify` 的 `review::NotFound` downcast；`archive.rs` 的 `guard_open_tickets`／`guard_stale_stamps`；CLI `verbs/station.rs` 的兩個 `StationCli` 常數；desktop `query.rs`（兩處 `freshness`、兩個工單檔名）、`cache.rs`（兩個工單檔名）、`verbs.rs`（兩處 `discard`）。
- **86 條門面測試搬進 `station.rs` 的測試模組**（review 51 條、verify 35 條，各成一個子模組），呼叫改帶 `&REVIEW`／`&VERIFY`，斷言字串一字不動。

**相容性影響**：零行為變化。所有 CLI 人眼輸出與 `--json`、server wire、desktop payload 逐位元不變；訊息字串全部來自 `Station` 常數與 `station.rs` 既有函式，搬家不改字面。不涉及 CLI 子指令、旗標、stdin、exit code；不涉及設定欄位；不涉及生成技能；golden 零變動。公開 API 變動只在 speclink-core crate 內部消費者（CLI、desktop-core 同 workspace，一併改）。

## Non-Goals

（本 change 建 design.md，範圍排除與已否決做法記錄在 design 的 Goals／Non-Goals。）

## Capabilities

### New Capabilities

（none）——step 3 掃描：review-station 與 verify-station 已涵蓋兩站工單動詞、蓋章、指紋錨與失效判定的全部可觀察行為；change-lifecycle 涵蓋封存的章失效守門。本 change 只換實作落點，不加能力。

### Modified Capabilities

（none）——零行為變化，沒有任何要求的字面需要改。

## Impact

- Affected specs: 無 delta
- Affected crates／apps: speclink-core、speclink-cli、apps/desktop/core
- Affected code:
  - New: （無新檔）
  - Modified: crates/speclink-core/src/lib.rs（移除兩個 mod）、crates/speclink-core/src/station.rs（StationId、REVIEW／VERIFY／兩個 DOC 常數、freshness 簽名、86 條測試搬入）、crates/speclink-core/src/model.rs（`anchors`）、crates/speclink-core/src/archive.rs（兩支 guard 與測試的常數路徑）、crates/speclink-core/src/command/mod.rs（8 臂、classify、測試的常數路徑）、crates/speclink-cli/src/verbs/station.rs（兩個常數）、apps/desktop/core/src/query.rs、apps/desktop/core/src/cache.rs、apps/desktop/core/src/verbs.rs、docs/product-status.md／docs/product-status.zh-TW.md（指向 review.rs 的連結改指 station.rs）
  - Removed: crates/speclink-core/src/review.rs、crates/speclink-core/src/verify.rs
- 測試面：86 條門面測試搬家不改斷言；新增 `model.rs` 的 `anchors` 測試（兩站各取對欄位、互不遮蔽）與 `station.rs` 的 `freshness(st, meta, …)` 測試。
