## Context

品質站的共通生命週期（工單 add-round／show／discard、蓋章、指紋、失效判定）住在 `crates/speclink-core/src/station.rs`，以 `Station` 常數組（工單檔名、meta 前綴、訊息用詞、`round_requires_tasks_complete`）參數化。`review.rs` 與 `verify.rs` 各持一個 `STATION` 常數、一個 `*_DOC` 常數、一組 `pub use` 轉出 station 的型別，以及 6 支一行轉發的公開函式（`add_round`、`show_with_content`、`discard`、`stamp`、`stamp_with_scope`、`freshness`）。

只有 `freshness` 有內容：把 `meta.reviewed_at`／`reviewed_tasks_total`／`reviewed_scope`（或 `verified_*`）組成 `StampAnchors` 再呼叫 `station::freshness`。同一份對映在 `archive.rs` 的 `guard_stale_stamps` 又手寫了兩臂。寫入端 `station::write_stamp` 用 `st.meta_prefix` 拼 `{prefix}_at` 等鍵名，讀取端用 `ChangeMeta` 的具名欄位——`Station` 讀不回自己寫的章。

呼叫端盤點（非測試碼）：`command/mod.rs` 8 個 Command 臂＋`classify` 的 `review::NotFound` downcast；`archive.rs` 的 `guard_open_tickets`（兩個 STATION）與 `guard_stale_stamps`（兩個 STATION＋兩份對映）；`crates/speclink-cli/src/verbs/station.rs` 的 `REVIEW_CLI`／`VERIFY_CLI`；desktop `query.rs`（`REVIEW_DOC`／`VERIFY_DOC`＋兩處 `freshness`）、`cache.rs`（兩個 DOC）、`verbs.rs`（兩處 `discard`）。測試碼：`review.rs` 51 條、`verify.rs` 35 條，`command/mod.rs` 與 `archive.rs` 的測試引用 `REVIEW_DOC`／`VERIFY_DOC`／`content_fingerprint`，desktop `query.rs` 測試引用 `content_fingerprint`。

verify-station-parity design D1 選門面的理由是遷移期回歸網（review 簽名不變，前置測試不改一字）；該 change 2026-08-06 封存，理由已用完。

## Goals / Non-Goals

**Goals:**

- 「這個站的章住在哪些 meta 欄位」只有一份：`ChangeMeta::anchors(st)`，且第三站來時由編譯器逼著補對映。
- 讀一個站的行為只看 `station.rs`：常數、函式、測試都在那一檔；門面層歸零。
- 零行為變化：所有訊息字串、輸出、守門順序不變。

**Non-Goals:**

- 不動封存守門鏈、`run_archive`、`--mark-tasks-complete` 的代勾位置、bulk 的跳過語意（候選 3，下一刀）。
- 不改 `Station` 的既有欄位語意、工單格式、章欄位形狀、`StampAnchors` 型別。
- 不改 speclink-host 的 `change_diff::StationNs`（CLI 側的 snapshot namespace，另一層的站別列舉）。
- 不動 speclink-protocol、server routes、Node SDK（沒有任何引用門面的地方）。
- 已否決做法（討論 Round 6）：保留門面只加 `anchors`（跳一層的稅還在，第三站仍要複製門面與測試）；`Station` 帶 `fn(&ChangeMeta) -> StampAnchors` 函式指標（常數變成半個 trait）；`StationKind` trait（兩個實例、零多型需求，improve-workspace-sync 已否決同型空轉接）；測試改參數化跑兩站（斷言字串兩站不同，參數化會把差異塞進 match，可讀性反而下降）。

## Decisions

### D1 ChangeMeta::anchors 以 StationId 窮舉對映

```rust
// station.rs
#[derive(Clone, Copy)]
pub enum StationId { Review, Verify }
pub struct Station { pub id: StationId, /* 既有欄位不變 */ }
pub const REVIEW_DOC: &str = "review.md";
pub const VERIFY_DOC: &str = "verify.md";
pub const REVIEW: Station = Station { id: StationId::Review, doc: REVIEW_DOC, meta_prefix: "reviewed", … };
pub const VERIFY: Station = Station { id: StationId::Verify, doc: VERIFY_DOC, meta_prefix: "verified", … };

// model.rs
impl ChangeMeta {
    pub fn anchors(&self, st: &Station) -> StampAnchors<'_> {
        match st.id {
            StationId::Review => StampAnchors { stamped_at: self.reviewed_at.as_deref(), tasks_total: self.reviewed_tasks_total, scope: &self.reviewed_scope },
            StationId::Verify => StampAnchors { stamped_at: self.verified_at.as_deref(), tasks_total: self.verified_tasks_total, scope: &self.verified_scope },
        }
    }
}
```

