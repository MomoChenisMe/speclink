## Why

speclink-core 的討論記錄（`openspec/discussions/<slug>.md`）只有「型別化的讀」（`DiscussionInfo`），沒有「型別化的寫」：frontmatter 的狀態轉移（`status` open→concluded→promoted、`promoted_to` 逗號累加、`hold` 旗標、`board_rank`）由 `mark_promoted`、`unlink_discarded`、`conclude`、`set_board_rank` 四支函式各自做字串手術，同一個 `status` 欄位並存兩套寫法（整份文字 `replacen` 與只認 frontmatter 的 `set_frontmatter_line`）。`replacen("status: open")` 不限於 frontmatter，內文出現同字串就會被誤改。讀取側同樣散開：`promoted_to`、`discussion_concluded`、`discussion_held`、`board_rank` 各自獨立讀檔，desktop 每張討論卡讀 3 次檔，host crate 的 `discussions_extras` 直接抓原文再呼叫 `promoted_to_in`／`concluded_in`，frontmatter 的文字格式知識已外洩到 speclink-host。

同時，「討論何時結束生命」的判準（無在途變更引用＋已有結論＋無 hold）在 `discuss::conclude` 與 `archive::archive` 各算一次，而且已經漂移：conclude 側把 meta 損壞的在途變更視為「仍引用」（fail-closed），archive 側只看 `from_discussions()`，meta 損壞的變更被當作不引用、討論會被掃進封存區。近一個月四個封存變更（conclusion-gated-discussion-archive、discussion-spinout-hold、discuss-search-recall、validate-merge-gate）每個都得同時補 `discuss.rs` 與 `archive.rs` 兩處以上。

本 change 是討論 improve-lifecycle-layer 結論的三刀之一（刀一，候選 1＋2），也是唯一帶行為變化的一刀；刀二（change meta 八個寫入點改走同一原語）與刀三（封存守門鏈收進 `archive()`＋站別門面刪除）在本 change 封存後再各自立案。

**目標使用者與情境**：透過 AI 代理跑 SDD 的開發者。受影響的 workflow 階段是討論生命週期——`/speclink-propose --from-discussion`、`/speclink-discuss` 的 conclude（含 `--hold`）、`/speclink-archive` 的討論隨行封存、discard 的解鏈，以及 desktop 看板與 server 討論列表的讀取。

## What Changes

- **新增逐行編輯原語 `KeyLines`**（新模組 `crates/speclink-core/src/keylines.rs`）：對「頂層 `key: value` 行」做原位手術，其餘位元組逐字保留、永不重新序列化。以 frontmatter（首行 `---` 到下一個 `---` 之間）為區域，提供 `get`、`set`（第一條原位代換、重複鍵丟掉、缺則補在 frontmatter 尾端）、`remove`（純量鍵只刪那一行；區塊鍵含縮排續行區塊）、`list`（逗號清單）、`text` 回寫。介面只長到討論側實際用到的形狀；刀二的 change meta 八個寫入點是第二個消費者，屆時再依真實需求長出來。
- **新增 `DiscussionHead`**（`discuss.rs` 內，建在 `KeyLines::frontmatter` 之上）：一次解析 frontmatter 為九個欄位——slug、topic、status（三值列舉 Open／Concluded／Promoted）、created、created_by、kind、promoted_to（`Vec<String>`）、hold（bool）、board_rank。缺 status 沿現況預設 open；未知 status 值不 fail-closed（手寫文件，讀端容錯是既有方向）。head 只管 frontmatter；Context／Rounds／Conclusion 的區段函式不動。
- **三條狀態轉移成為 head 的方法**：`promote(change) -> bool`（open／concluded→promoted、promoted_to 累加去重、**只有新名字才清 hold**）、`unlink(change) -> Option<Status>`（縮清單；清空時移除 promoted_to 行並回退 concluded 或 open）、`conclude(hold)`（open→concluded、hold 設或清）。`mark_promoted`、`unlink_discarded`、`conclude`、`set_board_rank` 一律改為「載入 head → 呼叫方法 → 回寫」。
- **`DiscussionInfo` 加兩個 `#[serde(skip)]` 欄位**：`concluded` 與整個 `head: DiscussionHead`（`promoted_to`、`hold`、`board_rank` 都在裡面），由同一趟解析產出。`discuss list --json` 與 `discuss show --json` 逐位元不變（skip 欄位不進 JSON）。三個內部轉接因此消失：desktop `entry()` 的兩次額外讀檔與 `board_sorted_active` 的一次、server `discussion_dtos`（原名 `discussion_dtos_with_extras`）的 extras 組裝改讀 info 欄位、host `bridge::discussions_extras` 整支刪除。
- **刪除五支文字探測函式與三支失去呼叫端的讀取函式**：`frontmatter_value`、`set_frontmatter_line`、`promoted_to_in`、`concluded_in`、`held_in`，以及 discuss.rs 非測試碼內對 frontmatter 的全部 `replacen`；`board_rank(store, slug)`、`discussion_concluded`、`discussion_held` 在轉接移除後沒有跨模組呼叫端，一併刪除。`promoted_to(store, slug)` 改由 head 實作、簽名不變（trace.rs 仍呼叫）。
- **新增 `discuss::close_if_finished(store, slug) -> Result<Option<String>>`**：收「無在途變更引用（meta 損壞的變更視為仍引用）＋已有結論＋無 hold」三條件與 `archive_discussion` 封存步。`conclude` 保留自己的「promoted_to 非空」觸發與 `closing_error` 包裝，改呼叫它；`archive()` 的討論 `filter_map` 段改為對每個來源 slug 呼叫它。
- **行為變化（唯一一處）**：變更封存的討論隨行封存路徑，對 meta 損壞的在途變更改為 fail-closed——該討論視同仍被引用、留在途不隨行封存。這是把 conclude 側既有的紀律對齊到 archive 側，`discussion-docs` 隨行封存要求 MODIFIED、補一條 scenario。

