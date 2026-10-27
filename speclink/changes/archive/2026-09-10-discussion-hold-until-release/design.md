## Context

討論記錄的 hold 旗標（`hold: true` frontmatter 行，2026-09-07 的 discussion-spinout-hold 落地）讓已結論的討論在「還欠一個尚未建立的變更」時留在途。現況的清除點是「任何轉出動詞累加新變更名時」：`speclink discuss promote`、`speclink new change --from-discussion`、`speclink discuss seal` 都經 speclink-core 的 `DiscussionHead::promote` 累加 promoted_to，並在那裡把旗標清掉。這個設計假設「先轉出、再 conclude --hold」；多刀系列的實際順序是「先 conclude --hold、再轉出刀一」，旗標在轉出刀一時就被清掉，刀一封存時記錄仍被隨行封存。

隨行封存與 conclude 閉環共用同一支判準 `close_if_finished`（無在途變更引用、Conclusion 已有內文、無 hold），已於 lifecycle-discussion-head 收成一份，本變更不動它。

Desktop 端的討論列表資料鏈有兩條，終點共用：引擎 `DiscussionInfo`（`head` 內含 hold，`serde(skip)` 不進 CLI JSON）→ **remote 專案**經 server 的 GET /discussions 於 route 邊緣組 wire DTO、**本地專案**經 Tauri 的 `list_discussions` 指令由 `apps/desktop/core/src/discussions.rs` 自組同形 JSON（兩邊目前都只回填 promotedTo 與 concluded）→ packages/ui 的 `DiscussionItem` → 看板 `isCollapsedPromoted` 與 tray 快照的 `promoted` 布林。hold 沒有上鏈，看板與系統匣無從區分「還欠一刀」與「做完等封存」。

限制：在途變更 `lifecycle-archive-gate-owner` 正在改 crates/speclink-core/src/archive.rs、crates/speclink-core/src/command/mod.rs、crates/speclink-cli/src/verbs/lifecycle.rs，本變更不碰這三檔。

## Goals / Non-Goals

**Goals:**

- 多刀系列只做一次 `conclude --hold`，記錄從第一刀到最後一刀都留在途，不需每刀重跑手續。
- 使用者在看板與系統匣上一眼分得出「這份討論還會回來」與「做完等最後一刀封存」。
- 改動面避開在途變更鎖住的三個檔案。

**Non-Goals:**

- 不加 `discuss reopen`／unarchive 救援動詞（來源討論 Deferred：再發生一次誤封存再立案）。
- 不改 `close_if_finished` 的三條件、不改 archive 的隨行封存邏輯。
- 不加 hold 的計數或 promote 的 `--last` 旗標（來源討論 Ruled out）。
- 上區全卡不列衍生 change 清單；帶 hold 的討論其衍生進度改在詳情面板看。
- 系統匣討論列不加「保留中」標籤（列只顯示 slug 與 topic）。
- CLI 的 `discuss list --json` 不增 hold 欄位（引擎 JSON 逐位元不變的既定原則）。

## Decisions

### D1：hold 只由使用者明示解除，轉出不清

`DiscussionHead::promote` 累加新變更名時不再改 hold。清除點剩兩個，都是使用者明示：不帶 --hold 的 `conclude`（既有：重述意圖）與手動 `discuss archive`（既有：無視旗標）。

替代方案與落敗理由：hold 帶計數（寫 3 實際立 2 就永遠不自動封，多一種可寫錯的狀態）；promote 加 `--last` 旗標（多一個入口，只為保住最後一刀自動收尾）；只改技能手續順序為「先 promote 再 conclude --hold 再 seal」（conclude 會對在途刀蓋 restale 章、再 seal 清掉，仍三步、一樣會忘）；只改技能文字（三步手續是遺忘來源）。

代價：帶 hold 的討論在最後一刀封存時不再自動隨行封存，使用者跑一次 `speclink discuss archive <slug>`。忘記的後果從「記錄被吞、git mv 搬回」變成「記錄留在途、看板看得到」，且整個系列只做一次。

