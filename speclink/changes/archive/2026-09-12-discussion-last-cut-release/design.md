## Context

討論記錄的 hold 旗標（frontmatter `hold: true`）讓已結論的討論在「還欠尚未建立的變更」時留在途。現況（discussion-hold-until-release，2026-09-10）：三條轉出路徑——`speclink discuss promote`、`speclink new change --from-discussion`、`speclink discuss seal`——都經 speclink-core 的 `promoted_text`／`mark_promoted` 呼叫 `DiscussionHead::promote` 累加 promoted_to，且刻意不碰 hold；hold 只由不帶 --hold 的 conclude 或 `speclink discuss archive` 解除。隨行封存與 conclude 閉環共用同一支判準 `close_if_finished`（無在途引用、Conclusion 已有內文、無 hold），本變更不動它。

結果：多刀系列的最後一刀封存後，記錄留在途，使用者得自己判斷「全部做完了」再跑 `discuss archive`。`improve-repo-layout` 三刀全封存後記錄仍掛著，正是這個代價第一次落到使用者身上。來源討論 hold-countdown-auto-close 查證了兩件事：（1）引擎沒有任何訊號能分辨「三刀已是全部」與「刀四還沒立案」；（2）寫結論時的刀數不可靠——`improve-lifecycle-layer` 結論寫三刀、實際四刀，刀三在立案期拆成兩個變更，後半在前半封存之後才立案，若以刀數倒數會在前半封存時吞掉記錄。「這是最後一刀」只在立最後一刀的那一刻才確定。

wire 現況：`PromoteDiscussionRequest { name }`、`CreateChangeRequest { name, schema, description, agent, from_discussion }`、`BindDiscussionRequest { change }`（link 與 seal 共用）；server 三個端點經 Command gateway 直通 `Command::DiscussPromote`／`NewChange`／`DiscussSeal`；typed client `discussion_promote`／`create_change`／`seal_discussion`。桌面「轉為變更」按鈕經 `apps/desktop/core/src/discussions.rs` 直呼引擎 `discuss::promote`，不經 Command。

限制：在途變更無；正典 discuss-skill、improve-skill 與 render golden 目前寫的是「最後一刀封存後手動 discuss archive 收尾」。

## Goals / Non-Goals

**Goals:**

- 多刀系列的最後一刀封存時，記錄自動隨行封存，全程不跑 `discuss archive`，也不靠人在封存後判斷。
- 訊號在最晚、最確定的時刻交給引擎：立最後一刀時，由 propose 技能依結論比對 promoted_to 決定。
- 不加新狀態、不加新動詞、不改封存程式碼；不帶 `--last` 的一切行為逐位元不變。
- 本機與 remote 同旗標、同行為，唯一實作落點在引擎。

**Non-Goals:**

- 不用刀數倒數（`--hold N`）：結論時的數字擋不住立案期的拆刀（來源討論 Ruled out，附實例）。
- 不加獨立的調數／釋放動詞、不允許從已封存記錄轉出、不在每次 propose 問「是不是最後一刀」（來源討論 Ruled out）。
- discard 掉帶過 `--last` 的那一刀不回補 hold：記錄不記「哪一刀帶了旗標」，重立通常緊接在 discard 之後、其他刀在途即撐住記錄；否則 git mv（來源討論 Deferred）。
- 封存時對 held 討論不加提示行；看板不顯示「還欠幾刀」；桌面「轉為變更」按鈕不提供最後一刀選項。
- 不改 `close_if_finished` 三條件、archive.rs 的隨行封存段、conclude 請求、`DiscussionInfo.hold`、看板與系統匣分區規則。
- docs/ 使用者文件不動（多刀流程只活在技能文字；workflow.md 的 promote 語法列 `[--name <change>]` 已足，`--last` 由 help 觀察）。

## Decisions

### D1：最後一刀在轉出時標記，旗標掛在既有三條轉出路徑

