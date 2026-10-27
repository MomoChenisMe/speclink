## MODIFIED Requirements

### Requirement: init 內建 Agent 工具選擇

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

所有 `speclink init`（filesystem 與 Remote Store）SHALL 在任何 Workspace 寫入前解析至少一個內建 Agent 工具。顯式 `--tools` SHALL 接受 `claude`、`codex`、`copilot` 中的一個，或其中任意多個以逗號分隔（重複名稱合併為一個）並跳過詢問；解析後為空或含未知名稱 SHALL 以非零 exit code 失敗，未知名稱的 stderr 單行訊息 SHALL 列出三個支援的工具名。未提供 `--tools` 且 stdin 為互動終端時，CLI SHALL 於 stderr 依序明示詢問 Claude、Codex、Copilot 三題並允許選任意非空組合；三題皆未選 SHALL NOT 開始 init，並 SHALL 繼續要求有效選擇。未提供 `--tools` 且 stdin 非互動終端時，CLI SHALL 在零寫入狀態以非零 exit code 失敗，stderr SHALL 為單行訊息並指出 `--tools`、三個工具名與逗號分隔的組合寫法，stdout SHALL 為空。init SHALL NOT 將 redirected／piped stdin 當作工具答案，SHALL NOT 新增 stdin payload 或 `--json` 介面。此行為是對早期 footprint 自動偵測的刻意分歧；顯式提供 `--tools` 時成功輸出的格式與 `--no-color` 行為 SHALL 維持既有基線（Generated files 清單內容隨受管集合縮減為技能檔與工作區檔）。

