---
topic: 把 Matt Pocock 的 improve-codebase-architecture 融入 speclink:模型發起的架構改進討論流程
slug: architecture-improve-flow
status: promoted
promoted_to: add-improve-flow
created: 2026-08-01
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 把 Matt Pocock 的 improve-codebase-architecture 融入 speclink:模型發起的架構改進討論流程

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者要把 Matt Pocock 的 improve-codebase-architecture 技能理念融入 speclink:新增一個與 discuss 同層、由模型掃描 codebase 主動提出改進 candidates 的討論流程,下游沿用 rounds → conclude → promote → propose 回歸 SDD。

模式:assumptions——本地相關檔案充足(speclink-discuss skill、speclink-cli、speclink-core 的 discuss 模組)。

Codebase scout:add-round 的 --mode 為自由字串(crates/speclink-cli/src/main.rs:689 僅 default "interview",core 的 add_round 不驗證值),討論記錄基建可零改動複用。In-flight changes code-review-stage 與 verify-station-parity 已佔用 review 語意(apply 後品質站)。

Source doc: https://github.com/mattpocock/skills/tree/main/skills/engineering/improve-codebase-architecture
(SKILL.md ＋ HTML-REPORT.md ＋ agents/openai.yaml;三段式 Explore → HTML 報告列 candidates → Grilling loop,依賴 /codebase-design 架構詞彙、/grilling 討論方法、CONTEXT.md 領域詞彙、ADR 防重提機制)

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-01)

**Focus**: 新流程以什麼形態掛進 speclink——記錄基建、記錄粒度、掃描範圍、呈現形式、命名
**Position**: 以 discuss 同層技能 `/speclink-improve` 落地,完全複用討論記錄基建,CLI 零改動;使用者確認全部五項假設並定案命名。
- add-round 的 mode 為自由字串(crates/speclink-cli/src/main.rs:689;core add_round 不驗證值),新技能可用自訂 mode 標籤寫回同一套記錄,new/context/conclude/promote/link/封存生命週期全數免費繼承——零新 seam、零 adapter
- 一次掃描＝一份討論記錄:Round 1 記 candidates 清單,後續 rounds 逐一 grill 使用者挑的 candidate,結論經既有 fan-out 機制扇出多個變更
- 被否決的 candidates 落 Ruled out;已封存討論即 ADR 替代品,未來掃描開場先讀以防重提
- 範圍先收斂(YAGNI):使用者指定方向優先,否則以 git log 熱點推斷,絕不全 repo 盲掃
- 呈現走對話內 markdown ＋討論記錄,不做 Matt 的臨時 HTML 報告(speclink 已有 GUI board 與討論抽屜)
- Matt 生態與 speclink 對應:/grilling ≈ discuss interview mode;CONTEXT.md ≈ LANGUAGE.md(語意偏使用者文案,有落差);ADR ≈ 已封存討論的 Ruled out(superset);seam/depth/deletion test ≈ discuss Step 4 的 interface depth check
**Ruled out**: 平行記錄類型(動 CLI/GUI/生命週期,成本大無收益);每 candidate 一份記錄(掃描本身的取捨推理無處安放,一次掃描灌爆 board);臨時 HTML 報告(第三個呈現面,YAGNI);含 review 的命名(與 in-flight 品質站 code-review-stage、verify-station-parity 撞名);全 repo 盲掃(candidates 不可比、討論失焦)
**Open**: 掃描機制(inline vs Explore subagent vs 混合);技能內文結構(自包含 vs 執行期引用 discuss);LANGUAGE.md 是否承載架構詞彙

### Round 2 — assumptions (2026-08-01)

**Focus**: 使用者三項裁定(subagent 上限、文件同步、desktop 卡片標示)的落地方式
**Position**: 三項皆採納;Round 1 的「CLI 零改動」修正為「記錄機件零改動,但卡片標示需開一條小 seam」。
- 掃描 subagent 硬上限 2 個 Explore,上限與觸發判準寫死在技能內文
- 技能檔是引擎生成物,不是手寫檔:新增 crates/speclink-core/assets/skills/improve.md 模板(claude＋codex 兩份生成檔)→ 乾淨樹 golden 再生 → 本 repo `speclink update` 落地 .claude/skills/speclink-improve/ 與 CLAUDE.md/AGENTS.md 注入區塊的 workflow 段(discuss?/improve? → propose);README.md 與 README.en.md 的 workflow 圖與 improve 一節——鏡像 code-review-stage tasks 5.2/5.3 的安排
- 卡片標示走 code-review-stage 的審查小章樣式:討論 frontmatter 新增 kind: improve(缺省即一般討論,舊記錄零遷移);discuss new 增 --kind 旗標(白名單驗證);DiscussionInfo(crates/speclink-protocol/src/query.rs:499)增選填欄位;DiscussionColumn 卡片行內小章(lucide icon＋Tooltip,不加文字列)＋DiscussionDrawer 同步標示;i18n tw/en 詞條;LANGUAGE.md 增正典詞(暫提「改進討論」)
**Ruled out**: 以 slug 前綴 improve-* 或 round mode 字串讓 GUI 推斷討論型別(stringly、無契約——測不到的邊界＝沒有契約);手寫 .claude/skills/speclink-improve/(技能同步已正典化為引擎模板,手寫會被 speclink update 蓋掉)
**Open**: LANGUAGE.md 正典詞定名(暫提「改進討論」);kind 欄位名(kind vs origin,傾向 kind)

