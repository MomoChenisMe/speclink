# workspace-tools Specification

## Purpose

AI 工具指令檔在工作區的生成與維護：內建與自訂 tools 描述子的接受與驗證、描述子的同步與清理生命週期、中性渲染目標，以及 init 時的工具選擇與事後的工作區補齊入口。本 capability 保證生成只寫入工作區內的受管檔——不碰 AI 工具的使用者設定檔，產物層版本戳與內嵌資產同源因而可探測指令檔過期，且技能內容隨 worktree 等政策條件式生成、受管檔再生帶降級守門。

## Requirements

### Requirement: tools 自訂描述子的接受與驗證

.speclink.yaml 的 tools 清單 SHALL 接受兩種元素形式：內建工具名字串（claude、codex、copilot、pi、cursor），或自訂描述子物件（欄位：name 必填、skills_dir 必填、instructions_file 選填且不再驅動任何生成、invocation 選填且值域為 cli 或 tool-call、預設 cli）。描述子驗證規則：name SHALL 為 kebab-case（2 至 50 字）且 SHALL NOT 與內建工具名（claude、codex、copilot、pi、cursor，以及 codex 的別名 agents）衝突；skills_dir 與（存在時）instructions_file SHALL 為專案根相對路徑，正規化後 SHALL NOT 逸出專案根——instructions_file 保留驗證僅為遺留 marker 剝除的定位所需。skills_dir SHALL 於驗證時削去結尾的 `/`，其後所有消費端（技能生成、足跡記錄與清理、過期探測的路徑回報）SHALL 一律使用削去後的形式。以下兩條拒絕 SHALL 比對詞法正規化後的路徑（丟棄 `.` 段、以 `..` 回退一段，零檔案系統存取），因此等價拼法與字面拼法同樣被拒：正規化後等同專案根本身的 skills_dir SHALL 被拒絕；正規化後等同任一內建工具 skills 目錄（`.claude/skills`、`.agents/skills`、`.github/skills`、`.pi/skills`、`.cursor/skills`）的 skills_dir SHALL 被拒絕，且這條比對 SHALL 不分大小寫（macOS 與 Windows 的檔案系統上 `.GitHub/skills` 就是 `.github/skills`）。描述子仍帶 instructions_file 欄位時，workspace 檢查 SHALL 輸出一行棄用提示（非錯誤、不影響 exit code）。驗證失敗時指令 SHALL 以非 0 exit code 結束並輸出單行語義化錯誤訊息（指明錯誤欄位與原因）。

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
| `.pi/skills` | 拒絕——內建 pi 的 skills 目錄 |
| `.wad/../.pi/skills` | 拒絕——正規化後等同內建目錄（等價拼法） |
| `.pi/agent` | 接受——與內建目錄不同 |
| `.cursor/skills` | 拒絕——內建 cursor 的 skills 目錄 |
| `.cursor//skills` | 拒絕——正規化後等同內建目錄（等價拼法） |
| `.Cursor/skills` | 拒絕——不分大小寫等同內建 cursor 目錄 |
| `.cursor/rules` | 接受——與內建目錄不同 |

#### Scenario: 名為 copilot 或指向 Copilot 目錄的既有描述子升級後被拒

- **WHEN** 升級前可用的 .speclink.yaml 的 tools 含描述子 name: copilot、skills_dir: .copilot-skills，或描述子 name: my-copilot、skills_dir: .github/skills，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息分別指明 name 與內建工具名衝突、或 skills_dir 屬內建 copilot 目錄；任何技能檔與 .github/skills 下既有檔案逐字元不變

#### Scenario: 名為 pi、cursor 或指向其目錄的既有描述子升級後被拒

- **WHEN** 升級前可用的 .speclink.yaml 的 tools 含描述子 name: cursor、skills_dir: .cursor/skills，或描述子 name: my-pi、skills_dir: .pi/skills，或描述子 name: pi、skills_dir: .pi-skills，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息分別指明 name 與內建工具名衝突、或 skills_dir 屬內建 pi 目錄；任何技能檔與 .cursor/skills、.pi/skills 下既有檔案逐字元不變


<!-- @trace
source: add-pi-cursor-tools
updated: 2026-11-10T21:23:31+08:00
-->

---
### Requirement: 描述子的同步與清理生命週期

speclink update SHALL 對描述子與內建工具一視同仁：在 tools 清單上的描述子重新生成其技能檔；描述子仍帶 instructions_file 欄位且該檔含 SPECLINK marker 區塊時，update SHALL 剝除該區塊（使用者自有內容保留，剝除後全空的檔案刪除）。自清單移除的描述子，其生成物 SHALL 被清理——skills_dir 下的 speclink- 前綴技能目錄移除（因而變空的目錄一併移除）。

#### Scenario: 移除描述子後生成物被清理

- **WHEN** 先以含 wad-harness 描述子的 tools 執行 speclink update，再將該描述子自 tools 移除並重新執行 speclink update
- **THEN** .wad/skills/ 下的 speclink- 前綴目錄被移除

#### Scenario: 更新時剝除描述子的遺留 marker

- **WHEN** 舊版引擎生成的 WAD.md 含 SPECLINK marker 區塊與使用者自有段落，tools 仍含該描述子（instructions_file: WAD.md），執行新版 speclink update
- **THEN** WAD.md 的 marker 區塊被剝除、使用者段落原樣保留；若剝除後全空則整檔刪除


<!-- @trace
source: remove-marker-injection
updated: 2026-08-23
-->

---
### Requirement: 中性渲染目標

描述子生成的技能檔 SHALL 使用中性渲染：內文 SHALL NOT 含 /speclink- slash 前綴與 plan mode 參照；對其他技能的引用 SHALL 渲染成 `speclink-<名稱>`（技能的名稱），SHALL NOT 渲染成與動詞相同的 `speclink <名稱>` 形式；開頭的呼叫說明 SHALL 寫明 `speclink-<名稱>` 是另一支技能、不是動詞，不得當成指令或工具的參數送出；speclink 動詞的措辭依 invocation 決定——cli 為「執行 speclink <動詞>」形式，tool-call 為「呼叫 speclink 工具（參數為 argv 陣列）」形式。內建 claude、codex、copilot、pi 與 cursor 的生成內容 SHALL 與 render golden 基線位元級一致；基線 SHALL 僅隨提案記載的刻意變更同批更新。

#### Scenario: tool-call 措辭

- **WHEN** 描述子 invocation 為 tool-call，執行 speclink update 後檢視生成的技能檔
- **THEN** 內文以「呼叫 speclink 工具」措辭引用動詞，且不含 /speclink- 前綴與 plan mode 字樣

#### Scenario: 內建工具輸出鎖定於 golden 基線

- **WHEN** tools 僅含 claude、codex、copilot、pi 與 cursor，執行 speclink update
- **THEN** 生成的 .claude/skills/、.agents/skills/、.github/skills/、.pi/skills/、.cursor/skills/ 技能內容與 render golden 基線完全一致

#### Scenario: 交棒寫成技能名稱

- **WHEN** 原稿的收尾段引用 apply、archive 與 review 三支技能，以中性的 tool-call 形式渲染
- **THEN** 結果以 `speclink-apply`、`speclink-archive`、`speclink-review` 表述這三處；不含反引號內以 `speclink apply`、`speclink archive <` 開頭的交棒寫法

#### Scenario: 呼叫說明分辨技能與動詞

- **WHEN** 檢視中性 cli 與 tool-call 兩種形式產生的任一技能檔的開頭說明
- **THEN** 說明含一句：寫成 `speclink-<name>` 的是另一支技能，不是動詞