### D2：分區規則對齊「引擎會自動收走的討論才收合」

看板欄底「已轉出」收合列與系統匣「已轉出」分區只收 promoted 且已結論且無 hold 的討論——這正是最後一刀封存時 `close_if_finished` 會自動封存的集合。帶 hold 的討論留看板上區全卡（狀態標「已轉出・保留中」）與系統匣「討論」分區。規則與 D1 對齊：hold 整個系列都在，記錄整個系列都留在上區。

替代方案與落敗理由：只在收合列加「保留中」小標（討論仍收在欄底、要展開才看得到，疑惑沒消）；用 promoted_to 的 change 狀態推論「還欠一刀」（刀 N 進行中時推不出來）。

### D3：hold 比照 concluded 上鏈，唯一實作落點在引擎

protocol `DiscussionInfo` 增選填 `hold`（`Option<bool>`、`serde(default, skip_serializing_if = "Option::is_none")`、camelCase）；server 的 `discussion_dtos` 自引擎 `info.head.hold` 恆填 `Some(true|false)`。判準只在引擎（head 解析）；組裝端純轉手、不重讀檔；ui 與 desktop 只消費。缺席＝未知，client 視同「不保留」沿既有分區，不得把缺席當 true。

**兩個組裝端，不是一個**：remote 專案經 server 的 route 邊緣（`crates/speclink-server/src/routes.rs`），本地專案經 Tauri 指令的 `entry()`（`apps/desktop/core/src/discussions.rs`）——後者比照隔壁的 `concluded` 多填一鍵即可。兩端都吃同一個引擎 `head.hold`，判準仍只有一份。CLI remote 模式的 wire 回填（verbs/discuss.rs 的「hold 與 board_rank 取預設」）不需改，CLI 不顯示 hold。

### D4：技能文字縮成一句話，並補救援路徑

discuss.md 與 improve.md 的分期轉出段改為：「結論規劃多刀依序立案時，conclude 帶 --hold 一次；hold 直到不帶 --hold 的 conclude 或手動 `discuss archive` 才解除；最後一刀封存後跑 `speclink discuss archive <slug>` 收尾」。刪掉「旗標由下一次轉出清除」與「discard 剛清掉旗標的變更不會還原旗標、要重跑 conclude --hold」（D1 後 discard 不再有旗標可清）。補一句：「討論被誤封存時，把檔案從 openspec/discussions/archive/ 搬回 openspec/discussions/ 即可續用」。改 asset 內文走 ASSET_VERSION、render golden、assets.lock 三連動；`.claude/skills/` 與 `.agents/skills/` 的 SKILL.md 由 `speclink update` 再生。

### D5：schema 的 tasks 指引註明 design 標題可只寫編號

內建 spec-driven schema 的 tasks instruction（fork.schema.yaml 的 Cross-referencing 段）第二句改為：design.md 存在時，每個 `###` 標題的本文或其編號（D1、決策一、Decision 1 任一樣式）至少出現在一個任務描述中即算引用。第一句（需求名以子字串出現在任務描述）不動。理由：analyzer 已於 analyze-rule-tuning 接受編號，指引不講代理人就不會用，任務描述繼續抄整串標題，且 design 改一字 tasks 就跟著紅。順手併入本變更是因為改動只有一行加一個斷言測試；schema 內文不進 golden 與 assets.lock，不需版號連動。

替代方案與落敗理由：單獨開 change（一行字，流程成本大於內容）；同時把規則抄進 propose 技能本文（規則會有兩份，schema 指引才是單一來源）。

## Implementation Contract

**引擎（speclink-core，`crates/speclink-core/src/discuss.rs`）**

