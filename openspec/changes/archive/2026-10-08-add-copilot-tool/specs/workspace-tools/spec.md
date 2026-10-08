## ADDED Requirements

### Requirement: Copilot 渲染目標

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

- **WHEN** 在 openspec/config.yaml 無 worktree 鍵的專案以 tools 為 `[codex, copilot]` 執行 speclink update
- **THEN** `.github/skills/` 與 `.agents/skills/` 下的 `speclink-` 目錄名稱集合相同，兩者都不含 `speclink-analyze`、`speclink-apply-with-worktree`、`speclink-worktree-merge`

#### Scenario: 選取 Copilot 不產生指令檔

- **WHEN** 在空目錄以 `--tools copilot` 執行 speclink init
- **THEN** 專案根不存在 `AGENTS.md`、`CLAUDE.md`，`.github/` 下只有 `skills/` 目錄，不存在 `.github/copilot-instructions.md`

#### Scenario: footprint 偵測不推斷 Copilot

- **WHEN** 專案含 `.claude/` 目錄與 `.github/workflows/ci.yml`、`.speclink.yaml` 不含 tools 清單，執行 speclink update
- **THEN** 只生成 `.claude/skills/` 下的技能檔，`.github/skills/` 不被建立，`.github/workflows/ci.yml` 位元級不變

## MODIFIED Requirements

### Requirement: tools 自訂描述子的接受與驗證
<!-- BEFORE: 內建工具名只有 claude、codex（另保留別名 agents），保留目錄只有 .claude/skills 與 .agents/skills -->

.speclink.yaml 的 tools 清單 SHALL 接受兩種元素形式：內建工具名字串（claude、codex、copilot），或自訂描述子物件（欄位：name 必填、skills_dir 必填、instructions_file 選填且不再驅動任何生成、invocation 選填且值域為 cli 或 tool-call、預設 cli）。描述子驗證規則：name SHALL 為 kebab-case（2 至 50 字）且 SHALL NOT 與內建工具名（claude、codex、copilot，以及 codex 的別名 agents）衝突；skills_dir 與（存在時）instructions_file SHALL 為專案根相對路徑，正規化後 SHALL NOT 逸出專案根——instructions_file 保留驗證僅為遺留 marker 剝除的定位所需。skills_dir SHALL 於驗證時削去結尾的 `/`，其後所有消費端（技能生成、足跡記錄與清理、過期探測的路徑回報）SHALL 一律使用削去後的形式。以下兩條拒絕 SHALL 比對詞法正規化後的路徑（丟棄 `.` 段、以 `..` 回退一段，零檔案系統存取），因此等價拼法與字面拼法同樣被拒：正規化後等同專案根本身的 skills_dir SHALL 被拒絕；正規化後等同任一內建工具 skills 目錄（`.claude/skills`、`.agents/skills`、`.github/skills`）的 skills_dir SHALL 被拒絕，且這條比對 SHALL 不分大小寫（macOS 與 Windows 的檔案系統上 `.GitHub/skills` 就是 `.github/skills`）。描述子仍帶 instructions_file 欄位時，workspace 檢查 SHALL 輸出一行棄用提示（非錯誤、不影響 exit code）。驗證失敗時指令 SHALL 以非 0 exit code 結束並輸出單行語義化錯誤訊息（指明錯誤欄位與原因）。

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

### Requirement: 中性渲染目標
<!-- BEFORE: golden 基線只鎖定內建 claude 與 codex 的生成內容 -->

描述子生成的技能檔 SHALL 使用中性渲染：內文 SHALL NOT 含 /speclink- slash 前綴與 plan mode 參照；speclink 動詞的措辭依 invocation 決定——cli 為「執行 speclink <動詞>」形式，tool-call 為「呼叫 speclink 工具（參數為 argv 陣列）」形式。內建 claude、codex 與 copilot 的生成內容 SHALL 與 render golden 基線位元級一致；基線 SHALL 僅隨提案記載的刻意變更同批更新。

#### Scenario: tool-call 措辭

- **WHEN** 描述子 invocation 為 tool-call，執行 speclink update 後檢視生成的技能檔
- **THEN** 內文以「呼叫 speclink 工具」措辭引用動詞，且不含 /speclink- 前綴與 plan mode 字樣

#### Scenario: 內建工具輸出鎖定於 golden 基線