#### Scenario: 真的動詞仍是動詞寫法

- **WHEN** 原稿的步驟寫 `speclink archive <name>`（執行封存動詞），以中性的 tool-call 形式渲染
- **THEN** 該步驟維持 `speclink archive <name>`


<!-- @trace
source: skills-verbs-second-wave
updated: 2026-11-12T16:54:51+08:00
-->

---
### Requirement: init 內建 Agent 工具選擇

所有 `speclink init`（filesystem 與 Remote Store）SHALL 在任何 Workspace 寫入前解析至少一個內建 Agent 工具。顯式 `--tools` SHALL 接受 `claude`、`codex`、`copilot`、`pi`、`cursor` 中的一個，或其中任意多個以逗號分隔（重複名稱合併為一個）並跳過詢問；解析後為空或含未知名稱 SHALL 以非零 exit code 失敗，未知名稱的 stderr 單行訊息 SHALL 列出五個支援的工具名。未提供 `--tools` 且 stdin 為互動終端時，CLI SHALL 於 stderr 依序明示詢問 Claude、Codex、Copilot、Pi、Cursor 五題並允許選任意非空組合；五題皆未選 SHALL NOT 開始 init，並 SHALL 繼續要求有效選擇。未提供 `--tools` 且 stdin 非互動終端時，CLI SHALL 在零寫入狀態以非零 exit code 失敗，stderr SHALL 為單行訊息並指出 `--tools`、五個工具名與逗號分隔的組合寫法，stdout SHALL 為空。init SHALL NOT 將 redirected／piped stdin 當作工具答案，SHALL NOT 新增 stdin payload 或 `--json` 介面。此行為是對早期 footprint 自動偵測的刻意分歧；顯式提供 `--tools` 時成功輸出的格式與 `--no-color` 行為 SHALL 維持既有基線（Generated files 清單內容隨受管集合縮減為技能檔與工作區檔）。

#### Scenario: filesystem init 顯式選擇 Codex

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools codex`
- **THEN** exit code 為 0，stdout 沿用既有 Initialized 與 Generated files 摘要，`.speclink.yaml` 的 built-in tools 僅含 `codex`，並生成 Codex Skills；不生成 `AGENTS.md`

#### Scenario: Remote init 顯式選擇兩個工具

- **WHEN** 在空目錄執行 Remote Store init 並顯式提供 `--tools claude,codex`、有效 project URL 與 repo
- **THEN** exit code 為 0，`.speclink.yaml` 同時含兩個 built-in tools 與 remote section，生成兩組 Skills，且不存在 `speclink/` 與任何指令檔

#### Scenario: 互動終端選擇 Claude 與 Codex

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude 與 Codex 都回答 yes、對 Copilot、Pi、Cursor 回答 no
- **THEN** 詢問文字依 Claude、Codex、Copilot、Pi、Cursor 順序只寫入 stderr，init 以 claude 與 codex 兩個工具執行，成功摘要寫入 stdout

#### Scenario: 互動終端不得提交空選集

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude、Codex、Copilot、Pi、Cursor 五題都回答 no
- **THEN** CLI 不建立 `.speclink.yaml`、`speclink/`、`.gitignore` 或 Skills，於 stderr 印一行要求至少選取一個工具，並從 Claude 起再次詢問五題

#### Scenario: 非互動 init 缺少 tools 零寫入失敗

- **WHEN** stdin 為 pipe 或 redirect，執行 init 且未提供 `--tools`
- **THEN** exit code 非 0、stdout 為空，stderr 單行訊息包含 `--tools`、`claude`、`codex`、`copilot`、`pi` 與 `cursor`，且目標目錄內容逐項不變

#### Scenario: 空或未知的顯式 tools 被拒

- **WHEN** 執行 init 並提供空的 `--tools` 值或含 `vscode` 的值
- **THEN** exit code 非 0，stderr 指出選集為空或 unknown tool（unknown tool 訊息列出 claude、codex、copilot、pi、cursor），且任何 Workspace 檔案都未建立或修改

#### Scenario: no-color 不改變工具選擇語意

- **WHEN** 以 `--no-color` 執行互動式 init 並選取任一有效工具
- **THEN** prompt 與成功輸出不含 ANSI escape sequence，exit code 與檔案效果和有色模式相同

#### Scenario: filesystem init 顯式選擇 Copilot

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools copilot`
- **THEN** exit code 為 0，stdout 的 Generated files 摘要列出 `copilot`，`.speclink.yaml` 的 built-in tools 僅含 `copilot`，並生成 `.github/skills/` 下的 Copilot Skills；不生成 `.claude/`、`.agents/`、`AGENTS.md`、`CLAUDE.md` 與 `.github/copilot-instructions.md`

#### Scenario: 顯式選擇三個工具

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools claude,codex,copilot`
- **THEN** exit code 為 0，`.speclink.yaml` 的 built-in tools 依 claude、codex、copilot 記錄三個工具，並生成 `.claude/skills/`、`.agents/skills/`、`.github/skills/` 三組 Skills

#### Scenario: filesystem init 顯式選擇 Pi

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools pi`
- **THEN** exit code 為 0，stdout 的 Generated files 摘要列出 `pi`，`.speclink.yaml` 的 built-in tools 僅含 `pi`，並生成 `.pi/skills/` 下的 Pi Skills；不生成 `.claude/`、`.agents/`、`.github/`、`.cursor/`、`AGENTS.md` 與 `CLAUDE.md`

#### Scenario: filesystem init 顯式選擇 Cursor

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools cursor`
- **THEN** exit code 為 0，stdout 的 Generated files 摘要列出 `cursor`，`.speclink.yaml` 的 built-in tools 僅含 `cursor`，並生成 `.cursor/skills/` 下的 Cursor Skills；`.cursor/` 下只有 `skills/` 目錄，不生成 `.cursor/agents/`、`.cursor/rules/`、`.claude/`、`.agents/`、`AGENTS.md` 與 `CLAUDE.md`

#### Scenario: 顯式選擇五個工具

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools claude,codex,copilot,pi,cursor`
- **THEN** exit code 為 0，`.speclink.yaml` 的 built-in tools 依 claude、codex、copilot、pi、cursor 記錄五個工具，並生成 `.claude/skills/`、`.agents/skills/`、`.github/skills/`、`.pi/skills/`、`.cursor/skills/` 五組 Skills

##### Example: --tools 值的解析

| `--tools` 值 | 結果 |
| --- | --- |
| `copilot` | 接受，選集為 copilot |
| `Copilot` | 接受，選集為 copilot（大小寫不拘） |
| `claude,copilot` | 接受，選集為 claude、copilot |
| `copilot,copilot` | 接受，選集為 copilot（重複合併） |
| `github-copilot` | 拒絕——unknown tool，訊息列出 claude、codex、copilot、pi、cursor |
| `pi` | 接受，選集為 pi |
| `Cursor` | 接受，選集為 cursor（大小寫不拘） |
| `codex,pi` | 接受，選集為 codex、pi |
| `cursor-agent` | 拒絕——unknown tool（不收別名） |
| `vscode` | 拒絕——unknown tool |


<!-- @trace
source: add-pi-cursor-tools
updated: 2026-11-10T21:23:31+08:00
-->

---
### Requirement: built-in tools 權威收斂