`DiscussionHead::promote(change, last)` 增布林參數：`last` 為 true 時把 `hold` 設為 false 並標記重述，`write_back` 因此移除 `hold:` 行；`last` 為 false 時行為與現況完全相同。`promoted_text`／`mark_promoted`／`promote`／`seal` 逐層帶過 `last`；`link` 不經此路、記錄逐位元不變。CLI 三個子指令加 `--last`，`new change` 的 `--last` 以 clap `requires = "from_discussion"` 綁定。

替代方案與落敗理由：刀數倒數（結論時的快照擋不住立案期拆刀，lifecycle 系列實例會被吞）；獨立 `discuss hold <slug> N` 調數動詞（多一個入口只為修補倒數）；允許從已封存記錄轉出、取消 hold（「已封存」不再等於「已完成」，看板「已轉出・保留中」失去意義、封存輸出「Discussion archived」誤導）；每次 propose 問一次（每刀一問）；引擎自行推論（沒有訊號）。09-10 否決「promote 加 --last」的理由是「多一個入口，只為保住最後一刀自動收尾」——自動收尾現在就是需求本身，且旗標不是新動詞。

### D2：`--last` 對已在清單的名字同樣解除 hold

`DiscussionHead::promote` 的 `last` 不以「本次累加了新名字」為條件：名字已在 promoted_to（重複 promote、re-ingest 的 seal）時，`last` 為 true 仍移除 hold 行。理由：忘了在立最後一刀時帶旗標，`speclink discuss seal <slug> <change> --last`（seal 對已在清單的名字冪等）就是不經 conclude、不重蓋 restale 章的唯一補救路徑；且判斷少一層、實作更簡單。來源討論結論寫「名字已在清單時旗標無效」，propose 期依此理由改為解除，記於此。已封存的變更 seal 讀不到 meta 而失敗（既有），所以全部封存後的補救仍是 `discuss archive`。

### D3：wire 只加 `last`，唯一實作落點在引擎

`PromoteDiscussionRequest`、`CreateChangeRequest`、`BindDiscussionRequest` 各增 `last: bool`（`#[serde(default, skip_serializing_if = "<&bool>::not")]`、camelCase 鍵 `last`），舊 payload 缺席即 false。server 三個 handler 只把 `req.last` 塞進對應 `Command`，不判斷、不重讀檔；link 端點收同一型別但不讀 `last`（link 從不碰討論側）。typed client 三個方法各增 `last: bool` 參數。CLI remote 臂與 fs 臂共用同一個 clap 旗標。

替代方案與落敗理由：為 seal 另立 `SealDiscussionRequest`（為一個欄位分裂型別，client 與 handler 都要多一組）；`last` 走 query 參數（其他布林旗標如 conclude 的 hold 都在 body）。

### D4：propose 技能判定最後一刀；discuss 與 improve 技能改寫多刀收尾

propose.md 在「Create the change directory」的 `--from-discussion` 段規定：讀討論結論 Decision 段的刀清單（刀一／刀二／…或 cut A／cut B）與記錄 frontmatter 的 promoted_to；本次立的變更是結論規劃的最後一刀時，`speclink new change` 帶 `--last`；立案期把一刀拆成多個變更時，只有拆出的最後一段帶；結論沒規劃分期（單刀）時不帶——單刀記錄本來就不帶 hold，帶了也是無害的 no-op。discuss.md 的「At convergence」與「Mid-discussion spin-out」段、improve.md 的扇出段改為：多刀系列 conclude --hold 一次；最後一刀由 propose 帶 `--last`，最後一個封存自動隨行封存；忘了帶時記錄留在途，`speclink discuss archive <slug>` 一次收尾。刪掉「最後一刀封存後手動 discuss archive 收尾」的常態敘述；救援路徑（誤封存搬回）保留。

### D5：`--last` 不改任何輸出

三個轉出動詞帶 `--last` 時，人眼與 `--json` 輸出與不帶時逐位元一致：轉出從不報告 hold（09-10 既定），效果由記錄的 `hold:` 行消失與看板「已轉出・保留中」標籤消失呈現；最後一個封存的輸出多的是既有的「Discussion archived」行。