- **WHEN** tools 僅含 claude、codex 與 copilot，執行 speclink update
- **THEN** 生成的 .claude/skills/、.agents/skills/、.github/skills/ 技能內容與 render golden 基線完全一致

### Requirement: init 內建 Agent 工具選擇
<!-- BEFORE: --tools 只接受 claude、codex；互動終端只問 Claude 與 Codex 兩題；非互動錯誤訊息列兩個工具與三種選法 -->

所有 `speclink init`（filesystem 與 Remote Store）SHALL 在任何 Workspace 寫入前解析至少一個內建 Agent 工具。顯式 `--tools` SHALL 接受 `claude`、`codex`、`copilot` 中的一個，或其中任意多個以逗號分隔（重複名稱合併為一個）並跳過詢問；解析後為空或含未知名稱 SHALL 以非零 exit code 失敗，未知名稱的 stderr 單行訊息 SHALL 列出三個支援的工具名。未提供 `--tools` 且 stdin 為互動終端時，CLI SHALL 於 stderr 依序明示詢問 Claude、Codex、Copilot 三題並允許選任意非空組合；三題皆未選 SHALL NOT 開始 init，並 SHALL 繼續要求有效選擇。未提供 `--tools` 且 stdin 非互動終端時，CLI SHALL 在零寫入狀態以非零 exit code 失敗，stderr SHALL 為單行訊息並指出 `--tools`、三個工具名與逗號分隔的組合寫法，stdout SHALL 為空。init SHALL NOT 將 redirected／piped stdin 當作工具答案，SHALL NOT 新增 stdin payload 或 `--json` 介面。此行為是對早期 footprint 自動偵測的刻意分歧；顯式提供 `--tools` 時成功輸出的格式與 `--no-color` 行為 SHALL 維持既有基線（Generated files 清單內容隨受管集合縮減為技能檔與工作區檔）。

#### Scenario: filesystem init 顯式選擇 Codex

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools codex`
- **THEN** exit code 為 0，stdout 沿用既有 Initialized 與 Generated files 摘要，`.speclink.yaml` 的 built-in tools 僅含 `codex`，並生成 Codex Skills；不生成 `AGENTS.md`

#### Scenario: Remote init 顯式選擇兩個工具

- **WHEN** 在空目錄執行 Remote Store init 並顯式提供 `--tools claude,codex`、有效 project URL 與 repo
- **THEN** exit code 為 0，`.speclink.yaml` 同時含兩個 built-in tools 與 remote section，生成兩組 Skills，且不存在 `openspec/` 與任何指令檔

#### Scenario: 互動終端選擇 Claude 與 Codex

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude 與 Codex 都回答 yes、對 Copilot 回答 no
- **THEN** 詢問文字依 Claude、Codex、Copilot 順序只寫入 stderr，init 以 claude 與 codex 兩個工具執行，成功摘要寫入 stdout

#### Scenario: 互動終端不得提交空選集

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude、Codex、Copilot 三題都回答 no
- **THEN** CLI 不建立 `.speclink.yaml`、`openspec/`、`.gitignore` 或 Skills，於 stderr 印一行要求至少選取一個工具，並從 Claude 起再次詢問三題

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

### Requirement: built-in tools 權威收斂
<!-- BEFORE: 完整期望狀態只涵蓋 Claude／Codex 集合，剝除對象為各 built-in 指令檔 -->

Workspace 工具同步 SHALL 將請求中的 Claude／Codex／Copilot 集合視為 built-in tools 的完整期望狀態。被選取的 built-in SHALL 生成或更新其 `speclink-*` Skills；未選取的 built-in SHALL 移除 Speclink 產生的 Skills。未選取的 copilot 其移除範圍 SHALL 僅限 `.github/skills/` 下名稱以 `speclink-` 為前綴的目錄（因而變空的 `.github/skills/` 一併移除；同步前就是空的目錄 SHALL 保留）；`.github/` 下其他內容 SHALL 位元級不變。任一未選取 built-in 的 skills 目錄本身是 symlink 時，同步 SHALL NOT 透過它移除任何目錄，symlink 本身保留。無論選取與否，同步 SHALL 剝除 claude 與 codex 指令檔（`CLAUDE.md`、`AGENTS.md`）中遺留的 `SPECLINK:START..END` 區塊——SHALL 保留區塊外的使用者內容，且指令檔在剝除後全空時才刪除；不含區塊的指令檔 SHALL 位元級不變。copilot 沒有指令檔：同步 SHALL NOT 建立、改寫或刪除 `.github/copilot-instructions.md`。同步 SHALL 保留 `.speclink.yaml` 內的 custom descriptor、unknown tool entry、remote、spec_dir 與其他頂層鍵。相同期望狀態重試 SHALL 收斂到相同檔案結果，不破壞使用者內容。

#### Scenario: Claude 切換為 Codex並保留自訂工具

- **WHEN** `.speclink.yaml` 含 `claude`、一個 custom descriptor、remote section 與未知頂層鍵，`CLAUDE.md` 同時含遺留 Speclink 區塊和使用者文字，然後同步 built-in 選集 `[codex]`
- **THEN** `tools` 保留 custom descriptor 並將 built-in 集合改為僅 `codex`，remote 與未知鍵值不變，Codex Skills 被補齊，Claude Skills 被移除，`CLAUDE.md` 的遺留區塊被剝除且使用者文字仍存在

##### Example: built-in 選集轉換

| 原 built-in | 新選集 | 保留 custom descriptor | 受管結果 |
| --- | --- | --- | --- |
| claude | codex | 是 | 移除 Claude Skills、補齊 Codex Skills |
| claude,codex | claude | 是 | 更新 Claude Skills、移除 Codex Skills |
| codex | claude,codex | 是 | 補齊兩組 Skills |
| claude | claude,copilot | 是 | 更新 Claude Skills、補齊 Copilot Skills |
| claude,copilot | claude | 是 | 更新 Claude Skills、移除 Copilot Skills |
| codex | copilot | 是 | 移除 Codex Skills、補齊 Copilot Skills |

#### Scenario: 既有選集缺少產物時自動補齊

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[codex]`，但任一 Codex Skill 缺少，然後再次同步 `[codex]`
- **THEN** 缺少或過期的技能檔被補齊至正典內容，其他使用者檔案維持不變，且同步成功