Workspace 工具同步 SHALL 將請求中的 Claude／Codex／Copilot／Pi／Cursor 集合視為 built-in tools 的完整期望狀態。被選取的 built-in SHALL 生成或更新其 `speclink-*` Skills；未選取的 built-in SHALL 移除 Speclink 產生的 Skills。未選取的 copilot、pi、cursor 其移除範圍 SHALL 僅限各自 skills 目錄（`.github/skills/`、`.pi/skills/`、`.cursor/skills/`）下名稱以 `speclink-` 為前綴的目錄（因而變空的 skills 目錄與其所在的 `.github/`、`.pi/`、`.cursor/` 一併移除；同步前就是空的目錄 SHALL 保留）；`.github/`、`.pi/`、`.cursor/` 下其他內容 SHALL 位元級不變。任一未選取 built-in 的 skills 目錄路徑上任何一層是 symlink 時（skills 目錄本身，或其上層目錄，例如 `.cursor` 連到 `.claude`），同步 SHALL NOT 透過它移除任何目錄，symlink 本身保留。同步 SHALL 在任何寫入之前（含 `.speclink.yaml` 的改寫）檢查 skills 目錄有沒有經 symlink 共用：每個已選取工具（內建工具與自訂描述子）的 skills 目錄順著 symlink 解析後，若和另一個已選取工具的 skills 目錄解析後是同一個目錄，或和一個未選取、路徑上沒有 symlink 的內建工具 skills 目錄（同步會清理它）是同一個目錄，同步 SHALL 以非零 exit code 失敗，輸出一行錯誤指出兩個目錄與所屬工具，且 SHALL NOT 寫入任何檔案；`speclink update --allow-downgrade` SHALL NOT 越過這項檢查。解析後指到其他位置（例如專案外的共用資料夾）的 symlink 不受影響，照常同步。無論選取與否，同步 SHALL 剝除 claude 與 codex 指令檔（`CLAUDE.md`、`AGENTS.md`）中遺留的 `SPECLINK:START..END` 區塊——SHALL 保留區塊外的使用者內容，且指令檔在剝除後全空時才刪除；不含區塊的指令檔 SHALL 位元級不變。copilot、pi、cursor 沒有指令檔：同步 SHALL NOT 建立、改寫或刪除 `.github/copilot-instructions.md`，也 SHALL NOT 建立 `.cursor/rules/` 或 `.cursor/agents/`。同步 SHALL 保留 `.speclink.yaml` 內的 custom descriptor、unknown tool entry、remote、spec_dir 與其他頂層鍵。相同期望狀態重試 SHALL 收斂到相同檔案結果，不破壞使用者內容。

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
| codex | codex,pi | 是 | 更新 Codex Skills、補齊 Pi Skills |
| codex,pi | codex | 是 | 更新 Codex Skills、移除 Pi Skills |
| claude | claude,cursor | 是 | 更新 Claude Skills、補齊 Cursor Skills |
| claude,cursor | claude | 是 | 更新 Claude Skills、移除 Cursor Skills |

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

#### Scenario: 取消 Cursor 只移除 speclink 技能目錄

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[claude, cursor]`，`.cursor/skills/` 下另有使用者自己的 `my-skill/SKILL.md`，`.cursor/rules/style.mdc` 存在，然後同步 built-in 選集 `[claude]`
- **THEN** `.cursor/skills/` 下所有 `speclink-` 前綴目錄被移除，`my-skill/SKILL.md` 與 `.cursor/rules/style.mdc` 位元級不變，`.cursor/skills/` 因仍有使用者技能而保留，Claude Skills 被更新

#### Scenario: 取消 Pi 後變空的目錄一併移除

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[codex, pi]`，`.pi/` 下只有 speclink 生成的 `skills/speclink-*` 目錄，然後同步 built-in 選集 `[codex]`
- **THEN** `.pi/skills/` 與 `.pi/` 都不存在，`.agents/skills/` 下的 Codex Skills 被更新

#### Scenario: 已勾選工具的 skills 目錄經 symlink 指到另一個工具的目錄時拒絕

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[claude, cursor]`，`.cursor/skills` 是指向 `../.claude/skills` 的 symlink，執行 speclink update，或執行 `speclink update --allow-downgrade`
- **THEN** exit code 非 0，stderr 為一行錯誤，指出 `.cursor/skills`（cursor）與 `.claude/skills`（claude）是同一個目錄；`.claude/skills/` 下的檔案、`.speclink.yaml` 與 symlink 位元級不變

#### Scenario: 指到未選取工具目錄的 symlink 同樣被拒

- **WHEN** `.claude/skills/` 下有 Claude Skills，`.cursor/skills` 是指向 `../.claude/skills` 的 symlink，同步 built-in 選集 `[cursor]`
- **THEN** 同步以一行錯誤失敗，`.claude/skills/` 下的檔案與 `.speclink.yaml` 位元級不變——不會先寫入 Cursor 版、再把它當成未選取的 claude 清掉

#### Scenario: 指到其他位置的 symlink 照常同步

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[claude]`，`.claude` 是指向專案外共用資料夾的 symlink，執行 speclink update
- **THEN** exit code 為 0，Claude Skills 寫入該共用資料夾下的 `skills/`