#### Scenario: filesystem init 顯式選擇 Codex

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools codex`
- **THEN** exit code 為 0，stdout 沿用既有 Initialized 與 Generated files 摘要，`.speclink.yaml` 的 built-in tools 僅含 `codex`，並生成 Codex Skills；不生成 `AGENTS.md`

#### Scenario: Remote init 顯式選擇兩個工具

- **WHEN** 在空目錄執行 Remote Store init 並顯式提供 `--tools claude,codex`、有效 project URL 與 repo
- **THEN** exit code 為 0，`.speclink.yaml` 同時含兩個 built-in tools 與 remote section，生成兩組 Skills，且不存在 `speclink/` 與任何指令檔

#### Scenario: 互動終端選擇 Claude 與 Codex

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude 與 Codex 都回答 yes、對 Copilot 回答 no
- **THEN** 詢問文字依 Claude、Codex、Copilot 順序只寫入 stderr，init 以 claude 與 codex 兩個工具執行，成功摘要寫入 stdout

#### Scenario: 互動終端不得提交空選集

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude、Codex、Copilot 三題都回答 no
- **THEN** CLI 不建立 `.speclink.yaml`、`speclink/`、`.gitignore` 或 Skills，於 stderr 印一行要求至少選取一個工具，並從 Claude 起再次詢問三題

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

### Requirement: 工具檔生成不寫入 AI 工具的使用者設定檔

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

工具檔生成（init、update、tools 收斂、工作區補齊 adopt 的所有路徑）SHALL NOT 建立或改寫 AI 工具的使用者設定檔（`.claude/settings.json`）。受管生成物 SHALL 僅限技能檔、`.speclink.yaml` 與 `.gitignore` 的 `.speclink/` 條目。既有的使用者設定檔 SHALL 視為使用者資料：任何工具同步後 SHALL 位元級不變，清理（prune）SHALL NOT 移除它。

#### Scenario: init 不產生使用者設定檔

- **WHEN** 對全新目錄以 tools=[claude] 執行 init
- **THEN** `.claude/skills/` 技能檔照常生成，`.claude/settings.json` 與 `CLAUDE.md` 皆不存在

#### Scenario: 既有使用者設定檔在工具同步後位元級不變

- **WHEN** 專案的 `.claude/settings.json` 含使用者自訂內容（如外掛啟用清單），執行 speclink update
- **THEN** 該檔位元級不變，其餘受管檔照常再生

##### Example: 自訂外掛設定不被清空

- **GIVEN** `.claude/settings.json` 內容為 `{"enabledPlugins":{"frontend-design":true},"includeGitInstructions":false}`
- **WHEN** 執行 speclink update
- **THEN** 該檔內容仍為 `{"enabledPlugins":{"frontend-design":true},"includeGitInstructions":false}`，位元級相同

#### Scenario: 工作區補齊不產生使用者設定檔

- **WHEN** 對「已有 speclink/ 但無 .speclink.yaml」的目錄以 tools=[claude] 執行工作區補齊（adopt）
- **THEN** 工作區檔照常補齊（.speclink.yaml、技能檔、.gitignore 條目），`.claude/settings.json` 不存在

### Requirement: worktree 技能的政策條件式生成

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

技能足跡生成（speclink init 與 speclink update）SHALL 依 speclink/config.yaml 的 worktree 檔值過濾兩顆 worktree 技能（speclink-apply-with-worktree 與 speclink-worktree-merge）：值為 true 時生成，false 或未設時不生成，且既有生成物依既有清理生命週期移除（技能目錄移除，因而變空的目錄一併移除）。生成判定 SHALL NOT 納入 SPECLINK_WORKTREE 環境變數（環境層僅影響執行期政策，不影響足跡）。其餘技能的生成集合不受 worktree 政策影響。此過濾對 claude、codex、copilot 與自訂描述子工具一視同仁。

#### Scenario: 政策關閉時生成集合不含 worktree 技能

- **WHEN** speclink/config.yaml 無 worktree 鍵（或值為 false）的專案執行 speclink update
- **THEN** 各工具 skills 目錄不含 speclink-apply-with-worktree 與 speclink-worktree-merge 目錄，其餘技能照常生成

#### Scenario: 政策開啟時注入兩顆技能

- **WHEN** speclink/config.yaml 含 worktree: true 的專案執行 speclink update
- **THEN** 各工具 skills 目錄含兩顆 worktree 技能，內容與 golden 對照一致

#### Scenario: 政策由開改關後再生即清理

- **WHEN** 先於 worktree: true 下執行 speclink update，再將該鍵改為 false 並重新執行 speclink update
- **THEN** 兩顆 worktree 技能目錄被移除，其餘技能保留

#### Scenario: 環境變數不影響生成

- **WHEN** speclink/config.yaml 無 worktree 鍵，但於 SPECLINK_WORKTREE=true 環境下執行 speclink update
- **THEN** 兩顆 worktree 技能不生成——環境層僅於技能執行期由 P1 政策檢查讀取

#### Scenario: 過期探測不把被政策排除的技能報成過期

- **WHEN** worktree 政策關閉的專案於技能檔已同步的狀態下執行技能檔過期探測
- **THEN** 探測結果不列出兩顆 worktree 技能——被政策排除者不屬於預期生成集合

### Requirement: update 清除孤兒技能目錄

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

每一個再生入口——`speclink update`、`speclink init` 與 `speclink init --force`（filesystem 與 Remote Store）、工具選集收斂、工作區補齊、`workflow-config` 寫入後的技能足跡同步、桌面的技能檔更新動作——於各生成目標（內建工具與自訂描述子）完成技能生成後，SHALL 清除該目標 skills 目錄下名稱以 speclink- 為前綴、且不屬於該目標本次應生成集合的目錄。本次應生成集合 SHALL 依既有規則計算：claude 為 registry 全集、codex、copilot 與自訂描述子為非 Claude 子集（registry 中 claude_only 為假的技能），worktree 政策關閉時排除兩顆 worktree 技能。名稱非 speclink- 前綴的目錄 SHALL NOT 被移除。任一目錄刪除失敗時該入口 SHALL 以非零 exit code（或單行錯誤）結束，已生成的檔案保留；重跑 SHALL 收斂到同一終態。本清理 SHALL 與既有三條清理路徑（工具自 tools 下架、自訂描述子移除、worktree 政策關閉）並存，不改變其行為。`speclink init --force` 的選集 SHALL 視為內建工具的完整期望狀態：未選工具的 speclink- 技能目錄 SHALL 移除，自訂描述子的足跡記錄 SHALL 隨設定檔重寫歸零，兩個內建指令檔的遺留 `SPECLINK:START..END` 區塊無論選取與否 SHALL 剝除（使用者內容保留）。init 的 stdout 摘要 SHALL 維持既有兩行，不新增清理明細。

#### Scenario: 技能改名後舊目錄被清除

- **WHEN** 工作區的 skills 目錄含舊版生成的 speclink-onboard 目錄，執行 speclink update
- **THEN** speclink-onboard 目錄不存在，speclink-baseline 目錄存在，兩份技能不並存

#### Scenario: 非前綴目錄不受清理影響

- **WHEN** skills 目錄含使用者自建、名稱非 speclink- 前綴的技能目錄（如 conventional-commit），執行 speclink update
- **THEN** 該目錄與其內容位元級不變

#### Scenario: 前綴保留給生成物

- **WHEN** skills 目錄含名稱以 speclink- 為前綴、但不在本次應生成集合內的目錄，執行 speclink update
- **THEN** 該目錄被清除——speclink- 前綴的目錄一律視為引擎生成物

#### Scenario: init --force 切換工具時清除下架足跡

- **WHEN** 工作區 `.speclink.yaml` 的 tools 為 `[claude]` 且 `.claude/skills/` 含現版技能檔，執行 `speclink init --force --tools codex`
- **THEN** exit code 為 0，stdout 仍為 Initialized 與 Generated files 兩行；`.speclink.yaml` 的 tools 僅含 codex；`.agents/skills/` 含 Codex 生成集合；`.claude/skills/` 下不存在任何 speclink- 目錄，因而變空的 `.claude/skills/` 與 `.claude/` 一併移除

##### Example: 各再生入口的清理面

| 入口 | 觸發 | 清理結果 |
| --- | --- | --- |
| speclink update | tools=[claude] 但 .claude/skills/ 含 speclink-onboard | speclink-onboard 移除 |
| speclink init --force --tools codex | 原 tools=[claude] | .claude/skills/speclink-* 全部移除、.agents/skills/ 補齊 |
| speclink init --force --tools claude | .claude/skills/ 含 speclink-onboard | speclink-onboard 移除、其餘現版 |
| 工具選集收斂 [codex] | 原 tools=[claude] | 同 update（既有行為） |

#### Scenario: init --force 清除改名技能的舊目錄與描述子足跡

- **WHEN** 工作區 `.claude/skills/` 含 speclink-onboard 目錄，且 `.speclink.yaml` 曾含一個描述子、`.speclink/generated-tools.yaml` 記錄其足跡、描述子 skills_dir 下有生成物，執行 `speclink init --force --tools claude`
- **THEN** speclink-onboard 目錄不存在；描述子 skills_dir 下的 speclink- 目錄移除；`.speclink/generated-tools.yaml` 不存在；`.speclink.yaml` 為樣板加 tools=[claude]

#### Scenario: init --force 剝除未選工具的遺留區塊

- **WHEN** `CLAUDE.md` 含遺留 `SPECLINK:START..END` 區塊與使用者段落，執行 `speclink init --force --tools codex`
- **THEN** `CLAUDE.md` 的區塊被剝除、使用者段落原樣保留；`AGENTS.md` 不存在時不被建立

#### Scenario: 不帶 force 的 init 改寫殘留技能檔

- **WHEN** 目錄無 `.speclink.yaml`、無規格資料夾，但 `.claude/skills/speclink-apply/SKILL.md` 為舊版內容且 `.claude/skills/speclink-onboard/` 殘留，執行 `speclink init --tools claude`
- **THEN** exit code 為 0；speclink-apply 的 SKILL.md 為現版內容；speclink-onboard 目錄不存在；其餘生成集合補齊

### Requirement: Copilot 渲染目標

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

內建工具 copilot 的技能檔 SHALL 生成於專案根的 `.github/skills/speclink-<name>/SKILL.md`。生成集合 SHALL 與 codex 相同：registry 中標記為非 Claude 工具也生成的技能（即除 analyze 以外的技能），worktree 政策關閉時排除兩顆 worktree 技能。frontmatter SHALL 依序僅含 name、description、license、compatibility、metadata（author、version、generatedBy），SHALL NOT 含 context、agent、disallowedTools、allowed-tools、argument-hint、user-invocable、disable-model-invocation。內文 SHALL NOT 含 Claude fork 前言（`## Claude fork context`）。技能之間的引用 SHALL 寫成 `/speclink-<name>`，SHALL NOT 出現 `$speclink-`；`{{TOOL}}` 的代換值 SHALL 為 `copilot`，因此技能指示建立變更與蓋章時帶 `--agent copilot`；`{{PLAN_DIR}}` 的代換值 SHALL 為空字串。除叫用前綴與 `{{TOOL}}` 代換值以外，Copilot 技能內容 SHALL 與同一技能的 codex 生成內容逐字相同。copilot SHALL NOT 產生、改寫或剝除任何指令檔（`.github/copilot-instructions.md`、`AGENTS.md`、`CLAUDE.md`）。工作區 footprint 偵測（`.speclink.yaml` 無 tools 清單時的遺留回退，以及 desktop checkout 的預選）SHALL NOT 因 `.github/` 或 `.github/skills/` 存在而推斷 copilot。