- 對映住在 `ChangeMeta` 上（它擁有那十個欄位），不放 `Station`（常數裡放函式指標會讓資料變半個 trait）。
- 用 `StationId` 列舉而不是對 `meta_prefix` 字串 match：字串 match 需要一個 fallthrough 臂，第三站漏補時靜默走進去；列舉 match 窮舉，漏補是編譯錯誤。討論 Round 6 寫的是「依 `st.meta_prefix` 選」，本決策達成同一目的、把檢查提前到編譯期。
- `StationId` 不對外有其他用途；CLI 的 `StationNs` 與它並存（不同層、不同關注）。

### D2 station::freshness 直接吃 meta

```rust
pub fn freshness(st: &Station, meta: &ChangeMeta, counts: &Counts, read_file: &ScopeReadFn<'_>) -> Freshness
```

- 函式開頭一行 `let anchors = meta.anchors(st);`，其餘判定本體不變——不另切私有轉接層（全 repo 只有這一個呼叫端）；`is_stamped`／`stale_reason` 維持公開且吃 `StampAnchors`（`guard_stale_stamps` 需要在判定前先以 `is_stamped` 過濾、再取破錨原因）。
- `guard_stale_stamps` 的兩臂手寫對映改為 `[&REVIEW, &VERIFY].into_iter().map(|st| (st, change.meta.anchors(st)))`。
- 語意零變化：Unknown／Stale／Fresh 判定規則不動。

### D3 常數與門面搬進 station.rs、兩檔刪除

- `review.rs`／`verify.rs` 刪除；`lib.rs` 移除 `pub mod review;`／`pub mod verify;`。
- 門面 `pub use` 曾轉出的名字（`content_fingerprint`、`content_fingerprint_bytes`、`fingerprint_scope`、`scope_union`、`Finding`、`Freshness`、`NotFound`、`Round`、`RoundPhase`、`ScopeReadFn`、`Severity`、`Ticket`）本來就定義在 `station`，呼叫端改路徑即可。
- 常數命名：`station::REVIEW`／`station::VERIFY`（站）、`station::REVIEW_DOC`／`station::VERIFY_DOC`（工單檔名）。文件註解的「design D1 review.rs 薄實例」措辭改成「兩個常數」。

### D4 呼叫端改寫清單

| 檔案 | 改動 |
| --- | --- |
| crates/speclink-core/src/command/mod.rs | 8 臂：`crate::review::add_round(store, &change, &content)` → `crate::station::add_round(&crate::station::REVIEW, store, &change, &content)`，其餘 `show_with_content`／`stamp_with_scope`／`discard` 同形；verify 臂帶 `&VERIFY`。`classify` 的 `crate::review::NotFound` → `crate::station::NotFound`。測試中的 `crate::review::REVIEW_DOC`／`content_fingerprint` 改 `crate::station::` |
| crates/speclink-core/src/archive.rs | `guard_open_tickets` 的 `[(&REVIEW, carry_review), (&VERIFY, carry_verify)]`；`guard_stale_stamps` 依 D2；測試中的 `REVIEW_DOC`／`VERIFY_DOC` 改路徑 |
| crates/speclink-cli/src/verbs/station.rs | `REVIEW_CLI.station = &core::station::REVIEW`、`VERIFY_CLI.station = &core::station::VERIFY` |
| apps/desktop/core/src/query.rs | `speclink_core::station::REVIEW_DOC`／`VERIFY_DOC`；`speclink_core::station::freshness(&REVIEW, &c.meta, &counts, &read_file)`（verify 同形）；測試的 `content_fingerprint`／`content_fingerprint_bytes` 改路徑 |
| apps/desktop/core/src/cache.rs | 兩個 DOC 常數改路徑 |
| apps/desktop/core/src/verbs.rs | `speclink_core::station::discard(&station::REVIEW, store, change)`（verify 同形） |

所有訊息字串來自 `Station` 欄位與 `station.rs` 既有函式，改寫不觸碰任何字面。

### D5 測試搬家：兩個子模組、呼叫加站別、斷言不動

- `station.rs` 的 `#[cfg(test)] mod tests` 內新增 `mod review` 與 `mod verify` 兩個子模組，分別承接 `review.rs` 51 條與 `verify.rs` 35 條測試，各自保留原本的 `META`／`ROUND_*`／`code_counts` 等 fixture（子模組隔離，不與 station 既有 3 條測試的名稱衝突）。
- 呼叫改寫是機械式：`add_round(&store, …)` → `add_round(&REVIEW, &store, …)`（review 51 處、verify 40 處）、`stamp(…)`／`stamp_with_scope(…)`／`discard(…)` 同形、`freshness(&meta, &counts, &rf)` → `freshness(&REVIEW, &meta, &counts, &rf)`（review 13 處、verify 10 處）、`station::show(&STATION, …)` → `show(&REVIEW, …)`。verify 子模組用 `&VERIFY`。
- 斷言字串、fixture 內容、測試名稱一字不動——搬家後 `cargo test -p speclink-core station::tests::review::` 與 `station::tests::verify::` 的測試數必須分別是 51 與 35。
- 不做參數化：兩站的斷言文字本來就不同（`re-review` vs `re-verify`、`review.md` vs `verify.md`），參數化會把差異塞進 match。