#### Scenario: 取消勾選工具的上層目錄是 symlink 時不清理

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[claude]`，`.cursor` 是指向 `.claude` 的 symlink（所以 `.cursor/skills` 就是 `.claude/skills`），執行 speclink update
- **THEN** exit code 為 0，`.claude/skills/` 下的 Claude Skills 全數保留，`.cursor` symlink 本身保留，清除摘要不列 cursor


<!-- @trace
source: add-pi-cursor-tools
updated: 2026-11-10T21:23:31+08:00
-->

---
### Requirement: 工作區補齊入口

引擎 SHALL 提供冪等的工作區補齊（adopt）入口：對「有規格資料夾但無 `.speclink.yaml`」的目錄啟用 speclink。規格資料夾是舊佈局的 `openspec/`（且沒有 `speclink/`）時，adopt SHALL 先以與 `speclink init` 匯入相同的程式把 `openspec/` 改名為 `speclink/`、每個 `.openspec.yaml` 改名為 `.meta.yaml`，檔案內容逐位元組不變；在 git repo 裡、有主目錄以外的 worktree 或 `openspec/` 有未提交修改時，adopt SHALL 回單行錯誤說明原因且零寫入。之後 SHALL 補 `speclink/` 骨架缺件（specs/ 與 changes/archive/ 目錄；config.yaml 僅在不存在時寫入範本）、於專案根寫入 `.speclink.yaml` 記錄所選 tools（不寫入 `spec_dir`）、為每個所選工具生成技能檔，並確保專案根 `.gitignore` 涵蓋 `.speclink/` 工作資料夾。既有規格文件（specs、changes、discussions、schemas 及既有 config.yaml）的內容 SHALL 不被改寫。入口 SHALL NOT 受 init 的「已初始化即擋下」限制；tools 空清單 SHALL 回錯誤。重複執行 SHALL 冪等收斂於相同結果。

#### Scenario: 匯入後既有內容位元級不變

- **WHEN** 對已 commit、含 openspec/（內有規格文件、一個含 .openspec.yaml 的變更與自訂 config.yaml）但無 .speclink.yaml 的目錄以 tools=[claude] 執行 adopt
- **THEN** openspec/ 不存在、speclink/ 存在；speclink/ 內的規格文件與 config.yaml 與原檔位元級相同，變更資料檔為 .meta.yaml 且內容與原 .openspec.yaml 相同；專案根產生 .speclink.yaml（tools 含 claude、沒有 spec_dir）與 .claude/skills/ 技能檔，不生成 CLAUDE.md

#### Scenario: 骨架缺件補齊

- **WHEN** 對 openspec/ 內缺 specs/ 目錄與 config.yaml 的未啟用目錄執行 adopt
- **THEN** 匯入後 speclink/specs/、speclink/changes/archive/ 目錄建立，speclink/config.yaml 以範本寫入

#### Scenario: 工作資料夾納入版控忽略

- **WHEN** 對 .gitignore 不存在、或存在但未涵蓋 `.speclink/` 的未啟用目錄執行 adopt
- **THEN** 專案根 .gitignore 涵蓋 `.speclink/`，且該檔既有內容原樣保留

##### Example: 既有 .gitignore 追加而非覆寫

- **GIVEN** 專案根 .gitignore 內容為 `node_modules/\ndist/\n`
- **WHEN** 以 tools=[claude] 執行 adopt
- **THEN** .gitignore 仍含 `node_modules/` 與 `dist/` 兩行，並多出 `.speclink/` 條目

#### Scenario: 重複執行冪等

- **WHEN** 對同一目錄以相同 tools 連續執行 adopt 兩次
- **THEN** 第二次成功且所有生成檔內容與第一次相同

#### Scenario: tools 空清單拒絕

- **WHEN** 以空的 tools 清單執行 adopt
- **THEN** 回單行錯誤，目錄零寫入

#### Scenario: 有未提交修改時拒絕匯入

- **WHEN** 未啟用目錄的 openspec/changes/demo/proposal.md 有未提交修改，以 tools=[claude] 執行 adopt
- **THEN** 回單行錯誤說明 openspec/ 有未提交修改，openspec/ 原封不動，沒有產生 .speclink.yaml、speclink/ 或任何技能檔


<!-- @trace
source: desktop-spec-dir-migration
updated: 2026-10-24T22:35:52+08:00
-->

---
### Requirement: 工具檔生成不寫入 AI 工具的使用者設定檔

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


<!-- @trace
source: spec-dir-path-rewrite
updated: 2026-10-25T21:44:20+08:00
-->

---
### Requirement: 產物層版本戳同源

生成的技能檔 frontmatter 版本欄位 SHALL 為單一產物層版號：同一次生成的所有工具、所有技能檔 SHALL 帶相同版號字串。該版號 SHALL 僅於內嵌資產的 render 內容變動時遞增，SHALL NOT 隨 app 或 CLI 發版自動變動。

#### Scenario: 生成物的版號一致

- **WHEN** tools 含 claude 與 codex，執行 speclink init 或 speclink update 後檢視 .claude/skills/ 與 .agents/skills/ 下任兩個技能檔的 frontmatter 版本欄位
- **THEN** 各處為相同的版號字串，無任何技能檔殘留固定值 "1.0"


<!-- @trace
source: remove-marker-injection
updated: 2026-08-23
-->

---
### Requirement: 內嵌資產版本鎖定紀律

repo SHALL 提交記錄產物層版號與全部 render 輸出指紋的鎖定檔。鎖定測試 SHALL 於 render 指紋與鎖定檔不符而版號未變時失敗，失敗訊息 SHALL 載明修復步驟（遞增版號後以指定環境變數重生鎖定檔）。鎖定檔重生 SHALL 於指紋變動而版號未變時拒絕改寫並失敗。僅遞增版號而 render 內容未變 SHALL 通過。

#### Scenario: 改資產未遞增版號即紅燈

- **WHEN** 修改內嵌技能資產內容而未遞增產物層版號，執行 speclink-core 測試
- **THEN** 鎖定測試失敗，測試輸出含遞增版號與重生鎖定檔的修復指引

#### Scenario: 防呆重生拒絕繞過

- **WHEN** 未遞增版號即以重生環境變數執行鎖定測試，且 render 指紋已變
- **THEN** 鎖定檔不被改寫，測試失敗

#### Scenario: 遞增並重生後通過

- **WHEN** 遞增產物層版號並於乾淨樹以重生環境變數更新鎖定檔後，正常執行測試
- **THEN** 鎖定測試通過，鎖定檔記錄新版號與新指紋

<!-- @trace
source: desktop-instruction-staleness-prompt
updated: 2026-07-31
code:
  - .agents/skills/speclink-apply/SKILL.md
  - .agents/skills/speclink-archive/SKILL.md
  - .agents/skills/speclink-audit/SKILL.md
  - .agents/skills/speclink-commit/SKILL.md
  - .agents/skills/speclink-config/SKILL.md
  - .agents/skills/speclink-discuss/SKILL.md
  - .agents/skills/speclink-drift/SKILL.md
  - .agents/skills/speclink-ingest/SKILL.md
  - .agents/skills/speclink-onboard/SKILL.md
  - .agents/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-analyze/SKILL.md
  - .claude/skills/speclink-apply/SKILL.md
  - .claude/skills/speclink-archive/SKILL.md
  - .claude/skills/speclink-audit/SKILL.md
  - .claude/skills/speclink-commit/SKILL.md
  - .claude/skills/speclink-config/SKILL.md
  - .claude/skills/speclink-discuss/SKILL.md
  - .claude/skills/speclink-drift/SKILL.md
  - .claude/skills/speclink-ingest/SKILL.md
  - .claude/skills/speclink-onboard/SKILL.md
  - .claude/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-verify/SKILL.md
  - AGENTS.md
  - CLAUDE.md
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/instructionUpdatePrompt.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/InstructionUpdatePrompt.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/instructionPrompt.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/tests/archive_readiness_gate.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/render_golden.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: worktree 技能的政策條件式生成

技能足跡生成（speclink init 與 speclink update）SHALL 依流程設定的 worktree 檔值過濾兩顆 worktree 技能——本機專案取自 speclink/config.yaml；remote 專案自 Server 取得技能時由 Server 依該範圍的流程設定過濾，remote 專案退回本機範本時（Server 沒有宣告技能或無法連線）取本機沒有流程設定的預設值——（speclink-apply-with-worktree 與 speclink-worktree-merge）：值為 true 時生成，false 或未設時不生成，且既有生成物依既有清理生命週期移除（技能目錄移除，因而變空的目錄一併移除）。生成判定 SHALL NOT 納入 SPECLINK_WORKTREE 環境變數（環境層僅影響執行期政策，不影響足跡）。其餘技能的生成集合不受 worktree 政策影響。此過濾對 claude、codex、copilot、pi、cursor 與自訂描述子工具一視同仁。

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

#### Scenario: remote 專案的 worktree 技能跟著 Server 的流程設定

- **WHEN** Server 上該範圍的流程設定為 worktree: true，在連到它的 checkout 執行 speclink update
- **THEN** checkout 出現 speclink-apply-with-worktree 與 speclink-worktree-merge 兩個技能目錄；之後 Server 的設定改成 false 並再次執行 speclink update，這兩個目錄被移除


<!-- @trace
source: remote-skill-delivery
updated: 2026-11-13T14:35:20+08:00
-->

---
### Requirement: 受管檔再生的降級守門

引擎的受管檔再生 SHALL 於任何寫入之前判定方向：即將被新建或改寫的技能檔中，任一檔的 frontmatter 版號數值領先當前產物層版號時 SHALL 拒絕執行——輸出單行英文說明（含工作區領先的版號與引擎現版版號）、exit code 非零、SHALL NOT 寫入任何檔案（含設定檔）。判定目標 SHALL 取自該次再生的實際寫入集（tools 清單選集、無清單時的目錄偵測、自訂描述子的技能檔），SHALL NOT 僅取內建工具。守門 SHALL 一體適用於所有經再生入口的路徑：`speclink update`、`speclink init --force` 的重建、工具選集收斂、`workflow-config` 寫入後的技能足跡同步（CLI 與桌面設定頁）、桌面的技能檔更新動作。remote 專案自 Server 取得技能時 SHALL NOT 套用本守門：技能版本跟著 Server，Server 提供的版本舊於 checkout 既有的版本時 SHALL 照常寫入，並於輸出說明技能已換成 Server 提供的版本；remote 專案退回本機範本時本守門照常適用。旗標 `--allow-downgrade`（`speclink update`）SHALL 為本機範本路徑上唯一的明示越過入口；`--force` SHALL NOT 被視為同意降級。缺失、過期、現版與無法判定情境下的既有行為 SHALL 不變。守門訊息 SHALL 為英文單行（與 update 既有輸出語言一致），不隨 locale 設定變化。

#### Scenario: 較新工作區拒絕 update

- **WHEN** 工作區技能檔版號新於引擎現版，執行 speclink update
- **THEN** stderr 單行說明含兩個版號、exit code 非零、工作區零檔案變動

#### Scenario: --allow-downgrade 越過守門

- **WHEN** 工作區技能檔版號新於引擎現版，執行 speclink update --allow-downgrade
- **THEN** 受管檔照常再生為引擎現版內容、exit code 0

#### Scenario: 過期工作區照常更新

- **WHEN** 工作區技能檔版號舊於引擎現版，執行 speclink update（不帶旗標）
- **THEN** 受管檔照常再生、exit code 0，行為與守門引入前相同

#### Scenario: --force 重建不等於同意降級

- **WHEN** 工作區技能檔版號新於引擎現版，執行 speclink init --force
- **THEN** 單行說明含兩個版號、exit code 非零、工作區零檔案變動；`--force` SHALL NOT 越過守門

#### Scenario: 無 tools 清單的工作區同受守門

- **WHEN** `.speclink.yaml` 未記錄 tools 清單（再生走目錄偵測），而既有技能檔的版號新於引擎現版，執行 speclink update
- **THEN** 拒絕執行、零檔案變動；SHALL NOT 因判定面只看內建清單而放行

#### Scenario: 自訂描述子的技能檔同受守門

- **WHEN** tools 清單只含自訂描述子，其 skills_dir 下技能檔的版號新於引擎現版，執行 speclink update
- **THEN** 拒絕執行、零檔案變動

#### Scenario: 技能足跡同步被拒時的失敗形狀

- **WHEN** 工作區技能檔版號新於引擎現版，執行 speclink workflow-config set worktree true
- **THEN** 設定檔已寫入新值，技能足跡同步被守門拒絕——單行說明含兩個版號、exit code 非零、受管檔零變動（沿用同步失敗的既有形狀）

#### Scenario: remote 專案換成 Server 較舊的版本

- **WHEN** remote 專案 checkout 的技能檔版號為 v1.60.0，Server 宣告並提供 v1.58.0，執行 speclink update（不帶 --allow-downgrade）
- **THEN** exit code 為 0，技能檔的版號變成 v1.58.0，stdout 說明技能已換成 Server 提供的版本

#### Scenario: remote 專案退回本機範本時守門仍然有效

- **WHEN** remote 專案 checkout 的技能檔版號領先本機引擎，Server 沒有宣告技能，執行 speclink update
- **THEN** exit code 非零，輸出既有的降級拒絕訊息，沒有任何檔案被寫入


<!-- @trace
source: remote-skill-delivery
updated: 2026-11-13T14:35:20+08:00
-->

---
### Requirement: 引擎版號查詢面

`speclink --version` SHALL 同時輸出套件版號、架構與產物層版號，格式為 `<套件版號> (<架構>, engine <產物層版號>)`；產物層版號 SHALL 與技能檔 frontmatter 版本欄位使用的版號同源。輸出 SHALL 為單行至 stdout、exit code 0。

#### Scenario: --version 含引擎版號

- **WHEN** 執行 speclink --version
- **THEN** stdout 單行同時含套件版號、架構與 engine 產物層版號，exit code 0

##### Example: 版號輸出格式

- **GIVEN** 套件版號 0.1.0、arm64 架構、產物層版號 v1.14.0
- **WHEN** 執行 speclink --version
- **THEN** 輸出含 `0.1.0 (arm64, engine v1.14.0)`


<!-- @trace
source: remove-marker-injection
updated: 2026-08-23
-->

---
### Requirement: 技能檔過期探測

引擎 SHALL 提供唯讀的技能檔過期探測：依 .speclink.yaml 的 tools 清單——內建工具與通過驗證的自訂描述子——讀取各工具 skills 目錄下技能檔 frontmatter 的版本欄位並與當前產物層版號比對，回報五態之一——缺失（任一工具的 skills 目錄下無任何 speclink- 技能檔，即從未安裝或整組移除）、過期（任一工具的技能版號舊於現版）、較新（任一工具的技能版號新於現版，即工作區檔案領先引擎）、現版、無法判定（設定解析失敗或技能檔存在但讀取錯誤）。逐工具方向資訊的工具名 SHALL 為內建名（claude、codex、copilot、pi、cursor）或描述子的 name。無法通過驗證的描述子 SHALL NOT 參與探測，也 SHALL NOT 使結果變為無法判定（壞描述子由 update 的錯誤告知）。方向 SHALL 以版號數值比較判定：去除 v 前綴、以點號拆段、逐段數值比較，段數不足補零；任一邊無法完整解析為數字段時，該工具 SHALL 退回字串相等判定（不等即過期），SHALL NOT 對無法解析的版號排序方向、SHALL NOT 據以判較新。聚合優先序 SHALL 為 較新 > 缺失 > 過期 > 現版：任一工具較新即整體回報較新。過期、缺失或較新時 SHALL 一併回報「更新將新建或改寫且內容與現版 render 不同」的受管檔清單（專案根相對路徑，描述子的路徑以其 skills_dir 起頭）與各工具的方向資訊；比對前 SHALL 正規化換行，僅換行形式差異的檔案 SHALL NOT 列入清單。探測 SHALL NOT 寫入任何檔案。

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

- **WHEN** tools 清單含 claude 與一個合法描述子（name 為 my-harness、skills_dir 為 .my-harness/skills），.claude/skills/ 為現版而 .my-harness/skills/ 下無任何 speclink- 技能檔，執行過期探測
- **THEN** 回報缺失；逐工具資訊含 tool 為 my-harness 且 missing 為真的一項；差異清單含以 `.my-harness/skills/speclink-` 起頭的描述子技能檔路徑（非 Claude 子集，worktree 政策關閉時不含兩顆 worktree 技能），不含任何 .claude/skills/ 路徑

##### Example: 描述子缺失時的回報

- **GIVEN** tools 為 `[claude, {name: my-harness, skills_dir: .my-harness/skills, …}]`，.claude/skills/ 全為現版，.my-harness/skills/ 不存在
- **WHEN** 執行過期探測
- **THEN** status 為 "missing"；tools 含 `{tool: "claude", missing: false, stale: false, newer: false}` 與 `{tool: "my-harness", missing: true, workspaceVersion: null}`；differingFiles 每一項皆以 `.my-harness/skills/speclink-` 起頭並以 `/SKILL.md` 結尾

#### Scenario: Pi 技能目錄缺少判缺失

- **WHEN** tools 清單含 codex 與 pi，.agents/skills/ 為現版而 .pi/skills/ 下無任何 speclink- 技能檔，執行過期探測
- **THEN** 回報缺失；逐工具資訊含 tool 為 pi 且 missing 為真的一項；差異清單每一項皆以 `.pi/skills/speclink-` 起頭，不含任何 .agents/skills/ 路徑

#### Scenario: 描述子技能檔過期判過期

- **WHEN** tools 清單含一個合法描述子，其 skills_dir 下技能檔的版號數值舊於現版，執行過期探測
- **THEN** 回報過期；逐工具資訊該描述子 stale 為真；差異清單列出其內容與現版 render 不同的技能檔路徑

#### Scenario: 無效描述子不影響探測

- **WHEN** tools 清單含 claude 與一個無法通過驗證的描述子（缺 skills_dir），.claude/skills/ 為現版，執行過期探測
- **THEN** 回報現版，逐工具資訊只含 claude，差異清單為空


<!-- @trace
source: add-pi-cursor-tools
updated: 2026-11-10T21:23:31+08:00
-->

---
### Requirement: update 清除孤兒技能目錄

每一個再生入口——`speclink update`、`speclink init` 與 `speclink init --force`（filesystem 與 Remote Store）、工具選集收斂、工作區補齊、`workflow-config` 寫入後的技能足跡同步、桌面的技能檔更新動作——於各生成目標（內建工具與自訂描述子）完成技能生成後，SHALL 清除該目標 skills 目錄下名稱以 speclink- 為前綴、且不屬於該目標本次應生成集合的目錄。本次應生成集合 SHALL 依既有規則計算：claude 為 registry 全集、codex、copilot、pi、cursor 與自訂描述子為非 Claude 子集（registry 中 claude_only 為假的技能），worktree 政策關閉時排除兩顆 worktree 技能。名稱非 speclink- 前綴的目錄 SHALL NOT 被移除。任一目錄刪除失敗時該入口 SHALL 以非零 exit code（或單行錯誤）結束，已生成的檔案保留；重跑 SHALL 收斂到同一終態。本清理 SHALL 與既有三條清理路徑（工具自 tools 下架、自訂描述子移除、worktree 政策關閉）並存，不改變其行為。`speclink init --force` 的選集 SHALL 視為內建工具的完整期望狀態：未選工具的 speclink- 技能目錄 SHALL 移除，自訂描述子的足跡記錄 SHALL 隨設定檔重寫歸零，兩個內建指令檔的遺留 `SPECLINK:START..END` 區塊無論選取與否 SHALL 剝除（使用者內容保留）。init 的 stdout 摘要 SHALL 維持既有兩行，不新增清理明細。

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


<!-- @trace
source: add-pi-cursor-tools
updated: 2026-11-10T21:23:31+08:00
-->

---
### Requirement: Copilot 渲染目標

內建工具 copilot 的技能檔 SHALL 生成於專案根的 `.github/skills/speclink-<name>/SKILL.md`。生成集合 SHALL 與 codex 相同：registry 中標記為非 Claude 工具也生成的技能（即除 analyze 以外的技能），worktree 政策關閉時排除兩顆 worktree 技能。frontmatter SHALL 依序僅含 name、description、license、compatibility、metadata（author、version、generatedBy），SHALL NOT 含 context、agent、disallowedTools、allowed-tools、argument-hint、user-invocable、disable-model-invocation。內文 SHALL NOT 含 Claude fork 前言（`## Claude fork context`）。技能之間的引用 SHALL 寫成 `/speclink-<name>`，SHALL NOT 出現 `$speclink-`；`{{TOOL}}` 的代換值 SHALL 為 `copilot`，因此技能指示建立變更與蓋章時帶 `--agent copilot`；`{{PLAN_DIR}}` 的代換值 SHALL 為空字串。除叫用前綴與 `{{TOOL}}` 代換值以外，Copilot 技能內容 SHALL 與同一技能的 codex 生成內容逐字相同。copilot SHALL NOT 產生、改寫或剝除任何指令檔（`.github/copilot-instructions.md`、`AGENTS.md`、`CLAUDE.md`）。工作區 footprint 偵測（`.speclink.yaml` 無 tools 清單時的遺留回退，以及 desktop checkout 的預選）SHALL NOT 因 `.github/` 或 `.github/skills/` 存在而推斷 copilot。

