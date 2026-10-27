## Context

`archive()` 的順序契約由三條規格釘住：change-lifecycle「單筆封存的任務完成度守門」（拒絕時 SHALL NOT 改動任何檔案；帶 `--mark-tasks-complete` 時先全勾再封存；守門 SHALL 於引擎封存流程本體生效、一體適用 CLI 與桌面）、「封存的章失效守門」（任務守門之後、任何封存檔案效果之前；`--mark-tasks-complete` 路徑 SHALL 於前置全勾寫入之前判定章失效）、archive-merge「兩階段合併計畫與零半套寫入」（驗證階段的任何違規 SHALL 使封存在零檔案效果下結束）。三條都只要求「拒絕路徑 tasks.md 逐位元不變」，沒有一條要求代勾先於 validate 或計畫階段。

今天的代勾在 `run_archive`，位於 `archive()` 之前。為了讓 merge 守門的拒絕也零寫入，`run_archive` 得在代勾前自己跑一次 `merge_violations`——但 `archive()` 內的同日撞名與結構 validate 兩道它沒抄，所以那兩道拒絕時 tasks.md 已經被勾了。這不是設計上的取捨，是抄不全的後果。

`archive()` 今天的順序：`guard_linked_worktree` → `require_valid_meta` → `guard_open_tickets` → 任務完成度（`!opts.mark_tasks_complete` 時判）→ `guard_stale_stamps` → 同日撞名 → 結構 validate（`!opts.no_validate`）→ 讀 evidence → 計畫階段（讀 delta、`capability_violations`、算合併文字；`!opts.skip_specs`）→ `merge_refusal` → 提交階段（snapshot → 正典寫回 → 目錄搬移 → archived_* 蓋章 → 討論隨行封存）。

bulk 今天自己算三個跳過條件：`merge_violations`（`!skip_specs`）、`validate_change_structural`（`!no_validate`）、`tasks::progress`（`!mark_tasks_complete`），命中第一條就 `Skipped: <name> — <why>` 並 `continue`；然後對每個 change 發 `Command::Archive`，`Err` 即中止並印三段報告。

## Goals / Non-Goals

**Goals:**

- 守門順序契約只在 `archive()` 一份；CLI 單筆、CLI bulk、server、desktop 四個入口語意等價。
- `--mark-tasks-complete` 的代勾成為「所有守門與純讀計畫全過之後」的第一個寫入；任何拒絕路徑 tasks.md 逐位元不變。
- bulk 的跳過判定讀 `archive()` 同一組守門的唯讀投影，不再自算。
- 所有凍結字串（`archive()` 的六種拒絕、bulk 的三段跳過理由、三段中止報告）逐字不變。

**Non-Goals:**

- 不改 bulk 的「跳過 vs 中止」分類：三種跳過、其餘中止，是刻意設計。
- 不改 `merge_violations` 的公開性與語意（validate／drift 共用）。
- 不改 desktop `archive_with` 與 server 路由的參數（它們不傳 `mark_tasks_complete`）。
- 不動 in-progress 標記、evidence 提示、討論隨行封存、`archived_*` 蓋章。
- 已否決做法（討論 Round 5）：代勾放在 `archive()` 開頭（違反零效果要求）；代勾留在 `run_archive` 只刪重複守門（merge 守門仍得外面先跑，重複刪不掉）；bulk 直接呼叫 `archive()` 吃錯誤決定跳過（凍結短字串會換成 archive 的長字串、跳過 vs 中止靠錯誤型別分類不可靠）；`skip_reason` 依 `archive()` 順序回報（多條同時成立時 bulk 的跳過行會變，凍結輸出破）。

## Decisions

### D1 代勾寫入落在計畫階段之後、提交階段的第一個效果

在 `archive()` 中，`merge_refusal` 判定之後、snapshot 寫入之前插入：

```rust
// --- 提交階段的第一個效果：--mark-tasks-complete 的代勾。所有守門與純讀計畫已過，
// 從這一行起才有檔案效果（spec change-lifecycle「單筆封存的任務完成度守門」、
// 「封存的章失效守門」；archive-merge「兩階段合併計畫與零半套寫入」）。
if opts.mark_tasks_complete {
    if let Some(text) = store.read_artifact(&change.name, "tasks.md") {
        let done = text.replace("- [ ] ", "- [x] ").replace("- [ ]\t", "- [x]\t")
                       .replace("* [ ] ", "* [x] ").replace("* [ ]\t", "* [x]\t");
        store.write_artifact(&change.name, "tasks.md", &done)?;
    }
}
```

