---
topic: 我想把所有名稱叫做正典規格的地方，全部都改成叫做正式規格，英文則維持specs
slug: canonical-spec-zh-wording
status: promoted
created: 2026-09-12
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: canonical-spec-zh-wording
---

# Discussion: 我想把所有名稱叫做正典規格的地方，全部都改成叫做正式規格，英文則維持specs

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者要把繁中文案裡的「正典規格」全部改成「正式規格」，英文維持 specs。第一輪回覆把目標磨利：本專案對照 OpenSpec，規格分兩種——變更裡的 delta 規格與合併後的 specs；使用者要的是這一對在中文裡區分清楚，不是把「正典」這個字全面清掉。目標可驗證，未經 grill 階段。

偵察：詞彙表無「正典規格」詞條，但 LANGUAGE.md 的「規格基準」「手冊」兩條定義已寫「正式規格」，baseline-skill、manual-skill 兩份規格與 CHANGELOG 亦然——漂移已朝使用者要的方向發生。活的檔案（不含封存區）裡「正典規格」128 處、「正典 spec(s)」52 處、「規格正典」9 處、光桿「正典」約 400 處（含正典值／正典化／正典詞彙等別義）；使用者看得到的真字串只有 packages/ui/src/i18n.tsx 兩條（specs.heading、specs.empty）。delta 側用詞已一致：i18n 與手冊寫「delta 規格」、docs 與規格寫「delta specs」。

相關規格：ui-copy-vocabulary（詞彙守門面：i18n、README、docs、技能資產）、spec-validation 與 server-read-api（Requirement 標題含舊詞）、manual-pages、desktop-app。先例變更：zh-tw-vocabulary-drawer-and-quality-station（詞條＋守門，design D1 接受規格散文舊詞漸進汰換）、spec-purpose-backfill（直編 67 份規格、零 delta）、docs-vocab-and-posix-paths（守門紅燈同批清）。在途變更 discussion-last-cut-release 與本題無交集。
Prior discussions: manual-generation-skill, spec-drawer-trace-links, rename-onboard-to-baseline

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-12)

**Focus**: 「正典規格」改「正式規格」要動哪些面、怎麼防漂回、同概念的其他寫法怎麼辦
**Position**: 六條假設成立，且使用者把目標磨利為「delta 規格 vs 正式規格這一對要在中文裡分清楚」，不是清掉「正典」這個字：
- 英文不動：桌面英文版 "Canonical specs"（packages/ui/src/i18n.tsx:428）、技能裡的 canon、CLI 的 --specs 都維持
- LANGUAGE.md 立「正式規格」詞條，定義點名對照詞「delta 規格」；avoid 收「正典規格」「正典 spec」「規格正典」（進機械守門）與「正典（指規格集合時）」（帶語境限定、守門跳過，因為光桿「正典」另有別義）
- 守門面同批清零：i18n.tsx 2 條、docs 20 處、README 4 處——avoid 一加立刻紅（scripts/docs/vocabulary-guard.test.mjs:14-20 掃描面）
- 規格散文、手冊、程式碼註解、測試直接改字，一個變更、零 delta（先例 spec-purpose-backfill）；spec-validation 一個 Requirement 標題、server-read-api 一個 Requirement 與一個 Scenario 標題改名，不經封存合併故無未宣告刪除雷，溯源不靠標題名配對（crates/engine/speclink-core/src/lifecycle/trace.rs:159）
- 手冊直接改字、不重生：直編規格不動 @trace 時戳，手冊不會被標「可能過期」
- 技能資產零命中，無 ASSET_VERSION／golden／assets.lock 三連動；零行為變化
- delta 側用詞已一致（i18n 與手冊「delta 規格」、docs 與規格「delta specs」），不另立詞條
**Ruled out**: 光桿「正典」全 repo 一律機械替換——它至少有四種意思（規格集合、正典詞彙、正典值、正典化、「本文是…的正典」），機械替換會改壞別義；走 delta MODIFIED 改規格散文——零語意變化卻要約 40 個區塊，標題改名還會踩未宣告刪除雷
**Open**: 規格散文裡指「規格集合」的光桿「正典」（155 行、其中約 40 行是別義）要一起改成「正式規格」，還是只改複合寫法、光桿留給漸進汰換；已封存 90 檔不回改（使用者未反對，待確認）

### Round 2 — interview (2026-09-12)