- 行為：對 frontmatter 帶 `hold: true` 的討論執行 `speclink discuss promote`、`speclink new change --from-discussion`、`speclink discuss seal`（累加新名字）後，promoted_to 累加該名字、`hold: true` 行逐字保留、status 為 promoted。已在清單的名字重跑：記錄逐位元不變（既有）。
- 介面：`DiscussionHead::promote(&mut self, change) -> bool` 簽名不變；回傳值語意不變（新名字 true）；不再改 `hold` 與 `hold_restated`。`mark_promoted`／`promoted_text` 的 doc 註解改寫為「轉出不清 hold」。
- 生命週期：帶 hold 的討論其所有轉出變更封存後，`close_if_finished` 因 hold 條件回 `Ok(None)`，記錄留在 openspec/discussions/；`speclink archive <change>` 的隨行封存清單不列它（既有行為，因旗標未被清）。`speclink discuss archive <slug>` 無視旗標移入 archive/（既有）。不帶 --hold 的 conclude 移除旗標並走閉環（既有）。
- 驗收：core 單元測試 `head_promote_new_change_clears_hold_and_returns_true` 改為「保留 hold 且回 true」；`mark_promoted_clears_the_hold_flag`、`link_leaves_the_hold_flag_untouched_and_seal_clears_it`、`mark_promoted_lands_promoted_to_on_a_crlf_record_and_clears_the_hold`、`head_conclude_and_promote_restate_the_hold_line_even_when_the_flag_value_is_unchanged` 改期望為旗標保留；CLI 整合測試 `staged_spin_out_lifecycle_holds_then_releases_the_record` 改為「刀 b 封存後記錄仍在途、`discuss archive` 後移入 archive/」；新增「promote 三路徑皆保留旗標」的 CLI 測試。`cargo test -p speclink-core --test it`、`cargo test -p speclink-cli --test it` 全綠。

**技能 asset（`crates/speclink-core/assets/skills/discuss.md`、`improve.md`）**

- 行為：渲染後的 speclink-discuss 與 speclink-improve 技能檔（claude 與 codex 兩工具）含 D4 的文字；不再含「the next spin-out clears the flag」與「Discarding the cut that cleared the flag does not bring the flag back」。
- 驗收：ASSET_VERSION bump；`UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden::` 再生五份 golden；`UPDATE_ASSETS_LOCK=1` 再生 crates/speclink-core/tests/golden/assets.lock；先 `cargo build` 再 `speclink update` 再生 SKILL.md；render_golden 測試全綠。

**schema 指引（`crates/speclink-core/assets/schema/spec-driven/fork.schema.yaml`、`crates/speclink-core/src/schema.rs`）**

- 行為：`speclink instructions tasks --change <name> --json` 的 instruction 欄位，Cross-referencing 段第二句含「或其編號（D1 等）」語意；第一句與其餘內容逐字不變。`speclink schema fork spec-driven` 的輸出與正典逐位元相同（既有守門）。
- 驗收：schema.rs 內容斷言測試（比照既有「fork.schema.yaml 須點名 `[M]` markers」）斷言該句含編號說明；`cargo test -p speclink-core schema::` 全綠。

**Protocol 與 server（`crates/speclink-protocol/src/query.rs`、`crates/speclink-server/src/routes.rs`）**

- 資料形狀：`DiscussionInfo.hold: Option<bool>`，JSON 鍵 `hold`，缺席省略。GET /discussions 每筆恆含 `hold: true` 或 `hold: false`；GET /discussions/{slug} 的 info 同。
- 驗收：protocol 序列化測試（true／false 出鍵、舊 payload 無鍵反序列化為 None、再序列化無鍵）；server 整合測試 `list_discussions_carries_concluded_for_every_record` 旁新增 hold 版本：一筆帶 `hold: true` 的記錄回 true、無 hold 行的回 false。`discuss list --json` 本地 CLI 輸出逐位元不變。

**看板（`packages/ui/src/adapter.ts`、`components/DiscussionColumn.tsx`、`i18n.tsx`）**

