## MODIFIED Requirements

### Requirement: 渲染 API
<!-- BEFORE: target 只有 claude／codex／neutral，未定義共享 Copilot 輸出。 -->
SDK SHALL 提供 skills.list()（回傳技能名與描述清單）、skills.render(name, options) 與 instructions.render(options)——options 涵蓋渲染矩陣：target（claude｜codex｜copilot｜neutral）、invocation（cli｜tool-call）、store（fs｜remote）；回傳字串內容 SHALL 與 CLI 以對等參數生成的內容一致。

#### Scenario: 中性 tool-call 渲染
- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call', store: 'remote' })
- **THEN** 回傳字串以「呼叫 speclink 工具」措辭表述動詞、不含 /speclink- 前綴與本地規格路徑句

#### Scenario: 與 CLI 生成一致
- **WHEN** 以 target claude、store fs 呼叫 skills.render('apply', …)，並與 speclink init 於 fs 專案生成的 .claude/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致

#### Scenario: Codex 與 Copilot SDK 輸出共用

- **WHEN** 對相同技能與 specDir，以 target codex及copilot 呼叫 skills.render
- **THEN** 回傳字串位元級相同，符合 CLI 在等效選集生成的 .agents/skills/speclink-*/SKILL.md，使用共享技能名稱與 CLI 執行者前言，不寫入檔案

#### Scenario: 舊渲染入口維持

- **WHEN** 以 target claude 或 neutral 呼叫既有技能渲染
- **THEN** 除同源資產版本戳更新，本文與既有輸出一致；neutral 的 toolName與cli／tool-call 語意不變

#### Scenario: 未知 target 或技能

- **WHEN** 呼叫 skills.render 的 target 為 vscode，或技能名為 no-such-skill
- **THEN** 明確拋錯且無檔案寫入；target 錯誤列出 claude、codex、copilot、neutral，技能錯誤維持 Unknown skill 語意；不轉用另一個目標

#### Scenario: SDK 輸出不新增命令通道

- **WHEN** 宿主取得 target copilot 的共享技能字串
- **THEN** 僅回傳字串、不新增 stdout／stderr／exit code／JSON 介面；字串指示使用既有 CLI 動詞，其人眼與 --json 契約及 --no-color 行為不改。tw／ja／en／未設定、中文弱偵測與遠端 revision／離線／認證仍由既有工作流處理，不在渲染 API 改寫
