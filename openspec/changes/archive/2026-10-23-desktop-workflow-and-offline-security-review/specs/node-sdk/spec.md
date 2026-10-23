## MODIFIED Requirements

### Requirement: 渲染 API
<!-- BEFORE: 承諾 instructions.render、store 渲染軸及四種 target；改為現行技能 API 與五種 target 的公開型別契約。 -->
SDK SHALL 提供 skills.list()（回傳技能名與描述清單）與 skills.render(name, options)（回傳技能 Markdown 字串，不寫入檔案）。options SHALL 包含 target（claude｜codex｜copilot｜antigravity｜neutral），並接受 invocation（cli｜tool-call，預設 cli）、specDir（預設 openspec）與 toolName（neutral 專用，預設 speclink）。回傳字串內容 SHALL 與 CLI 以對等參數生成的內容一致。SDK 的公開 TypeScript 宣告 SHALL 接受上述五種 target 與選項，並拒絕未知 target、未知 invocation 及 store／worktree 渲染選項；SDK SHALL NOT 提供 instructions.render 指令檔介面。執行時未知 target SHALL 拋出錯誤並列出五個合法值，未知 invocation 與未知技能 SHALL 明確拋出錯誤。此為規格對齊既有技能 API，加上 Antigravity 型別修正；CLI 人眼輸出、JSON、生成檔內容與退出碼 SHALL 維持既有行為。

#### Scenario: 中性 tool-call 渲染
- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call' })
- **THEN** 回傳字串以「呼叫 speclink 工具」措辭表述動詞並使用 argv、不含 /speclink- 與 $speclink- 前綴，且沒有未替換模板；不承諾移除技能內的所有本機路徑

#### Scenario: 與 CLI 生成一致
- **WHEN** 以 target claude 呼叫 skills.render('apply', …)，並與相同 specDir 的 speclink init 生成的 .claude/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致

#### Scenario: copilot target 與 CLI 生成一致
- **WHEN** 以 target copilot 呼叫 skills.render('apply', …)，並與相同 specDir 的 speclink init --tools copilot 生成的 .github/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致，且回傳字串的技能引用為 `/speclink-` 形式、不含 `$speclink-`

#### Scenario: 未知 target 被拒
- **WHEN** 執行 skills.render('apply', { target: 'github-copilot' })
- **THEN** 呼叫拋出錯誤，訊息含 `github-copilot` 與 claude、codex、copilot、antigravity、neutral 五個合法值

#### Scenario: Copilot 原生渲染與 CLI 同源

- **WHEN** 以 target copilot 呼叫 skills.render，並以相同 specDir 生成 CLI Copilot 技能
- **THEN** SDK 回傳字串與 .github/skills 對應技能一致；採 /speclink- 語法，Codex 仍採 $speclink-，不要求兩工具逐位元相同；不寫入檔案。未知 target 或技能明確拋錯，neutral 的 invocation 語意不變。

#### Scenario: Antigravity 原生渲染與 CLI 同源

- **WHEN** SDK 以 target antigravity 渲染各對外技能，並以 CLI init --tools codex,antigravity 產生技能
- **THEN** SDK 與 .agent/skills 對應檔內容一致、採 /speclink-；Codex 的 .agents/skills 仍採 $speclink- 且內容不被覆寫

#### Scenario: 公開 TypeScript 宣告可供使用者編譯

- **WHEN** TypeScript 使用者從套件入口匯入 skills 與 RenderOptions，以五種合法 target 分別呼叫，並傳入 invocation、specDir、toolName
- **THEN** strict、noEmit 編譯成功；未知 target／invocation、store／worktree 選項及 instructions 匯入均為編譯錯誤

#### Scenario: 選填參數與預設值

- **WHEN** 比較只給 target neutral 與明給 invocation cli、specDir openspec、toolName speclink 的渲染，並另以 specDir specs-local、toolName project-tool、invocation tool-call 渲染 verify
- **THEN** 前兩者內容一致；自訂渲染使用 specs-local/changes/ 與 --agent project-tool，不殘留未替換模板；tool-call 仍以呼叫 speclink 工具表述動詞，toolName 只替換技能內的工具身分

#### Scenario: 未知 invocation 被拒

- **WHEN** JavaScript 呼叫 skills.render('apply', { target: 'neutral', invocation: 'shell' })
- **THEN** 呼叫拋出錯誤，訊息含 shell、cli 與 tool-call

## REMOVED Requirements

### Requirement: 渲染 API 的 worktree 軸

**Reason**: 指令檔渲染介面已退役；SDK 只提供技能渲染，不再承諾 instructions.render 或其 worktree／store 選項。
**Migration**: 呼叫 skills.list() 與 skills.render(name, {target, invocation, specDir, toolName}) 取得技能字串；技能的生成選擇沿 CLI init／update 與工作流政策，不以 SDK marker 開關控制。
