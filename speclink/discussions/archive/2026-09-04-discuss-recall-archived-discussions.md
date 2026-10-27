---
topic: discuss 是否需要查看舊討論，避免重複討論
slug: discuss-recall-archived-discussions
status: promoted
promoted_to: discuss-search-recall
created: 2026-09-04
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: discuss 是否需要查看舊討論，避免重複討論

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者問 /speclink-discuss 的參考順序（規格先還是討論先），以及有沒有讀舊討論避免重提。查證後發現 discuss 只查在途討論以接續，封存討論不在偵察範圍；improve 則依規格明文做防重提檢查（含 --archived）。使用者接著問這樣設計好不好、discuss 是否該查舊討論，題目轉為決策。

Grill 階段：不需要。目標可驗證——「討論題目撞到過去已否決或已延後的項目時，技能在第一輪假設前就照出來」。

相關規格：discuss-skill（正典先行漏斗偵察、事實決策分診）、improve-skill（防重提檢查）、discussion-docs（list/show/archive 動詞與 kind 欄位）。
相關在途變更：無。
封存現況：121 筆，其中 kind: improve 4 筆；121 筆全有 Ruled out、73 筆 Deferred 非空。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-04)

**Focus**: discuss 開場是否要查封存討論；若要，查的範圍、深度、用法、引擎支援與落點各是什麼
**Position**: 現況是缺口不是刻意取捨，建議在 discuss 偵察加一段有時間盒的「舊討論查核」，形狀比照正典掃描：
- 決策樹：A 範圍（只在途／在途＋封存以關鍵字命中 topic 與 slug／封存全讀 Ruled out）、B 深度（命中 ≤3 份只讀 Conclusion＋各輪 Ruled out＋Deferred／讀全文）、C 用法（已否決就擋／進假設清單當第四種分診列／只在 Context 提一句）、D 引擎（不動引擎只改技能文字／list --archived 加結論摘要或新增 search 動詞）、E 落點（discuss-skill 規格加 scenario＋discuss.md asset 三連動）
- 證據：discuss-skill 規格只要求「正典先行的漏斗偵察」，沒有讀封存討論的要求；improve-skill 規格明文要求防重提檢查含 --archived；speclink discuss list --archived --json 只給 slug/topic/status/created/rounds/path，沒有內文；封存 121 筆全有 Ruled out 行、73 筆 Deferred 非 none
- 建議：A 選在途＋封存關鍵字命中；B 選 ≤3 份、只讀 Conclusion／Ruled out／Deferred；C 選第四種分診列——曾否決（附當時理由，重開須說明理由已失效，規則抄 improve）、曾延後（可接手）、已落地（正典會照出，不重複列），不擋；D 先不動引擎，Context 加一行 Prior discussions 讓 propose 看得到；E 規格 scenario＋asset（ASSET_VERSION／golden／assets.lock 連動）
- 介面深度檢查不觸發：無新模組、指令或儲存層
**Ruled out**: 「已否決就擋」——違反 discuss 自身原則「使用者需求是目標，正典是證據不是裁決」；「封存全讀」——每次開討論讀 121 檔，撞時間盒
**Open**: 五條假設待使用者確認；使用者提出新分支「依 kind 切分：improve 只讀 improve 建的討論、discuss 只讀非 improve 的討論」待裁定；已知弱點：topic 與 slug 中英混雜，關鍵字命中率待觀察（命中率太低即為引擎 search 動詞的觸發條件）

### Round 2 — interview (2026-09-04)

