## Why

內嵌宿主的 Agent 用真實模型照 propose 技能寫變更時，專案設了 `locale: tw`、`spec_locale: tw`，規格差異的內文卻寫成英文。原因是同一份 `speclink instructions specs --json` 的回應先講「English by default」、最後才補一句實際的語言，propose 技能又講一次「規格預設英文」，而回應的 `locale` 欄位對規格根本不準（它永遠是專案的 `locale`）。同一輪還發現：開了 tdd 或 audit 時，任務的指示叫模型去執行 `speclink instructions --skill tdd`，這個命令經宿主的 `dispatch` 一定被拒絕，模型白跑兩次。

目標使用者：透過 AI agent 寫變更的開發者，以及在內嵌宿主的對話裡請 Agent 寫提案的非開發者（PO、PM）。使用情境：propose 與 ingest 技能建立或更新 specs、tasks 這兩份產出物的階段。

來源：討論 embedded-host-needs-8 的 cut 1（G1 與 G2）。相關的正式規格掃描結果：workflow-config（政策解析與 instructions 的政策欄位）、propose-skill（規格語言指向 `workflow-config show`）、ingest-skill（語言說明與 propose 相同）、workflow-schemas（instruction 文字的單一來源）、node-sdk（`dispatch` 拒絕帶 `--skill` 的 `instructions`）。

## What Changes

- `speclink instructions specs --change <名稱> --json` 的 `locale` 欄位改成規格的語言：`spec_locale` 是 `tw`、`ja`、`en` 時用它，`auto` 時跟著 `locale`，沒有設定時是 English。其他產出物（proposal、design、tasks、apply）的 `locale` 不變。沿用現有的三層解析（環境變數 `SPECLINK_SPEC_LOCALE` 優先於 `speclink/config.yaml`，再來是內建預設）。
- 規格的指示文字不再有「English by default. If the project sets spec_locale in speclink/config.yaml…」那一段。引擎改成一律在指示最後寫出解析後的語言（英文也寫），並保留「結構標記與 SHALL、MUST 維持英文」以及中文模糊用語的提醒。
- propose 與 ingest 技能拿掉「規格預設英文，除非 `speclink workflow-config show --json` 的 `specLocale`…」的例外，改成「回應的 `locale` 就是這份產出物的語言」。影響全部工具（claude、codex、copilot、pi、cursor、neutral）產生的 speclink-propose 與 speclink-ingest 兩支技能；技能原稿有改，內嵌資產版號、golden 與 assets.lock 同批更新。
- 任務的指示在開了 tdd、audit 時加的兩句話，括號裡不再叫模型執行 `speclink instructions --skill tdd` 或 `--skill audit`，改成說明「實作時 apply 會提供完整的紀律，寫任務不用去取得」。一句摘要保留。
- 不新增或變更任何 CLI 子指令、旗標、stdin 行為與 exit code；不新增設定欄位，`spec_locale` 的合法值與預設值不變。

### 相容性影響

- `instructions specs --json`：`locale` 的值在「`spec_locale` 解析結果和 `locale` 不同」時改變（例如 `locale: tw`、沒設 `spec_locale` 時由 Traditional Chinese 變成 English）；欄位名稱與型別不變。`instruction` 的文字改變。人眼輸出跟著同一份資料改變。
- `instructions tasks --json`：只有開了 tdd 或 audit 時 `instruction` 的文字改變。
- 沒有程式讀 `instructions` 的 `locale`（apps、packages、crates 都搜尋過），只有技能讀。舊版技能配新引擎仍然寫出正確的語言，因為舊技能自己套用「規格預設英文」的例外，而引擎的指示寫明了語言。
- 回歸對照：技能的 golden 快照與 assets.lock 要同批更新；固定指示文字的測試要同批更新。
- 使用者不需要遷移；用 `speclink update` 重新產生技能檔即可拿到新文字。

### 三種形態

- 本機 repo：可用。`speclink instructions specs` 直接讀 `speclink/config.yaml` 與環境變數。
- 連到遠端的 checkout：可用。回應由 Server 用同一個函式產生，CLI 原樣輸出；遠端專案的技能也取自 Server，所以技能文字和回應的版本一致。連到還沒升級的 Server 時，回應是舊的語意、技能也是舊的文字，行為和現在相同。
- 沒有本機規格檔的內嵌宿主：可用。`host.dispatch(['instructions', 'specs', '--change', <名稱>, '--json'])` 得到同一份回應；任務的指示不再指到 `dispatch` 拒絕的命令。

## Non-Goals (optional)

設計文件會建立，範圍排除寫在 design 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-config`: 新增兩條需求。第一條規定 instructions 對 specs 產出物回報解析後的規格語言（locale 欄位與指示文字）；第二條規定任務指示提到 tdd 與 audit 紀律時不指向帶 --skill 的 instructions。
- `propose-skill`: 改寫「artifact 的讀取與改寫一律經動詞」裡規格撰寫語言的來源，由 workflow-config show 的 specLocale 改成 instructions 回應的 locale。ingest-skill 的正式規格寫的是「語言說明與 propose 技能相同」，所以不需要自己的 delta。

## Impact

- Affected specs: workflow-config、propose-skill
- Affected crates: speclink-core（指示的產生、schema 原稿、技能原稿、內嵌資產版號）、speclink-cli（整合測試）、speclink-node（經 dispatch 的測試）。speclink-host、speclink-server、apps/desktop 沒有程式改動。
- Affected code:
  - Modified:
    - crates/engine/speclink-core/src/workspace/instructions.rs
    - crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml
    - crates/engine/speclink-core/assets/skills/propose.md
    - crates/engine/speclink-core/assets/skills/ingest.md
    - crates/engine/speclink-core/src/workspace/init.rs
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
