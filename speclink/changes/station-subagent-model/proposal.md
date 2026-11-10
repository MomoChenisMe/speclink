## Why

品質關卡（審查、驗證，及兩站合跑的 `/speclink-quality`）派出的子代理目前一律沿用主對話的模型，技能原稿沒有任何模型相關指示。透過 AI agent 開發的開發者（團隊情境下含 PO、PM）在跑 `/speclink-review`、`/speclink-verify`、`/speclink-quality` 時，沒有辦法依站指定子代理模型——例如審查用 opus、驗證用 sonnet。只靠工具自己的全域設定（CLAUDE_CODE_SUBAGENT_MODEL、Codex 的 default_subagent_model）會改到所有子代理、不能分站，也不跟著專案走。

本變更是討論 station-subagent-model 結論的第一刀（cut 1）：引擎設定、CLI 讀寫與兩站技能。第二刀 desktop-subagent-model-page（desktop「品質關卡 › 子代理模型」頁）之後另開，讀寫靠本刀的引擎函式。預設不變：沒設定的格子沿用主對話的模型，設定只是給需要的人的選項；這與 quality-gate-speed 否決的「為加速改預設用小模型」不衝突。

## What Changes

- `speclink/config.yaml` 新增頂層區塊 `subagent_model`（預設：缺席＝每站每個 AI 工具都沿用主對話）：
  - `review`、`verify` 兩站各以 AI 工具名稱對應模型名稱，例 `review: { claude: opus, codex: gpt-6.1-sol }`；`/speclink-quality` 沿用兩站的值，不另設；審查站兩軸（Standards、Correctness）共用一個值。
  - `choices` 是各 AI 工具的自訂模型清單，例 `choices: { codex: [gpt-6.1-sol, gpt-6-astra] }`，供第二刀的 desktop 下拉使用；站上的值不必出現在 `choices`。
  - 結構從嚴：區塊內只收 `review`、`verify`、`choices`，拼錯即整份 config 無法解析並點名該鍵（比照 `review` 區塊）；AI 工具名稱收任何 kebab-case 名稱（自訂描述子的工具也能用）；模型名稱不檢查。頂層維持寬鬆，舊版 CLI 略過此區塊。沒有環境變數覆寫，也不讀 `.speclink.yaml`。
- `speclink workflow-config show --json` 新增兩個欄位：`subagentModel`（整個區塊，缺席時為三個空物件）與 `builtinSubagentModels`（引擎內建清單，只有 `claude`：sonnet、opus、haiku、fable——這四個簡稱不帶版本號、不會過期）。人眼輸出不變。
- 新增寫入子指令 `speclink workflow-config subagent-model`，三種形式皆支援 `--dry-run` 與 `--no-color`：
  - `subagent-model review <tool> <model>`、`subagent-model verify <tool> <model>`：設定一格；`<model>` 為空字串時移除該格（回到沿用主對話）。
  - `subagent-model choices <tool> --stdin`：以 stdin 整段代換該 AI 工具的自訂清單（一行一個、頭尾空白去除、空行略過；空 stdin 移除該清單）；未帶 `--stdin` 時以非零 exit code 說明用法。
  - `<tool>` 不是 kebab-case、或第一個參數不是 `review`、`verify`、`choices` 時，以非零 exit code 拒絕且無檔案效果。成功 exit code 0、stdout 單行成功訊息。
  - 寫入只代換 `subagent_model` 區塊，其他鍵、註解與空行逐位元保留；remote 模式走與其他寫入子指令相同的讀-改-寫與版本衝突檢查；壞檔 fail-closed。
