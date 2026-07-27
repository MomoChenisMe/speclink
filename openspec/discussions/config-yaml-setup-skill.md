---
topic: config.yaml 的 context 寫法優化與設定技能
slug: config-yaml-setup-skill
status: open
created: 2026-07-27
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: config.yaml 的 context 寫法優化與設定技能

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者要新增一支 speclink 專屬技能，用於自動整理 `openspec/config.yaml`：搭配 CLI 時依 remote 或 local 模式決定設定存放，locale / spec_locale / tdd / audit 等政策逐項詢問，並以本專案的 config.yaml 為範本——但範本本身要先優化。追加需求：技能每次呼叫都能依當下 codebase 做一次迭代優化（codebase 會持續演進），且本專案現有的 context 寫得不夠好，需先考據 OpenSpec 對 context 的既定寫法。

模式：assumptions——codebase scout 找到 `crates/speclink-core/src/config.rs`（WorkflowConfig / update_workflow_config_text / WorkflowPolicyFields）、`crates/speclink-core/src/instructions.rs`（context 與 rules 的注入點）、`crates/speclink-core/src/init.rs`（config.yaml 範本）、`crates/speclink-cli/src/commands.rs`（config 動詞）、`apps/desktop/src-tauri/src/lib.rs`（本機／遠端寫入 command）、`apps/desktop/src/views/ProjectSettingsView.tsx`（既有設定頁），足以直接形成立場。

相關規格：workflow-config（正典規格對 `context` 一字未提，只規範四層解析、deprecation 警告、init 範本政策寫入位置）、desktop-config、command-runtime。目前無進行中的變更。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-27)

**Focus**: 一支「搭配 CLI、依 local/remote 決定儲存」的設定技能，現有基建撐得住嗎？
**Position**: 撐不住——core 有改寫器但 CLI 沒有任何動詞可寫 `openspec/config.yaml`，remote 路徑更只能經動詞：
- `speclink config path` 實測回 `~/Library/Application Support/speclink/config.yaml`：`config get/set/list/unset/reset/edit` 全部管全域設定，完全不碰專案的 `openspec/config.yaml`。
- core 的 `config.rs:503 update_workflow_config_text` 是完整的 text→text 改寫器（政策四欄為完整目標態、context 三態、rules 整份代換，未知鍵原樣保留），但目前唯一消費者是 desktop 的 Tauri command（`apps/desktop/src-tauri/src/lib.rs:263` 本機／`:644` 遠端）。
- remote 模式不可能靠寫檔：`remote_write_workflow_config` 要求 `expected_revision: u64` 樂觀鎖，且 remote checkout 根本不建本機 `openspec/`（`connections.rs:681` 測試斷言）。故 local/remote 的分流必須在 CLI/Host 層裁決，技能側不該有分支。
- 逐項詢問的四欄正好是 `WorkflowPolicyFields`（locale / spec_locale / tdd / audit），與 desktop `ProjectSettingsView` 的政策區塊同源；`context` 與 `rules` 則屬可從 codebase 推導的內容層，問使用者無意義。
**Ruled out**: 技能直接以檔案工具寫 `openspec/config.yaml`——local 可行但 remote 永久無解，兩條路徑不對等。
**Open**: CLI 動詞的形狀（擴充 `speclink config` 加 `--project` 面，或另立動詞）？context/rules 的產出如何避免每次呼叫都全量改寫（迭代收斂判準）？

### Round 2 — assumptions (2026-07-27)

**Focus**: OpenSpec 對 `context` 有無既定寫法？本專案這份該怎麼優化才算對？
**Position**: 上游只給定位不給長度，但注入機制本身就是硬判準——context 進四個 artifact、rules 只進對應 artifact，據此本專案的 context 可砍掉三分之二：
- 專案內既有說明只有三處且都極簡：`init.rs:38-45` 範本註解舉的例子是四行（Tech stack / conventional commits / Domain）；`docs/configuration.md:100` 一句「Project context shown to AI when creating artifacts」；正典規格 `openspec/specs/workflow-config/spec.md` 對 context **一字未提**（只規範四層解析、deprecation 警告、init 範本政策寫入位置）。
- 上游 OpenSpec 官方 customization 文件明載：「Context appears in ALL artifacts」對比「Rules ONLY appear for the matching artifact」，範例同樣是四行技術棧＋慣例；**未給任何長度或精簡建議**。
- 程式碼比文件更硬：`instructions.rs:178` 每次 `build_artifact` 都以 `wf.context_text()` 整份帶入，`rules` 則是 `wf.rules_for(&artifact.id)` 分 artifact。context 長度＝proposal/specs/design/tasks 每次產生都要付四遍的固定成本。
- 本專案 context（約 60 行實體內容）有三類該砍：(a) **與自動注入重複**——`tdd: true` 已讓 tasks instruction 自動帶 Red-Green-Refactor（`instructions.rs:150-157`）、`audit: true` 同理（:158-165）、`spec_locale: tw` 已自動注入「散文用繁中／結構標記與 SHALL 保留英文」且多帶中文弱語言偵測（:120-135），而「紅線約束」把 TDD 與語言兩條又複述一遍；(b) **只對單一 artifact 咬合、該降級為 rules**——「分層邊界」與 rules.design 既有條目重疊，「`--json` 欄位一律 camelCase」在 rules.specs 與 rules.tasks 各已有一條；(c) **對 AI 無作用的 meta**——「精確版本一律以各 Cargo.toml 為準（會過時）」是寫給人看的免責聲明。
- 該留下的是四個 artifact 都需要的專案身分：Speclink 是什麼、Local Repo 與 Remote Store 雙路徑、crate 地圖、雙輸出路徑（人眼＋`--json`）、正典 vs delta、設定分兩層，以及三份 docs 的指路（product-status / platform-architecture / roadmap）——指路成本低、收益高，保留。
**Ruled out**: 把 context 當「專案知識全集」寫——注入機制決定它是四份 artifact 共同前言，不是知識庫；artifact 專屬內容有 rules 這個更精準的位置。
**Open**: 「每次呼叫依 codebase 迭代優化」如何收斂而非每跑一次就重寫一遍？技能是否應只產 diff 交使用者裁決、不自動寫入？優化後的 config.yaml 要不要反過來成為技能內嵌的參考範例（三處內嵌技能同步的維護成本）？

## Conclusion

<!-- Written by `speclink discuss conclude`:
**Decision** / **Rationale** / **Rejected alternatives** / **Deferred** / **Capture to** / **Next** -->