#### Scenario: 壞設定在寫入前被拒

- **WHEN** `.speclink.yaml` 無法解析，然後請求同步任一 built-in 選集
- **THEN** 同步以單行解析錯誤失敗，原設定、Skills 與指令檔逐字元不變

#### Scenario: 更新時剝除內建工具的遺留 marker

- **WHEN** 舊版引擎注入過 marker 的專案（`CLAUDE.md` 含 SPECLINK 區塊與使用者段落）以 tools=[claude] 執行新版 speclink update
- **THEN** 技能檔再生為現版內容，`CLAUDE.md` 的區塊被剝除、使用者段落原樣保留，stdout 摘要列出被剝除的檔案

#### Scenario: 取消 Copilot 只移除 speclink 技能目錄

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[claude, copilot]`，`.github/skills/` 下另有使用者自己的 `my-skill/SKILL.md`，`.github/workflows/ci.yml` 與 `.github/copilot-instructions.md` 存在，然後同步 built-in 選集 `[claude]`
- **THEN** `.github/skills/` 下所有 `speclink-` 前綴目錄被移除，`my-skill/SKILL.md`、`.github/workflows/ci.yml` 與 `.github/copilot-instructions.md` 位元級不變，Claude Skills 被更新

#### Scenario: symlink 的 Copilot 技能目錄不被穿透清除

- **WHEN** `.github/skills` 是指向 `../.claude/skills` 的 symlink，`.speclink.yaml` 的 built-in tools 為 `[claude]`，執行 speclink update
- **THEN** `.claude/skills/` 下的 Claude Skills 全數保留，`.github/skills` symlink 本身保留，清除摘要不列 copilot

#### Scenario: 同步前就空的 Copilot 技能目錄保留

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[claude]`，`.github/skills/` 是同步前就存在的空目錄，執行 speclink update
- **THEN** `.github/skills/` 仍存在

### Requirement: 技能檔過期探測
<!-- BEFORE: 逐工具資訊的內建工具名只列 claude、codex -->

