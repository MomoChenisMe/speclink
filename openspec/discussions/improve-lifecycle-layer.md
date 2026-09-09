---
topic: 討論與變更生命週期層的結構改善——討論 frontmatter 文字手術、討論收尾判準雙源、封存守門鏈三處重複、站別門面
slug: improve-lifecycle-layer
status: concluded
created: 2026-09-09
created_by: MomoChen <momochenisme@gmail.com>
kind: improve
hold: true
---

# Discussion: 討論與變更生命週期層的結構改善——討論 frontmatter 文字手術、討論收尾判準雙源、封存守門鏈三處重複、站別門面

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者於 2026-09-09 執行 `/speclink-improve`，未指定方向，範圍由 git 熱區推斷。

**熱區與排除**：近三個月與近一個月的 `git log --name-only` 統計，程式碼側最熱的是 `crates/speclink-core/src/init.rs`（29 次／月）＋四份 golden 快照，其次是 `command/mod.rs`（14）、`archive.rs`（12）、`discuss.rs`（8）、`validate.rs`（8）。Step 1 查 `speclink discuss search` 與 `speclink list`：
- `init.rs` 熱度來自剛封存的 `improve-workspace-sync`（刀 A `workspace-sync-plan`、刀 B `workspace-sync-entrypoints`，2026-09-07 封存），該區剛重構完，排除。
- CLI 動詞層已由 `improve-cli-command-layer`、`improve-wire-convert-seam`、`improve-cli-verb-layer`、`cli-verb-family-modules` 四份記錄處理完（最後一刀 `cli-typed-engine-entry` 2026-09-01 封存），排除；其 Deferred（server routes 約 20 個 wrong_outcome 臂、node SDK 8 處拆封遷到 core TryFrom）屬既定後續小刀，不重提。
- 在途變更只有 `release-changelog-whats-new`（desktop／release 面），與本次範圍不重疊。

**收斂後的範圍**：speclink-core 的「討論與變更生命週期」層——`discuss.rs`（3051 行）、`archive.rs`（2362 行）、`station.rs`／`review.rs`／`verify.rs`、`command/mod.rs` 的 `run_archive`。理由：扣掉已處理區後，近一個月四個封存變更（`conclusion-gated-discussion-archive`、`discussion-spinout-hold`、`discuss-search-recall`、`validate-merge-gate`）全落在這一層，且每個都同時動 `discuss.rs` 與 `archive.rs` 兩檔以上。

**再提案檢查**：以 frontmatter／型別化／門面／守門鏈／run_archive／掃除搜舊討論，無已否決方案。`archive-readiness-gating` 曾裁定 run_archive 補任務完成度守門（豁免沿 --mark-tasks-complete），是守門「存在」的裁定，不是守門「住在 run_archive」的裁定；`verify-station-parity` design D1 選了「command 層只認站別門面」的分層，候選 4 是對那個決定的回訪，在候選內明示。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — scan (2026-09-09)

**Focus**: 討論與變更生命週期層有哪些結構改善
**Position**: 這一層的兩份文件（討論記錄 `.md` 的 frontmatter、change 的 `.openspec.yaml`）都只有型別化的「讀」沒有型別化的「寫」，狀態轉移靠各動詞自己做字串手術；同一條生命週期規則因此散在兩到三處，近一個月每個 change 都得同時補多處。候選如下：

---

### 候選 1：討論 frontmatter 的狀態改由一個型別擁有，取代散在四支函式的字串取代

- **Files**：`crates/speclink-core/src/discuss.rs`（`frontmatter_value` :36、`mark_promoted` :652、`unlink_discarded` :710、`set_frontmatter_line` :982、`promoted_to_in`／`concluded_in`／`held_in` :1050-1081、`board_rank` :970、`conclude` :1138）；`crates/speclink-core/src/store.rs:25`（`DiscussionDoc` 只帶 raw text）；`crates/speclink-host/src/bridge.rs:119-126`；`apps/desktop/core/src/discussions.rs:16-27`。
- **Problem**：訊號 1（一個概念散在多處）＋訊號 4（格式知識外洩）。討論的狀態機（`status` open→concluded→promoted、`promoted_to` 逗號累加、`hold`）沒有型別擁有，每個動詞各自改文字：`mark_promoted` 三處 `replacen`、`unlink_discarded` 三處、`conclude` 一處直接 `text.replacen("status: promoted", …)`；而 `hold`／`board_rank` 走另一條 `set_frontmatter_line`（38957e63 才為 hold 補上的）——同一個 `status` 欄位有兩套寫法並存。`replacen("status: open")` 不限於 frontmatter，內文出現同字串就會被誤改；`escape_colliding_lines` :166 的存在證明「內文撞結構」已經是發生過的事故類型。讀取側：`DiscussionInfo` 被凍結（`discuss list --json` 逐位元不變），所以 `promoted_to`／`discussion_concluded`／`board_rank`／`discussion_held` 各自獨立讀檔——desktop `entry()` 每張卡讀 3 次檔，host bridge `discussions_extras` 自己抓 raw text 再呼叫 `promoted_to_in`／`concluded_in`，frontmatter 的文字格式知識已經外洩到 host crate。
- **Solution**：`discuss.rs` 內立一個 `DiscussionHead`：一次解析 frontmatter 為欄位（status 列舉、promoted_to `Vec`、hold、board_rank、kind、created、created_by…）並保留原始行序與未知鍵，提供改欄位＋回寫。`mark_promoted`／`unlink_discarded`／`conclude`／`set_board_rank` 一律「載入 head → 改欄位 → 回寫」；`DiscussionInfo` 維持 JSON 投影不變但由同一個 head 產出；bridge 與 desktop 改拿 head 欄位。`frontmatter_value`／`set_frontmatter_line`／`promoted_to_in`／`concluded_in`／`held_in` 五支併入型別。
- **Wins**：狀態轉移規則只有一份可讀；「status: open 出現在內文」的誤改在結構上不可能；desktop 討論列表每卡 4 次讀檔變 1 次；host crate 不再知道 frontmatter 長什麼樣；下一個新欄位（hold 是第三個）不必再選「replacen 還是 set_frontmatter_line」。刪除測試：五支文字探測函式與十來處 replacen 消失，工作集中到一個型別——複雜度集中，不是搬家。
- **Recommendation**：**strongly recommended**

