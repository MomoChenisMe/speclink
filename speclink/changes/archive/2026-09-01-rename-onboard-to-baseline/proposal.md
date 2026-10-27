## Why

Speclink 的 onboard 技能與 OpenSpec 的 Onboard 同名但行為不同：OpenSpec 帶新使用者走完一次完整 workflow；Speclink 盤點既有 codebase、依目前行為建立第一批正式 specs。兩專案關係緊密，同名讓使用者誤以為行為一致。改名為 baseline（生成名 speclink-baseline）以「requirements baseline」的業界標準語意準確表達「建立目前行為的規格基準」。定案記錄在來源討論 rename-onboard-to-baseline。

目標使用者是透過 AI 代理跑 SDD 的開發者；使用情境是工作流程的「既有專案首次建規格」站（原 onboard 站，改名後為 baseline 站）。

## What Changes

- Skill registry（crates/speclink-core/src/skills.rs）的技能 id 由 onboard 改為 baseline，description 同步改寫為 baseline 情境句；asset 檔由 onboard.md 改名為 baseline.md，內文的自指名稱與交棒句同步更新。
- ASSET_VERSION 由 v1.24.0 bump 至 v1.25.0，assets.lock 與五份 golden snapshots（claude、claude-worktree、codex、neutral-cli、neutral-tool-call）同批再生——golden 屬刻意變更，於本提案記載。
- **speclink update 新增 registry 差集 prune**：生成後列舉各工具 skills 目錄下 speclink- 前綴的目錄，不在該工具本次應生成集合內者刪除。舊專案更新後不會同時存在 speclink-onboard 與 speclink-baseline。
- repo 自身的 .claude/skills/ 與 .agents/skills/ 生成物再生：speclink-onboard 目錄移除、speclink-baseline 目錄新增；全部 SKILL.md 的 frontmatter 版本戳隨 ASSET_VERSION 變為 v1.25.0。
- 八份使用者文件更新站名與呼叫方式（README.md、README.en.md、docs/getting-started 中英、docs/workflow 中英、docs/product-status 中英）；docs/workflow 兩語言的站別段補一句「舊稱 onboard」。
- openspec/LANGUAGE.md 新增詞彙釘義：「規格基準」（baseline 技能的產出）與「Apply baseline」（品質站凍結點）分立；docs/workflow.md 兩處裸寫的 baseline 補上修飾詞。
- 三份正典 specs 以 delta 更新：skill-routing（入口情境聯集、交棒句邊集、出口 Scenario 站名）、user-documentation（兩條點名站名的 requirement）、workspace-tools（新增 update 孤兒清理的行為承諾）。

## Non-Goals (optional)

- 不保留 deprecated alias、不同時生成兩份技能——pre-1.0（0.1.3）、npm 未發佈、無程式碼引用固定 skill ID，雙份會造成 Agent 技能清單重複觸發。
- 不改技能的行為內容：盤點、capability map 確認、寫規格、validate 的流程不變；不新增 CLI 子指令（本站維持無 speclink baseline 子指令）。
- 不新建 baseline-skill capability spec——行為未變，只更新點名它的既有條文。
- 不回改歷史：封存的 changes 與 discussions、@trace 檔案清單、workspace-chooser-onboarding（desktop 首啟流程）、server e2e 的 team onboarding 字樣全部保留。
- 不採 tombstone 舊名清單清理——registry 差集一次到位，對未來改名同樣生效。

## Capabilities

### New Capabilities

（無——本次改名不引入新 capability。討論階段的 specs 掃描結果：skill-routing 與 user-documentation 直接點名站名、workspace-tools 承載 update 清理承諾，三者以 delta 修改即可；review-skill 與 delivery-baseline 僅為 baseline 撞名證據，不需修改。）

### Modified Capabilities

- `skill-routing`: 入口情境聯集、交棒句邊集與「出口不帶命令總表」條文中的站名由 onboard 改為 baseline；Scenario 改名帶 REMOVED-SCENARIO 宣告。
- `user-documentation`: 「完整工作流指南說明用途與使用時機」與「工作流正典逐站列出技能與完成判準」兩條 requirement 的站名清單由 onboard 改為 baseline。
- `workspace-tools`: 新增 requirement——speclink update 生成後清除孤兒的 speclink- 前綴技能目錄（registry 差集 prune），並界定與既有三條清理路徑（worktree 政策、工具下架、描述子移除）的關係。

## Impact

- 影響的 crate / app：crates/speclink-core（registry、init、assets、golden 測試）。speclink-cli 僅透過既有 update 動詞間接受影響，無指令簽名變更；Desktop、Server、Node SDK 不引用固定 skill ID，不受影響。
- 相容性影響：
  - speclink update 行為變更——多刪除孤兒的 speclink- 前綴目錄。使用者自建但以 speclink- 開頭的目錄會被視為生成物清除，與既有 prune 的前綴所有權語意一致；自建技能的遷移指引是改用非 speclink- 前綴的目錄名。非 speclink- 前綴的使用者技能不受影響。
  - 生成的 SKILL.md frontmatter 版本戳全數變為 v1.25.0；speclink --version 顯示的 engine 版號同步。
  - 五份 golden snapshots 刻意變更，同批以再生指令更新；人眼輸出沿 update 既有樣式，--json 欄位 shape 不變。
  - 舊版 CLI 的使用者升級後執行一次 update 即完成遷移；未升級者維持舊技能可用。
- Affected code:
  - New: crates/speclink-core/assets/skills/baseline.md、.claude/skills/speclink-baseline/SKILL.md、.agents/skills/speclink-baseline/SKILL.md
  - Modified: crates/speclink-core/src/skills.rs、crates/speclink-core/src/init.rs、crates/speclink-core/tests/golden/assets.lock、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、README.md、README.en.md、docs/getting-started.md、docs/getting-started.zh-TW.md、docs/workflow.md、docs/workflow.zh-TW.md、docs/product-status.md、docs/product-status.zh-TW.md、openspec/LANGUAGE.md、.claude/skills/ 與 .agents/skills/ 底下全部 speclink-* SKILL.md（版本戳）
  - Removed: crates/speclink-core/assets/skills/onboard.md、.claude/skills/speclink-onboard/SKILL.md、.agents/skills/speclink-onboard/SKILL.md
