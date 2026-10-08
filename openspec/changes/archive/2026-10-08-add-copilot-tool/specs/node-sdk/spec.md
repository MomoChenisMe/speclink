## MODIFIED Requirements

### Requirement: 渲染 API
<!-- BEFORE: target 值域只有 claude｜codex｜neutral -->
SDK SHALL 提供 skills.list()（回傳技能名與描述清單）、skills.render(name, options) 與 instructions.render(options)——options 涵蓋渲染矩陣：target（claude｜codex｜copilot｜neutral）、invocation（cli｜tool-call）、store（fs｜remote）；回傳字串內容 SHALL 與 CLI 以對等參數生成的內容一致。target 不在值域內時 SHALL 拋出錯誤，訊息 SHALL 列出 claude、codex、copilot、neutral 四個合法值。

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
- **THEN** 呼叫拋出錯誤，訊息含 `github-copilot` 與 claude、codex、copilot、neutral 四個合法值
