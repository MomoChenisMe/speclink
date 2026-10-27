---
topic: discuss 叉出多個變更時，結論已寫但仍欠一個尚未建立的變更，卻在第一個變更封存時被連帶封存
slug: discussion-planned-spinout-hold
status: promoted
promoted_to: discussion-spinout-hold
created: 2026-09-07
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: discuss 叉出多個變更時，結論已寫但仍欠一個尚未建立的變更，卻在第一個變更封存時被連帶封存

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

**起因**：`improve-workspace-sync` 的結論規劃兩刀（刀 A 零行為變化、刀 B 有行為變化），Deferred 寫「刀 A 封存後回本討論再轉出」。刀 A（workspace-sync-plan）封存時，引擎依「已有結論＋無在途變更引用」的規則把討論隨行封存，刀 B 無從在原討論上 promote。

**判讀**：引擎沒有做錯——它照 2026-08-31 討論定下的「討論的生命由結論決定」實作（archive.rs:752、discuss.rs:1116）。缺口是「還欠一個尚未建立的變更」只存在於結論散文，沒有機器可讀的訊號。掃全部封存討論的 Deferred 行，只有本案一份寫了「回本討論再轉出」；先例 improve-cli-verb-layer 是兩刀同日一起 promote，improve-cli-command-layer 的後續則是開新討論。

**需求已夠銳利**（可驗證：封存刀 A 後討論留在途、刀 B 可 promote），未經 grill 直接進 assumptions。

**相關規格**：discussion-docs（「討論以 link 動詞併入既有變更」的隨行封存條件、「conclude 於全數轉出變更已封存時順手封存討論」）、discuss-skill（「中途轉出教學」）。相關變更：無在途變更。
Prior discussions: discussion-auto-archive-before-conclusion, remote-remaining-gaps, discuss-recall-archived-discussions

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-07)

**Focus**: 引擎怎麼知道「這份討論還欠一個尚未建立的變更」？六條假設全數由使用者確認。
**Position**: 走 A1——conclude 帶保留旗標，frontmatter 多一行機器可讀訊號，下次轉出自動清掉。
- 引擎沒做錯：archive.rs:752 與 discuss.rs:1116 都照 08-31 討論「討論的生命由結論決定」實作；本案結論已寫、promoted_to 只有 workspace-sync-plan，兩條件成立即封存
- 缺口：「刀 B 之後回本討論再轉出」只在結論 Deferred 散文，引擎讀不懂散文；掃全部封存討論只有本案一份這樣寫
- 決策樹：A 記錄放機器可讀訊號（A1 conclude 旗標／A2 結構化計數／A3 獨立動詞）、B 先立刀 B 骨架擋封存、C unarchive 救援動詞、D 只改技能文字
- 旗標只在 conclude 給（排除 A3）：「還欠一刀」在寫結論那一刻就知道，獨立動詞是多一個入口
- 技能文字同步改：discuss 技能「最後一個變更封存即自動封存」與 improve 流程「刀 1 先立、刀 2 隨後」合起來就是本案的坑；要明說想回原討論再轉出就 conclude 帶旗標，否則後續刀走新討論
- 本案救援：手動把 archive/2026-09-07-improve-workspace-sync.md 搬回 discussions/，promote 守門只看是否在 archive/（discuss.rs:839），搬回即可轉出刀 B
**Ruled out**: A2 結構化計數——多一種可寫錯的狀態（寫 3 實際立 2 就永遠不封）；A3 獨立動詞——同一件事多一個入口；B 先立刀 B 骨架——Round 5 of improve-workspace-sync 已否決先立刀 B，且空卡長掛污染「待收尾」語意；C unarchive 動詞——08-31 討論已否決（治標、留髒狀態），且手動搬檔即可救；D 只改技能——留下「同一討論不能分期轉出」的硬限制
**Open**: 旗標由誰清掉（promote 還是 seal）？手動 discuss archive 是否無視旗標？desktop 看板要不要標示？

