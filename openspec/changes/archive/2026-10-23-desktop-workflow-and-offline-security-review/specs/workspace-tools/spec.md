## MODIFIED Requirements

### Requirement: tools 自訂描述子的接受與驗證

.speclink.yaml 的 tools 清單 SHALL 接受兩種元素形式：內建工具名字串（claude、codex、copilot、antigravity），或自訂描述子物件（欄位：name 必填、skills_dir 必填、instructions_file 選填且不再驅動任何生成、invocation 選填且值域為 cli 或 tool-call、預設 cli）。描述子驗證規則：name SHALL 為 kebab-case（2 至 50 字）且 SHALL NOT 與內建工具名（claude、codex、copilot、antigravity，以及 codex 的別名 agents）衝突；skills_dir 與（存在時）instructions_file SHALL 為專案根相對路徑，正規化後 SHALL NOT 逸出專案根——instructions_file 保留驗證僅為遺留 marker 剝除的定位所需。skills_dir SHALL 於驗證時削去結尾的 `/`，其後所有消費端（技能生成、足跡記錄與清理、過期探測的路徑回報）SHALL 一律使用削去後的形式。以下兩條拒絕 SHALL 比對詞法正規化後的路徑（丟棄 `.` 段、以 `..` 回退一段，零檔案系統存取），因此等價拼法與字面拼法同樣被拒：正規化後等同專案根本身的 skills_dir SHALL 被拒絕；正規化後等同任一內建工具 skills 目錄（`.claude/skills`、`.agents/skills`、`.github/skills`、`.agent/skills`）的 skills_dir SHALL 被拒絕，且這條比對 SHALL 不分大小寫（macOS 與 Windows 的檔案系統上 `.GitHub/skills` 就是 `.github/skills`）。描述子仍帶 instructions_file 欄位時，workspace 檢查 SHALL 輸出一行棄用提示（非錯誤、不影響 exit code）。驗證失敗時指令 SHALL 以非 0 exit code 結束並輸出單行語義化錯誤訊息（指明錯誤欄位與原因）。

#### Scenario: 合法描述子生成對應工具檔

- **WHEN** .speclink.yaml 的 tools 含描述子 name: wad-harness、skills_dir: .wad/skills，執行 speclink update
- **THEN** 生成 .wad/skills/speclink-*/SKILL.md 技能檔，exit code 為 0，且不生成任何指令檔區塊

#### Scenario: 名稱與內建工具衝突被拒

- **WHEN** tools 含描述子 name: claude，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息指明 name 與內建工具名衝突

#### Scenario: 路徑逸出專案根被拒

- **WHEN** tools 含描述子 skills_dir: ../outside/skills，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息指明 skills_dir 逸出專案根

#### Scenario: 殘留 instructions_file 欄位得棄用提示

- **WHEN** tools 含描述子 name: wad-harness、skills_dir: .wad/skills、instructions_file: WAD.md，執行 speclink update
- **THEN** exit code 為 0、技能檔照常生成，stderr 帶一行棄用提示指明 instructions_file 已不生效

#### Scenario: 結尾分隔符於驗證時削去

- **WHEN** tools 含描述子 skills_dir: .wad/skills/，執行 speclink update 後再執行技能檔過期探測
- **THEN** 技能檔生成於 .wad/skills/，且探測回報的受管檔路徑以 `.wad/skills/speclink-` 起頭、不含連續斜線

#### Scenario: 削去後等同專案根的 skills_dir 被拒

- **WHEN** tools 含描述子 skills_dir 為 `/`、`./`、`.` 或 `.wad/..`（正規化後皆等同專案根），執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息指明 skills_dir 等同專案根；專案根不得出現任何 speclink- 目錄

#### Scenario: 與內建工具 skills 目錄相撞被拒

- **WHEN** tools 含 claude 與描述子 skills_dir 為 `.claude/skills`，或其等價拼法 `./.claude/skills`、`.claude/skills/.`、`.claude//skills`、`.wad/../.claude/skills`，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息指明該目錄屬內建工具；.claude/skills 下既有技能檔逐字元不變

##### Example: skills_dir 的驗證結果