- 四條 `replace`（含星號條列與 tab，凍結規則）逐字從 `run_archive` 搬過來；`[M]` 任務同樣被代勾（既有語意：旗標＝「全部視為完成」，章失效守門已在前面擋掉不該代勾的情境）。
- 位置在 snapshot 之前：snapshot 是 unarchive 的備份，與 tasks.md 無關；代勾先落，接著 snapshot → 正典 → 目錄搬移，搬走的是已全勾的 tasks.md（與今天封存結果相同）。
- 失敗狀態：代勾寫入失敗 → `Err` 回傳，此時正典未動、無 snapshot、change 目錄原位、tasks.md 可能已寫（單一寫入，不是半套）；與今天 `run_archive` 代勾失敗的可觀察狀態相同。
- 比今天更強的保證：同日撞名、結構 validate、merge 計畫三道拒絕時 tasks.md 逐位元不變（今天前兩道拒絕時已被勾）；desktop 若未來帶旗標直呼 `archive()`，同樣受保護。

### D2 run_archive 縮為 resolve_change 加 archive()

```rust
fn run_archive(store, ws, actor, change, opts) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_FLAG)?;
    let host = host_workspace(ws);
    let outcome = crate::archive::archive(&host, store, &change, &opts, actor).map_err(classify)?;
    Ok(CommandOutcome::Archive(outcome))
}
```

- `guard_meta` 也刪：`archive()` 第二道 `require_valid_meta` 回 `MetaError`，`classify` 映射為 `ErrorCode::InvalidConfig` 與 `invalid openspec/changes/<name>/.openspec.yaml: <reason>`——與 `guard_meta` 產生的 `CommandError` 完全相同（同一個 `classify`）。守門順序也相同：`archive()` 內 linked worktree 在 meta 之前，與今天 `run_archive` 一致。
- `Refusal` 型別的守門（worktree、未結工單、任務、章失效、merge）經 `classify` 為 `Refused`；`"Validation failed:\n…"`、同日撞名是 `bail!` → `Error`——與今天 `archive()` 段的分類相同，因為今天這些本來就在 `archive()` 內、經同一個 `map_err(classify)`。
- 「in-progress 標記於封存時不動」的凍結行為留在 `archive()`（它本來就不碰），註解搬過去。
- **實作時發現的第二個可觀察差異——merge 拒絕的位次**：今天 `run_archive` 的 merge 守門跑在 `archive()` 之前，所以 CLI 單筆與 server 路由對「delta 過期」的 change 一律先收到 merge 拒絕，即使它同時任務未完成、章失效、同日撞名或結構不合法；desktop 直呼 `archive()` 則先收到後面那四道的拒絕——同一個 change、兩種入口、兩種訊息。收斂後四個入口一律走 `archive()` 的既有順序：任務完成度 → 章失效 → 同日撞名 → 結構 validate → merge，merge 拒絕成為最後一道（規格只釘任務守門先於章失效守門，對 merge 守門與其他四道的先後無規定）。回歸測試：archive.rs 釘「任務」「章失效」「結構 validate」三道各先於 merge 拒絕，command 層釘「任務先於 merge」（CLI 單筆與 server 都經 `Command::Archive`）。CLI 測試 `validate_and_drift_name_the_same_stale_operation` 原本靠舊順序（fixture 任務未勾）拿到 merge 拒絕，改為先勾齊任務再單筆封存——它驗的是四處指向同一過期操作，不是守門序。

### D3 guard 函式縮回 archive.rs 私有

- `guard_linked_worktree`、`guard_open_tickets`、`guard_stale_stamps`、`merge_refusal`：`pub(crate) fn` → `fn`。刪掉三處「runtime 於前置寫入前先喚一次、archive 內再守一次」的註解，改為一句「守門只在此處」。
- `merge_violations` 維持 `pub fn`：validate.rs（change 驗證的合併守門）、drift.rs（Specs 維度）仍呼叫，規格「過期判定單源共用」點名四處共用同一實作——bulk 這一處改經 `skip_reason` 間接共用，仍是同一支。
- `capability_violations`、`parse_delta` 等 `pub(crate)` 不動（validate.rs 用）。
- `validate_change_structural` 順手收成 `pub(crate)` 並拿掉從未使用的 `_schema` 參數：bulk 不再直接呼叫它之後，crate 外零呼叫者，公開面成了本 change 造成的孤兒（review 站發現）。