#### Scenario: 選取 Copilot 生成正確的技能檔

- **WHEN** 在空目錄以 `--tools copilot` 執行 speclink init，檢視 `.github/skills/speclink-propose/SKILL.md`
- **THEN** frontmatter 只有 name、description、license、compatibility、metadata 五個頂層鍵，name 為 `speclink-propose`；內文引用其他技能寫成 `/speclink-apply` 這類形式、不含 `$speclink-`、不含 `## Claude fork context`，建立變更的指令帶 `--agent copilot`

##### Example: 內建工具的渲染差異

| 工具 | skills 目錄 | 技能引用寫法 | `--agent` 值 | Claude 專用 frontmatter | 含 analyze |
| --- | --- | --- | --- | --- | --- |
| claude | `.claude/skills` | `/speclink-apply` | `claude` | fork 技能有 context、agent；唯讀技能有 disallowedTools | 是 |
| codex | `.agents/skills` | `$speclink-apply` | `codex` | 無 | 否 |
| copilot | `.github/skills` | `/speclink-apply` | `copilot` | 無 | 否 |
| pi | `.pi/skills` | `/skill:speclink-apply` | `pi` | 無 | 否 |
| cursor | `.cursor/skills` | `/speclink-apply` | `cursor` | 無 | 否 |