### Round 2 — interview (2026-09-07)

**Focus**: 旗標的清除點、手動封存的優先權、desktop 呈現
**Position**: 旗標在 mark_promoted 一處清掉；手動 archive 無視旗標；desktop 不改資料鏈、只在既有「已轉出」收合列加一個小標。
- 清除點是事實不是決策：promote（discuss.rs:858）與 seal（discuss.rs:943）都經同一支 mark_promoted 累加 promoted_to，旗標在那裡清掉，兩條轉出路徑（discuss promote、propose --from-discussion 後 seal）都涵蓋
- 兩個自動封存點都守旗標：變更封存的隨行封存（archive.rs:752）與 conclude 閉環（discuss.rs:1116）；旗標未清就留在途
- 手動 speclink discuss archive 是明示動詞，無視旗標——使用者決定不做刀 B 時的出口，不需新動詞
- 再次 conclude 可再帶旗標（第三刀情境）；不帶旗標的 re-conclude 清掉旗標——結論改寫即重述意圖
- desktop：討論留在途本身就是可觀察結果；「已轉出」收合列加「保留中」小標屬文案，資料鏈是否加欄位留 propose/design 決定
**Ruled out**: 旗標分別在 promote 與 seal 各清一次——兩支已共用 mark_promoted，分開寫是重複；desktop 本輪定資料鏈——非本案主軸，留設計期
**Open**: 無

## Conclusion

**Decision**: 討論結論可帶「保留在途」旗標，明示「還欠一個尚未建立的變更」。`speclink discuss conclude <slug> --hold`（旗標名於 propose 期定案）在記錄 frontmatter 寫一行機器可讀訊號。兩個自動封存點都守它：變更封存的隨行封存（archive.rs:752）與 conclude 閉環（discuss.rs:1116）在旗標未清時不封存，討論留在途。旗標在 mark_promoted（discuss.rs:650）一處清掉——promote 與 seal 都經此累加 promoted_to，兩條轉出路徑皆涵蓋。手動 `speclink discuss archive` 無視旗標，是放棄後續刀的出口。不帶旗標的 re-conclude 清掉旗標；帶旗標的 re-conclude 可續保留（第三刀情境）。技能同步：discuss 技能的中途轉出段落與 improve 流程明說「結論規劃之後回本討論再轉出時，conclude 必帶旗標；否則後續刀走新討論」。本案 improve-workspace-sync 以手動搬檔救回（archive/2026-09-07-improve-workspace-sync.md 搬回 discussions/），刀 B 直接 promote，刀 B 封存時隨行封存即為正確結局。
**Rationale**: 引擎沒做錯——它照 08-31 討論「討論的生命由結論決定」實作；缺口是「還欠一刀」只存在結論散文，引擎讀不懂散文。旗標在寫結論那一刻給，正是 Deferred 寫下「回本討論再轉出」的同一時刻；清除點與兩個守門點各只有一處，改動面最小。
**Rejected alternatives**: A2 結構化計數（Planned changes: N）——多一種可寫錯的狀態，寫 3 實際立 2 就永遠不封；A3 獨立 hold／release 動詞——同一件事多一個入口；B 先把刀 B 立成骨架擋封存——improve-workspace-sync Round 5 已否決先立刀 B（tasks 會寫在尚不存在的介面上），且空卡長掛污染「待收尾」語意；C unarchive 救援動詞——08-31 討論已否決（治標、留髒狀態），手動搬檔即可救；D 只改技能文字——留下「同一討論不能分期轉出」的硬限制；旗標在 promote 與 seal 各清一次——兩支已共用 mark_promoted。
**Deferred**: 旗標的 CLI 名稱與 frontmatter 鍵名；desktop「已轉出」收合列是否加「保留中」小標與資料鏈是否加欄位——留 propose/design。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion discussion-planned-spinout-hold
