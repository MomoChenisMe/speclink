## Summary

`[M]` 標記的語意自「手動測試」放寬為「任何 agent 無法代行、需要使用者親手操作的任務」，同步收斂正典詞彙（「手動任務」／「待手動」）、desktop 徽章與章的文案、六支技能資產的敘述；引擎行為零改動。

## Motivation

現行 `[M]` 的正典定義綁死在「測試」：LANGUAGE.md 詞條叫「手動測試」，desktop 任務列徽章寫「手動測試」四字，技能資產寫 "manual-verification"。但實務上需要人動手的不只測試——去外部服務建 OAuth app、把金鑰放進 keychain、開通第三方帳號，這些 agent 都做不到，卻沒有正當的標記歸屬。使用者看到「手動測試」徽章，會以為只有測試類任務才能標 `[M]`，起草時就漏標，導致 agent 卡在自己做不到的任務上。

引擎側完全不涉語意：解析器只剝前綴、「寫碼任務全完成」預測子只看 manual 旗標、位置 lint 只看字面位置——放寬只發生在人讀的文字層，成本極低。

目標使用者是透過 AI 代理跑 SDD 的開發者；使用情境橫跨 propose／ingest 起草任務時的標記判斷、apply 遇到手動前置任務時的交棒、以及 desktop 看板與任務列的標示判讀。

本提案承接已結論的同名討論（討論記錄 manual-marker-scope-beyond-tests，2026-08-14）。

## Proposed Solution

- **正典詞彙改名**（openspec/LANGUAGE.md）：「手動測試」詞條改為「手動任務」、「待手測」詞條改為「待手動」，各自更新 definition／avoid／why 與裁定日期。「手動測試」進「手動任務」的 avoid 並帶語境限定「（此概念上）」——docs 內「每日手動測試」這類指真測試的正當用法不受機械守門誤傷；「待手測」為自造詞、無其他正當用法，進「待手動」的 avoid 不帶限定。
- **UI 文案**（packages/ui/src/i18n.tsx）：任務列徽章 tw「手動測試」→「手動」、en "Manual test" → "Manual"；看板章 tw「待手測」→「待手動」（含 tooltip「待手動·剩 N 項」）、en "Awaiting manual test" → "Awaiting manual"。英文一併改是語意放寬所致，非詞彙守門所致。對應 UI 測試期望值同步更新。
- **六支技能資產放寬敘述**（crates/speclink-core/assets/skills/ 的 apply、ingest、propose、review、verify、quality）："manual-verification" 敘述改為放寬後的手動任務語意；propose 的「Code tasks and automated tests never carry it」改寫為「agent 做得到的都不帶」方向；apply 補一句「寫碼任務依賴未勾的 `[M]` 時，停下來請使用者先做，不要繞過去」。
- **產物三連動**：資產內文改動觸發 MARKER_VERSION 自 v1.19.13 進版、golden 快照重生、assets.lock 重生。
- **規格對齊**：manual-task-marker（標記需求改名為手動任務語意、apply／ingest 技能需求的定義與指示放寬、apply 補「被擋即停」條文）、desktop-app（徽章與章兩條需求改名改文）、propose-skill（起草標記需求的定義放寬）。含舊詞的需求名以 REMOVED＋ADDED 成對宣告改名改文。
- **守門面擴充**（ui-copy-vocabulary ＋ scripts/vocabulary-guard.test.mjs）：packages/ui/src/i18n.tsx 納入使用者可見文案面——本次改的徽章與章字串正住在這個檔，卻不在守門面內，不補上的話新詞漂回去沒人擋。納面時一併修正該檔既有的 avoid 詞違規（若守門實測揭露）。
- **Purpose 直編**：manual-task-marker 與 propose-skill 兩份正典規格的 Purpose 段含「手動測試」語意，delta 合併不觸及 Purpose，比照 spec-purpose-backfill 前例直編。

## Non-Goals