#### Scenario: Copilot 與 Codex 生成集合相同

- **WHEN** 在 speclink/config.yaml 無 worktree 鍵的專案以 tools 為 `[codex, copilot]` 執行 speclink update
- **THEN** `.github/skills/` 與 `.agents/skills/` 下的 `speclink-` 目錄名稱集合相同，兩者都不含 `speclink-analyze`、`speclink-apply-with-worktree`、`speclink-worktree-merge`

#### Scenario: 選取 Copilot 不產生指令檔

- **WHEN** 在空目錄以 `--tools copilot` 執行 speclink init
- **THEN** 專案根不存在 `AGENTS.md`、`CLAUDE.md`，`.github/` 下只有 `skills/` 目錄，不存在 `.github/copilot-instructions.md`

#### Scenario: footprint 偵測不推斷 Copilot

- **WHEN** 專案含 `.claude/` 目錄與 `.github/workflows/ci.yml`、`.speclink.yaml` 不含 tools 清單，執行 speclink update
- **THEN** 只生成 `.claude/skills/` 下的技能檔，`.github/skills/` 不被建立，`.github/workflows/ci.yml` 位元級不變


<!-- @trace
source: add-pi-cursor-tools
updated: 2026-11-10T21:23:31+08:00
-->

---
### Requirement: Pi 渲染目標

