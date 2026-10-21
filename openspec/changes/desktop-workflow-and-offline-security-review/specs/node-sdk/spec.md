## MODIFIED Requirements

### Requirement: 渲染 API
SDK SHALL 提供 skills.list()（回傳技能名與描述清單）、skills.render(name, options) 與 instructions.render(options)——options 涵蓋渲染矩陣：target（claude｜codex｜copilot｜antigravity｜neutral）、invocation（cli｜tool-call）、store（fs｜remote）；回傳字串內容 SHALL 與 CLI 以對等參數生成的內容一致。target 不在值域內時 SHALL 拋出錯誤，訊息 SHALL 列出 claude、codex、copilot、antigravity、neutral 五個合法值。

#### Scenario: 中性 tool-call 渲染
- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call', store: 'remote' })
- **THEN** 回傳字串以「呼叫 speclink 工具」措辭表述動詞、不含 /speclink- 前綴與本地規格路徑句

#### Scenario: 與 CLI 生成一致
- **WHEN** 以 target claude、store fs 呼叫 skills.render('apply', …)，並與 speclink init 於 fs 專案生成的 .claude/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致

#### Scenario: copilot target 與 CLI 生成一致
- **WHEN** 以 target copilot、store fs 呼叫 skills.render('apply', …)，並與 speclink init --tools copilot 於 fs 專案生成的 .github/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致，且回傳字串的技能引用為 `/speclink-` 形式、不含 `$speclink-`

#### Scenario: 未知 target 被拒
- **WHEN** 執行 skills.render('apply', { target: 'github-copilot' })
- **THEN** 呼叫拋出錯誤，訊息含 `github-copilot` 與 claude、codex、copilot、antigravity、neutral 五個合法值

#### Scenario: Copilot 原生渲染與 CLI 同源

- **WHEN** 以 target copilot 呼叫 skills.render，並以相同 specDir 生成 CLI Copilot 技能
- **THEN** SDK 回傳字串與 .github/skills 對應技能一致；採 /speclink- 語法，Codex 仍採 $speclink-，不要求兩工具逐位元相同；不寫入檔案。未知 target 或技能明確拋錯，neutral 的 invocation/store 語意不變。

#### Scenario: Antigravity 原生渲染與 CLI 同源

- **WHEN** SDK 以 target antigravity 渲染各對外技能，並以 CLI init --tools codex,antigravity 產生技能
- **THEN** SDK 與 .agent/skills 對應檔內容一致、採 /speclink-；Codex 的 .agents/skills 仍採 $speclink- 且內容不被覆寫