### 候選 2：「討論何時結束生命」的判準只留一份

- **Files**：`crates/speclink-core/src/discuss.rs:1186-1200`（`conclude` 的 closing step）；`crates/speclink-core/src/archive.rs:770-796`（`archive()` 的討論隨行封存）；`discuss.rs:652-694`（`mark_promoted` 清 hold）。
- **Problem**：訊號 1，而且已經漂移。「無在途 change 引用＋已有結論＋未 hold」這條規則在兩處各算一次，公式不同：`conclude` 那份把 `c.meta_error.is_some()` 當作「仍被引用」（fail-closed），`archive.rs` 那份只看 `c.meta.from_discussions()`，meta 壞掉的 change 被當作不引用。兩處各有 10 行註解解釋同一個道理。`discussion-spinout-hold`（5a0e30b3）落地時就必須兩處各補一個 `held` 條件；下一個條件一樣要補兩處。
- **Solution**：`discuss.rs` 提供一支 `close_if_finished(store, slug) -> Result<Option<file>>`，內含引用檢查（fail-closed 版）、concluded、held 三條件；`conclude` 與 `archive()` 都改呼叫它。`archive()` 端「排除自己」的需求現況已由「change 已搬走、`list_changes` 看不到」成立，不需額外參數。注意這會讓 archive 路徑對 meta 壞掉的 change 也 fail-closed（行為對齊 = 規格要補一個 scenario）。
- **Wins**：判準單源；fail-closed 語意自動對齊到 archive 路徑（現況是實際落差）；未來加條件改一處。刪除測試：`archive.rs` 那段 `filter_map` 整段消失，行為集中到 `discuss.rs`。可與候選 1 同刀，也可獨立先做（約 40 行）。
- **Recommendation**：**strongly recommended**

### 候選 3：封存守門鏈在三個入口各跑一遍，根因是 `--mark-tasks-complete` 的代勾寫入落在 `archive()` 外面

- **Files**：`crates/speclink-core/src/command/mod.rs` `run_archive`（`guard_linked_worktree`、`guard_meta`、`guard_open_tickets`、`merge_violations`、`guard_stale_stamps`、tasks.md 代勾）；`crates/speclink-core/src/archive.rs:590-660`（同五道守門再跑一次）；`crates/speclink-cli/src/verbs/lifecycle.rs:115-200`（bulk 再跑 `merge_violations`／`validate_change_structural`／任務完成度）；`apps/desktop/core/src/verbs.rs` `archive_with`（直呼 `archive()`，不經 `run_archive`）。
- **Problem**：訊號 4（順序契約跨縫外洩）。`archive()` 已收 `opts.mark_tasks_complete`，但只拿它當「豁免」，代勾寫入在 `run_archive`；為了讓拒絕路徑「零寫入」，`run_archive` 得在預寫之前把 `archive()` 內部的守門全部再抄一遍、順序也要一樣（註解明寫「core archive flow gates again for entry points that call it directly (desktop)」）。`validate-merge-gate` 剛剛就必須同時動 `run_archive`、`archive()`、lifecycle bulk 三處。desktop 直呼 `archive()` 時代勾語意不存在（desktop 不傳此旗標所以沒出事，但入口間語意不等價）。
- **Solution**：代勾寫入搬進 `archive()`，落在任務完成度守門與章失效守門之後、計畫階段之前——`archive()` 本來就是「守門全過才有檔案效果」的擁有者，預寫就是第一個檔案效果。`run_archive` 縮成 `resolve_change`＋`guard_meta`＋`archive()`。bulk 的「跳過而非失敗」語意保留，但改吃守門的乾跑形式（例如 `archive::refusal(store, ws, change, opts) -> Option<String>`），不再自己拼一次。
- **Wins**：守門順序契約只在 `archive()` 一份；CLI 單筆、CLI bulk、desktop 三個入口語意等價；下一道守門加一處。刪除測試：`run_archive` 的守門鏈整段消失、工作集中到 `archive()`。
- **Recommendation**：**worth exploring**——方向清楚，但拒絕訊息是凍結輸出（`Validation failed`、`merge_refusal`、`tasks incomplete (n/m)`）要逐字保住，bulk 乾跑的形狀還沒定。

### 候選 4：`review.rs`／`verify.rs` 兩個站別門面——寫章靠字串前綴、讀章靠型別欄位，門面只是在補這條縫

- **Files**：`crates/speclink-core/src/review.rs:1-100`、`verify.rs:1-101`（各 6 支一行轉發＋`STATION` 常數）；`station.rs:465-500`（`{prefix}_at` 字串拼 meta 行）、`station.rs:607` `strip_stamp_lines`（用前綴剝行）；`model.rs` `ChangeMeta` 的 `reviewed_*`／`verified_*` 十個具名欄位；`command/mod.rs` 8 個 Review／Verify 臂；`apps/desktop/core/src/query.rs`（只用到 re-export 的 `content_fingerprint`）。
- **Problem**：訊號 2（介面與實作一樣淺）＋訊號 4。門面 12 支公開函式全是一行轉發，唯一有內容的是 `freshness`：把 `meta.reviewed_at`／`reviewed_tasks_total`／`reviewed_scope` 手工塞進 `StampAnchors`。「這個站的章住在哪些 meta 欄位」有兩份知識：寫入靠 `Station.meta_prefix` 字串，讀取靠 `ChangeMeta` 具名欄位——所以 `Station` 常數讀不回自己寫的章，門面存在的理由就是這個縫。
- **Solution**：`ChangeMeta` 提供 `fn stamp(&self, st: &Station) -> StampAnchors`（依 st 選欄位）；`station::freshness` 直接吃 `(meta, st)`；`STATION` 常數與 re-export 移到 `station.rs`（`station::REVIEW`／`station::VERIFY`），`review.rs`／`verify.rs` 刪除，呼叫端改 `station::xxx(&station::REVIEW, …)`；兩檔約 1600 行測試搬到 `station.rs` 或改參數化。
- **Wins**：讀一個站的行為只看 `station.rs` 一檔；第三個站只加一個常數，不用再複製 100 行門面＋900 行測試。刪除測試過（轉發層消失、知識集中），但實際省下的非測試碼約 200 行。
- **Recommendation**：**worth exploring**——`verify-station-parity` design D1 明文選了「command 層只認站別門面」，本候選是回訪那個決定；收益以可讀性為主。