### D4 bulk 改吃 skip_reason 唯讀投影、字串與優先序留在 bulk

```rust
// archive.rs
pub enum SkipReason {
    MergeRefused(Vec<MergeViolation>),
    StructuralInvalid,
    TasksIncomplete { complete: usize, total: usize },
}
/// bulk 預檢的唯讀投影：依 opts 三旗標的豁免語意，以 merge → validate → tasks 的固定
/// 順序判定，命中第一條即回、後面的條件不再算；零寫入。順序是 bulk 的凍結輸出契約，
/// 不是 archive() 的守門序。
pub fn skip_reason(store: &dyn Store, change: &Change, opts: &ArchiveOptions) -> Option<SkipReason>
```

- 條件與今天 bulk 逐字對齊：`!opts.skip_specs` 才算 `merge_violations`（非空→`MergeRefused`）；`!opts.no_validate` 才跑 `validate_change_structural(store, change, false)`（`!valid`→`StructuralInvalid`）；`!opts.mark_tasks_complete` 才算 `tasks::progress`（`total > 0 && complete < total`→`TasksIncomplete`）。任務條件與 `archive()` 內的任務守門共用一支私有 `incomplete_tasks(store, name) -> Option<TaskShortfall>`（私有具名結構 `{ complete, total }`，不靠 tuple 順序；`skip_reason` 再包成 `TasksIncomplete`，守門端的 `if let` 對 `Some` 內容不可反駁、不會靜默放行），兩處判斷同源。
- bulk 端：`impl From<&ArchiveArgs> for ArchiveOptions` 把五個旗標抄一次，`Command::Archive` 也從同一份 `opts` 組；對每個 change `skip_reason(...)`，渲染字串沿今天三段（`MergeRefused(vs)` → `"{n} delta operation(s) archive would refuse — run /speclink-drift {name}"` 加 Purpose 點名段；`StructuralInvalid` → `"validation failed"`；`TasksIncomplete` → `"tasks incomplete ({complete}/{total})"`），`continue` 語意不變；其後的 `Command::Archive` 派發、中止報告、結尾統計逐字不變。
- 回 `Option` 而非 `Vec`：唯一呼叫端只要第一條，回全部理由是沒被要求的彈性（CLAUDE.md「程式碼取捨」；review 站發現），而且 merge 已成立時還會白跑結構 validate 與任務計數。短路後與今天 bulk 的三段 `continue` 逐字等價。

### D5 零行為變化的驗證面

- **凍結輸出回歸網**：CLI `archive_readiness_gate.rs`（含 `refused_archive_leaves_the_mark_tasks_complete_pre_write_undone`、`mark_tasks_complete_archives_and_checks_every_task`、bulk 在 worktree 內拒絕）、`archive_merge_gate.rs`（bulk 預檢字串、Purpose 點名、`a_refused_archive_leaves_tasks_untouched_even_with_mark_tasks_complete`、`--skip-specs` 不預檢）、`manual_task_gates.rs`（`mark_tasks_complete_leaves_tasks_untouched_when_stale_refuses`、`bulk_archive_fails_fast_on_a_stale_stamp`）、`archive_evidence_gate.rs`；command 層 `archive_on_corrupt_meta_refuses_without_moving_or_merging`（mark 兩值皆零寫入）、未結工單拒絕零寫入；全部斷言不改。
- **一條既有測試的斷言要改**：archive.rs `mark_tasks_complete_flag_passes_the_gate_without_pre_write` 今天斷言「`archive()` 帶旗標不寫 tasks.md」（因為代勾在外面）；改後 `archive()` 自己代勾，斷言改為「封存後的 tasks.md 全部已勾」，測試名改為 `mark_tasks_complete_flag_checks_every_task_inside_archive`。這是內部函式行為的刻意變化，CLI 可觀察結果不變。
- **新增測試**：archive.rs「帶旗標、結構 validate 拒絕時 tasks.md 逐位元不變」（今天會失敗——是本 change 的收斂）、「帶旗標、同日撞名拒絕時 tasks.md 逐位元不變」、`skip_reason` 對三旗標各自豁免與 merge→validate→tasks 順序、「任務／章失效／結構 validate 三道先於 merge 拒絕」；command 層「`run_archive` 對 linked worktree／未結工單／壞 meta 的錯誤碼與訊息與改前相同」以既有測試釘住，另新增「任務守門先於 merge」一條。同日撞名測試把今天與明天兩個 dated_name 都放進封存區，跨午夜不 flaky。
- golden：`render_golden::` 零 diff。

