## MODIFIED Requirements

### Requirement: 渲染 API
<!-- BEFORE: 敘述含 instructions.render 與 store 選項，兩者都不在實作裡；沒有 specInCheckout -->

SDK SHALL 提供 skills.list()（回傳技能名與描述清單）與 skills.render(name, options)——options 涵蓋渲染矩陣：target（claude｜codex｜copilot｜pi｜cursor｜neutral）、invocation（cli｜tool-call，只對 neutral 有作用）、specInCheckout（布林，預設 true：規格資料夾是否位於使用這份技能的 checkout；不是布林值時——包含 null——SHALL 拋出錯誤，SHALL NOT 靜默當成預設值）、specDir 與 toolName；內嵌引擎而沒有本機規格檔的宿主 SHALL 以 specInCheckout 為 false 渲染給自己 Agent 的技能；回傳字串內容 SHALL 與 CLI 以對等參數生成的內容一致。target 不在值域內時 SHALL 拋出錯誤，訊息 SHALL 列出 claude、codex、copilot、pi、cursor、neutral 六個合法值。

#### Scenario: 中性 tool-call 渲染
- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call', specInCheckout: false })
- **THEN** 回傳字串以「呼叫 speclink 工具」措辭表述動詞、不含 /speclink- 前綴與本地規格路徑句

#### Scenario: 與 CLI 生成一致
- **WHEN** 以 target claude（不傳 specInCheckout）呼叫 skills.render('apply', …)，並與 speclink init 於本機專案生成的 .claude/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致

#### Scenario: copilot target 與 CLI 生成一致
- **WHEN** 以 target copilot（不傳 specInCheckout）呼叫 skills.render('apply', …)，並與 speclink init --tools copilot 於本機專案生成的 .github/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致，且回傳字串的技能引用為 `/speclink-` 形式、不含 `$speclink-`

#### Scenario: pi 與 cursor target 與 CLI 生成一致
- **WHEN** 以 target pi 與 target cursor（不傳 specInCheckout）呼叫 skills.render('apply', …)，並分別與 speclink init --tools pi,cursor 於本機專案生成的 .pi/skills/speclink-apply/SKILL.md、.cursor/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩組內容各自一致；pi 的技能引用為 `/skill:speclink-` 形式，cursor 的技能引用為 `/speclink-` 形式，兩者都不含 `$speclink-`

#### Scenario: 未知 target 被拒
- **WHEN** 執行 skills.render('apply', { target: 'github-copilot' })
- **THEN** 呼叫拋出錯誤，訊息含 `github-copilot` 與 claude、codex、copilot、pi、cursor、neutral 六個合法值

#### Scenario: specInCheckout 為 false 的渲染

- **WHEN** 以 skills.render('archive', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, specDir: 'openspec' }) 渲染
- **THEN** 回傳的字串不含 `openspec/`，交棒以 `speclink-<名稱>` 表述

#### Scenario: 省略 specInCheckout 等於本機專案

- **WHEN** 分別以省略 specInCheckout 與 specInCheckout 為 true 渲染同一支技能
- **THEN** 兩個回傳字串相同，且與 CLI 在本機專案產生的該技能內容一致

#### Scenario: 非布林的 specInCheckout 被拒

- **WHEN** 以 specInCheckout 為字串 'false'、數字 0、物件或 null 呼叫 skills.render('commit', { target: 'claude', … })
- **THEN** 每一種都拋出錯誤，沒有任何一種被當成 true 渲染