### 候選 5：change 的 `.openspec.yaml` 同樣沒有型別化寫入器，五個模組各自 `push_str`／`replacen` 改 meta

- **Files**：`model.rs:88` `from_text`（只有讀）、`model.rs:210` `set_board_rank`、`inprogress.rs:145-150`（started_*）、`newcmd.rs:31-37`（created_*／from_discussion）、`command/mod.rs:1649`（claimed_*）、`station.rs:465-500`（章）、`discuss.rs:760-838`（`restale_from` 累加／清除）。
- **Problem**：候選 1 的鏡像。`ChangeMeta` 有型別化讀取沒有回寫，五個模組各自「讀原文→push_str 或 replacen→寫回」，每支都自己處理行尾換行、重複鍵剝除、YAML 純量守門（`strip_stamp_lines`、restale 的 replacen 累加、`set_board_rank` 的原地替換是三套不同做法）。
- **Solution**：與候選 1 同型——`ChangeMeta` 補一個保留原文行序與未知鍵的回寫，各模組改用它。
- **Wins**：新 meta 欄位不用再發明第四套字串手術。
- **Recommendation**：**speculative**——每支手術都有「其餘欄位逐位元保留」的測試盯著，事故證據薄；改動橫跨五個模組且 remote bridge 的 staged commit 也讀這份文字，改動面大於證據。若候選 1 的型別回寫形狀證明好用，再回訪。

---

**我的首選**：候選 1，候選 2 順道同刀。理由：證據最厚（剛修過的 hold 走新路、status 走舊路兩套並存；escape 函式證明撞名已發生；host bridge 直接吃 raw text），刪除測試最乾淨，而且候選 2 的 `close_if_finished` 正好是 head 型別上的第一個高階動詞。候選 3 第二順位，它是「每個 change 都得動三處」的直接來源，但凍結字串多，適合在 1＋2 之後單刀。

**Open**：使用者要先深挖哪一個候選？

### Round 2 — interview (2026-09-09)

**Focus**: 使用者裁定五個候選全做。第一個上游問題——候選 1（討論 head）與候選 5（change meta 回寫）的接縫落點：兩份文件的回寫要不要共用同一個逐行編輯原語。
**Position**: 要共用，而且原語是「逐行編輯」不是「重新序列化」。證據：(a) 兩份文件的寫入現況都明文選了位元組保留——`model.rs:207` 註解「沿 started_* 的文字手術機制…永不重新序列化」、`station.rs:471` 「文字手術：先剝除既有站別區塊…其餘欄位逐位元組保留」；`ChangeMeta` 讀取走 `config::parse_lenient_or_reason`（serde 寬鬆解析），寫入卻刻意不走 serde，因為使用者手寫的註解與鍵序要保住。(b) 同一個「頂層鍵原位代換／去重／尾端補行」演算法已有三份實作：`discuss.rs:982` `set_frontmatter_line`（43 行，只認 `---` 圍欄內）、`model.rs:225-240` `set_board_rank` 內嵌迴圈（只認第 0 欄鍵）、`station.rs:607` `strip_stamp_lines`（多剝一個縮排區塊）。三者差異只有兩點：作用區域（frontmatter 圍欄內 vs 整份文件）與是否處理縮排區塊（`reviewed_scope:` 之下的 `- path/hash`）。提議的形狀：core 立一個 `KeyLines`（名稱 propose 期定）——以區域（`Frontmatter` / `Whole`）建構，提供 `get(key)`、`set(key, value)`（原位代換第一條、丟重複、缺則補在區域尾）、`remove(key)`（含縮排續行）、`text()` 回寫；`DiscussionHead`（候選 1）與 `ChangeMeta` 的回寫（候選 5）都建在它上面，`station.rs` 的章寫入也改走它。候選 5 因此從「橫跨五個模組各自發明」降為「五個寫入點改呼叫同一原語」，改動面明顯縮小。
**Ruled out**: 討論與 change 各自一份型別化回寫——同一演算法會留兩份，候選 5 的 speculative 評級就是因為改動面大，共用原語正是讓它變便宜的關鍵；改成 serde 重新序列化——違反兩處明文的位元組保留決策，使用者手寫註解與鍵序會被吃掉，且 remote bridge 的 staged commit 比對的是原文。
**Open**: 使用者是否同意「一個共用的逐行編輯原語，兩份文件的型別化回寫都建在它上面」？同意後下一題談候選 1 head 的深度（欄位型別化到什麼程度、`DiscussionInfo` 凍結 JSON 如何投影）。

### Round 3 — interview (2026-09-09)