#### Scenario: 選取 Copilot 生成正確的技能檔

- **WHEN** 在空目錄以 `--tools copilot` 執行 speclink init，檢視 `.github/skills/speclink-propose/SKILL.md`
- **THEN** frontmatter 只有 name、description、license、compatibility、metadata 五個頂層鍵，name 為 `speclink-propose`；內文引用其他技能寫成 `/speclink-apply` 這類形式、不含 `$speclink-`、不含 `## Claude fork context`，建立變更的指令帶 `--agent copilot`

##### Example: 三個內建工具的渲染差異

| 工具 | skills 目錄 | 技能引用寫法 | `--agent` 值 | Claude 專用 frontmatter | 含 analyze |
| --- | --- | --- | --- | --- | --- |
| claude | `.claude/skills` | `/speclink-apply` | `claude` | fork 技能有 context、agent；唯讀技能有 disallowedTools | 是 |
| codex | `.agents/skills` | `$speclink-apply` | `codex` | 無 | 否 |
| copilot | `.github/skills` | `/speclink-apply` | `copilot` | 無 | 否 |

#### Scenario: Copilot 與 Codex 生成集合相同

- **WHEN** 在 speclink/config.yaml 無 worktree 鍵的專案以 tools 為 `[codex, copilot]` 執行 speclink update
- **THEN** `.github/skills/` 與 `.agents/skills/` 下的 `speclink-` 目錄名稱集合相同，兩者都不含 `speclink-analyze`、`speclink-apply-with-worktree`、`speclink-worktree-merge`

#### Scenario: 選取 Copilot 不產生指令檔

- **WHEN** 在空目錄以 `--tools copilot` 執行 speclink init
- **THEN** 專案根不存在 `AGENTS.md`、`CLAUDE.md`，`.github/` 下只有 `skills/` 目錄，不存在 `.github/copilot-instructions.md`

#### Scenario: footprint 偵測不推斷 Copilot

- **WHEN** 專案含 `.claude/` 目錄與 `.github/workflows/ci.yml`、`.speclink.yaml` 不含 tools 清單，執行 speclink update
- **THEN** 只生成 `.claude/skills/` 下的技能檔，`.github/skills/` 不被建立，`.github/workflows/ci.yml` 位元級不變
