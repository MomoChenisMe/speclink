---
topic: 多刀系列的討論記錄一直被 change 封存時連帶封存
slug: multi-cut-discussion-hold
status: promoted
created: 2026-09-10
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: discussion-hold-until-release
---

# Discussion: 多刀系列的討論記錄一直被 change 封存時連帶封存

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

**起因**：`improve-lifecycle-layer` 的結論拆成四刀依序立案。每立一刀，前一刀封存時討論記錄就被隨行封存，下一刀要 `--from-discussion` 時記錄已不在途，只能手動從 `openspec/discussions/archive/` 搬回（本系列被連帶封存三次、手動搬回兩次，其中一次已 commit 要用 git mv）。

**病因（已查證，2026-09-10 main）**：`--hold` 的設計前提是「先轉出刀 A，再 conclude --hold」；本系列反過來「先 conclude --hold 再轉出刀一」，而 `DiscussionHead::promote`（discuss.rs:180）對新名字一律清 hold，旗標在轉出刀一時就消失。要補救得每刀封存前重跑 `conclude --hold`，而 conclude 會經 `stamp_restale` 對在途刀蓋 restale 章、還得再 seal 清掉。手續寫在原討論 Round 8，實務每次都忘。

**需求已夠銳利**（可驗證：多刀系列從頭到尾不用手動搬檔），未經 grill 直接進 assumptions。

**相關規格**：discussion-docs（「conclude 以 --hold 保留討論在途」的「轉出清除旗標」scenario 與「分期兩刀的生命週期」Example；「討論以 link 動詞併入既有變更」的隨行封存三條件）、discuss-skill（「分期轉出帶 --hold」scenario）。相關變更：在途 `lifecycle-archive-gate-owner` 正在改 archive.rs、command/mod.rs、verbs/lifecycle.rs，本案改動面要避開這三檔。
Prior discussions: discussion-planned-spinout-hold, discussion-auto-archive-before-conclusion, discuss-recall-archived-discussions

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-10)

**Focus**: 多刀系列的記錄要怎麼活到最後一刀，而不靠每刀重跑一套手續？六條假設全數由使用者確認。
**Position**: 走 A1——轉出不再清 hold，hold 只由「不帶 --hold 的 conclude」或手動 `discuss archive` 解除。
- 病因是順序：`--hold` 設計前提是「先轉出刀 A 再 conclude --hold」；本系列先 conclude --hold 再轉出刀一，`DiscussionHead::promote`（discuss.rs:180）對新名字清 hold，旗標從沒生效
- 決策樹：A 改 hold 語意（A1 轉出不清／A2 計數／A3 promote 加 --last）、B 只改手續順序、C reopen 動詞、D 只改技能文字
- A1 改動面只有 `promote` 一行加測試與 doc 註解（discuss.rs:806、:822），避開在途 change 正在改的 archive.rs、command/mod.rs、lifecycle.rs
- 代價：最後一刀封存時不再自動隨行封存，改由使用者 `discuss archive` 一次；忘記的後果從「記錄被吞、git mv 搬回」變成「記錄多留在途、看板看得到」，整個系列只做一次
- 推翻 09-07 討論「旗標在 mark_promoted 一處清掉」的決策，理由：該設計只涵蓋「先轉出再 hold」的順序，對「先 hold 再轉出」的系列無效
- 要 MODIFIED 的正典：discussion-docs scenario「轉出清除旗標」（spec.md:808）、Example「分期兩刀的生命週期」（:813）；discuss-skill scenario「分期轉出帶 --hold」（:235）；技能文字縮成一句「多刀系列 conclude --hold 一次，最後一刀封存後 discuss archive 收尾」
**Ruled out**: A2 計數——多一種狀態，09-07 否決理由仍成立；A3 promote 加 --last——多一個入口，只為保住最後一刀自動收尾；B 只改手續順序——先 promote 再 conclude --hold 會蓋 restale 章再 seal，仍是三步、一樣會忘；D 只改技能——每刀三步手續是遺忘來源，文字救不了
**Open**: C reopen 動詞要不要當安全網（09-07「手動搬檔即可救」的理由已變弱，但 A1 落地後只剩「一開始忘帶 --hold」會走到）；Desktop 看板上「已轉出但還欠一刀」的討論被收進「已轉出」收合列，使用者會以為討論結束了，怎麼呈現？

### Round 2 — interview (2026-09-10)