**Focus**: 舊討論查核要不要依 kind 切分——improve 只讀 improve 建的討論、discuss 只讀其餘討論
**Position**: 不切分，兩個技能都讀全部封存與在途討論，篩選軸是題目關鍵字，kind 只當看板標籤與 improve 的讀取排序提示：
- 「討論過了」與誰起的頭無關：improve 掃出被否決的候選與使用者自開討論否決的方向，同為定案；切分後 discuss 看不到 improve 的否決、improve 看不到一般討論的否決，而後者是 improve 規格「提使用者否決過的東西比不提更糟」的主要來源（一般討論 117 筆 vs improve 4 筆）
- 題目確實跨線：關鍵字 review 出現在 9 筆一般討論 topic 與 3 筆 improve 記錄，archive 為 3 對 3，verify 為 5 對 1
- 省讀量的效果不成立：依 kind 切後 discuss 側仍有 117 筆要靠關鍵字篩，improve 側僅 4 筆本就少；壓讀量靠題目相關，不靠來源
- kind 可當排序：封存中有 improve 記錄 topic 標「候選 4 回訪」，improve 掃同範圍時先讀同範圍舊 improve 記錄合理，但那是順序不是過濾
- 使用者同意不切分
**Ruled out**: 依 kind 切分——兩邊各漏一半定案，且不減讀量
**Open**: 第一輪的五條假設待確認（範圍已由本輪定為在途＋封存關鍵字命中）：B 深度 ≤3 份只讀 Conclusion／Ruled out／Deferred、C 第四種分診列不擋、D 先不動引擎、E 規格 scenario＋asset 落點；中英混雜命中率弱點

### Round 3 — interview (2026-09-04)

**Focus**: 關鍵字只比 topic 與 slug 的查核範圍會不會太窄
**Position**: 太窄，搜尋軸改為 grep 記錄全文但只抓帶決定的行，讀取上限不變：
- 實測封存 121 筆：drawer 在 topic 命中 0 筆、在 Ruled out／Decision／Deferred 行命中 6 筆；golden 為 0 對 15；sse 為 0 對 12；worktree 為 6 對 14；review 為 9 對 18；archive 為 3 對 21——定案藏在回合的 Ruled out 行，topic 照不出來
- 只抓四種決定行：`**Ruled out**`、`**Decision**`、`**Rejected alternatives**`、`**Deferred**`，皆為技能模板固定格式、一筆一行；全文命中不收（drawer 全文 27 筆中 21 筆是 Evidence 提到檔名的雜訊）
- 時間盒改形狀：命中的決定行不設上限直接列出；整份 Conclusion 讀取維持最多 3 份，topic 命中者優先
- 中英文各跑一次：中文用使用者題目關鍵字，英文用正典掃描轉譯出的 capability 名
- 仍不動引擎：Grep 工具掃 openspec/discussions/ 與 archive/ 兩層，grep 121 檔實測 4 毫秒
**Ruled out**: 只比 topic／slug——漏掉大多數定案；全文命中全收——Evidence 行雜訊多於決定行三倍以上；調高整份讀取上限——問題在搜尋軸不在上限
**Open**: C 第四種分診列不擋、D 先不動引擎、E 規格 scenario＋asset 落點，待使用者確認後收斂

### Round 4 — interview (2026-09-04)