**Focus**: 回寫原語共用已裁定。本輪談候選 1 的深度——`DiscussionHead` 型別化到什麼程度、狀態轉移住在哪、凍結的 `DiscussionInfo` JSON 怎麼投影。
**Position**: head 要「擁有轉移」，不只是「裝欄位」。證據與提案：
- 欄位：實際記錄（128 份）的 frontmatter 鍵只有 slug／topic／status／created／promoted_to／created_by／kind，加上程式會寫的 hold／board_rank；status 實值只有 open（1）／concluded（7）／promoted（120）三種。head 就是這九個欄位：`status` 做三值列舉、`promoted_to` 做 `Vec<String>`、`hold` 做 bool，其餘 `Option<String>`；缺 status 沿現況預設 open（`info_from_doc` :231 現在就這樣），未知值不 fail-closed（記錄由使用者手寫，讀端容錯是既有方向）。head 只管 frontmatter；Context／Rounds／Conclusion 的區段函式（`section_body_range`、`replace_section`、`conclusion_body`）維持不動。
- 深度：三條轉移規則搬成 head 的方法——`promote(change) -> bool`（open／concluded→promoted、累加去重、**新名字才清 hold**，即 `mark_promoted` :652-694 的全部語意）、`unlink(change) -> Option<Status>`（`unlink_discarded` :710-747 的縮清單／清空回退語意）、`conclude(hold)`（open→concluded、hold 設或清）。「它只是轉發」的淺介面在這裡不成立：這三支方法各自帶著現在散在註解裡的規則（hold 只在新名字時清、回退到 concluded 還是 open 看有無結論）。
- 投影：`DiscussionInfo` 加 `promoted_to`／`concluded`／`held`／`board_rank` 四欄並標 `#[serde(skip)]`——`discuss list --json` 逐位元不變（凍結保住），`Deserialize` 端取預設值不影響 remote 解碼；`info_from_doc` 已握有全文（`DiscussionDoc.text`），同一趟解析就能算出 concluded（`conclusion_body`）。三個消費端因此不再各自讀檔：desktop `discussions.rs:23-26` 的 `promoted_to`／`discussion_concluded` 兩次讀檔消失；server `routes.rs:1531-1560` 的 extras 組裝直接讀 info 欄位，`speclink_host::bridge::discussions_extras`（bridge.rs:99-130）整支刪除；`promoted_to_in`／`concluded_in` 兩支 raw-text 公開函式隨之刪除。
- 介面深度四項：(1) 接縫＝`discuss.rs` 內，建在 `KeyLines` 之上；(2) 轉接數＝邊緣三個投影（CLI 凍結 JSON、server wire DTO、desktop camelCase）保留，內部轉接（bridge extras、`*_in` 函式、四支獨立讀取函式）歸零；(3) 深度＝三條轉移規則＋累加語意藏在型別後面；(4) 刪除測試＝今天刪掉它就回到 5 支探測函式＋11 處 replacen＋每卡 4 次讀檔。
**Ruled out**: head 只裝欄位、轉移留在動詞函式——規則仍散在 `mark_promoted`／`unlink_discarded`／`conclude` 三處，只是 replacen 換成 setter，候選 1 的主要收益（規則一份）沒拿到；`DiscussionInfo` 直接加進 JSON 輸出——動凍結輸出，remote parity 測試與 desktop 都要跟著改，超出本候選；status 未知值 fail-closed——記錄是手寫文件，現況讀端容錯，改嚴是行為變化且無事故證據。
**Open**: 使用者是否同意「轉移住在 head 方法上、`DiscussionInfo` 以 serde(skip) 欄位吸收四個獨立讀取」？同意後下一題談候選 2 併入同刀時，archive 路徑 fail-closed 對齊要不要進規格。

### Round 4 — interview (2026-09-09)

**Focus**: 候選 2 併入同刀——兩處判準統一成一支時，公式差異怎麼收、archive 路徑的 fail-closed 對齊要不要進規格。
**Position**: 統一的是「三條件裁決＋封存步」，兩邊的觸發條件各自保留；archive 路徑改 fail-closed 是行為變化，補一條 scenario 進 `discussion-docs`。查證結果：
- 兩處公式其實有兩個差異，不只 meta_error 一個。差異一（觸發）：conclude 側多一條「promoted_to 非空」（規格 discussion-docs:683 明文），archive 側沒有——因為 link 型討論規格明定 `link` 不寫討論側（:73「SHALL NOT 寫 promoted_to」），archive 側的「曾轉出」證據是變更自己的 from_discussion。所以「promoted_to 非空」是 conclude 的觸發條件，不是共同裁決的一部分——留在 conclude。差異二（裁決）：conclude 側 `c.meta_error.is_some() ||`（discuss.rs:1188，註解「the same discipline as link」）視壞 meta 為仍引用；archive 側（archive.rs:783-785）只看 `from_discussions()`，壞 meta 被當作沒引用，該討論會被掃進封存區。
- 對齊方向選 fail-closed：`link`／`seal` 對壞 meta 拒絕已入規格（:549、:554），同一段隨行封存要求已對「討論記錄讀取失敗」明定「視同未寫入結論、留在途」（:73）——壞 change meta 沒被寫到只是漏了，不是刻意放寬。反向對齊（conclude 放寬）會讓一條已寫成規格精神的守門倒退。
- 形狀：`discuss::close_if_finished(store, slug) -> Result<Option<String>>`——內含「無在途引用（壞 meta 視為引用）＋已有結論＋無 hold」三條件與 `archive_discussion` 步；`conclude` 的呼叫端保留自己的「promoted_to 非空」前置與 `closing_error` 包裝，archive 的 `filter_map` 換成對每個 slug 呼叫它（archive 本身已搬走、不在 `list_changes`，「排除自己」現況就成立）。head 落地後這支就是 head 上的第一個高階動詞。
- 規格：`discussion-docs` 的隨行封存要求（:73 那條）MODIFIED，補一條 scenario「在途變更 meta 損壞時視同仍引用、討論不隨行封存」；conclude 側既有 scenario 不動（行為不變）。
**Ruled out**: 兩邊都放寬（拿掉 meta_error）——倒退一條已入規格的 fail-closed 精神，且 link／seal 已對同型錯誤拒絕，同一層出現兩種紀律；把「promoted_to 非空」也塞進共同裁決——會讓 link 型討論永遠不隨行封存，違反 :106 scenario「併入後隨變更自動封存」；只統一不改行為（共同函式吃一個 `fail_closed: bool` 參數）——參數化保留了分歧，判準仍是兩份。
**Open**: 使用者是否同意「archive 路徑對壞 meta 改為 fail-closed，補一條 scenario；promoted_to 非空留在 conclude 端當觸發」？同意後下一題談候選 3（守門鏈收進 archive()）的 bulk 乾跑形狀與凍結字串。

### Round 5 — interview (2026-09-09)