替代方案與落敗理由：`--json` 加 `released` 欄位（多一個沒有消費者的鍵；desktop 不呼叫帶旗標的轉出）。

## Implementation Contract

**引擎（speclink-core，`crates/engine/speclink-core/src/lifecycle/discuss.rs`、`crates/engine/speclink-core/src/command/mod.rs`）**

- 介面：`DiscussionHead::promote(&mut self, change: &str, last: bool) -> bool`（回傳值語意不變：新名字 true）；`promoted_text(store, slug, change, last)`、`mark_promoted(store, slug, change, last)`、`promote(store, slug, name, actor, last)`、`seal(store, slug, change, last)`；`Command::DiscussPromote { slug, name, last }`、`Command::DiscussSeal { slug, change, last }`、`Command::NewChange { …, last }`。
- 行為：對帶 `hold: true` 的記錄以 `last = true` 轉出後，promoted_to 累加（或不變，名字已在清單時）、status 為 promoted、`hold:` 行消失，其餘位元組不變；`last = false` 時記錄與現況逐位元相同。記錄無 hold 行時 `last = true` 是 no-op。`new change --from-discussion --last` 與 `promote --last` 先算記錄文字再建變更目錄（既有順序），建目錄失敗時記錄不落檔。
- 生命週期：帶 `--last` 轉出後，最後一個轉出變更封存時 `close_if_finished` 三條件成立、記錄移入 openspec/discussions/archive/，封存輸出列它；不論封存順序。
- 驗收：core 單元測試——`promote(change, true)` 對帶 hold 的記錄移除 hold 行且回 true、對無 hold 記錄逐位元不變、對已在清單的名字回 false 且移除 hold 行、CRLF 記錄同斷言（保留既有 CRLF 測試）；`cargo test -p speclink-core discuss::` 全綠。

**CLI（speclink-cli，`crates/adapters/speclink-cli/src/verbs/discuss.rs`、`crates/adapters/speclink-cli/src/verbs/new.rs`）**

- 介面：`speclink discuss promote <slug> [--name <change>] [--last] [--json]`、`speclink discuss seal <slug> <change> [--last] [--json]`、`speclink new change <name> --from-discussion <slug> [--last]`；`new change --last` 缺 `--from-discussion` 時 clap 拒絕（exit code 2、stderr 說明相依），不落檔。
- 行為：fs 與 remote 兩臂帶同一旗標；人眼與 `--json` 輸出與本變更前逐位元一致（帶或不帶 `--last` 皆同）。
- 驗收：CLI 整合測試（`crates/adapters/speclink-cli/tests/it/discuss_conclude_auto_archive.rs`）——`staged_spin_out_lifecycle_holds_across_every_cut_until_archived_by_hand` 改為三刀劇本：conclude --hold 一次，promote cut-a、cut-b 不帶旗標，promote cut-c 帶 `--last`；封存 cut-a、cut-b 後記錄留在途且封存輸出不列它，封存 cut-c 後記錄移入 archive/ 且封存輸出列它，全程不跑 discuss archive；新增測試斷言三路徑（promote、new change --from-discussion、link 後 seal）帶 `--last` 皆移除 hold 行、人眼與 --json 輸出與不帶旗標時相同；新增測試斷言 `seal --last` 對已在清單的名字移除 hold 行；`new change --last` 缺 `--from-discussion` 時 exit code 2 且不建目錄。remote_verb_parity 增一筆 `promote --last`：mock server 收到的請求 body 含 `last: true`，人眼輸出與 fs 模式同形。`cargo test -p speclink-cli --test it` 全綠。

**Protocol、remote client 與 server（`crates/protocol/speclink-protocol/src/command.rs`、`crates/protocol/speclink-remote/src/client.rs`、`crates/host/speclink-server/src/api/routes.rs`）**

