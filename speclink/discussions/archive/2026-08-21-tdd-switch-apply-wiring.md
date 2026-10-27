---
topic: speclink 的 tdd 開關是否如 Spectra 般接進 apply 與 debug
slug: tdd-switch-apply-wiring
status: promoted
promoted_to: tdd-switch-apply-wiring
created: 2026-08-21
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: speclink 的 tdd 開關是否如 Spectra 般接進 apply 與 debug

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在改用 speclink 前曾把 TDD 寫進 OpenSpec config.yaml 的 rules.tasks 與 CLAUDE.md，痛點是「開關只作用在產 tasks，apply 用另一套 instructions」。看到 Spectra 的接法（.spectra.yaml 設 tdd: true → apply 技能另跑 instructions --skill tdd → 每 task Red→Green→Refactor、spec Example 轉第一批測試）後，想確認 speclink 是否同樣接法。需求明確（驗證接線現況＋評估），無 grill 階段。

掃描結果：speclink 已有同款接法且更完整（tasks 端與 apply 端都接），但抓到消費端漂移。相關 specs：workflow-config（tdd 政策欄位與四層解析）、config-skill；相關資產：crates/speclink-core/assets/skills/apply.md、tdd.md、crates/speclink-core/src/skills.rs。無相關進行中變更（desktop-schema-panel 無涉）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-21)

**Focus**: speclink 的 tdd 開關是否已如 Spectra 般接進 apply，接線有無缺口
**Position**: speclink 已有同款接法且更完整，但四項漂移假設全數獲使用者確認，修法方向已定：
- 現況已接：tdd 為正式政策欄位（正典 openspec/config.yaml，workflow-config spec）；apply.md 步驟 5 開啟時跑 `speclink instructions --skill tdd`、每 task Red→Green→Refactor；spec Example 的 GIVEN/WHEN/THEN 轉第一批 RED 測試（apply.md:170＋tdd.md「Using Spec Examples」）；instructions tasks payload 也含 TDD 紀律——tasks 端與 apply 端雙接，比 Spectra 描述多一端；worktree 版 apply 組合 apply.md 本體，掛勾自動繼承
- 漂移 1（確認）：apply.md:131 叫 agent 讀 `.speclink.yaml`，但正典已搬 openspec/config.yaml——本 repo 的 .speclink.yaml 無 tdd 鍵，照字面執行開關靜默失效，只靠 CLAUDE.md 硬規定兜住
- 漂移 2（確認）：修法採「payload 帶有效值」——instructions apply --json 已帶 locale（引擎解析完的結果）但缺 tdd/audit；補上後 skill 不可能讀錯層，環境變數覆寫自然生效；audit 同機制一併帶入
- 漂移 3（確認）：tdd.md「set in `.speclink.yaml`」過時字句隨手修
- 漂移 4（確認）：tdd.md 自述可 standalone 呼叫但 registry() 不渲染技能檔，入口是死文字——修的方向待定（見 Open）
- 額外決定（使用者裁定）：移除 `.speclink.yaml` 舊鍵相容層（含棄用警告）——第一個正式版發布時就已無舊鍵，相容層是死重；四層解析縮為三層（環境變數 ＞ openspec/config.yaml ＞ 內建預設），workflow-config spec 隨之改
**Ruled out**: skill 改跑 `workflow-config show` 自查政策——多一次指令且 skill 文字仍可能漂移，payload 帶值更穩
**Open**: 漂移 4 的修向——渲染 standalone tdd 技能檔（補上 debug 端入口，對齊 Spectra「apply 與 debug」）或刪除 tdd.md 的 standalone 描述（YAGNI）

### Round 2 — interview (2026-08-21)

**Focus**: 漂移 4 的修向——渲染 standalone tdd 技能檔，或刪除死文字
**Position**: 刪除 tdd.md 的 standalone 描述——TDD 只作為 apply 的內嵌紀律（經 `instructions --skill tdd` 取得），不渲染獨立技能檔。
- debug 端走 TDD 交由各專案自備技能（如本 repo 的 tdd-workflow），不由 speclink 引擎承擔
- tdd.md 的「Usage Modes」段與 `/speclink:tdd` 入口描述隨修復一併移除
**Ruled out**: 渲染 standalone tdd 技能檔（registry 加一筆）——使用者裁定 YAGNI；補入口的收益不及維護面擴大
**Open**: 無——全部節點已決，進入結論

## Conclusion

**Decision**: speclink 的 tdd 開關已如 Spectra 般接進 apply（且 tasks 端多接一端），不需新建機制；要修的是消費端接線的四處漂移：
1. `instructions apply --json` payload 新增 tdd 與 audit 有效值欄位（引擎四層解析完的結果，如既有 locale 欄位）
2. apply.md 步驟 5 改依 payload 欄位判斷，移除「Read `.speclink.yaml`」字句
3. tdd.md 移除「set in `.speclink.yaml`」過時字句，並刪除 standalone 模式描述（Usage Modes 段與 `/speclink:tdd` 入口）——TDD 只作為 apply 內嵌紀律
4. 移除 `.speclink.yaml` 舊鍵相容層（含棄用警告）：解析縮為三層（環境變數 ＞ openspec/config.yaml ＞ 內建預設），workflow-config spec 的舊鍵 scenario 隨之刪改
**Rationale**: 開關存在但 apply.md 叫 agent 讀已搬家的 `.speclink.yaml`，照字面執行 TDD 紀律靜默失效（本 repo 靠 CLAUDE.md 硬規定兜住）；payload 帶引擎解析完的有效值讓 skill 不可能讀錯層、環境變數覆寫自然生效，也消除 skill 文字再漂移的面。相容層在第一個正式版發布時即無使用者，屬死重。
**Rejected alternatives**: skill 自跑 `workflow-config show` 自查政策（多一次指令、skill 文字仍可能漂移）；渲染 standalone tdd 技能檔補 debug 端入口（使用者裁定 YAGNI，debug 端交各專案自備技能）；保留舊鍵相容層（無歷史使用者可相容）
**Deferred**: none
**Capture to**: proposal（轉為變更；波及 workflow-config spec 的解析層數與 payload 欄位、apply.md／tdd.md 資產——踩 MARKER_VERSION／golden／assets.lock 三連動，deprecation_warning.rs 測試隨相容層移除）
**Next**: /speclink-propose --from-discussion tdd-switch-apply-wiring
