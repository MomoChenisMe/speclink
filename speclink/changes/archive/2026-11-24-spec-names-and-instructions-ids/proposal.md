## Why

內嵌宿主的 Agent 用真實模型照 ingest 技能，把一份英文的 delta 規格改成專案設定的規格語言。結果有兩個問題。第一，引擎的指示與技能都只說「結構標記維持英文」，沒說 `### Requirement:` 與 `#### Scenario:` 後面的名稱要怎麼處理：模型把 3 條需求、9 個情境的名稱全部翻掉，回答裡也沒提。這份 delta 是 ADDED 所以沒出事，但同樣的請求用在 MODIFIED、REMOVED、RENAMED 時，名稱必須和正式規格一字不差。第二，delta 規格在三個動詞有三種寫法（讀 `artifact cat specs/<capability>`、取指示 `instructions specs`、寫回 `new artifact spec <capability>`），技能沒有講清楚，`instructions` 的錯誤訊息又不列可用的代號，模型試到第 5 次才成功。propose 技能有反方向的同一個坑。

目標使用者：透過 AI agent 寫變更的開發者，以及在內嵌宿主的對話裡請 Agent 修改提案的非開發者（PO、PM）。使用情境：propose 與 ingest 技能建立或更新 specs 產出物的階段。

來源：討論 embedded-host-needs-9（H1 與 H2）。相關的正式規格掃描結果：workflow-config（instructions 對 specs 產出物回報解析後的規格語言）、ingest-skill（既有 artifact 的更新經動詞整份寫回）、propose-skill（artifact 的讀取與改寫一律經動詞）、spec-validation（change 驗證納入合併守門，已在封存前擋下對不上正式規格的名稱，這次不改）、workflow-schemas（instruction 文字的單一來源，這次不改）。

## What Changes

- `speclink instructions specs --change <名稱> --json` 的 `instruction` 最後那一段語言說明多一句名稱的規則：需求與情境的名稱算內文，跟著規格語言寫；指向正式規格的名稱照正式規格的原文，一字不改。指向正式規格的名稱有四種：MODIFIED 與 REMOVED 的需求名稱、RENAMED 的舊名稱（FROM）、MODIFIED 區塊裡沿用的正式規格情境名稱、REMOVED-SCENARIO 註記裡的情境名稱。`locale` 欄位與其他產出物的指示不變。
- propose 與 ingest 技能講 `locale` 的段落各加一句，說明 specs 的指示會寫明哪些名稱照正式規格的原文。完整的規則只由引擎的指示說一次。
- ingest 技能最後的摘要多列一項：這次寫回改到既有的需求或情境名稱時，逐一列出「舊名稱 → 新名稱」；沒有改到就不列。
- ingest 技能在取指示的步驟寫明 delta 規格的三種寫法：取指示用 `speclink instructions specs`（整個變更一份，不帶能力名稱），讀用 `speclink artifact cat specs/<capability>`，寫回用 `speclink new artifact spec <capability>`。
- propose 技能讀前置產出物的步驟寫明：前置產出物是 `specs` 時，照 `speclink show <name> --json` 的 `deltaSpecs` 逐一以 `speclink artifact cat specs/<capability>` 讀（現在照字面執行 `artifact cat specs` 會被拒絕）。
- `speclink instructions <代號>` 找不到代號時，錯誤訊息在原本的句子後面列出可用的代號（這個變更的產出流程定義的產出物，加上 `apply`）。exit code 與錯誤類別不變。
- 影響全部工具（claude、codex、copilot、pi、cursor、neutral）產生的 speclink-propose 與 speclink-ingest 兩支技能；技能原稿有改，內嵌資產版號、golden 與 assets.lock 同批更新。
- 不新增或變更任何 CLI 子指令、旗標、stdin 行為與 exit code；不新增設定欄位；`instructions` 不多收任何代號的寫法。

### 相容性影響

- `instructions specs --json`：`instruction` 的文字改變（語言說明那一段多一句），欄位名稱、型別與 `locale` 的值不變。人眼輸出跟著同一份文字改變。
- `instructions <不存在的代號>`：stderr 的訊息變長，仍以 `Artifact '<代號>' not found in schema` 開頭；`--json` 與人眼模式都只有這一句錯誤，exit code 仍非 0；經宿主 `dispatch` 的錯誤碼仍是 `not_found`。全 repo 沒有程式或測試比對這一句。
- 回歸對照：技能的 golden 快照與 assets.lock 要同批更新；固定指示文字與技能用語的測試要同批更新。
- 舊版技能配新引擎：技能段落沒有那一句指向，但引擎的指示已帶完整規則，結果相同。新版技能配舊引擎：指示沒有名稱規則，行為和現在相同。
- 使用者不需要遷移；用 `speclink update` 重新產生技能檔即可拿到新文字。

### 三種形態

- 本機 repo：可用。`speclink instructions specs` 與錯誤訊息由同一個引擎函式產生。
- 連到遠端的 checkout：可用。回應與錯誤由 Server 用同一個函式產生，CLI 原樣輸出；遠端專案的技能也取自 Server。連到還沒升級的 Server 時是舊的指示與舊的錯誤訊息，行為和現在相同。
- 沒有本機規格檔的內嵌宿主：可用。`host.dispatch(['instructions', 'specs', '--change', <名稱>, '--json'])` 得到同一份指示；代號寫錯時 `not_found` 的訊息列出可用的代號。

## Non-Goals (optional)

設計文件會建立，範圍排除寫在 design 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-config`: 改寫「instructions 對 specs 產出物回報解析後的規格語言」，語言說明加上名稱的規則；新增一條需求，規定 instructions 找不到產出物代號時的錯誤訊息列出可用的代號。
- `ingest-skill`: 改寫「既有 artifact 的更新經動詞整份寫回」，加上 delta 規格三種寫法的說明與摘要列出改名的規定。
- `propose-skill`: 改寫「artifact 的讀取與改寫一律經動詞」，加上名稱規則的指向，以及前置產出物是 specs 時逐一讀 delta 規格的寫法。

## Impact

- Affected specs: workflow-config、ingest-skill、propose-skill
- Affected crates: speclink-core（指示的產生、instructions 命令的錯誤訊息、技能原稿、內嵌資產版號）、speclink-cli（整合測試）、speclink-node（經 dispatch 的測試）。speclink-host、speclink-server、speclink-remote、apps/desktop 沒有程式改動。
- Affected code:
  - Modified:
    - crates/engine/speclink-core/src/workspace/instructions.rs
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/assets/skills/propose.md
    - crates/engine/speclink-core/assets/skills/ingest.md
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/tests/it/skill_verbization.rs
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-remote.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/copilot.snapshot.md
    - crates/engine/speclink-core/tests/golden/cursor.snapshot.md
    - crates/engine/speclink-core/tests/golden/pi.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call-remote.snapshot.md
    - crates/adapters/speclink-cli/tests/it/instructions_policy.rs
    - crates/adapters/speclink-node/__test__/workflow-config-verbs.spec.ts
    - docs/configuration.md
    - docs/configuration.zh-TW.md
  - New: (none)
  - Removed: (none)
- 這個 repo 自己安裝的技能檔（.claude/skills 與 .agents/skills 底下的 speclink 技能）在資產版號更新後用 speclink update 重新產生，屬於產生物。