**Focus**: 候選 3——守門鏈收進 `archive()` 的落點、`--mark-tasks-complete` 代勾的新位置、bulk「跳過而非失敗」的乾跑形狀、凍結字串。
**Position**: 代勾搬進 `archive()` 並落在**計畫階段之後、提交階段的第一個效果**；`run_archive` 縮成三行；bulk 吃一個「跳過理由」列舉、自己保留渲染字串。查證：
- 現況三份守門序：`run_archive`（command/mod.rs）＝ linked worktree → meta → open tickets → merge_violations → [帶旗標時：stale stamps → 代勾寫入] → `archive()`；`archive()`（archive.rs:596-660）＝ linked worktree → meta → open tickets → 任務完成度（旗標豁免）→ stale stamps → 同日撞名 → 結構 validate → 計畫階段（merge 守門在此）→ 提交階段；bulk（lifecycle.rs:141-186）＝ merge_violations → 結構 validate → 任務完成度，三者任一不過就「跳過」，其餘守門交給 `Command::Archive` 且失敗即「中止」。`run_archive` 的前四道與 `archive()` 前三道逐字重複，存在理由只有一個：代勾必須在 merge 守門之後寫，而 merge 守門住在 `archive()` 的計畫階段，所以 `run_archive` 得在外面先抄一遍。
- 新位置的規格依據：change-lifecycle:703「拒絕時 --mark-tasks-complete 前置寫入零效果」、:737「章失效守門之後、任何封存檔案效果之前」、archive-merge「兩階段合併計畫與零半套寫入」——三條都只要求「拒絕路徑 tasks.md 逐位元不變」，沒有要求代勾先於 validate。把代勾放在計畫階段結束後、snapshot 之前，等於「所有守門與純讀計畫全過，才有第一個寫入」，三條同時滿足且比現況更強（現況 merge 拒絕靠 `run_archive` 提前檢查來保護，desktop 直呼 `archive()` 帶旗標時沒有這層保護）。零行為變化，不需規格 delta。
- `run_archive` 剩：`resolve_change` → `guard_meta`（保留，因它產的是帶 ErrorCode 的 CommandError，`archive()` 內 `require_valid_meta` 是 anyhow）→ `archive()`。`guard_linked_worktree`／`guard_open_tickets`／`merge_violations`／`guard_stale_stamps` 四道在 command 層的重複呼叫全部刪除；`guard_*` 的 `pub(crate)` 可見度隨之縮回 archive.rs 私有（除 `merge_violations`：validate.rs 與 drift.rs 仍是共用者，規格 archive-merge「過期判定單源共用」）。
- bulk：新增 `archive::skip_reasons(store, change, opts) -> Vec<SkipReason>`，`SkipReason` 三值列舉 `MergeRefused(Vec<MergeViolation>)`／`StructuralInvalid`／`TasksIncomplete { complete, total }`，內含 `skip_specs`／`no_validate`／`mark_tasks_complete` 三個旗標的豁免語意（現在 bulk 自己判）。bulk 保留自己的字串（`"{n} delta operation(s) archive would refuse — run /speclink-drift {name}"`、`"validation failed"`、`"tasks incomplete ({c}/{t})"`）與 merge→validate→tasks 的既有優先序，只是不再自己算條件。`archive()` 的拒絕字串（`Refusal("change '…' has {c}/{t} tasks complete — …")`、`"Validation failed:\n…"`、`merge_refusal`）逐字不動。
- 介面深度四項：(1) 接縫＝`archive()` 是唯一「守門全過才有檔案效果」的擁有者，代勾是檔案效果，歸它；(2) 轉接＝`run_archive` 從「半個 archive」降為純 command 包裝，desktop `archive_with` 與 bulk 都直接對到同一支；(3) 深度＝守門順序契約、豁免語意、代勾都藏在 `archive()` 後面，`skip_reasons` 是同一組守門的唯讀投影；(4) 刪除測試＝刪掉 `run_archive` 的守門鏈與 bulk 的三段自算，行為集中到 archive.rs。
**Ruled out**: 代勾放在 `archive()` 開頭（守門之前）——違反 :703／:737 零效果要求；代勾留在 `run_archive`、只刪重複守門——merge 守門仍得在外面先跑，重複刪不掉；bulk 改直接呼叫 `archive()` 吃錯誤來決定跳過——bulk 的「跳過 vs 中止」是刻意分類（三種跳過、其餘中止），靠錯誤型別分類會把凍結的短字串換成 archive 的長字串；`skip_reasons` 改用 archive 的順序回報——多條同時成立時 bulk 的跳過行會變，凍結輸出破。
**Open**: 使用者是否同意「代勾成為提交階段第一個效果、bulk 走 `skip_reasons` 唯讀投影、零行為變化不動規格」？同意後下一題談候選 4（站別門面刪除）——是否回訪 verify-station-parity design D1 的分層決定。

### Round 6 — interview (2026-09-09)