內建工具 pi 的技能檔 SHALL 生成於專案根的 `.pi/skills/speclink-<name>/SKILL.md`。生成集合 SHALL 與 codex 相同：registry 中 claude_only 為假的技能（即除 analyze 以外的技能），worktree 政策關閉時排除兩顆 worktree 技能。frontmatter SHALL 依序僅含 name、description、license、compatibility、metadata（author、version、generatedBy），SHALL NOT 含 context、agent、disallowedTools、allowed-tools、disable-model-invocation。內文 SHALL NOT 含 Claude fork 前言（`## Claude fork context`）。技能之間的引用 SHALL 寫成 `/skill:speclink-<name>`（pi 的叫用語法），技能檔 SHALL NOT 含 `$speclink-` 與 `/speclink-`；`{{TOOL}}` 的代換值 SHALL 為 `pi`，因此技能指示建立變更與蓋章時帶 `--agent pi`；`{{PLAN_DIR}}` 的代換值 SHALL 為空字串。除叫用前綴與 `{{TOOL}}` 代換值以外，pi 技能內容 SHALL 與同一技能的 codex 生成內容逐字相同。pi SHALL NOT 產生、改寫或剝除任何指令檔（`AGENTS.md`、`CLAUDE.md`），也 SHALL NOT 寫入 `.pi/` 下 `skills/` 以外的任何檔案。同時選取 codex 與 pi 時，兩組技能 SHALL 各自生成於 `.agents/skills/` 與 `.pi/skills/`，不合併、不互相略過。工作區 footprint 偵測（`.speclink.yaml` 無 tools 清單時的遺留回退，以及 desktop checkout 的預選）SHALL NOT 因 `.pi/` 或 `.pi/skills/` 存在而推斷 pi。

#### Scenario: 選取 Pi 生成正確的技能檔

- **WHEN** 在空目錄以 `--tools pi` 執行 speclink init，檢視 `.pi/skills/speclink-propose/SKILL.md`
- **THEN** frontmatter 只有 name、description、license、compatibility、metadata 五個頂層鍵，name 為 `speclink-propose`；內文引用其他技能寫成 `/skill:speclink-apply` 這類形式、不含 `$speclink-` 與 `/speclink-`、不含 `## Claude fork context`，建立變更的指令帶 `--agent pi`

#### Scenario: Pi 與 Codex 並存時各寫各的目錄

- **WHEN** 在 speclink/config.yaml 無 worktree 鍵的專案以 tools 為 `[codex, pi]` 執行 speclink update
- **THEN** `.pi/skills/` 與 `.agents/skills/` 下的 `speclink-` 目錄名稱集合相同，兩者都不含 `speclink-analyze`、`speclink-apply-with-worktree`、`speclink-worktree-merge`；`.pi/skills/speclink-apply/SKILL.md` 把 `/skill:speclink-` 換成 `$speclink-`、`--agent pi` 換成 `--agent codex` 之後，與 `.agents/skills/speclink-apply/SKILL.md` 逐字相同

#### Scenario: footprint 偵測不推斷 Pi

- **WHEN** 專案含 `.claude/` 目錄與 `.pi/settings.json`、`.speclink.yaml` 不含 tools 清單，執行 speclink update
- **THEN** 只生成 `.claude/skills/` 下的技能檔，`.pi/skills/` 不被建立，`.pi/settings.json` 位元級不變

<!-- @trace
source: add-pi-cursor-tools
updated: 2026-11-10T21:23:31+08:00
-->

---
### Requirement: Cursor 渲染目標

內建工具 cursor 的技能檔 SHALL 生成於專案根的 `.cursor/skills/speclink-<name>/SKILL.md`。生成集合 SHALL 與 codex 相同：registry 中 claude_only 為假的技能（即除 analyze 以外的技能），worktree 政策關閉時排除兩顆 worktree 技能。frontmatter SHALL 依序僅含 name、description、license、compatibility、metadata（author、version、generatedBy），SHALL NOT 含 context、agent、disallowedTools、paths、disable-model-invocation、icon、color。內文 SHALL NOT 含 Claude fork 前言（`## Claude fork context`）。技能之間的引用 SHALL 寫成 `/speclink-<name>`，SHALL NOT 出現 `$speclink-`；`{{TOOL}}` 的代換值 SHALL 為 `cursor`，因此技能指示建立變更與蓋章時帶 `--agent cursor`；`{{PLAN_DIR}}` 的代換值 SHALL 為空字串。除叫用前綴與 `{{TOOL}}` 代換值以外，Cursor 技能內容 SHALL 與同一技能的 codex 生成內容逐字相同。cursor SHALL NOT 產生、改寫或剝除任何指令檔（`AGENTS.md`、`CLAUDE.md`、`.cursor/rules/` 下的檔案），也 SHALL NOT 產生 `.cursor/agents/` 下的子代理檔。工作區 footprint 偵測 SHALL NOT 因 `.cursor/` 或 `.cursor/skills/` 存在而推斷 cursor。

#### Scenario: 選取 Cursor 生成正確的技能檔

- **WHEN** 在空目錄以 `--tools cursor` 執行 speclink init，檢視 `.cursor/skills/speclink-propose/SKILL.md`
- **THEN** frontmatter 只有 name、description、license、compatibility、metadata 五個頂層鍵，name 為 `speclink-propose`（與目錄名相同）；內文引用其他技能寫成 `/speclink-apply` 這類形式、不含 `$speclink-`、不含 `## Claude fork context`，建立變更的指令帶 `--agent cursor`

#### Scenario: Cursor 與 Codex 生成集合相同

- **WHEN** 在 speclink/config.yaml 無 worktree 鍵的專案以 tools 為 `[codex, cursor]` 執行 speclink update
- **THEN** `.cursor/skills/` 與 `.agents/skills/` 下的 `speclink-` 目錄名稱集合相同，兩者都不含 `speclink-analyze`、`speclink-apply-with-worktree`、`speclink-worktree-merge`

#### Scenario: footprint 偵測不推斷 Cursor

- **WHEN** 專案含 `.claude/` 目錄與 `.cursor/rules/style.mdc`、`.speclink.yaml` 不含 tools 清單，執行 speclink update
- **THEN** 只生成 `.claude/skills/` 下的技能檔，`.cursor/skills/` 不被建立，`.cursor/rules/style.mdc` 位元級不變

<!-- @trace
source: add-pi-cursor-tools
updated: 2026-11-10T21:23:31+08:00
-->

---
### Requirement: 技能依規格是否在 checkout 渲染

技能的渲染 SHALL 有一個「規格資料夾是否位於這個 checkout」的輸入，並依它處理原稿的條件區塊。speclink init 於本機模式產生的技能 SHALL 以「在」渲染；speclink init --store remote 產生的技能 SHALL 以「不在」渲染；speclink update 與其他同步技能檔的入口 SHALL 依專案設定決定：`.speclink.yaml` 含 remote 區段時以「不在」渲染，否則以「在」渲染，SHALL NOT 依規格資料夾是否存在於磁碟判斷。技能檔過期的探測 SHALL 用相同的判斷產生比對基準。本機專案以「在」渲染的技能中，對規格檔的 git 步驟與規格路徑的顯示 SHALL 保留；以「不在」渲染的技能 SHALL NOT 含這些內容。render golden 基線 SHALL 增列 Claude 與中性 tool-call 兩種目標以「不在」渲染的快照。

#### Scenario: 本機專案的 worktree 技能保留規格檔的 commit 步驟

- **WHEN** 在開啟 worktree 政策的本機專案執行 speclink update，檢視產生的 apply-with-worktree 技能
- **THEN** 它含「變更的 artifact 尚未 commit 時先 commit」的步驟與對規格資料夾的 git 指令

#### Scenario: remote 專案的技能沒有規格檔的 git 步驟

- **WHEN** 在 `.speclink.yaml` 含 remote 區段的 checkout 執行 speclink update，檢視產生的 commit 技能
- **THEN** 它不含規格資料夾的路徑與對規格檔的 git 步驟，並含一句說明這次 commit 只含程式碼的檔案

#### Scenario: 殘留的規格資料夾不影響判斷

- **WHEN** remote 專案的 checkout 裡留有一個舊的 speclink 資料夾，執行 speclink update
- **THEN** 產生的技能仍然以「不在」渲染