## Implementation Contract

**可觀察行為**：`speclink archive <name>`（含 `--mark-tasks-complete`、`--carry-*`、`--skip-specs`、`--no-validate`）、`speclink archive --all`／多筆、server 封存路由、desktop 封存的 stdout／stderr／exit code／`--json`／payload 逐位元不變。兩條差異：（1）帶 `--mark-tasks-complete` 被同日撞名或結構 validate 拒絕時，tasks.md 逐位元不變（今天已被全勾）；（2）CLI 單筆與 server 路由的 merge 拒絕退到最後一道——「delta 過期」同時伴隨任務未完成、章失效、同日撞名或結構不合法時，先收到的是那四道的拒絕（見 D2）。

**介面**

- 新增：`archive::SkipReason`、`archive::skip_reason(store, &Change, &ArchiveOptions) -> Option<SkipReason>`。
- 變更：`validate::validate_change_structural` 收成 `pub(crate)`、移除未使用的 `_schema` 參數。
- 變更：`run_archive` 私有函式縮為兩步；`guard_linked_worktree`／`guard_open_tickets`／`guard_stale_stamps`／`merge_refusal` 由 `pub(crate)` 改私有。
- 不變：`archive::archive` 簽名與 `ArchiveOptions`／`ArchiveOutcome`；`merge_violations` 公開；`Command::Archive` 形狀；bulk 的 CLI 旗標。

**失敗模式**：`archive()` 六種拒絕的型別（`Refusal`／`MetaError`／`bail!`）與字串不變；代勾寫入失敗回儲存層錯誤、零正典效果；bulk 的跳過與中止語意不變。

**驗收條件**

1. `cargo test -p speclink-core`、`cargo test -p speclink-core --test it`（`render_golden::` 零 diff）全綠。
2. `cargo test -p speclink-cli --test it` 全綠（上列四個守門測試檔）。
3. `cargo test -p speclink-desktop-core` 全綠。
4. `grep -n 'guard_linked_worktree\|guard_open_tickets\|guard_stale_stamps\|merge_refusal\|merge_violations' crates/speclink-core/src/command/mod.rs` 零命中；`grep -n 'merge_violations\|validate_change_structural\|tasks::progress' crates/speclink-cli/src/verbs/lifecycle.rs` 零命中；`grep -n 'pub(crate) fn guard_' crates/speclink-core/src/archive.rs` 零命中。
5. `grep -n '"- \[ \] "' crates/speclink-core/src/command/mod.rs` 零命中（代勾四條 replace 只在 archive.rs）。

**範圍邊界**

- In：`archive.rs`（代勾、`SkipReason`／`skip_reason`、`incomplete_tasks`、可見度、測試）、`command/mod.rs` 的 `run_archive` 與其測試、`lifecycle.rs` 的 `cmd_archive_bulk`、`validate.rs` 的 `validate_change_structural` 簽名與可見度、CLI 測試 `validate_specs.rs` 的一條 fixture。
- Out：`archive()` 的其他階段、討論隨行封存、evidence、in-progress、desktop／server 呼叫端、validate.rs 的其他部分／drift.rs、規格。

## Risks / Trade-offs

- [守門順序或分類漂移] → `archive()` 內順序不動，只在計畫階段之後插入代勾；CLI 四個守門測試檔與 command 層測試釘住每一道的訊息、exit code 與零寫入。
- [bulk 跳過字串漂移] → 字串留在 bulk 端逐字不動；`archive_merge_gate.rs` 的 bulk 測試釘住 `Skipped:`、`archive would refuse`、`## Purpose` 點名。
- [代勾之後、目錄搬移之前的 I/O 失敗] → 與今天 `run_archive` 代勾後 `archive()` 失敗的狀態相同（tasks.md 已勾、其餘零效果）；規格允許的方向（tasks.md 全勾是使用者以旗標宣告的意圖）。
- [`guard_*` 改私有後 host／node 若有引用] → 已 grep：無 core 以外的呼叫者；改私有是編譯期即知。
- [golden／CLI 回歸] → 改碼前跑一次建立基準；任何 diff 視為缺陷。
- [跨平台] → 純流程搬家；`guard_linked_worktree` 的 git 判定與 Windows 路徑行為不動。

## Migration Plan

無資料遷移、無格式變化。部署即生效；回滾即還原程式碼。

## Open Questions

（無。這是本討論的最後一刀，封存時討論記錄隨行封存。）