**Focus**: 候選 4——是否回訪 verify-station-parity design D1「review.rs／verify.rs 薄實例＋command 層只認門面」的分層，把門面刪掉、讓 `Station` 自己讀回自己寫的章。
**Position**: 回訪成立，D1 當初的理由已經用完。查證：
- D1 原文（verify-station-parity design.md:25-29）：門面的存在理由是「review 的公開函式簽名不變，兩個前置 change 的測試不改一字即為回歸網」——這是**遷移期**的保護，不是分層主張；那次遷移已封存（2026-08-06），回歸網已經證明 station.rs 正確。今天門面 12 支函式全是一行轉發（review.rs:37-82、verify.rs 同形）。
- 真正的縫：「這個站的章住在哪些 meta 欄位」現有**四份**手寫對映——review.rs:90、verify.rs:91、archive.rs:94、archive.rs:102 各自把 `meta.reviewed_at`／`reviewed_tasks_total`／`reviewed_scope`（或 verified_*）塞進 `StampAnchors`；寫入端 station.rs:472-495 用 `meta_prefix` 字串拼行，讀取端用具名欄位。`Station` 常數因此讀不回自己寫的章，每個要判 freshness 的地方都得自己對一次欄位。
- 形狀：`ChangeMeta::anchors(&self, st: &Station) -> StampAnchors<'_>`（依 `st.meta_prefix` 選 reviewed_*／verified_* 三欄，唯一一份對映，第三站來時只加一個 match 臂）；`station::freshness(st, meta, counts, read_file)` 直接吃 meta；`STATION` 常數改名 `station::REVIEW`／`station::VERIFY`，連同 `REVIEW_DOC`／`VERIFY_DOC` 一起搬進 station.rs；review.rs／verify.rs 兩檔刪除；呼叫端改寫：command/mod.rs 8 個臂（`crate::review::add_round(store, …)` → `crate::station::add_round(&station::REVIEW, store, …)`）、archive.rs 的 `guard_open_tickets`／`guard_stale_stamps`（後者四份對映降為 `[REVIEW, VERIFY].map(|st| (st, meta.anchors(st)))`）、desktop query.rs:80／97 的兩支 `freshness`＋`content_fingerprint` re-export、`classify` 的 `review::NotFound` downcast。兩檔約 1600 行測試搬到 station.rs 的測試模組，呼叫改指向 `&REVIEW`／`&VERIFY`，斷言字串不動。
- 介面深度四項：(1) 接縫＝站別差異全部收在 `Station` 常數＋`ChangeMeta::anchors` 的一個 match，不再有「每站一個模組」；(2) 轉接＝門面層歸零，command 臂直接對 station；(3) 深度＝現況門面零深度（一行轉發），刪除後 station.rs 是唯一有行為的地方；(4) 刪除測試＝今天刪門面，呼叫端多帶一個 `&REVIEW` 參數，四份 anchors 對映仍散著——所以本候選的核心是 `anchors`，門面刪除是它的自然結果。
- 與候選 5 的關係：station.rs:472-495 的章寫入是候選 5 要改走 `KeyLines` 的五個寫入點之一（`strip_stamp_lines` 的縮排區塊剝除正是 `KeyLines::remove` 要涵蓋的能力）。候選 4 不等它——`anchors` 是讀端、與寫端的原語無關。
**Ruled out**: 保留門面、只加 `anchors`——四份對映降一份是主要收益沒錯，但門面 12 支一行轉發仍是讀者每次要跳一層的稅，且第三站仍要複製 100 行門面＋900 行測試；`Station` 帶 fn pointer 讀 meta（`anchors: fn(&ChangeMeta) -> StampAnchors`）——常數裡放函式指標讓 `Station` 從資料變成半個 trait，`ChangeMeta` 上一個 match 更直白；做成 trait `StationKind`——兩個實例、零多型需求，是 improve-workspace-sync 已否決過的同型「空轉接」。
**Open**: 使用者是否同意「回訪 D1：`ChangeMeta::anchors(st)` 收四份對映、門面刪除、常數與測試搬進 station.rs」？同意後下一題談候選 5 的範圍（五個寫入點全改走原語、還是先改三個）與五個候選的切刀順序。

### Round 7 — interview (2026-09-09)

**Focus**: 候選 5 的範圍——change meta 的哪些寫入點改走共用原語、原語在 change 側的包裝長什麼樣、`new change` 的樣板建檔算不算。
**Position**: 「編輯既有 meta」的八個點全改走原語；「從零建檔」的 `new change` 不算。查證：
- 編輯型寫入點盤點（非測試碼）：model.rs:210 `set_board_rank`（原位代換／尾端補行）、inprogress.rs:121 `add`（三行尾端追加）、inprogress.rs:63 `remove`（過濾三個 `started_*` 行）、command/mod.rs:1649 claim（兩行尾端追加）、station.rs:472 章寫入（`strip_stamp_lines` 剝頂層行＋縮排區塊，再追加五個欄位）、discuss.rs:894 `link`（`from_discussion` 逗號累加）、discuss.rs:753 `stamp_restale`／:794 `clear_restale`（`restale_from` 逗號累加／縮清單／清行）。八處各自處理「行尾換行」（四處 `if !meta.ends_with('\n')`）、「原位 vs 追加」、「縮排區塊」、「逗號清單」，且每處都先 `from_text`／`check_meta_text` fail-closed 再動刀——這段前奏也是八份。
- newcmd.rs:29-38 是 `format!` 組整份新檔，沒有既有內容要保留，不是編輯——留原樣。
- change 側包裝：`model::edit_meta(store, name, |m: &mut KeyLines| …) -> Result<()>`，內含四步：讀原文（不存在→錯誤）、fail-closed 解析（沿 `check_meta_text` 的訊息）、套用閉包、寫回。八個寫入點各縮成「edit_meta ＋ 一到三個 set／remove／list 呼叫」，前奏八份歸一。`KeyLines` 為此需要的能力剛好對上 round 2 的四支：`get`、`set`（原位／尾補）、`remove`（含縮排續行——`strip_stamp_lines` 的需求）、加上 `push_list`／`remove_list`（逗號累加——`from_discussion`、`restale_from`、討論側 `promoted_to` 三個鍵同一語意，現在三處各寫一遍 split／trim／filter／join）。
- 行為：零變化。八處的「其餘欄位逐位元保留」測試（`*_preserves_existing_fields_verbatim` 一族）不改一字就是回歸網；唯一可觀察差異是重複鍵——現況 `set_board_rank` 只換第一條、`remove` 過濾全部，`KeyLines::set` 沿 `set_frontmatter_line` 的既有語意「第一條原位代換、其餘重複鍵丟掉」，對只有一條鍵的正常檔零差異，對手寫出重複鍵的壞檔是收斂而非破壞（討論側已是這個語意）。
- 評級變動：候選 5 從 speculative 升為 worth exploring——共用原語裁定後，它的改動面是「八處各 5-15 行換成 3 行」，事故證據薄仍成立，但成本已經低到值得在原語落地後順手收。
**Ruled out**: 只改三個「有逗號清單語意」的點——原語只吃一半，前奏仍是五份；`ChangeMeta` 加完整 setter 集合（`set_started`、`set_claimed`…）——每個欄位族一支 setter 等於把八個寫入點的形狀搬進 model.rs，`edit_meta` 閉包已足夠；`new change` 也改走原語——沒有既有內容，用原語只是多一層。
**Open**: 使用者是否同意「八個編輯型寫入點全改走 `edit_meta`＋`KeyLines`，`new change` 不動，零行為變化」？同意後最後一題：五個候選的切刀與順序（我會提案四刀＋`conclude --hold`）。