- `/speclink-review`、`/speclink-verify` 技能派子代理前以 `speclink workflow-config show --json` 讀 `subagentModel.<站>.<{{TOOL}}>`：有值才帶 model、沒值不帶；技能檔不寫模型值本身，只靠既有的渲染變數 `{{TOOL}}`，改設定不必重新生成技能檔。
  - 結果呈現開頭一行寫明本輪子代理用的模型；`/speclink-quality` 的合併摘要保留這一行。
  - 工具的子代理呼叫沒有 model 參數（例：Codex 未開 `expose_spawn_agent_model_overrides`）：照常派子代理、用主對話的模型，報告寫一行說明並提示開法。
  - 工具不認得模型名稱：該站停在派子代理那一步，原文回報工具的錯誤，並指出要改 config.yaml 的 `subagent_model`；不自動退回主對話的模型。
  - 工具不能派子代理：該站在主對話裡跑、不套用這個設定，報告寫明。
- `speclink/LANGUAGE.md` 新增詞條「子代理模型」。

相容性影響：

- `workflow-config show --json` 多兩個欄位，既有欄位不變；人眼輸出逐位元不變。
- 舊版 CLI 讀到含 `subagent_model` 的 config 時略過此區塊；新版 CLI 讀到拼錯的 `subagent_model` 子鍵時整份 config fail-closed——與 `review` 區塊相同的取捨。
- 技能原稿改動：ASSET_VERSION、golden 與 assets.lock 一併更新；使用者升級後要跑 `speclink update`，舊技能檔不讀這個設定。

## Capabilities

### New Capabilities

（無）step 3 掃描找到 workflow-config、review-skill、verify-skill、quality-skill、desktop-config 五個相關規格；本刀的行為全都落在前四個既有 capability 內，desktop-config 屬第二刀。

### Modified Capabilities

- `workflow-config`: 新增 `subagent_model` 區塊的嚴格解析需求與 `workflow-config subagent-model` 寫入需求；`workflow-config show` 的 `--json` 帶出 `subagentModel` 與 `builtinSubagentModels`。
- `review-skill`: 審查站派兩個子代理前讀子代理模型設定、有值才帶 model，呈現開頭寫明模型，以及工具不支援、名稱被拒、不能派子代理三種情況的行為。
- `verify-skill`: 驗證站派子代理前讀同一設定，套用與呈現規則同審查站。
- `quality-skill`: 三個子代理各帶自己站的模型；任一子代理因模型名稱被拒時本輪兩站都不記錄；合併摘要保留各站的模型行。

## Impact

- 目標使用者與情境：透過 AI agent 開發、跑品質關卡（`/speclink-review`、`/speclink-verify`、`/speclink-quality`）的開發者；設定由專案共用，團隊中 PO、PM 不直接操作。
- Affected specs: workflow-config、review-skill、verify-skill、quality-skill
- 影響的 crate 與 app：speclink-core（設定模型、改寫函式、技能原稿）、speclink-cli（show 與新寫入子指令）、apps/desktop（只為編譯跟上 `ContentEdit` 新欄位，行為不變）；speclink-server 透過同一引擎解析自動拒收拼錯的區塊，程式不改、只加回歸測試。
- 影響的技能與工具：review、verify、quality 三個技能的原稿，經 `speclink update` 再生成到 claude、codex、copilot 三個工具（含自訂描述子工具）。
- Affected code:
  - New: (none)
  - Modified:
    - crates/engine/speclink-core/src/workspace/config.rs
    - crates/engine/speclink-core/src/workspace/config/tests.rs
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/assets/skills/station-shared.md
    - crates/engine/speclink-core/assets/skills/review.md
    - crates/engine/speclink-core/assets/skills/verify.md
    - crates/engine/speclink-core/assets/skills/quality.md
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/copilot.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/adapters/speclink-cli/src/verbs/config.rs
    - crates/adapters/speclink-cli/tests/it/workflow_config.rs
    - crates/adapters/speclink-cli/tests/it/config_fail_closed.rs
    - crates/host/speclink-server/tests/it/api/policy_write.rs
    - apps/desktop/core/src/settings.rs
    - apps/desktop/src-tauri/src/lib.rs
    - docs/configuration.md
    - docs/configuration.zh-TW.md
    - speclink/LANGUAGE.md
    - 本 repo 進版控、由 speclink update 再生的 speclink 技能檔（各工具的技能目錄）
  - Removed: (none)