引擎 SHALL 提供唯讀的技能檔過期探測：依 .speclink.yaml 的 tools 清單——內建工具與通過驗證的自訂描述子——讀取各工具 skills 目錄下技能檔 frontmatter 的版本欄位並與當前產物層版號比對，回報五態之一——缺失（任一工具的 skills 目錄下無任何 speclink- 技能檔，即從未安裝或整組移除）、過期（任一工具的技能版號舊於現版）、較新（任一工具的技能版號新於現版，即工作區檔案領先引擎）、現版、無法判定（設定解析失敗或技能檔存在但讀取錯誤）。逐工具方向資訊的工具名 SHALL 為內建名（claude、codex、copilot）或描述子的 name。無法通過驗證的描述子 SHALL NOT 參與探測，也 SHALL NOT 使結果變為無法判定（壞描述子由 update 的錯誤告知）。方向 SHALL 以版號數值比較判定：去除 v 前綴、以點號拆段、逐段數值比較，段數不足補零；任一邊無法完整解析為數字段時，該工具 SHALL 退回字串相等判定（不等即過期），SHALL NOT 對無法解析的版號排序方向、SHALL NOT 據以判較新。聚合優先序 SHALL 為 較新 > 缺失 > 過期 > 現版：任一工具較新即整體回報較新。過期、缺失或較新時 SHALL 一併回報「更新將新建或改寫且內容與現版 render 不同」的受管檔清單（專案根相對路徑，描述子的路徑以其 skills_dir 起頭）與各工具的方向資訊；比對前 SHALL 正規化換行，僅換行形式差異的檔案 SHALL NOT 列入清單。探測 SHALL NOT 寫入任何檔案。

#### Scenario: 舊版工作區判過期並列差異檔

- **WHEN** 工作區技能檔的版號數值舊於當前產物層版號，執行過期探測
- **THEN** 回報過期，並列出內容與現版 render 不同的技能檔相對路徑

#### Scenario: 工作區檔案領先引擎判較新

- **WHEN** 工作區技能檔的版號數值新於當前產物層版號（如新引擎再生後以舊版 app 探測），執行過期探測
- **THEN** 回報較新，並回報差異檔清單；SHALL NOT 與過期混同

##### Example: 引擎 v1.11.0 探測 v1.14.0 工作區

- **GIVEN** 引擎產物層版號 v1.11.0，工作區技能檔 frontmatter 版本 v1.14.0
- **WHEN** 執行過期探測
- **THEN** status 為 "newer"（2026-08-05 事故情境：舊判準回報「過期」並導致按「更新」降級 30 檔）

#### Scenario: 較新優先於缺失與過期

- **WHEN** tools 清單含 claude 與 codex，.claude/skills/ 技能版號新於現版而 .agents/skills/ 下無任何 speclink- 技能檔，執行過期探測
- **THEN** 回報較新（非缺失）——任何會改寫領先檔案的動作都不應被提供

#### Scenario: 無法解析的版號退回相等判定

- **WHEN** 工作區技能檔的版本欄位為無法解析為數字段的字串（如手改壞的 frontmatter），與現版不等，執行過期探測
- **THEN** 該工具判過期（字串不等），SHALL NOT 判較新

#### Scenario: 現版工作區不過期

- **WHEN** 工作區全部技能檔由當前版本的 init 或 update 生成，執行過期探測
- **THEN** 回報現版，差異清單為空

#### Scenario: 技能目錄缺少判缺失

- **WHEN** tools 清單含 claude 與 codex，.claude/skills/ 為現版而 .agents/skills/ 下無任何 speclink- 技能檔（如 clone 後技能未進版控），執行過期探測
- **THEN** 回報缺失，並列出更新將新建或改寫且內容與現版 render 不同的受管檔相對路徑；不與無法判定混同

#### Scenario: 設定損壞回報無法判定

- **WHEN** .speclink.yaml 無法解析，執行過期探測
- **THEN** 回報無法判定；SHALL NOT 與現版或過期混同

#### Scenario: 換行差異不誤報

- **WHEN** 工作區技能檔內容與現版 render 僅換行形式不同（CRLF 對 LF），執行過期探測
- **THEN** 該檔不出現在差異清單

#### Scenario: 描述子技能檔缺失判缺失

- **WHEN** tools 清單含 claude 與一個合法描述子（name 為 cursor、skills_dir 為 .cursor/skills），.claude/skills/ 為現版而 .cursor/skills/ 下無任何 speclink- 技能檔，執行過期探測
- **THEN** 回報缺失；逐工具資訊含 tool 為 cursor 且 missing 為真的一項；差異清單含以 `.cursor/skills/speclink-` 起頭的描述子技能檔路徑（非 Claude 子集，worktree 政策關閉時不含兩顆 worktree 技能），不含任何 .claude/skills/ 路徑