- 資料形狀：三個請求型別的 `last` 為 camelCase 布林、serde default、false 不序列化；回應型別不變。
- 行為：POST /discussions/{slug}/promote、POST /changes、POST /discussions/{slug}/seal 把 `last` 直通引擎命令；POST /discussions/{slug}/link 忽略 `last`、記錄逐位元不變。判準與寫入只在引擎。
- 驗收：protocol 單元測試——三型別缺 `last` 的舊 payload 反序列化為 false、false 不出鍵、true 出鍵；server 整合測試（`crates/host/speclink-server/tests/it/api/discussion_routes.rs`）——對帶 hold 的討論以 `last: true` 呼叫 promote 端點後 GET /discussions 該筆 `hold: false`，不帶 `last` 時仍 `hold: true`；`cargo test -p speclink-protocol`、`cargo test -p speclink-server --test it discussion_routes` 全綠。

**桌面呼叫端（`apps/desktop/core/src/discussions.rs`）**

- 行為：`promote_discussion_at` 呼叫引擎 `discuss::promote` 傳 `last: false`；桌面轉出行為與輸出 JSON 不變。
- 驗收：`cargo test -p speclink-desktop-core` 全綠。

**技能 asset（`crates/engine/speclink-core/assets/skills/propose.md`、`discuss.md`、`improve.md`）**

- 行為：渲染後的 speclink-propose 技能檔（claude 與 codex）在 `--from-discussion` 建變更那一步含 D4 的判定規則與帶 `--last` 的指令範例；speclink-discuss 與 speclink-improve 技能檔含「最後一刀由 propose 帶 --last、最後一個封存自動收尾、忘帶時 discuss archive 一次」，不再含「最後一刀封存後手動 discuss archive 收尾」作為常態步驟。
- 驗收：ASSET_VERSION bump（`crates/engine/speclink-core/src/workspace/init.rs`）；`UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden::` 再生五份 golden；`UPDATE_ASSETS_LOCK=1` 再生 assets.lock；先 `cargo build` 再 `speclink update` 再生 SKILL.md；不帶環境變數的 render_golden 測試全綠。

**範圍邊界**

- 在範圍內：上述五組；六份規格 delta。
- 不在範圍內：`close_if_finished`、archive.rs、conclude 動詞與其 wire、`DiscussionInfo`、看板與系統匣、docs/、discard 的 hold 回補、桌面轉出的最後一刀選項。

## Risks / Trade-offs

- **回歸對照**：五份 render golden 與 assets.lock 必須在乾淨樹上再生，且 ASSET_VERSION 已 bump 才允許重寫 lock；bump 後先 `cargo build` 再 `speclink update`，否則 binary 帶舊版號。CLI 整合測試「分期生命週期」的期望值刻意改變（第三刀帶 `--last` 後自動收尾），屬本變更的行為變更，不是回歸；不帶 `--last` 的既有 pinned 輸出（discuss_promote_snapshot、remote_verb_parity）必須逐位元不變。
- **跨平台**：discuss.rs 的 CRLF 記錄測試保留 CRLF 斷言，新增的移除 hold 行斷言同樣以 CRLF 記錄跑一次；無新的路徑或 git 互動。
- **標錯的代價**：在不是最後一刀的變更上帶了 `--last`，記錄會在最後一個在途變更封存時被隨行封存，其後再立一刀要把記錄自 openspec/discussions/archive/ 搬回（引擎錯誤訊息即指向此路，技能文字保留救援句）。忘了帶則退回今天的行為（記錄留在途、看板標「已轉出・保留中」、手動 archive 一次），不遺失。
- **wire 相容**：三個請求多一個選填鍵；舊 server 忽略未知鍵（效果等同未帶 `--last`，記錄留在途，仍可手動 archive），舊 client 不送即 false。無回應型別變動。
- **版號波及**：ASSET_VERSION bump 會再生 `.claude/skills/` 與 `.agents/skills/` 下 37 份 SKILL.md，收尾 commit 靠 git status 盤點，不進 evidence。
- **簽名連鎖**：`discuss::promote`／`seal` 簽名改動會讓 desktop core 與 server 編譯失敗，直到呼叫端跟進——在同一刀內完成，`cargo build --workspace` 是守門。