### Round 8 — interview (2026-09-09)

**Focus**: 最後一題——五個候選怎麼切刀、什麼順序、討論記錄怎麼跨刀存活。
**Position**: 三刀依序、全走 main、`conclude --hold`，每刀封存後回本討論再轉出下一刀。切法依「有無行為變化」與「檔案面」：
- **刀一（候選 1＋2）**：`KeyLines` 原語＋`DiscussionHead`＋`close_if_finished`。檔案面：新原語（落點 propose 期定，`util.rs` 或 `model.rs`）、discuss.rs、archive.rs 的討論掃除段（:770-796）、host bridge.rs（刪 `discussions_extras`）、server routes.rs:1531-1560、desktop discussions.rs:16-27。**唯一有行為變化的一刀**：archive 路徑對壞 meta fail-closed，`discussion-docs` 補一條 scenario；其餘（凍結 JSON、轉移語意）零變化。先做的理由：原語與 head 是後面兩刀的地基，且它帶規格 delta，單獨一刀讓 review 分得清「刻意的行為變化」與「純重構」。
- **刀二（候選 5）**：`edit_meta`＋八個寫入點改走 `KeyLines`。檔案面：model.rs、inprogress.rs、command/mod.rs 的 claim、station.rs:472、discuss.rs 的 link／restale。零行為變化。排第二的理由：它是原語的第二個消費者，趁刀一的形狀還熱時用八個真實案例驗證 `set`／`remove`／`push_list` 的介面夠不夠——若原語形狀有錯，這裡最早發現、修正成本最低；規模小（八處各縮成三行）。
- **刀三（候選 3＋4）**：守門鏈收進 `archive()`＋站別門面刪除＋`ChangeMeta::anchors`。檔案面：archive.rs（守門段＋`guard_stale_stamps`＋新 `skip_reasons`）、command/mod.rs（`run_archive`＋8 個站臂＋`classify`）、station.rs、review.rs／verify.rs 刪除、model.rs（`anchors`）、lifecycle.rs bulk、desktop query.rs。合一刀的理由：兩個候選都是「archive.rs 與 command/mod.rs 裡的重複刪除」、都零行為變化、檔案面高度重疊（archive.rs 與 command/mod.rs 都各動兩段），分兩刀等於同兩檔連續 review 兩次；review 面雖大但全是機械式（一行轉發改參數、守門呼叫刪除、測試搬家），凍結字串清單明確（round 5）。若 propose 期盤出 review 面超過可讀範圍，拆回 3 與 4 兩刀，順序 4 先 3 後（4 動 station.rs 測試搬家，先落地讓 3 的 archive.rs diff 乾淨）。
- **順序上的相依**：刀二依賴刀一（原語）；刀三不依賴刀一二，但排最後是因為它的 review 面最大、且 station.rs:472 的章寫入會在刀二改走原語——刀三搬 station.rs 測試時吃到的是已改好的版本，不會兩刀在同一段對撞。
- **不用 worktree 平行**：三刀彼此檔案面重疊（archive.rs、command/mod.rs、station.rs、discuss.rs 至少兩刀共用），平行必撞；且 improve-workspace-sync 的教訓（先立後刀的 tasks 寫在尚不存在的介面上必 drift）在此同樣成立——每刀封存後再 propose 下一刀。
- **記錄存活**：`conclude --hold`，先轉出刀一；刀一封存時記錄因 hold 留在途，回本討論「再轉出一個變更」轉出刀二（promote 清 hold），刀二封存前再 `conclude --hold` 一次保留給刀三；刀三轉出後記錄隨刀三封存自動封存。
**Ruled out**: 五個候選五刀——候選 2 只有 40 行且是 head 的第一個高階動詞，單開是空刀；候選 3 與 4 各自單刀（預設）——同兩檔 review 兩次，改為 propose 期視 review 面決定是否拆；一刀全做——刀一有規格 delta，混進 1600 行測試搬家與守門刪除，review 分不出刻意變化；刀三先於刀一（「先做零行為變化的」）——刀三不依賴刀一是事實，但刀二需要刀一，而刀二又該在刀三之前（station.rs:472 的順序理由），刀一先行讓三刀成直線。
**Open**: 使用者是否同意「三刀依序（1＋2 → 5 → 3＋4）、全走 main、`conclude --hold`、每刀封存後回本討論轉出下一刀」？同意即寫結論。

## Conclusion