### Round 3 — assumptions (2026-08-01)

**Focus**: Matt 原文精髓段(scope-before-you-scan、有機探索 friction 清單、deletion test)是否逐條保留進技能模板
**Position**: 逐條保留,寫入 improve.md 模板的 Step 2(範圍收斂)與 Step 3(掃描),僅兩處在地化改寫——此為對 propose/apply 的契約,不得濃縮。
- Step 2 照搬原文:使用者點名方向(模組/子系統/痛點)即用之、跳過推斷;否則走 git log 熱點推斷,近期常變區域加權(deepening 的回報在讓未來變更更容易);熱點分散無焦點就放寬網
- 在地化增補:熱點推斷輔以 openspec/changes/archive 的 touched 記錄——比裸 git log 多了「哪個意圖動了哪些檔」的訊號
- Step 3 五條 friction 訊號逐條照搬:概念理解需跳多個小模組、interface 複雜度逼近實作的 shallow module、為測試抽純函式但 bug 藏在呼叫端(無 locality)、緊耦合跨 seam 洩漏、難以透過現行 interface 測試的區域;並保留「有機探索、不逐條打勾」的原文精神
- deletion test 為 candidate 准入判準:刪掉後複雜度「集中」才算訊號,「只是搬家」不算
- 兩處在地化(前輪已裁):讀 CONTEXT.md/ADR → 讀 LANGUAGE.md(Step 0)＋範圍內 openspec/specs ＋已封存討論防重提(Step 1);Explore subagent → 混合機制、硬上限 2
**Ruled out**: 改寫或濃縮 friction 清單與 deletion test(原文即判準,濃縮會把「怎麼認出 shallow」的操作性丟掉)

## Conclusion

**Decision**: 新增與 discuss 同層、由模型發起的架構改進流程 `/speclink-improve`:六步骨架(載入詞彙 → 防重提檢查 → 範圍收斂 → 掃描 → 建記錄呈現 candidates → grilling 收斂),掃描產出寫入既有討論記錄(Round 1 = candidates,mode 標籤 scan),grilling 即 discuss 的 interview 紀律(depth check 無條件執行),經 conclude → promote/link 扇出變更回歸 SDD。含 desktop 討論卡片的 improve 標示與全套文件同步。Step 2/Step 3 逐條保留 Matt 原文精髓——scope-before-you-scan(使用者方向優先、git log 熱點推斷加權近期變更、分散則放寬網)、五條 friction 訊號、有機探索精神、deletion test 准入判準——僅兩處在地化(CONTEXT.md/ADR → LANGUAGE.md＋specs＋已封存討論;Explore subagent → 混合機制上限 2),熱點推斷輔以已封存變更的 touched 記錄(Round 3 契約,模板不得濃縮此段)。
**Rationale**: speclink 已具備 Matt Pocock 技能生態的全部下游機件(grilling ≈ interview mode、ADR ≈ 已封存討論的 Ruled out、seam/depth 詞彙 ≈ depth check、CONTEXT.md ≈ LANGUAGE.md),唯一缺的是「模型播種決策樹」的前段;完全複用討論記錄基建讓扇出、封存生命週期、propose 銜接全數免費繼承,新增面積僅剩:引擎技能模板一份、frontmatter kind 欄位一條、GUI 小章一枚。
**Rejected alternatives**: 平行記錄類型(動 CLI/GUI/生命週期,無對應收益);每 candidate 一份討論記錄(掃描取捨推理無處安放、灌爆 board);臨時 HTML 報告(speclink 已有 GUI 呈現面);含 review 的命名(與 in-flight 品質站撞名);全 repo 盲掃(candidates 不可比);slug 前綴/mode 字串推斷型別(無契約);手寫技能檔(會被 speclink update 蓋掉);LANGUAGE.md 擴充承載架構詞彙(章程是使用者文案,agent 詞彙留在技能內文);執行期引用 discuss skill(載入鏈不確定,自包含精簡版勝出);改寫或濃縮 friction 清單與 deletion test(原文即判準,濃縮丟操作性)
**Deferred**: LANGUAGE.md 正典詞定名(暫提「改進討論」)與 kind 欄位名(kind vs origin)於 propose 時定稿;candidates 卡片 icon 選型留給實作
**Capture to**: proposal(轉出新變更)
**Next**: /speclink-propose --from-discussion architecture-improve-flow