**相容性影響**：所有 CLI 人眼輸出與 `--json` 形狀逐位元不變（`DiscussionInfo` 的新欄位標 `serde(skip)`；server wire DTO 的 `promotedTo`／`concluded` 值來源改變但形狀與值不變；desktop 的 camelCase payload 不變）。可觀察差異：（1）封存一個 change 時，若另有在途 change 的 `.openspec.yaml` 解析失敗，來源討論從「被掃進封存區」變成「留在途」；既有使用者無需遷移（修好那份 meta 後再封存任一引用它的 change 即可收尾）。（2）既有缺陷的收斂：CRLF 行尾的討論記錄，promote／seal／`new change --from-discussion` 現在真的寫入 `promoted_to`（沿 CRLF）並清 `hold`，舊版在 CRLF 上落空；新插的 `promoted_to` 行落在 frontmatter 尾端而不是 `status:` 之後。golden 快照預期零變動。不涉及 CLI 子指令、旗標、stdin、exit code 的變更；不涉及設定欄位；不涉及生成技能。

## Non-Goals

（本 change 建 design.md，範圍排除與已否決做法記錄在 design 的 Goals／Non-Goals。）

## Capabilities

### New Capabilities

（none）——step 3 掃描：`discussion-docs` 已涵蓋討論記錄的全部行為（link／seal／conclude／hold／隨行封存），`change-lifecycle` 與 `archive-merge` 涵蓋封存流程本身；本 change 沒有新的可觀察能力，`KeyLines` 與 `DiscussionHead` 是內部型別。

### Modified Capabilities

- `discussion-docs`: 「討論以 link 動詞併入既有變更」要求中的變更封存隨行封存段——在途變更 meta 損壞時視同仍引用，該討論不隨行封存（補一條 scenario）；其餘要求不變。

## Impact

- Affected specs: `discussion-docs`（MODIFIED 一條要求）
- Affected crates／apps: speclink-core（主要）、speclink-host、speclink-server、speclink-cli、apps/desktop/core
- Affected code:
  - New: crates/speclink-core/src/keylines.rs
  - Modified: crates/speclink-core/src/lib.rs（註冊 keylines 模組）、crates/speclink-core/src/discuss.rs（DiscussionHead、close_if_finished、四支寫入動詞、info_from_doc、刪五支探測函式與 replacen）、crates/speclink-core/src/archive.rs（討論隨行封存段改呼叫 close_if_finished）、crates/speclink-host/src/bridge.rs（刪 discussions_extras）、crates/speclink-server/src/routes.rs（discussion_dtos_with_extras 改名 discussion_dtos、改讀 info 欄位）、crates/speclink-cli/src/verbs/discuss.rs（to_discussion_info 的結構體字面補新欄位）、apps/desktop/core/src/discussions.rs（entry 與 board_sorted_active 改讀 info 欄位）
  - Removed: （無檔案刪除；刪除的是函式）
- 測試面：discuss.rs 與 archive.rs 既有測試不改斷言；新增 keylines.rs 單元測試、archive.rs 一條「meta 損壞的在途變更擋住隨行封存」測試、discuss.rs 的 head 轉移規則測試。