**Decision**: 五個候選全部落地，分三刀依序、全走 main。刀一（候選 1＋2，唯一有行為變化）：core 立一個逐行編輯原語 `KeyLines`（以區域 Frontmatter／Whole 建構；`get`／`set` 第一條原位代換、重複鍵丟、缺則區域尾補／`remove` 含縮排續行／`push_list`／`remove_list` 逗號清單／`text` 回寫；永不重新序列化、其餘位元組保留）；discuss.rs 立 `DiscussionHead`（九欄：slug／topic／status 三值列舉／created／created_by／kind／promoted_to Vec／hold bool／board_rank；缺 status 沿現況預設 open、未知值不 fail-closed；只管 frontmatter，區段函式不動），三條轉移成為方法——`promote(change)`（open／concluded→promoted、累加去重、新名字才清 hold）、`unlink(change)`（縮清單／清空回退 concluded 或 open）、`conclude(hold)`；`DiscussionInfo` 加 promoted_to／concluded／held／board_rank 四欄標 `#[serde(skip)]`（`discuss list --json` 逐位元不變），desktop `entry()`、server extras、host `discussions_extras` 三個內部轉接與 `promoted_to_in`／`concluded_in`／`held_in`／`frontmatter_value`／`set_frontmatter_line` 隨之刪除；`close_if_finished(store, slug)` 收「無在途引用（壞 meta 視為仍引用）＋已有結論＋無 hold」三條件與封存步，conclude 保留自己的「promoted_to 非空」觸發與 closing_error 包裝，archive.rs 的 filter_map 改呼叫它；規格 `discussion-docs` 隨行封存要求 MODIFIED 補一條 scenario「在途變更 meta 損壞時視同仍引用、討論不隨行封存」。刀二（候選 5，零行為變化）：`model::edit_meta(store, name, |m: &mut KeyLines| …)` 四步（讀、fail-closed 解析、套閉包、寫回），八個編輯型寫入點改走它——set_board_rank、in-progress add／remove、claim、station 章寫入（strip_stamp_lines 由 `remove` 取代）、link 的 from_discussion、stamp_restale／clear_restale；`new change` 樣板建檔不動。刀三（候選 3＋4，零行為變化）：`--mark-tasks-complete` 代勾搬進 `archive()` 成為計畫階段之後、提交階段的第一個效果，`run_archive` 縮為 resolve_change＋guard_meta＋archive()，四道重複守門呼叫刪除、`guard_*` 縮回私有（merge_violations 仍共用）；bulk 改吃 `archive::skip_reasons(store, change, opts) -> Vec<SkipReason>`（MergeRefused／StructuralInvalid／TasksIncomplete，含三旗標豁免語意），保留自己的短字串與 merge→validate→tasks 優先序；`ChangeMeta::anchors(&self, st: &Station) -> StampAnchors` 收四份 meta→anchors 對映（review.rs:90、verify.rs:91、archive.rs:94／:102），`station::freshness` 改吃 (st, meta)，STATION 常數與 REVIEW_DOC／VERIFY_DOC 搬進 station.rs 為 `station::REVIEW`／`station::VERIFY`，review.rs／verify.rs 刪除，command/mod.rs 8 臂與 classify、archive.rs 兩支 guard、desktop query.rs 改呼叫 station，約 1600 行測試搬進 station.rs 且斷言字串不動；archive() 的拒絕字串與 bulk 跳過字串逐字凍結。刀三 propose 期若盤出 review 面超過可讀範圍，拆回候選 4 先、候選 3 後兩刀。
**Rationale**: 這一層的兩份文件（討論 frontmatter、change `.openspec.yaml`）都只有型別化的讀沒有型別化的寫，每個動詞各自做字串手術，同一條生命週期規則因此散在兩到三處——近一個月四個封存變更每個都得同時補多處，且已出現實際漂移（討論收尾判準對壞 meta 一邊 fail-closed 一邊放行；status 一個欄位兩套寫法並存）。共用一個逐行原語是關鍵槓桿：兩份文件都明文選了位元組保留、永不重新序列化，同一演算法已有三份實作，原語落地後候選 5 從 speculative 降為「八處各換三行」。切三刀依「有無行為變化」與「檔案面」：刀一單獨承載規格 delta 讓 review 分得清刻意變化與純重構；刀二緊接刀一用八個真實案例驗證原語形狀，最早發現介面錯誤；刀三兩候選同為 archive.rs 與 command/mod.rs 的重複刪除、檔案面高度重疊，合一刀免同兩檔連續 review 兩次，且排最後讓 station.rs:472 已在刀二改走原語、測試搬家不對撞。介面深度四項檢查五個候選皆過站（Round 3、4、5、6、7）。不用 worktree 平行：三刀檔案面互相重疊必撞，且後刀的 tasks 不得寫在前刀尚不存在的介面上（improve-workspace-sync 同一教訓）。
**Rejected alternatives**: 討論與 change 各自一份型別化回寫——同一演算法留兩份，候選 5 的成本降不下來；serde 重新序列化——違反兩處明文的位元組保留決策、吃掉手寫註解與鍵序、remote bridge 比對原文；head 只裝欄位、轉移留在動詞——規則仍散三處，replacen 換 setter 而已；DiscussionInfo 直接加進 JSON 輸出——動凍結輸出，超出候選；status 未知值 fail-closed——手寫文件、無事故證據的行為變化；討論收尾判準兩邊都放寬——倒退已入規格的 fail-closed 精神（link／seal 已對同型錯誤拒絕、同段規格對討論讀取失敗已 fail-closed）；把「promoted_to 非空」塞進共同裁決——link 型討論永遠不隨行封存，違反 scenario「併入後隨變更自動封存」；共同函式吃 fail_closed 布林——參數化保留分歧；代勾放 archive() 開頭——違反 change-lifecycle:703／:737 零效果要求；代勾留 run_archive 只刪重複守門——merge 守門仍得外面先跑；bulk 靠錯誤型別分類跳過——凍結短字串換成長字串；skip_reasons 用 archive 順序回報——多條同時成立時跳過行變；保留門面只加 anchors——跳一層的稅還在、第三站仍複製 100 行門面＋900 行測試；Station 帶 fn pointer 讀 meta——資料變半個 trait；StationKind trait——兩個實例零多型需求，improve-workspace-sync 已否決的空轉接；候選 5 只改三個逗號清單點——前奏仍五份；ChangeMeta 加完整 setter 集合——寫入點形狀搬進 model.rs；new change 也改走原語——沒有既有內容、多一層；五候選五刀——候選 2 單開是空刀；一刀全做——規格 delta 混進大量機械搬家；刀三先於刀一——刀二需要刀一且該在刀三前，刀一先行三刀成直線；worktree 平行——檔案面重疊必撞。
**Deferred**: `KeyLines` 落點（util.rs 或 model.rs）與命名——刀一 propose 期定；刀三是否拆回兩刀——刀三 propose 期依 review 面決定；improve-cli-verb-layer 已 Deferred 的 server routes wrong_outcome 臂與 node SDK 拆封遷移——不在本範圍、維持原記錄的後續小刀。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion improve-lifecycle-layer（先立刀一；刀一封存後回本討論「再轉出一個變更」立刀二，刀二封存前再 conclude --hold 一次保留給刀三）
