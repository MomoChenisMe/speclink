## MODIFIED Requirements

### Requirement: Getting Started 僅使用已驗證入口
<!-- BEFORE: 入門文件只需說明 Claude slash command、Codex $skill 與直接 CLI 三種入口 -->

中英文 getting-started SHALL 提供一條可複製的 Local Repo 最短成功路徑；文件中的 CLI 子指令與旗標 SHALL 可由目前 `speclink --help` 或對應子指令 help 觀察，Agent skill 名 SHALL 存在於相應生成 surface。文件 SHALL 分別說明 Claude slash command、Codex `$skill`、Copilot slash command 與直接 CLI 的差異，並 SHALL 載明三個工具各自的技能目錄（`.claude/skills/`、`.agents/skills/`、`.github/skills/`）；SHALL NOT 將 optional artifact 寫成固定必產，SHALL NOT 將不存在的 skill 寫成可呼叫入口。結構檢查 SHALL 使用目前存在的 validate／analyze，實作驗證或 evidence 若無公開 skill 入口 SHALL 以目前限制標示並導向 product-status。

#### Scenario: Codex 使用者照入門文件操作

- **WHEN** Codex 使用者依 getting-started 建立並完成範例 change
- **THEN** 文件使用 `$speclink-*` skill 語法，底層 CLI 命令與旗標皆存在，且不要求呼叫未安裝的 `$speclink-verify`

#### Scenario: Claude 使用者照入門文件操作

- **WHEN** Claude 使用者依 getting-started 執行相同流程
- **THEN** 文件使用該 Host 生成的 slash command 語法，並與 Codex 版本產生相同 Speclink artifacts 與生命週期結果

#### Scenario: optional design 被正確說明

- **WHEN** 範例 change 不符合 design artifact 的建立條件
- **THEN** getting-started 說明 propose 只需完成 applyRequires 鏈上的必要 artifacts，design 可依指令條件跳過，SHALL NOT 宣稱每個 change 固定產出四份 artifact

#### Scenario: Copilot 使用者照入門文件操作

- **WHEN** GitHub Copilot 使用者依 getting-started 以 `speclink init --tools copilot` 初始化並完成範例 change
- **THEN** 文件使用 `/speclink-*` skill 語法並說明技能位於 `.github/skills/`，底層 CLI 命令與旗標皆存在，不要求呼叫 Copilot 未生成的 `/speclink-analyze`，且與 Claude、Codex 版本產生相同 Speclink artifacts 與生命週期結果
