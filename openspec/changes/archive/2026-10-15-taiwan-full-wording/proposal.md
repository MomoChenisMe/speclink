## Summary

把正式規格、手冊與 openspec/config.yaml 裡的「全量」換成台灣慣用詞（完整測試／全部／完整），並在 openspec/LANGUAGE.md 新增「完整測試」詞條，擋住這個詞漂回使用者可見文件。

## Motivation

- 「全量」不是台灣慣用詞，讀者讀到要停下來猜意思。使用者在討論 test-speed-and-wording（2026-10-15）裁定換掉，英文維持 full test suite 不動（技能原稿已如此）。
- 同一個詞在專案內有五種意思，換詞要按意思分流：測試範圍（全量測試）、全部重新載入或查詢（全量重載／重查／收斂）、任務總數（全量計數）、手冊全部重生（全量重生）、一整套文件（全量投影、還原全量）。
- 目標使用者：透過 AI 代理跑 SDD 的開發者／PO／PM。使用情境：
  - 讀 config、review、verify 三個技能對「任務驗證步驟跑哪些測試」的規定（propose 寫 tasks、quality 兩站跑測試時）
  - 讀手冊的 manual、policy-config、remote-context、remote-offline 四頁
  - 讀 delivery-baseline、remote 系列、desktop-app 等正式規格
- 為什麼排在現在：在途變更 config-skill-rule-routing 的 delta 有 5 處「全量」，封存時會整塊蓋回 config-skill 正式規格。本變更以 `speclink change depends` 宣告等它封存後才開始實作，改字一次到位。

## Proposed Solution

- **替換原則**：修飾動作時用「全部」，指一整套東西時用「完整」。逐處對照表見 design D1。
  - 全量測試 → 完整測試；delivery-baseline 需求標題「root 單一指令全量驗證」→「root 單一指令完整驗證」
  - 全量重載／全量重查 → 全部重新載入／全部重新查詢；全量收斂 → 全部重新收斂；全量重建 → 全部重新建立
  - 全量計數 → 全部任務計數
  - 全量重生 → 全部重新產生
  - 全量投影 → 完整投影；還原全量 → 還原全部
- **openspec/LANGUAGE.md 新增「完整測試」詞條**：definition 指 repo 慣例的整套測試（root `npm run test:all`），對照詞「只跑受影響面的測試」；avoid 收「全量」；why 記裁定日期與另四種意思的替換詞。vocabulary-guard 的掃描面（三份 i18n、README、docs、技能原稿）目前零處「全量」，加 avoid 不會打紅。
- **正式規格 13 份直接改字**：其餘 12 份共 28 處，config-skill 以封存 config-skill-rule-routing 後的字面為準（依目前 delta 預估 7 處），含 4 個標題改名。直接改字、不經封存合併，不動 `@trace` 時戳（前例 canonical-spec-zh-wording）。
- **手冊 4 頁直接改字、不重生**：共 7 處；frontmatter 的 `generated` 一字不動。直編規格不動 `@trace`，手冊不會被標「可能過期」，所以手動改。
- **openspec/config.yaml 的 tasks 規則改一句**：「全量 `npm run test:all` 只由 CI 守門」→「完整測試 `npm run test:all` 只由 CI 守門」，經 `speclink workflow-config rules tasks` 寫入。
- **一行註解跟著改名**：scripts/release/delivery-gate.test.mjs 的段落註解引用了被改名的需求標題，同步改成新標題，免得留下指向不存在標題的引用。
- **零行為變化、零 delta、零版號**：不改識別符、CLI 旗標、`--json` 形狀或技能原稿。

### 相容性影響

- 人眼輸出：CLI 與桌面、server-web 介面零變動（三份 i18n 零命中）。`--json` 欄位名與 shape 完全不變。
- 回歸對照：技能原稿零命中，golden 與 assets.lock 不動，不進 ASSET_VERSION。
- 設定欄位：openspec/config.yaml 不新增、不改任何欄位與預設值，只改 rules 的 tasks 段其中一條規則的字面；本 repo 的 `speclink instructions tasks` 輸出的這條規則字面跟著變。
- 遷移：無。既有工作區不需執行任何指令。

### 影響的 crate 與 app

無任何 crate 或 app 的程式碼變動（speclink-core、speclink-cli、speclink-host、apps/desktop、apps/server-web、packages/ui 都不動）。唯一的非文件檔是 scripts/release/delivery-gate.test.mjs 的一行註解。

### 技能與工具影響

無。技能原稿（crates/engine/speclink-core/assets/skills/）零命中，claude 與 codex 兩工具的產出不變。新詞條經 `speclink language show` 立即對代理生效，不需再生。

## Alternatives Considered

- 英文改成「Full Test」——單用讀成「把某物徹底測一次」，不道地，原稿已用 full test suite。
- 只改「測試」這個意思的「全量」——另四種意思照留，問題只解一半。
- 先改正式規格、不等 config-skill-rule-routing 封存——封存會把 5 處「全量」蓋回 config-skill。
- 走 delta（MODIFIED）改字——純字面、零行為變化；13 份規格的需求要整塊 MODIFIED，改標題還要 REMOVED＋ADDED，成本遠大於收益。前例 canonical-spec-zh-wording 直接改 25 份規格、零 delta。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

（無）——字面更動不是需求變更，理由與規格掃描結果見 Impact 的「規格掃描」一段。

## Impact

- 規格掃描（step-3）：命中三份——ui-copy-vocabulary（守門詞由 LANGUAGE.md 動態解析，新增詞條自動生效，需求不變）、user-documentation（要求繁中文件使用詞彙表的詞，本變更是讓文件回到該要求）、manual-pages（頁格式契約不變，只改內文字面）。4 個標題改名是字面更動、不是行為變更，所以無 delta。
- Affected specs: 無 delta。直接改字的正式規格 13 份：openspec/specs/config-skill/spec.md、openspec/specs/context-projection/spec.md、openspec/specs/delivery-baseline/spec.md、openspec/specs/desktop-app/spec.md、openspec/specs/manual-skill/spec.md、openspec/specs/manual-task-marker/spec.md、openspec/specs/phase2-acceptance/spec.md、openspec/specs/remote-resilience/spec.md、openspec/specs/remote-workspace-data/spec.md、openspec/specs/review-skill/spec.md、openspec/specs/server-context-api/spec.md、openspec/specs/server-event-stream/spec.md、openspec/specs/verify-skill/spec.md
- Affected code:
  - New: 無
  - Modified:
    - openspec/LANGUAGE.md
    - openspec/config.yaml
    - openspec/manual/manual.md、openspec/manual/policy-config.md、openspec/manual/remote-context.md、openspec/manual/remote-offline.md
    - scripts/release/delivery-gate.test.mjs
    - 上列 13 份正式規格
  - Removed: 無
- 不動：openspec/changes/archive/、openspec/discussions/（含本討論記錄）、CHANGELOG.md、apps/desktop/src/release-notes/release-notes.json、技能原稿、三份 i18n，以及程式碼註解與測試名稱裡的「全量」（約 25 檔，漸進汰換）。
- 前置：config-skill-rule-routing 必須先封存（以 `speclink change depends` 宣告）。