**Focus**: 舊討論查核靠本機 grep 夠不夠，引擎是否該提供 search 動詞
**Position**: 立場翻轉——改為新增引擎動詞 speclink discuss search，本機 grep 不再是方案：
- remote 模式：discussion-docs 規格「討論動詞於 remote 模式與本機同語意」，list／show／new 皆經 speclink-remote 打 server，記錄在 server 端，本機無 openspec/discussions/ 可 grep
- 儲存後端不只檔案：workspace 有 store-fs、store-sqlite、store-postgres，記錄在資料庫時 grep 碰不到
- 四種工具形狀：技能渲染給 claude、codex、neutral-cli、neutral-tool-call，各工具檔案搜尋能力不一，引擎動詞對四種一致
- improve 第一步今日為 list --archived 後逐筆 show，同樣弱點；一個動詞兩個技能共用
- 動詞形狀刻意做小：`speclink discuss search <關鍵字>... [--json]`；比對 topic、slug 與四種決定行（Ruled out／Decision／Rejected alternatives／Deferred，引擎自寫格式故知去處）；不分大小寫子字串、多關鍵字任一命中即算；預設在途與封存皆搜；輸出每筆命中的 slug、topic、status、archived、created、kind 與命中行及行種類，topic 命中優先；不做索引、模糊或語意搜尋
- 使用者提出「grep 不一定夠、每個專案都不一定」，查證後成立
**Ruled out**: 只改技能文字靠 Grep 工具——remote 模式與資料庫後端下無檔可 grep，且各工具搜尋能力不一；模糊／語意搜尋——幾百筆小記錄子字串即足，YAGNI
**Open**: C 用法（第四種分診列不擋，已向使用者解釋分診意涵）待確認；E 落點需擴為 discussion-docs 規格新 requirement（search 動詞）＋ discuss-skill 規格 scenario ＋ discuss.md 與 improve.md 兩份 asset；store trait 是否已有讀取全部討論的介面留給 propose 查證

## Conclusion

**Decision**: discuss 技能開場加入「舊討論查核」，由新的引擎動詞 speclink discuss search 支撐；improve 的防重提檢查改用同一動詞。
- 引擎動詞：`speclink discuss search <關鍵字>... [--json]`。比對 topic、slug 與四種決定行（Ruled out／Decision／Rejected alternatives／Deferred）；不分大小寫子字串、多關鍵字任一命中即算；預設在途與封存皆搜；每筆命中輸出 slug、topic、status、archived、created、kind、命中行與行種類，topic 命中排前；fs 與 remote 模式同語意。不做索引、模糊或語意比對。
- discuss 偵察順序改為：在途討論接續 → 詞彙 → 正典 → 舊討論查核（關鍵字用使用者題目原文加正典轉譯的英文詞）→ 程式碼。
- 時間盒：命中的決定行全列；整份 Conclusion 最多讀 3 份，topic 命中優先。
- 用法：假設清單加第四種分類「舊討論已定案」，細分曾否決（附當時理由；重開須說明理由已失效）、曾延後（可接手）、已落地（正典會照出，不重列）。不擋方向。Context 加一行 `Prior discussions: <slug 清單>` 供 propose 讀取。
- kind 不作過濾條件；improve 掃同範圍時可用它把舊 improve 記錄排前。
**Rationale**: 定案藏在各輪的 Ruled out 行，topic 照不出（實測 drawer 為 0 對 6、golden 為 0 對 15）；remote 模式、sqlite／postgres 後端與四種工具形狀讓本機 grep 不可靠。關鍵取捨是「引擎多一個小動詞」對「每次討論都可能重審已定案，且風險隨封存數成長（現 121 筆）」，前者一次付清。
**Rejected alternatives**: 維持現況不查——封存的 Ruled out 無人讀；依 kind 切分——兩技能各漏一半定案且不減讀量；只比 topic／slug——漏掉大多數定案；全文命中全收——Evidence 行雜訊逾決定行三倍；只改技能文字靠 Grep 工具——remote 與資料庫後端無檔可 grep；已否決就擋——違反「使用者需求是目標、正典是證據」；只在 Context 提一句——決策當下看不到；模糊／語意搜尋——YAGNI。
**Deferred**: store trait 是否已有讀取全部討論的介面，由 propose 查證；中英文命中率實測後再決定是否加同義詞比對；search 的 AND 模式（--all）先不做。
**Capture to**: proposal（新變更）。規格落點：discussion-docs 新 requirement（search 動詞）、discuss-skill 新 scenario（舊討論查核與第四種分類）、improve-skill 既有防重提 requirement 改用 search；技能落點：crates/speclink-core/assets/skills/discuss.md 與 improve.md，連動 ASSET_VERSION、golden、assets.lock。
**Next**: /speclink-propose --from-discussion discuss-recall-archived-discussions