- **不追討其他規格散文的舊詞**：client-protocol、quality-skill、review-skill、verify-skill 的條文僅字面提及「手動測試」，其釘住的行為契約（時序、守門、點名義務）在放寬後不變，沿 zh-tw-vocabulary-drawer-and-quality-station 變更 design D1 的界線不回改。程式碼註解原則同樣不追討；例外（品質關卡回合經使用者裁定納入）：引用「已被本變更改名之需求」的註解與旁測敘述更新為新名，引擎的使用者可見英文拒絕訊息與相鄰 doc 註解一併放寬——留舊名會指向不存在的需求、留舊訊息會讓同一次 CLI 流程出現兩套說法。
- **不改 `[M]` 字母**：既有 tasks.md 與封存區的標記全部繼續有效，零遷移。
- **不加第二種標記**：不區分「前置手動」與「驗收手動」——manual 欄位維持布林，client-protocol 的欄位形狀不動。
- **不改引擎行為**：解析、預測子、各守門、位置 lint 零改動；speclink-cli／speclink-host／speclink-server 行為不動。
- **不動 openspec/changes/archive/ 與 openspec/discussions/archive/**：已封存內容是稽核資料，不回改。

## Alternatives Considered

- **加第二種標記區分前置／驗收手動**：manual 欄位要從布林變值域，波及 client-protocol 與 desktop，語意放寬用不到——落選。
- **改 `[M]` 字母**：既有 tasks.md 與封存區全部作廢，代價不成比例——落選。
- **起草指引加 `[M]` 排序規則**：任務起草通則已有依賴排序，專屬規則等於把通則抄一次——落選。
- **章用「待動手」**：使用者裁定語感不對，「待手動」才是「輪到你動手」且與徽章「手動」同詞根——落選。

## Impact

- Affected specs: manual-task-marker、desktop-app、propose-skill、ui-copy-vocabulary
- Affected code:
  - Modified:
    - openspec/LANGUAGE.md
    - openspec/specs/manual-task-marker/spec.md（Purpose 直編）
    - openspec/specs/propose-skill/spec.md（Purpose 直編）
    - packages/ui/src/i18n.tsx
    - packages/ui/src/__tests__/taskList.test.tsx
    - packages/ui/src/__tests__/awaitingManualBadge.test.tsx
    - scripts/vocabulary-guard.test.mjs
    - crates/speclink-core/assets/skills/apply.md
    - crates/speclink-core/assets/skills/ingest.md
    - crates/speclink-core/assets/skills/propose.md
    - crates/speclink-core/assets/skills/review.md
    - crates/speclink-core/assets/skills/verify.md
    - crates/speclink-core/assets/skills/quality.md
    - crates/speclink-core/src/init.rs
    - crates/speclink-core/tests/golden/claude.snapshot.md
    - crates/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/speclink-core/tests/golden/codex.snapshot.md
    - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/speclink-core/tests/golden/remote-claude.marker.md
    - crates/speclink-core/tests/golden/assets.lock
  - New: （無）
  - Removed: （無）

### 相容性影響

- 人眼輸出：desktop 任務列徽章與看板章的繁中／英文字面改動；`--json` 欄位名與 shape **完全不變**（manual 布林與 codeTotal／codeComplete／codeRemaining 三欄不動）。
- 回歸對照：golden 快照與 assets.lock 屬**刻意變更**，同批重生。
- 遷移：既有工作區執行 speclink update 即取得新技能文案；既有 tasks.md 的 `[M]` 標記不需任何遷移（字母與位置規則不變）。
- **BREAKING（產物層）**：MARKER_VERSION 自 v1.19.13 進版，既有工作區跑 speclink update 會整套再生受管檔（本 repo 的 .claude/skills/、.agents/skills/ 兩目錄與注入檔 CLAUDE.md、AGENTS.md 亦隨之再生——後兩者只動版號標記行，皆不逐一列入上表）。

### 影響的 crate 與 app

`speclink-core`（技能資產、MARKER_VERSION、golden）、`packages/ui`（i18n 與測試；desktop 經由它取得新文案）。不動 `speclink-cli`、`speclink-host`、`speclink-server` 的行為。

### 技能與工具影響

影響 apply、ingest、propose、review、verify、quality 六支技能的內文，claude 與 codex 兩個工具的產出（.claude/skills/、.agents/skills/）皆隨資產再生。