- 資料形狀：`DiscussionItem.hold?: boolean`，缺席＝未知。
- 行為：`isCollapsedPromoted(d)` 為 `d.status === "promoted" && d.concluded !== false && d.hold !== true`。promoted 且 concluded 為 true 且 hold 為 true 的討論呈上區全尺寸卡，狀態標「已轉出・保留中」（英文 "Promoted · on hold"），warning 語意色（與「已轉出・尚無結論」同色系），無任何動詞按鈕；欄計數徽章計入。promoted 且 concluded 為 false（不論 hold）維持「已轉出・尚無結論」。KanbanBoard 的拖排落點清單共用 `isCollapsedPromoted`，上區卡集與可拖集恆等。
- 驗收：discussionColumn.test.tsx 新增「帶 hold 的已結論 promoted 討論留上區、標保留中、無動詞、徽章計入、收合列不列」與「hold 缺席時沿既有分區」；kanban.test.tsx 的可拖集斷言跟著更新。`npm run test -w packages/ui` 全綠。

**本地列表組裝（`apps/desktop/core/src/discussions.rs`）**

- 資料形狀：`entry()` 的 JSON 恆含 `hold` 鍵（true／false），值取自引擎 `info.head.hold`，位置與樣式比照隔壁的 `concluded`。
- 驗收：該模組新增測試——帶 `hold: true` 的記錄回 true、無 hold 行的回 false；`cargo test -p speclink-desktop-core` 全綠。

**系統匣（`apps/desktop/src/adapter/remoteDataSource.ts`、`apps/desktop/src/tray.ts`）**

- 資料形狀：wire 型別增 `hold?: boolean`，`toDiscussionItem` 原樣帶過；本地路徑的 `DiscussionLists` 由 Tauri 指令直出，型別已含此欄。
- 行為：tray 快照 `promoted` 為 `promotedTo.length > 0 && concluded !== false && hold !== true`。帶 hold 的已結論 promoted 討論列於「討論」分區、計入「討論 N」；不在「已轉出」分區。TrayPanel 與原生選單都吃同一布林，無各自判準。
- 驗收：tray.test.ts 既有「promoted 且 concluded 非 false 才歸已轉出」測試擴一筆 `hold: true` 案例；trayPanel.test.tsx 對應分區斷言。`npm run test -w apps/desktop` 全綠。

**範圍邊界**

- 在範圍內：上述七組；六份規格 delta。
- 不在範圍內：archive.rs／command/mod.rs／verbs/lifecycle.rs；`close_if_finished`；討論詳情面板（DiscussionDrawer）的呈現；CLI 人眼與 JSON 輸出；reopen 動詞。

## Risks / Trade-offs

- **回歸對照**：五份 render golden 與 assets.lock 必須在乾淨樹上再生，且 ASSET_VERSION 已 bump 才允許重寫 lock；bump 後要先 `cargo build` 再 `speclink update`，否則 binary 帶舊版號。CLI 整合測試 `staged_spin_out_lifecycle_holds_then_releases_the_record` 的期望值刻意改變，屬本變更的行為變更，不是回歸。remote_verb_parity 的 conclude --hold 對照不受影響（conclude 行為不變）。
- **跨平台**：discuss.rs 的 CRLF 記錄測試（`mark_promoted_lands_promoted_to_on_a_crlf_record_and_clears_the_hold`）改期望時保留 CRLF 斷言；無新的路徑或 git 互動。
- **行為變更的可見性**：既有使用者若依賴「最後一刀封存自動收走討論」，現在要手動 archive 一次；技能文字與封存後看板上留著的卡都會提醒。若真忘了，記錄留在途、不會遺失。
- **版號波及**：ASSET_VERSION bump 會再生 `.claude/skills/` 與 `.agents/skills/` 下 37 份 SKILL.md，收尾 commit 要靠 git status 盤點，不進 evidence。
- **平行變更**：`lifecycle-archive-gate-owner` 若先封存並改動 discuss.rs 周邊（例如把守門鏈收進 archive()），本變更的 discuss.rs 改動只在 `DiscussionHead::promote` 與 doc 註解，衝突面小；合併時以重跑測試為準。
