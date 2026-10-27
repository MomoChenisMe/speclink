---
topic: Baseline Skill 明確載入並套用 openspec/config.yaml 的 rules.specs
slug: baseline-apply-spec-rules
status: promoted
promoted_to: baseline-apply-spec-rules
created: 2026-09-02
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: Baseline Skill 明確載入並套用 openspec/config.yaml 的 rules.specs

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

Baseline 直接寫正式 specs、不建 change，因此 rules.specs 經 instructions payload 注入的路徑對它不生效；現行 baseline.md 的 Step 1 只直讀 config.yaml 取 context 與 spec_locale，全文沒有 rules。使用者要求 Baseline 盤點前取得有效 workflow config、rules.specs 存在時明確套用到每一份正式 spec、在 capability map 確認與最終報告揭露套用的規則，並保留既有六項邊界（只記現況、不改 code、不建 change、gap filling、map 確認後才寫、最後 strict validation）。需求已含可驗證目標與邊界，無 grill 階段，直接以假設清單開場，使用者一次全數確認。

查證事實：instructions.rs:177 只在 change 的 artifact payload 放 rules；無 change 時 speclink instructions specs --json 印 No active changes（exit 0）；workflow-config show --json 的 fs 與 remote 共用 print_workflow_config（verbs/config.rs:617），payload 含 context、specLocale、rules，兩模式同形；技能內容測試落在 crates/speclink-core/tests/it/render_golden.rs（skill_for_both_tools）；ASSET_VERSION 現為 v1.26.1，改名與新技能皆走 minor。

相關 specs：workflow-config（show --json 形狀與 remote 同形，covered）、skill-routing（baseline 的 description 與出口句）、config-skill（技能契約寫法範本）、user-documentation（workflow 兩語言每站載明輸入）。Baseline 的行為契約 canon 沉默——改名討論 rename-onboard-to-baseline（2026-09-01）明文因行為未變而不建 baseline-skill spec，本次行為改變故理由反轉。無 in-flight changes。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-02)

**Focus**: Baseline 套用 rules.specs 的九條設計假設（規格落點、設定入口、規則用法、版號、測試、文件、模式範圍）一次盤點
**Position**: 使用者全數確認九條假設，設計定案：
- 規格落點：新 capability baseline-skill，比照 config-skill 三條 requirement——渲染與 golden 保護、盤點前取得 workflow config 並套用 rules.specs（含揭露與「規則是 Agent 產生時的指令，validate --strict 只查結構」）、既有六項邊界寫成一條以防後續默默改掉
- 設定入口：Step 1 改跑 speclink workflow-config show --json 取 context、specLocale、rules.specs；指令失敗即停下回報，不退回手讀 YAML（config.rs:617 兩模式共用同一輸出函式；config.md 已有失敗即 STOP 先例）
- 規則用法：全部逐字套用、不做「哪條適用 baseline」的篩選；capability map 確認訊息固定帶「本輪套用的 specs 規則」逐條原文，未設定時明寫不套用；最終報告固定一段揭露是否套用與套了哪幾條
- 版號與衍生物：ASSET_VERSION v1.26.1→v1.27.0（新增步驟走 minor，同改名與新技能先例），同批再生 5 份 golden 與 assets.lock，repo 內 .claude／.agents 的 SKILL.md 由 speclink update 再生
- 測試：crates/speclink-core/tests/it/render_golden.rs 加一支，以 skill_for_both_tools 斷言五要點（取得 config、讀 rules.specs、每份 spec 遵守、未設定照常、報告揭露）
- 文件：只動 docs/workflow.md 與 docs/workflow.zh-TW.md 的 baseline 站 Input 行；configuration 兩份不動
- 模式範圍：Baseline 的 Local／Remote 支援不變，只換讀設定的入口
**Ruled out**: 規格放進 workflow-config 或 skill-routing——職責混入，Baseline 行為契約仍無家；手讀 config.yaml 或自寫 YAML 解析——remote 模式讀不到本機檔且重複引擎邏輯；沿用 speclink instructions specs --json——無 change 時只回 No active changes；規則適用性篩選——判準無法固定，報告不可預期；只升 patch 版號——與新增步驟的份量不符
**Open**: 無——九條假設皆已定案

## Conclusion

**Decision**: Baseline Skill 的 Step 1 改以 speclink workflow-config show --json 取得有效 workflow config（context、specLocale、rules.specs），指令失敗即停下回報；rules.specs 存在時逐字套用到本輪產生的每一份正式 spec，並在 capability map 確認訊息與最終報告各固定一段揭露套用的規則（未設定時明寫不套用）；規則是 Agent 產生內容時的指令，validate --strict 只查結構。新建 capability baseline-skill 承載此契約與既有六項邊界（只記現況、不改 code、不建 change、gap filling、map 確認後才寫、最後 strict validation）。ASSET_VERSION v1.26.1→v1.27.0，同批再生 5 份 golden、assets.lock 與 repo 內生成 SKILL.md；render_golden.rs 加一支內容斷言測試；docs/workflow 兩語言 baseline 站 Input 行補 workflow config。Local／Remote 支援範圍不變。
**Rationale**: workflow-config show --json 是既有 Dual 入口，兩模式同形且已有規格與測試保護，沿用即免手刻 YAML 解析且在 remote 模式也讀得到正確文件；規則逐字套用不篩選，報告才可預期；行為改變後 Baseline 首次需要行為契約，比照其他技能各自一個 capability 最一致。
**Rejected alternatives**: 規格放 workflow-config 或 skill-routing（職責混入）；手讀 config.yaml 或自寫 YAML 解析（remote 模式讀不到本機檔、重複引擎邏輯）；沿用 speclink instructions specs --json（無 change 時只回 No active changes）；規則適用性篩選（判準無法固定）；只升 patch 版號（與新增步驟份量不符）；擴張 remote 支援（spec 寫回 server 是另一個 change）。
**Deferred**: none
**Capture to**: proposal（新 change：baseline.md asset、skills.rs 不動、ASSET_VERSION、golden×5、assets.lock、render_golden.rs 新測試、baseline-skill 新 spec、docs/workflow 兩語言、repo 內 .claude／.agents 生成物）
**Next**: /speclink-propose --from-discussion baseline-apply-spec-rules
