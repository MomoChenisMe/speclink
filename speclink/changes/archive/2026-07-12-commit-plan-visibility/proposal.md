## Why

使用者實際回報（含截圖）：verify → archive → commit 連跑時，speclink-commit 技能在沒有輸出任何 commit 計畫或 commit 訊息的情況下，直接以 AskUserQuestion 問「依上述計畫 commit？」——「上述」在對話中不存在，使用者被迫盲簽。成因有兩層：技能的「Display commit plan」是散文式指示、無防呆，執行模型把計畫留在內部推理就提問；且技能把 commit 訊息生成排在使用者確認之後，「Show the generated message and allow editing」無確認把關，實務上被直接輾過——即使照章執行，使用者在確認當下也永遠看不到 commit 訊息。

目標使用者是透過 AI 代理跑 SDD 的開發者；情境是 commit 工具技能（utility skill，不屬工作流步驟）的確認閘門——這是使用者對「哪些檔案、以什麼訊息進版控」的唯一把關點，必須所見即所簽。

## What Changes

- 重排 speclink-commit 技能步驟順序：收集檔案（artifacts＋tracked source＋unrelated 分組）後，**先**生成 commit 訊息，再把「commit 計畫＋commit 訊息」以單一可見訊息一次輸出，**之後**才呼叫 AskUserQuestion 做單一確認閘門。
- 移除確認之後無把關的「Show the generated message to the user and allow editing before proceeding」路徑——訊息的檢視與修改機會併入確認閘門之前（使用者可經 Customize 或自由文字要求改訊息）。
- archive 子流程（Archive first, then commit together）執行後檔案集與訊息內容（Archived: yes）都會改變，SHALL 重新輸出更新後的計畫＋訊息並再次經 AskUserQuestion 確認，維持「每次提交前都有一次所見即所簽」的閘門語意。
- 新增 guardrail：呼叫 AskUserQuestion 前，commit 計畫與 commit 訊息必須已作為可見文字訊息輸出；確認問題的文字不得指涉對話中未曾輸出的內容（如「上述計畫」）。
- 三處同步：事實來源 crates/speclink-core/assets/skills/commit.md 修改後，repo 技能實例（.claude/skills、.agents/skills）同步更新，render golden 四份 snapshot 於乾淨樹上以 UPDATE_GOLDEN=1 再生並審視 diff。

## Non-Goals

- 不改 speclink 引擎程式碼——speclink-core 的 Rust 邏輯與 speclink-cli 的指令、旗標、人眼／--json 輸出皆不動。
- 不改其他技能（archive、apply、verify 等）的確認流程；本次只修 speclink-commit。
- 不改 commit 訊息格式本身（speclink(<change-name>): <summary> 與 Change/Tasks/Archived 欄位維持原樣）。
- 不採兩段確認（先確認檔案、再確認訊息）——討論已排除：互動成本較高、無對應收益（見來源討論 commit-plan-visibility）。
- 不動 .speclink.yaml 與 openspec/config.yaml——無新增或變更設定欄位。

## Capabilities

### New Capabilities

- `commit-skill`: 內嵌 speclink-commit 技能的提交確認閘門行為——commit 訊息於確認前生成、計畫與訊息以可見文字輸出後才確認、archive 子流程後重新確認；由 render golden（cargo test）驗證渲染產物內容。

### Modified Capabilities

(none)

## Impact

- Affected specs: `commit-skill`（新增）
- Affected code:
  - Modified:
    - crates/speclink-core/assets/skills/commit.md（事實來源）
    - crates/speclink-core/tests/render_golden.rs（新增確認閘門內容測試）
    - .claude/skills/speclink-commit/SKILL.md（render 產物，claude 工具）
    - .agents/skills/speclink-commit/SKILL.md（render 產物，codex 工具）
    - crates/speclink-core/tests/golden/claude.snapshot.md
    - crates/speclink-core/tests/golden/codex.snapshot.md
    - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - New: (none)
  - Removed: (none)
- 影響 crate：speclink-core（僅內嵌資產 assets 與 render golden 測試快照；引擎程式碼不動）、speclink-cli 不受影響。
- 影響技能與工具：speclink-commit 技能；claude 與 codex 兩個渲染目標（另含 neutral-cli、neutral-tool-call 兩個渲染變體的 golden）。
- 相容性影響：CLI 人眼與 --json 輸出皆不變，既有回歸對照（parity／color suite）不受影響；render golden 四份 snapshot 屬刻意更新；既有使用者專案於下次 speclink update 同步技能時取得新版 speclink-commit。