##### Example: 描述子缺失時的回報

- **GIVEN** tools 為 `[claude, {name: cursor, skills_dir: .cursor/skills, …}]`，.claude/skills/ 全為現版，.cursor/skills/ 不存在
- **WHEN** 執行過期探測
- **THEN** status 為 "missing"；tools 含 `{tool: "claude", missing: false, stale: false, newer: false}` 與 `{tool: "cursor", missing: true, workspaceVersion: null}`；differingFiles 每一項皆以 `.cursor/skills/speclink-` 起頭並以 `/SKILL.md` 結尾

#### Scenario: 描述子技能檔過期判過期

- **WHEN** tools 清單含一個合法描述子，其 skills_dir 下技能檔的版號數值舊於現版，執行過期探測
- **THEN** 回報過期；逐工具資訊該描述子 stale 為真；差異清單列出其內容與現版 render 不同的技能檔路徑

#### Scenario: 無效描述子不影響探測

- **WHEN** tools 清單含 claude 與一個無法通過驗證的描述子（缺 skills_dir），.claude/skills/ 為現版，執行過期探測
- **THEN** 回報現版，逐工具資訊只含 claude，差異清單為空

### Requirement: update 清除孤兒技能目錄
<!-- BEFORE: 應生成集合的計算只列 claude 為全集、codex 與自訂描述子為 for_codex 子集 -->

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

- **WHEN** 目錄無 `.speclink.yaml`、無 `openspec/`，但 `.claude/skills/speclink-apply/SKILL.md` 為舊版內容且 `.claude/skills/speclink-onboard/` 殘留，執行 `speclink init --tools claude`
- **THEN** exit code 為 0；speclink-apply 的 SKILL.md 為現版內容；speclink-onboard 目錄不存在；其餘生成集合補齊

### Requirement: worktree 技能的政策條件式生成
<!-- BEFORE: worktree 過濾一視同仁的對象只列 claude、codex 與自訂描述子 -->

技能足跡生成（speclink init 與 speclink update）SHALL 依 openspec/config.yaml 的 worktree 檔值過濾兩顆 worktree 技能（speclink-apply-with-worktree 與 speclink-worktree-merge）：值為 true 時生成，false 或未設時不生成，且既有生成物依既有清理生命週期移除（技能目錄移除，因而變空的目錄一併移除）。生成判定 SHALL NOT 納入 SPECLINK_WORKTREE 環境變數（環境層僅影響執行期政策，不影響足跡）。其餘技能的生成集合不受 worktree 政策影響。此過濾對 claude、codex、copilot 與自訂描述子工具一視同仁。

#### Scenario: 政策關閉時生成集合不含 worktree 技能

- **WHEN** openspec/config.yaml 無 worktree 鍵（或值為 false）的專案執行 speclink update
- **THEN** 各工具 skills 目錄不含 speclink-apply-with-worktree 與 speclink-worktree-merge 目錄，其餘技能照常生成

#### Scenario: 政策開啟時注入兩顆技能

- **WHEN** openspec/config.yaml 含 worktree: true 的專案執行 speclink update
- **THEN** 各工具 skills 目錄含兩顆 worktree 技能，內容與 golden 對照一致

#### Scenario: 政策由開改關後再生即清理

- **WHEN** 先於 worktree: true 下執行 speclink update，再將該鍵改為 false 並重新執行 speclink update
- **THEN** 兩顆 worktree 技能目錄被移除，其餘技能保留

#### Scenario: 環境變數不影響生成

- **WHEN** openspec/config.yaml 無 worktree 鍵，但於 SPECLINK_WORKTREE=true 環境下執行 speclink update
- **THEN** 兩顆 worktree 技能不生成——環境層僅於技能執行期由 P1 政策檢查讀取

#### Scenario: 過期探測不把被政策排除的技能報成過期

- **WHEN** worktree 政策關閉的專案於技能檔已同步的狀態下執行技能檔過期探測
- **THEN** 探測結果不列出兩顆 worktree 技能——被政策排除者不屬於預期生成集合