## Implementation Contract

**可觀察行為**：無變化。`speclink review ...`／`speclink verify ...` 全部子指令、`speclink archive` 的未結工單守門與章失效守門、desktop 的 `reviewStatus`／`verifyStatus`／封存頁的 `reviewedNotPassed`／`verifiedNotPassed`，人眼輸出、`--json`、exit code 逐位元不變。

**介面**

- 新增：`station::StationId`、`station::REVIEW`／`VERIFY`／`REVIEW_DOC`／`VERIFY_DOC`、`Station.id` 欄位、`ChangeMeta::anchors(&self, &Station) -> StampAnchors<'_>`。
- 變更：`station::freshness(st, meta, counts, read_file)`（原以 `StampAnchors` 為首參數的版本併入，不留私有轉接）。
- 刪除：`speclink_core::review` 與 `speclink_core::verify` 模組（含 `STATION`、`REVIEW_DOC`／`VERIFY_DOC`、6 支門面函式、`pub use` 轉出）。
- 不變：`station::add_round`／`show`／`show_with_content`／`discard`／`stamp`／`stamp_with_scope`／`is_stamped`／`stale_reason`／`open_ticket_disposal` 的簽名；`StampAnchors`、`Freshness`、`NotFound`、`Ticket` 等型別。

**失敗模式**：無新增。`NotFound`／`Refusal` 的產生點與字串不動；`classify` 的映射不動。

**驗收條件**

1. `cargo test -p speclink-core` 全綠；`station::tests::review::` 51 條、`station::tests::verify::` 35 條、`model::tests::anchors_` 新增測試全綠；`--test it render_golden::` 零 diff。
2. `cargo test -p speclink-cli --test it` 全綠（review／verify 家族、archive 守門家族、`remote_verb_parity`）。
3. `cargo test -p speclink-desktop-core` 全綠。
4. `test -e crates/speclink-core/src/review.rs || test -e crates/speclink-core/src/verify.rs` 皆不存在；`grep -rn 'review::\|verify::' --include='*.rs' crates apps` 只剩 host `change_diff` 或字串內容等與門面無關的命中（預期零命中於 `crate::review::`／`speclink_core::review::`／`core::review::` 與 verify 對應形）。
5. `grep -n 'reviewed_at.as_deref()\|verified_at.as_deref()' crates/speclink-core/src/*.rs` 非測試碼只命中 `model.rs` 的 `anchors` 一處（四份對映歸一）。

**範圍邊界**

- In：`station.rs`、`model.rs`（`anchors`）、`lib.rs`、`archive.rs` 兩支 guard 與測試常數路徑、`command/mod.rs` 8 臂＋classify＋測試常數路徑、CLI `verbs/station.rs` 兩常數、desktop `query.rs`／`cache.rs`／`verbs.rs`、86 條測試搬家，以及 `docs/product-status.md`／`docs/product-status.zh-TW.md` 兩處指向 `review.rs` 的連結改指 `station.rs`（刪檔的必然後果）。
- Out：`archive()` 的守門順序與 `run_archive`、bulk、`--mark-tasks-complete`（候選 3）；`write_stamp` 的寫入（刀二已改走 KeyLines）；host `StationNs`；protocol／server／node。

## Risks / Trade-offs

- [測試搬家時漏搬或改到斷言] → 搬家前記下 `cargo test -p speclink-core review::`／`verify::` 的測試數（51／35），搬家後子模組數量必須相同；`git diff` 以 `--color-moved` 檢視，斷言行應全部顯示為搬移而非修改。
- [`freshness` 簽名改變漏改呼叫端] → 編譯期即失敗；desktop-core 是獨立 package，`cargo test -p speclink-desktop-core` 必跑。
- [golden／CLI 回歸] → 所有字串來自常數與既有函式；改碼前跑一次 `render_golden::` 與 CLI review／verify／archive 家族建立基準，任何 diff 視為缺陷。
- [第三站的擴充點] → `StationId` 加一值、`anchors` 補一臂、`ChangeMeta` 補五欄、常數加一組——都是編譯器會提醒的位置。
- [跨平台] → 純程式碼搬家，不碰路徑與行尾；`content_fingerprint` 的 CRLF 正規化規則不動。
- [station.rs 檔案變長（約 2400 行，其中 1600 行是測試）] → 讀者只看非測試碼約 800 行；測試以子模組分區。刻意不拆測試到獨立檔（repo 只在 server 整合測試用過 `#[path]`，不引入新慣例）。

## Migration Plan

無資料遷移、無格式變化。部署即生效；回滾即還原程式碼。同 workspace 的 CLI 與 desktop-core 一併改，沒有外部消費者。

## Open Questions

（無。候選 3 於本 change 封存後另立 change。）