| skills_dir | 結果 |
| --- | --- |
| `.wad/skills` | 接受，原樣使用 |
| `.wad/skills/` | 接受，削為 `.wad/skills` |
| `../outside/skills` | 拒絕——逸出專案根 |
| `/` | 拒絕——正規化後等同專案根 |
| `.wad/..` | 拒絕——正規化後等同專案根（等價拼法） |
| `.claude/skills` | 拒絕——內建 claude 的 skills 目錄 |
| `./.claude/skills` | 拒絕——正規化後等同內建目錄（等價拼法） |
| `.claude/skills-extra` | 接受——與內建目錄不同 |
| `.github/skills` | 拒絕——內建 copilot 的 skills 目錄 |
| `./.github/skills` | 拒絕——正規化後等同內建目錄（等價拼法） |
| `.github/skills-extra` | 接受——與內建目錄不同 |
| `.GitHub/skills` | 拒絕——不分大小寫等同內建 copilot 目錄 |

#### Scenario: 名為 copilot 或指向 Copilot 目錄的既有描述子升級後被拒

- **WHEN** 升級前可用的 .speclink.yaml 的 tools 含描述子 name: copilot、skills_dir: .copilot-skills，或描述子 name: my-copilot、skills_dir: .github/skills，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息分別指明 name 與內建工具名衝突、或 skills_dir 屬內建 copilot 目錄；任何技能檔與 .github/skills 下既有檔案逐字元不變

保留舊 name: copilot 描述子的遷移指引：驗證錯誤 SHALL 提示改名 copilot-custom 可保留 skills_dir/invocation，或明確改為內建字串 copilot；SHALL NOT 靜默覆寫自訂設定。

### Requirement: init 內建 Agent 工具選擇

所有 `speclink init`（filesystem 與 Remote Store）SHALL 在任何 Workspace 寫入前解析至少一個內建 Agent 工具。顯式 `--tools` SHALL 接受 `claude`、`codex`、`copilot`、`antigravity` 中的一個，或其中任意多個以逗號分隔（重複名稱合併為一個）並跳過詢問；解析後為空或含未知名稱 SHALL 以非零 exit code 失敗，未知名稱的 stderr 單行訊息 SHALL 列出四個支援的工具名。未提供 `--tools` 且 stdin 為互動終端時，CLI SHALL 於 stderr 依序明示詢問 Claude、Codex、Copilot、Antigravity 四題並允許選任意非空組合；四題皆未選 SHALL NOT 開始 init，並 SHALL 繼續要求有效選擇。未提供 `--tools` 且 stdin 非互動終端時，CLI SHALL 在零寫入狀態以非零 exit code 失敗，stderr SHALL 為單行訊息並指出 `--tools`、四個工具名與逗號分隔的組合寫法，stdout SHALL 為空。init SHALL NOT 將 redirected／piped stdin 當作工具答案，SHALL NOT 新增 stdin payload 或 `--json` 介面。此行為是對早期 footprint 自動偵測的刻意分歧；顯式提供 `--tools` 時成功輸出的格式與 `--no-color` 行為 SHALL 維持既有基線（Generated files 清單內容隨受管集合縮減為技能檔與工作區檔）。