**Focus**: Desktop 看板與系統匣上，「已轉出但還欠一刀」的討論該怎麼呈現？
**Position**: 收合列與 tray「已轉出」分區的規則改成「引擎會自動收走的討論才收」——promoted 且已結論且無 hold；帶 hold 的討論留看板上區全卡（標「已轉出・保留中」）與 tray「討論」分區。使用者確認。
- 使用者情境拆成兩種：「先轉出 A、B 繼續討論（未結論）」已由 08-31 討論解掉——留上區全卡標「已轉出・尚無結論」（`DiscussionColumn.tsx:105` 的 `isCollapsedPromoted` 只判 promoted 且 concluded）；「結論已寫、帶 hold、還欠一刀」未解——收進收合列，與「做完等封存」長得一樣
- 根因：`hold` 沒上資料鏈——protocol `DiscussionInfo`（query.rs:580）只有 `concluded` 沒有 `hold`，看板不可能知道
- 資料鏈比照 `concluded`：core `DiscussionInfo.head.hold` → server route（routes.rs:1538 組裝處）→ protocol 加 `hold: Option<bool>`（缺席＝未知，不當 false）→ ui adapter `DiscussionItem` → `isCollapsedPromoted` 加 `hold !== true` → i18n 加「已轉出・保留中」；KanbanBoard 拖排落點清單共用判準，跟著變
- tray：`tray.ts:404` 的 `promoted` 是單一導出布林，TrayPanel 兩個分區與原生選單都吃它，加 `hold !== true` 一處即可；tray 討論列只顯示 slug 與 topic、無狀態標籤，held 討論回到「討論」分區就夠，不加標籤
- 「A、B 都結論但 B 依賴 A 還沒 propose」正是 conclude 該帶 --hold 的情境，帶了就留上區；規則與 A1 對齊：hold 整個系列都在，記錄整個系列都留上區
- 接受的取捨：上區全卡不列衍生 change 清單，held 討論的衍生進度改在詳情面板看
- 改動面都不碰在途 `lifecycle-archive-gate-owner` 鎖住的 archive.rs、command/mod.rs、verbs/lifecycle.rs
**Ruled out**: 只在收合列加「保留中」小標（09-07 原提案）——討論仍收在欄底、要展開才看得到，疑惑沒消；用 promoted_to 的 change 狀態推論「還欠一刀」——刀 N 進行中時推不出來，截圖正是這個時候
**Open**: C reopen 動詞要不要當安全網（留結論裁定）

## Conclusion

**Decision**: 兩件事一刀落地。（1）引擎：hold 語意改成「轉出不清 hold」——`DiscussionHead::promote`（discuss.rs:180）對新名字不再清旗標；hold 只由「不帶 --hold 的 conclude」或手動 `speclink discuss archive` 解除。`close_if_finished` 三條件不動。代價是最後一刀封存時不再自動隨行封存，改由使用者 `discuss archive` 一次收尾；忘記的後果從「記錄被吞、git mv 搬回」變成「記錄留在途、看板看得到」。（2）Desktop：看板欄底「已轉出」收合列與系統匣「已轉出」分區的規則改成「引擎會自動收走的討論才收」——promoted 且已結論且無 hold；帶 hold 的討論留看板上區全卡、標「已轉出・保留中」（與「已轉出・尚無結論」並列），tray 回到「討論」分區、不加標籤。資料鏈比照 concluded：core `DiscussionInfo.head.hold` → server route（routes.rs:1538）→ protocol `DiscussionInfo` 加 `hold: Option<bool>`（缺席＝未知）→ ui adapter → `isCollapsedPromoted`（DiscussionColumn.tsx:105）與 `tray.ts:404` 各加 `hold !== true`。正典同步：discussion-docs 的 scenario「轉出清除旗標」與 Example「分期兩刀的生命週期」MODIFIED；discuss-skill 的 scenario「分期轉出帶 --hold」MODIFIED，技能文字縮成「多刀系列 conclude --hold 一次，最後一刀封存後 discuss archive 收尾」，並補一句救援路徑「誤封存時把檔案從 discussions/archive/ 搬回 discussions/ 即可續用」。改動面避開在途 `lifecycle-archive-gate-owner` 鎖住的 archive.rs、command/mod.rs、verbs/lifecycle.rs。
**Rationale**: 09-07 的 hold 設計只涵蓋「先轉出再 hold」的順序；多刀系列是「先 hold 再轉出」，旗標在轉出刀一時就被清掉，每刀都得重跑 conclude --hold 加 seal 清 restale，實務每次都忘。把清除點從「下一次轉出」改成「使用者明示解除」後，整個系列只做一次 hold、一次 archive，忘記的代價變便宜且看得到。看板規則對齊同一條線：hold 在，討論就還沒完，留在上區。
**Rejected alternatives**: A2 hold 帶計數——多一種可寫錯的狀態，09-07 否決理由仍成立；A3 promote 加 --last——多一個入口，只為保住最後一刀自動收尾；B 只改手續順序（先 promote 再 conclude --hold 再 seal）——仍三步、一樣會忘；D 只改技能文字——三步手續是遺忘來源，文字救不了；只在收合列加「保留中」小標——討論仍收在欄底、要展開才看得到；用 promoted_to 的 change 狀態推論「還欠一刀」——刀 N 進行中時推不出來。推翻 09-07「旗標在 mark_promoted 一處清掉」的決策，理由如上。
**Deferred**: C `discuss reopen` 救援動詞——A1 落地後只剩「一開始就忘帶 --hold」會走到誤封存，且封存輸出會列出隨行封存的討論、當場看得見；先不立，再發生一次就立案。上區全卡不列衍生 change 清單，held 討論的衍生進度改在詳情面板看，先接受。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion multi-cut-discussion-hold