**Focus**: 規格散文、手冊、docs 裡指「specs 這一側」的光桿「正典」要不要一起改成「正式規格」
**Position**: 使用者選 B——連光桿也改，判準是「指 openspec/specs/ 合併後的規格集合（相對 delta）才改，其他意思不動」：
- 改：規格 155 行光桿「正典」中約 115 行、手冊 49 行、docs 與 README 約 50 行；含約 15 個 Requirement／Scenario 標題（archive-merge、change-analysis、capability-naming-guard、context-projection、discuss-skill、desktop-app、server-read-api）
- 不改：正典詞彙／正典詞（ui-copy-vocabulary、user-documentation）、正典值／正典歸屬（workflow-config）、正典化（review-skill、quality-skill）、正典載入／正典 YAML（workflow-schemas）、正典順序（worktree 兩技能）、「本文是…的正典」這類泛指唯一真相的用法、程式碼註解裡的光桿「正典」
- 程式碼註解只改複合寫法與「引用了改名標題」的那幾行（newcmd.rs:281、capname.rs:165、validate_specs.rs:2、command/tests.rs:84、validate/tests.rs:178），其餘不動
- 已封存 90 檔不回改：使用者未反對，視為接受
**Ruled out**: A（只改複合寫法、光桿留給漸進汰換）——delta 與 specs 的對照最密集的句子（archive-merge、spec-validation）改完仍是「正典」，正是使用者要分清楚的地方
**Open**: 無——進入結論

## Conclusion

**Decision**: 繁中把 OpenSpec 的 specs 統一叫「正式規格」，與變更裡的「delta 規格」成對；英文維持 specs／canonical specs 不改。一個變更、零 delta、零行為變化，直接改字：
- `openspec/LANGUAGE.md` 新增「正式規格」詞條：definition 點名對照詞「delta 規格」與 `openspec/specs/<capability>/spec.md` 的位置；avoid 收「正典規格」「正典 spec」「正典 specs」「規格正典」（進機械守門）與「正典（指這個規格集合時）」（帶語境限定，守門跳過）；why 記「正典」讀不出「合併後的定案」、與 delta 成對才分得清，並列出不改的別義。
- 使用者可見面同批清零：`packages/ui/src/i18n.tsx` 的 specs.heading 改「正式規格」、specs.empty 改「此專案尚無正式規格」，`packages/ui/src/__tests__/specList.test.tsx:162` 斷言同步；docs 與 README 的複合寫法與指 specs 側的光桿「正典」全改（README「共用同一份規格正典」→「共用同一份正式規格」）。
- 正式規格散文（約 44 份）、手冊（19 頁）：複合寫法全改；光桿「正典」依判準逐句改——指 openspec/specs/ 合併後的規格集合才改，正典詞彙／正典值／正典化／正典載入／正典順序／「本文是…的正典」不動。含約 15 個 Requirement／Scenario 標題改名（如 spec-validation「validate --specs 驗證正式規格」、server-read-api「正式規格 spec 內文可讀」→ 以「正式規格內文可讀」為準）。直編不經封存合併，無未宣告刪除雷；不動 @trace 時戳；手冊直接改字、不重生。
- 程式碼與測試：只改複合寫法（crates 31、apps 12、packages 12、scripts 1）與引用改名標題的註解；光桿「正典」註解不動。
- 範例：archive-merge 的「封存套用 delta 至正典時…ADDED 需求名已存在於正典」改為「封存套用 delta 至正式規格時…ADDED 需求名已存在於正式規格」。
- 驗收：scripts 的 vocabulary-guard 與 docs-parity 綠、`speclink validate --specs --strict` 綠、ui vitest 綠、活的檔案 grep「正典規格」「正典 spec」「規格正典」歸零。
**Rationale**: 專案對照 OpenSpec，規格分變更裡的 delta 與合併後的 specs 兩種；使用者要的是這一對在中文裡分清楚。「正式規格」已是 LANGUAGE.md 兩條定義、兩份規格與三份九月討論的用詞，本決定是把舊寫法收斂到已經在發生的方向。光桿「正典」連同改是因為 delta 與 specs 對照最密集的句子（archive-merge、spec-validation）用的正是光桿；別義不改是因為「正典」在專案內另有四種以上意思。
**Rejected alternatives**: 只改複合寫法、光桿留給漸進汰換（先例 D1 作法）——對照最密集的句子改完仍是「正典」，沒達到分清楚的目標；光桿「正典」全 repo 機械替換——會改壞正典詞彙／正典值／正典化等別義；走 delta MODIFIED 改規格散文——零語意變化卻要約 40 個區塊，標題改名還踩未宣告刪除雷；回改已封存 90 檔——違反 LANGUAGE.md「歷史 artifacts 不回改」，且封存區 delta 是溯源證據；改英文標題為 "Specs"——使用者明示英文維持。
**Deferred**: 「delta」這個英文詞直出於繁中文案（i18n「delta 規格」）是否要比照 worktree／config.yaml 立明文例外——本題未裁；「正典」的其他意思（正典詞彙、正典值、正典化）是否另改名——不在本題；程式碼註解裡的光桿「正典」——留給漸進汰換。
**Capture to**: LANGUAGE.md（詞彙漂移：舊寫法「正典規格」收斂為「正式規格」）、proposal（範圍與判準）、tasks
**Next**: /speclink-propose --from-discussion canonical-spec-zh-wording