#### Scenario: 探測用相同的基準

- **WHEN** remote 專案剛以 speclink update 產生技能檔，隨即探測技能檔是否過期
- **THEN** 探測結果是目前的版本，沒有任何檔案被列為不同

<!-- @trace
source: skills-verbs-second-wave
updated: 2026-11-12T16:54:51+08:00
-->

---
### Requirement: remote 專案的技能取自 Server

`.speclink.yaml` 含 remote 區段的專案執行 speclink update 時，引擎 SHALL 先握手：Server 宣告技能時，SHALL 以 `.speclink.yaml` 的工具清單向 Server 請求技能全文，並把回應寫成各工具技能目錄下的受管技能檔——檔案集合與內容 SHALL 等於 Server 的回應，checkout 多出來的 speclink- 技能目錄 SHALL 依既有的清理生命週期移除。Server 對某個工具的回應不含任何技能時，SHALL NOT 寫入或移除該工具技能目錄下的任何檔案，並 SHALL 於 stdout 以一行英文列出這些工具、說明它們的技能檔維持原狀；其餘工具照常安裝。Server 沒有宣告技能、沒有可用的憑證、無法連線、或握手沒有成功時，SHALL 退回本機的範本（以「規格資料夾不在 checkout」渲染），並於 stderr 以單行英文說明原因，exit code 為 0。握手宣告了技能而技能請求失敗時 SHALL 以非零 exit code 結束，SHALL NOT 寫入任何檔案，SHALL NOT 退回本機的範本。Server 的回應與 checkout 對不上時——技能名稱不是由小寫英數與連字號組成、至多 64 字元的名稱，同一個工具的技能名稱重複，回應缺少全文，或回應的工具不等於請求的工具——SHALL 同樣以非零 exit code 結束且 SHALL NOT 寫入任何檔案：技能名稱會成為資料夾名稱，不合形式的名稱 SHALL NOT 成為路徑。stdout 的摘要 SHALL 指出技能的來源（server 或 local）與版本。speclink init --store remote 的流程不變，結尾 SHALL 多一行提示：登入後執行 speclink update 以取得 Server 提供的技能。本機專案 SHALL NOT 向任何 Server 請求技能。寫入的技能檔 SHALL 與本機產生的技能檔同形（同樣的路徑與 frontmatter），既有的探測與清理 SHALL 能照常處理它們。

#### Scenario: 自 Server 安裝

- **WHEN** remote 專案的 .speclink.yaml 的 tools 為 claude 與 codex，Server 宣告技能，執行 speclink update
- **THEN** .claude/skills 與 .agents/skills 下每個 speclink- 技能檔的內容等於 Server 對該工具回應的內容；stdout 指出來源為 server 與其版本；exit code 為 0

#### Scenario: 舊 Server 退回本機範本

- **WHEN** Server 的握手沒有宣告技能，執行 speclink update
- **THEN** 技能檔由本機的範本產生，stderr 有一行說明 Server 沒有提供技能，exit code 為 0

#### Scenario: 無法連線退回本機範本

- **WHEN** Server 不可達，執行 speclink update
- **THEN** 技能檔由本機的範本產生，stderr 有一行說明無法連線，exit code 為 0

#### Scenario: 宣告了卻取不到

- **WHEN** Server 的握手宣告技能，但技能請求回應 500，執行 speclink update
- **THEN** exit code 非零，stderr 說明取得技能失敗，checkout 的技能檔逐位元不變

#### Scenario: 回應的技能名稱是路徑

- **WHEN** Server 回應的技能名稱為 `../../escape`，執行 speclink update
- **THEN** exit code 非零，checkout 的技能檔逐位元不變，checkout 之外沒有新檔案

#### Scenario: Server 不再提供的技能被清掉

- **WHEN** checkout 有 speclink-improve 技能目錄，Server 的回應不含 improve，執行 speclink update
- **THEN** speclink-improve 目錄被移除

#### Scenario: Server 對某個工具不提供任何技能

- **WHEN** .speclink.yaml 的 tools 為 claude 與 codex，兩者都已有技能檔；Server 對 claude 回應 apply、對 codex 回應空的清單，執行 speclink update
- **THEN** exit code 為 0；.claude/skills 的技能檔等於 Server 的回應；.agents/skills 下的每個檔案逐位元不變；stdout 有一行指出 codex 沒有拿到技能、技能檔維持原狀

#### Scenario: 自訂工具的技能自 Server 取得

- **WHEN** .speclink.yaml 含一個 invocation 為 tool-call 的自訂描述子，Server 宣告技能，執行 speclink update
- **THEN** 該描述子的技能目錄下的技能檔內容等於 Server 對它回應的中性渲染

#### Scenario: 本機專案不連線

- **WHEN** 在沒有 remote 區段的專案執行 speclink update
- **THEN** 沒有發出任何網路請求，行為與輸出和既有行為相同

<!-- @trace
source: remote-skill-delivery
updated: 2026-11-13T14:35:20+08:00
-->

---
### Requirement: CLI 的技能版本提醒

remote 動詞握手之後，Server 宣告的技能版號在數值上新於本機 CLI 的技能範本版號時，CLI SHALL 於 stderr 印出一行英文提醒，內容含兩邊的版號與更新 Speclink 的方式。speclink update 自己的握手 SHALL NOT 印出這一行，也 SHALL NOT 用掉當天的那一次：它的摘要已經指出技能的來源與版本。同一個 Server 位址同一個日曆日 SHALL 最多提醒一次；記錄 SHALL 存於使用者層級設定目錄，SHALL NOT 寫入專案內的任何檔案。提醒 SHALL NOT 改變 stdout、`--json` 的內容與 exit code，SHALL NOT 阻擋動詞的執行。任一邊的版號無法解析為數字段、Server 沒有宣告、或本機不舊於 Server 時 SHALL NOT 提醒。記錄檔無法讀寫時 SHALL NOT 提醒也 SHALL NOT 報錯；記錄檔不存在、或內容不是記錄時視為沒有記錄。CLI SHALL NOT 自行下載或安裝更新。

#### Scenario: 本機較舊時提醒一次

- **WHEN** 本機 CLI 的技能版號為 v1.57.0，Server 宣告 v1.58.0，當天第一次執行 speclink list --json
- **THEN** stdout 是清單的 JSON，exit code 為 0，stderr 有一行含 v1.57.0、v1.58.0 與更新方式

#### Scenario: 同一天不重複

- **WHEN** 同一天對同一台 Server 再執行一次 speclink list
- **THEN** stderr 沒有提醒

#### Scenario: 隔天再提醒

- **WHEN** 隔一個日曆日再執行 speclink list
- **THEN** stderr 再次出現提醒

#### Scenario: 本機較新或相同時不提醒

- **WHEN** 本機 CLI 的技能版號等於或新於 Server 宣告的版號，執行 speclink list
- **THEN** stderr 沒有提醒

#### Scenario: update 不提醒也不用掉當天的那一次

- **WHEN** 本機 CLI 的技能版號舊於 Server 宣告的版號，當天先執行 speclink update，再執行 speclink list
- **THEN** speclink update 的 stderr 沒有提醒；之後的 speclink list 出現提醒

#### Scenario: 另一台 Server 各自計算

- **WHEN** 當天已對 Server 甲提醒過，改連到同樣較新的 Server 乙執行 speclink list
- **THEN** stderr 出現對 Server 乙的提醒

<!-- @trace
source: remote-skill-delivery
updated: 2026-11-13T14:35:20+08:00
-->