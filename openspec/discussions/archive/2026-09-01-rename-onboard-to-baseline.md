---
topic: Speclink onboard Skill 重新命名為 baseline
slug: rename-onboard-to-baseline
status: promoted
promoted_to: rename-onboard-to-baseline
created: 2026-09-01
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: Speclink onboard Skill 重新命名為 baseline

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

onboard 名稱源自 OpenSpec，但兩邊行為已分歧：OpenSpec Onboard 帶新使用者走完一次完整 workflow；Speclink onboard 盤點既有 codebase、依目前行為建立初始正式 specs，不建 change、不教學。兩專案關係近，同名易讓使用者誤以為行為一致，故評估改名為 baseline（生成名 speclink-baseline）。需求明確（改名目標與邊界已陳述），無 grill 階段，直接以假設清單開場。

相關 specs：skill-routing（3 條 requirement 點名 onboard：入口情境聯集、交棒句邊集、出口不帶命令總表）、user-documentation（2 條 requirement 點名 onboard）、workspace-tools（update 清理承諾，現況不含 registry 差集清理）、review-skill 與 delivery-baseline（baseline 撞名證據）。onboard 是唯一沒有 per-skill capability spec 的流程技能。無相關 in-flight changes（cli-typed-engine-entry、remote-ctx-workspace 皆無關）。

公開狀態：版本 0.1.3（pre-1.0）、npm registry 未發佈、CLI 有 install scripts 與 Homebrew tap、無 speclink onboard CLI 子指令、無程式碼引用固定 skill ID。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-01)

**Focus**: onboard 改名的名稱定案與遷移方案（六條假設一次盤點）
**Position**: 使用者全數確認假設——改名為 baseline，同場在 openspec/LANGUAGE.md 釘住雙義：
- 名稱 baseline 語意最準：requirements baseline 是「第一批核可規格、後續變更以它為基準」的業界標準詞，不會被誤讀成教學（OpenSpec Onboard）或目錄初始化（init）
- 撞名證據：baseline 在現行文案已有五個用法——Apply baseline（品質站凍結點，crates/speclink-cli/src/verbs/station.rs、docs/workflow.md:126）、delivery-baseline spec、smell baseline（openspec/specs/review-skill/spec.md:5）、查核基線（docs/product-status.md:63）、drift 的變更假設基準（docs/workflow.md:39,166）；五處皆為帶修飾詞複合用法，可共存，但 LANGUAGE.md 須釘住「規格基準＝baseline 技能產出」vs「Apply baseline＝品質站凍結點」，並補齊 docs/workflow.md 兩處裸寫 baseline 的修飾詞
- 不留 deprecated alias：pre-1.0（0.1.3）、npm 空、無 CLI 子指令、無程式碼引用固定 ID、路由靠 description、無其他技能交棒到 onboard（入邊只在 skill-routing 路由表描述）；留 alias 反而造成 Agent 技能清單重複觸發
- update 現況不清孤兒目錄（init.rs 僅三條刪除路徑：worktree 政策關閉、工具下架、自訂描述子移除），change 須新增 registry 差集 prune：列舉各工具 skills 目錄下 speclink- 前綴目錄，不在本次應生成集合者刪除；沿用 init.rs:545 既有前綴所有權語意，對未來改名同樣生效
- change 範圍：skills.rs:132（name／description／asset 常數）、asset 檔改名與內文、ASSET_VERSION v1.24.0→v1.25.0、assets.lock、5 份 golden、repo 自身 .claude/.agents 生成物、8 份文件（README×2、workflow×2、getting-started×2、product-status×2）、3 份 specs delta（skill-routing、user-documentation、workspace-tools 補 prune）；skill-routing 的 Scenario 改名須帶 REMOVED-SCENARIO 宣告
- 不新建 baseline-skill capability spec（行為未變）；歷史全不動（封存 28 檔、@trace 檔案清單、workspace-chooser-onboarding、server e2e 的 team onboarding），僅 docs/workflow 兩語言站別段補一句「舊稱 onboard」
**Ruled out**: backfill——零撞名且涵蓋 gap-filling 模式，但「第一批基準」意味弱，敗給語意準確度；adopt——skill-routing 以「既有專案採用」描述此入口，但 adopt 是 init＋建規格整段採用的統稱，單獨命名建規格步驟會與 init 職責糊掉；codify——「正典化」已被「技能模板正典化生成」佔用；snapshot／seed——被 golden snapshot 與 propose 的 seed 語意佔用；deprecated alias——無消費者且製造重複觸發；tombstone 舊名清單——每次改名須維護，registry 差集更簡且語意一致
**Open**: 無——六條假設與命名皆已定案

## Conclusion

**Decision**: onboard Skill 改名為 baseline（Skill ID `baseline`、生成名 `speclink-baseline`），乾淨改名不留 alias；`speclink update` 新增 registry 差集 prune 清除孤兒的 speclink- 前綴目錄；openspec/LANGUAGE.md 同場釘住雙義——「規格基準」（baseline 技能的產出：依目前行為建立的第一批正式 specs）vs「Apply baseline」（品質站凍結點），docs/workflow.md 兩處裸寫的 baseline 補上修飾詞。
**Rationale**: baseline 是 requirements baseline 的業界標準用法，最準確表達「建立目前行為的規格基準」，且與 OpenSpec Onboard（教學走 workflow）明確切開；撞名的五處既有用法皆為帶修飾詞複合詞，以詞彙釘義即可共存；pre-1.0、npm 未發佈、無程式碼依賴固定 skill ID，直接改乾淨的成本最低、風險最小。
**Rejected alternatives**: backfill（零撞名、涵蓋 gap-filling 模式，但「第一批基準」意味弱）；adopt（是 init＋建規格整段「採用」的統稱，單獨命名此步驟會與 init 職責混淆）；codify（「正典化」已被技能模板正典化生成佔用）；snapshot／seed（被 golden snapshot 與 propose 的 seed 佔用）；deprecated alias（無消費者，且在 Agent 技能清單造成重複觸發）；tombstone 舊名清單（每次改名須維護，registry 差集沿用既有前綴所有權語意且一次到位）。
**Deferred**: none
**Capture to**: proposal（新 change：skills.rs 註冊項、asset 改名與內文、ASSET_VERSION v1.24.0→v1.25.0、assets.lock、5 份 golden、repo 自身 .claude/.agents 生成物、8 份文件、skill-routing／user-documentation／workspace-tools 三份 specs delta、update 的 registry 差集 prune 與 migration tests；skill-routing 的 Scenario 改名帶 REMOVED-SCENARIO 宣告；docs/workflow 站別段補「舊稱 onboard」）；LANGUAGE.md 詞彙釘義隨該 change 落地，不先行修改
**Next**: /speclink-propose --from-discussion rename-onboard-to-baseline