#### Scenario: filesystem init 顯式選擇 Codex

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools codex`
- **THEN** exit code 為 0，stdout 沿用既有 Initialized 與 Generated files 摘要，`.speclink.yaml` 的 built-in tools 僅含 `codex`，並生成 Codex Skills；不生成 `AGENTS.md`

#### Scenario: Remote init 顯式選擇兩個工具

- **WHEN** 在空目錄執行 Remote Store init 並顯式提供 `--tools claude,codex`、有效 project URL 與 repo
- **THEN** exit code 為 0，`.speclink.yaml` 同時含兩個 built-in tools 與 remote section，生成兩組 Skills，且不存在 `openspec/` 與任何指令檔

#### Scenario: 互動終端選擇 Claude 與 Codex

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude 與 Codex 都回答 yes、對 Copilot 與 Antigravity 回答 no
- **THEN** 詢問文字依 Claude、Codex、Copilot、Antigravity 順序只寫入 stderr，init 以 claude 與 codex 兩個工具執行，成功摘要寫入 stdout

#### Scenario: 互動終端不得提交空選集

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude、Codex、Copilot、Antigravity 四題都回答 no
- **THEN** CLI 不建立 `.speclink.yaml`、`openspec/`、`.gitignore` 或 Skills，於 stderr 印一行要求至少選取一個工具，並從 Claude 起再次詢問四題

#### Scenario: 非互動 init 缺少 tools 零寫入失敗

- **WHEN** stdin 為 pipe 或 redirect，執行 init 且未提供 `--tools`
- **THEN** exit code 非 0、stdout 為空，stderr 單行訊息包含 `--tools`、`claude`、`codex` 與 `copilot`，且目標目錄內容逐項不變

#### Scenario: 空或未知的顯式 tools 被拒

- **WHEN** 執行 init 並提供空的 `--tools` 值或含 `vscode` 的值
- **THEN** exit code 非 0，stderr 指出選集為空或 unknown tool（unknown tool 訊息列出 claude、codex、copilot），且任何 Workspace 檔案都未建立或修改

#### Scenario: no-color 不改變工具選擇語意

- **WHEN** 以 `--no-color` 執行互動式 init 並選取任一有效工具
- **THEN** prompt 與成功輸出不含 ANSI escape sequence，exit code 與檔案效果和有色模式相同

#### Scenario: filesystem init 顯式選擇 Copilot

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools copilot`
- **THEN** exit code 為 0，stdout 的 Generated files 摘要列出 `copilot`，`.speclink.yaml` 的 built-in tools 僅含 `copilot`，並生成 `.github/skills/` 下的 Copilot Skills；不生成 `.claude/`、`.agents/`、`AGENTS.md`、`CLAUDE.md` 與 `.github/copilot-instructions.md`

#### Scenario: 顯式選擇三個工具

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools claude,codex,copilot`
- **THEN** exit code 為 0，`.speclink.yaml` 的 built-in tools 依 claude、codex、copilot 記錄三個工具，並生成 `.claude/skills/`、`.agents/skills/`、`.github/skills/` 三組 Skills

##### Example: --tools 值的解析

| `--tools` 值 | 結果 |
| --- | --- |
| `copilot` | 接受，選集為 copilot |
| `Copilot` | 接受，選集為 copilot（大小寫不拘） |
| `claude,copilot` | 接受，選集為 claude、copilot |
| `copilot,copilot` | 接受，選集為 copilot（重複合併） |
| `github-copilot` | 拒絕——unknown tool，訊息列出 claude、codex、copilot |
| `vscode` | 拒絕——unknown tool |

## ADDED Requirements

### Requirement: Antigravity 技能生成與獨立清理

Antigravity SHALL 為內建工具名 antigravity，使用專案根 .agent/skills 下的 speclink-* 技能目錄。生成 SHALL 沿既有 Tool::ALL 生成計畫，使用非 Claude 技能 frontmatter 與 /speclink- 呼叫樣式；SHALL NOT 生成 AGENTS.md、GEMINI.md 或修改使用者全域設定。更新、過期探測及取消清理 SHALL 與其他內建工具一致；取消 SHALL 只清理受管技能並保留其他工具、自有技能與 .agent 下其他檔案。舊自訂描述子撞名或撞 .agent/skills SHALL 在寫入前拒絕並給明確遷移提示，不靜默覆寫。

#### Scenario: 同時使用 Codex 與 Antigravity

- **WHEN** 使用者以 tools codex,antigravity 初始化或更新專案
- **THEN** tools 保存兩種名稱，Codex 技能生成於 .agents/skills 並沿 $speclink-，Antigravity 技能生成於 .agent/skills 並沿 /speclink-，兩者內容互不覆寫

#### Scenario: 取消 Antigravity 保留使用者檔案

- **WHEN** 已生成 Antigravity 技能的專案將它自 tools 移除並更新
- **THEN** .agent/skills 下受管 speclink-* 移除，自有技能、.agent 其他檔案與 .agents 下 Codex 內容保持不變；連結目錄不被跟隨清理

#### Scenario: 舊自訂描述子與新內建工具衝突

- **WHEN** tools 含 name antigravity 的描述子，或自訂 skills_dir 正規化等同 .agent/skills
- **THEN** 更新以非零退出、提示撞名或撞目錄及改名 antigravity-custom／改用內建工具的可行方式，工作區檔案不變
