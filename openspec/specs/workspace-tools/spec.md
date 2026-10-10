# workspace-tools Specification

## Purpose

AI 工具指令檔在工作區的生成與維護：內建與自訂 tools 描述子的接受與驗證、描述子的同步與清理生命週期、中性渲染目標，以及 init 時的工具選擇與事後的工作區補齊入口。本 capability 保證生成只寫入工作區內的受管檔——不碰 AI 工具的使用者設定檔，產物層版本戳與內嵌資產同源因而可探測指令檔過期，且技能內容隨 worktree 等政策條件式生成、受管檔再生帶降級守門。

## Requirements

### Requirement: tools 自訂描述子的接受與驗證

.speclink.yaml 的 tools 清單 SHALL 接受兩種元素形式：內建工具名字串（claude、codex、copilot；agents 仍為 codex 別名），或自訂描述子物件（欄位：name 必填、skills_dir 必填、instructions_file 選填且不再驅動任何生成、invocation 選填且值域為 cli 或 tool-call、預設 cli）。描述子驗證規則：name SHALL 為 kebab-case（2 至 50 字）且 SHALL NOT 與內建工具名或 agents 別名衝突；既有 name: copilot 描述子 SHALL 在任何寫入前拒絕，stderr 單行錯誤 SHALL 指明 name 與內建 copilot 衝突並提供兩種人工遷移：改名 copilot-custom 保留 skills_dir／invocation，或改為內建字串 copilot。SHALL NOT 自動改寫該物件，exit code SHALL 為 1、配置及技能檔逐位元不變；skills_dir 與（存在時）instructions_file SHALL 為專案根相對路徑，正規化後 SHALL NOT 逸出專案根——instructions_file 保留驗證僅為遺留 marker 剝除的定位所需。skills_dir SHALL 於驗證時削去結尾的 `/`，其後所有消費端（技能生成、足跡記錄與清理、過期探測的路徑回報）SHALL 一律使用削去後的形式。以下兩條拒絕 SHALL 比對詞法正規化後的路徑（丟棄 `.` 段、以 `..` 回退一段，零檔案系統存取），因此等價拼法與字面拼法同樣被拒：正規化後等同專案根本身的 skills_dir SHALL 被拒絕；正規化後等同任一內建工具 skills 目錄（`.claude/skills`、`.agents/skills`）的 skills_dir SHALL 被拒絕。描述子仍帶 instructions_file 欄位時，workspace 檢查 SHALL 輸出一行棄用提示（非錯誤、不影響 exit code）。驗證失敗時指令 SHALL 以非 0 exit code 結束並輸出單行語義化錯誤訊息（指明錯誤欄位與原因）。

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

#### Scenario: 原 copilot 描述子零寫入拒絕

- **WHEN** tools 含物件 name: copilot、skills_dir: .copilot-skills、invocation: tool-call，執行 speclink update
- **THEN** exit code 為 1、stdout 不列成功摘要，stderr 指明 name 衝突並包含 copilot-custom 與內建字串 copilot 的遷移指引；.speclink.yaml、技能及 .speclink 足跡逐位元不變

#### Scenario: 改名後保持自訂語意

- **WHEN** 使用者把上述描述子的 name 改為 copilot-custom、其餘欄位保留，再執行 speclink update
- **THEN** exit code 為 0，.copilot-skills 的技能仍使用 tool-call 中性內容，沒有 .agents/skills 的內建生成；使用者自有技能保留

#### Scenario: 內建共享目錄仍拒絕自訂重用

- **WHEN** 描述子 name 是 my-agent，skills_dir 為 .agents/skills 或其詞法等價拼法，執行 speclink update
- **THEN** exit code 為 1、stderr 指明該目錄屬內建工具，既有 .agents/skills 技能不被覆寫


<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
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

描述子生成的技能檔 SHALL 使用中性渲染：內文 SHALL NOT 含 /speclink- slash 前綴與 plan mode 參照；speclink 動詞的措辭依 invocation 決定——cli 為「執行 speclink <動詞>」形式，tool-call 為「呼叫 speclink 工具（參數為 argv 陣列）」形式。內建 claude 的本文與既有 custom 中性本文 SHALL 保持既有 golden 基線（產物版號戳同源更新除外）。內建 codex 與 copilot SHALL 改為共用中性本文，技能引用 SHALL 使用 speclink- 名稱而非固定 $ 或 / 前綴，操作命令 SHALL 仍為有效 speclink CLI 動詞；SHALL NOT 把 apply／propose 等技能變成不存在的 CLI 子指令。共享本文 SHALL 不含工具專屬 Plan Mode 提醒、未解析模板或 Claude 專屬前言。共享本文及 Codex golden 的變更 SHALL 為本提案明示的刻意變更，同批遞增資產版本及鎖；更新後生成內容 SHALL 與新 golden 位元級一致。

#### Scenario: tool-call 措辭

- **WHEN** 描述子 invocation 為 tool-call，執行 speclink update 後檢視生成的技能檔
- **THEN** 內文以「呼叫 speclink 工具」措辭引用動詞，且不含 /speclink- 前綴與 plan mode 字樣

#### Scenario: 內建工具輸出鎖定於 golden 基線

- **WHEN** tools 僅含 claude 與 codex，執行 speclink update
- **THEN** 生成的 .claude/skills/、.agents/skills/ 技能內容與 render golden 基線完全一致

#### Scenario: 三種共享選集的本文一致

- **WHEN** 在相同 spec_dir、worktree 政策及資產版本下，分別使用 tools=[codex]、[copilot]、[codex,copilot] 生成技能
- **THEN** .agents/skills 每份 SKILL.md 位元級相同；本文以 speclink-propose／speclink-apply 引用技能、以 speclink new change／speclink instructions apply 等有效動詞執行操作，不含 $speclink-、/speclink- 或未解析模板

#### Scenario: 生成模式不隨語言政策分裂

- **WHEN** locale 分別為 tw、ja、en 或未設定，生成 Codex 與 Copilot 共享技能
- **THEN** 每種政策下兩工具的共享本文相同，流程產出語言仍由既有 instructions 政策決定；不新增中文弱偵測、不改既有語言檢查


<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->

---
### Requirement: init 內建 Agent 工具選擇

所有 `speclink init`（filesystem 與 Remote Store）SHALL 在任何 Workspace 寫入前解析至少一個內建 Agent 工具。顯式 --tools SHALL 接受 claude、codex、copilot 或逗號分隔的非空組合並跳過詢問（agents 別名仍解析成 codex）；解析後為空或含未知名稱 SHALL 以非零 exit code 失敗。未提供 `--tools` 且 stdin 為互動終端時，CLI SHALL 於 stderr 依序詢問 Claude、Codex、Copilot 並允許任一非空組合；三者皆未選 SHALL NOT 開始 init，並 SHALL 繼續要求有效選擇。未提供 `--tools` 且 stdin 非互動終端時，CLI SHALL 在零寫入狀態以非零 exit code 失敗，stderr SHALL 指出 --tools 與 claude、codex、copilot 的非空組合選法，stdout SHALL 為空。init SHALL NOT 將 redirected／piped stdin 當作工具答案，SHALL NOT 新增 stdin payload 或 `--json` 介面。此行為是對早期 footprint 自動偵測的刻意分歧；顯式提供 `--tools` 時成功輸出的格式與 `--no-color` 行為 SHALL 維持既有基線（Generated files 清單內容隨受管集合縮減為技能檔與工作區檔）。

#### Scenario: filesystem init 顯式選擇 Codex

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools codex`
- **THEN** exit code 為 0，stdout 沿用既有 Initialized 與 Generated files 摘要，`.speclink.yaml` 的 built-in tools 僅含 `codex`，並生成 Codex Skills；不生成 `AGENTS.md`

#### Scenario: Remote init 顯式選擇兩個工具

- **WHEN** 在空目錄執行 Remote Store init 並顯式提供 `--tools claude,codex`、有效 project URL 與 repo
- **THEN** exit code 為 0，`.speclink.yaml` 同時含兩個 built-in tools 與 remote section，生成兩組 Skills，且不存在 `openspec/` 與任何指令檔

#### Scenario: 互動終端選擇 Claude 與 Codex

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude 與 Codex 回答 yes、對 Copilot 回答 no
- **THEN** 詢問文字只寫入 stderr，init 以兩個工具執行，成功摘要寫入 stdout

#### Scenario: 互動終端不得提交空選集

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude、Codex 與 Copilot 都回答 no
- **THEN** CLI 不建立 `.speclink.yaml`、`openspec/`、`.gitignore` 或 Skills，並再次要求至少選取一個工具

#### Scenario: 非互動 init 缺少 tools 零寫入失敗

- **WHEN** stdin 為 pipe 或 redirect，執行 init 且未提供 `--tools`
- **THEN** exit code 非 0、stdout 為空，stderr 單行訊息包含 `--tools`、`claude` 與 `codex`，且目標目錄內容逐項不變

#### Scenario: 空或未知的顯式 tools 被拒

- **WHEN** 執行 init 並提供空的 `--tools` 值或含 `vscode` 的值
- **THEN** exit code 非 0，stderr 指出選集為空或 unknown tool，且任何 Workspace 檔案都未建立或修改

#### Scenario: no-color 不改變工具選擇語意

- **WHEN** 以 `--no-color` 執行互動式 init 並選取任一有效工具
- **THEN** prompt 與成功輸出不含 ANSI escape sequence，exit code 與檔案效果和有色模式相同

#### Scenario: filesystem 僅選 Copilot

- **WHEN** 在空目錄執行 speclink init --tools copilot
- **THEN** exit code 為 0，stdout 的 Initialized／Generated files 摘要列 copilot 與 .agents/skills 的實際檔案，.speclink.yaml 的 tools 僅含 copilot，生成一套共享技能；不生成 .github/skills、AGENTS.md 或 Copilot 指令檔

#### Scenario: 共選兩工具去重產物

- **WHEN** 在空目錄執行 speclink init --tools codex,copilot
- **THEN** exit code 為 0、stdout 工具名含 codex 與 copilot，每個共享檔案路徑只列一次，tools 保留兩個名稱，.agents/skills 只有一套技能

#### Scenario: 互動僅選 Copilot

- **WHEN** 未提供 --tools 且 stdin 是互動終端，對 Claude、Codex 回答 no、Copilot 回答 yes
- **THEN** 詢問只寫 stderr，成功 exit 0，stdout 摘要列 copilot，tools 僅含 copilot並生成共享技能

#### Scenario: 非互動錯誤提示含新增工具

- **WHEN** stdin 為 pipe、未提供 --tools，執行 init
- **THEN** exit code 為 1、stdout 空白、stderr 提示 --tools 及 claude、codex、copilot，目標目錄零寫入

#### Scenario: remote checkout 僅選 Copilot

- **WHEN** 在空目錄以有效遠端 project URL、repo 與 --tools copilot 執行 remote init
- **THEN** exit code 為 0，生成 .speclink.yaml 的 tools=[copilot] 與 remote section，以及 .agents/skills 的共享技能，不新建可寫的 openspec；CLI 使用既有遠端動詞存取規格

#### Scenario: no-color 的 Copilot 選擇

- **WHEN** 以 --no-color 與 --tools copilot 執行 init
- **THEN** exit code 與檔案效果和有色模式相同，stdout 與 stderr 不含 ANSI；init 仍不提供 --json 或 stdin payload


<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->

---
### Requirement: built-in tools 權威收斂

Workspace 工具同步 SHALL 將請求中的 Claude／Codex／Copilot 集合視為 built-in tools 的完整期望狀態。被選取的 built-in SHALL 生成或更新其 speclink-* Skills；Codex／Copilot 任一仍選取時 SHALL 保留並只生成一份 .agents/skills 共享技能，兩者皆未選取時才 SHALL 移除該目錄的 Speclink 受管技能，其他使用者技能 SHALL 保留。Claude 仍按自身選取狀態生成／清理 .claude/skills。空選集 SHALL 在寫入前拒絕。無論選取與否，同步 SHALL 剝除各 built-in 指令檔（`CLAUDE.md`、`AGENTS.md`）中遺留的 `SPECLINK:START..END` 區塊——SHALL 保留區塊外的使用者內容，且指令檔在剝除後全空時才刪除；不含區塊的指令檔 SHALL 位元級不變。同步 SHALL 保留 `.speclink.yaml` 內的 custom descriptor、unknown tool entry、remote、spec_dir 與其他頂層鍵。相同期望狀態重試 SHALL 收斂到相同檔案結果，不破壞使用者內容。

#### Scenario: Claude 切換為 Codex並保留自訂工具

- **WHEN** `.speclink.yaml` 含 `claude`、一個 custom descriptor、remote section 與未知頂層鍵，`CLAUDE.md` 同時含遺留 Speclink 區塊和使用者文字，然後同步 built-in 選集 `[codex]`
- **THEN** `tools` 保留 custom descriptor 並將 built-in 集合改為僅 `codex`，remote 與未知鍵值不變，Codex Skills 被補齊，Claude Skills 被移除，`CLAUDE.md` 的遺留區塊被剝除且使用者文字仍存在

##### Example: built-in 選集轉換

| 原 built-in | 新選集 | 保留 custom descriptor | 受管結果 |
| --- | --- | --- | --- |
| claude | codex | 是 | 移除 Claude Skills、補齊 Codex Skills |
| claude,codex | claude | 是 | 更新 Claude Skills、移除 Codex Skills |
| codex | claude,codex | 是 | 補齊兩組 Skills |

#### Scenario: 既有選集缺少產物時自動補齊

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[codex]`，但任一 Codex Skill 缺少，然後再次同步 `[codex]`
- **THEN** 缺少或過期的技能檔被補齊至正典內容，其他使用者檔案維持不變，且同步成功

#### Scenario: 壞設定在寫入前被拒

- **WHEN** `.speclink.yaml` 無法解析，然後請求同步任一 built-in 選集
- **THEN** 同步以單行解析錯誤失敗，原設定、Skills 與指令檔逐字元不變

#### Scenario: 更新時剝除內建工具的遺留 marker

- **WHEN** 舊版引擎注入過 marker 的專案（`CLAUDE.md` 含 SPECLINK 區塊與使用者段落）以 tools=[claude] 執行新版 speclink update
- **THEN** 技能檔再生為現版內容，`CLAUDE.md` 的區塊被剝除、使用者段落原樣保留，stdout 摘要列出被剝除的檔案

#### Scenario: 停用其中一個共享工具

- **WHEN** 原 tools=[codex,copilot]，再同步 tools=[copilot]，或由 [copilot] 改為 [codex]
- **THEN** .speclink.yaml 保留真實選取名稱，.agents/skills 受管技能不被刪除且本文不變，不產生第二套目錄

#### Scenario: 最後一個共享工具停用才清理

- **WHEN** 原 tools=[copilot] 且 .agents/skills 另含使用者技能 my-skill，再同步 tools=[claude]
- **THEN** .agents/skills 下 speclink-* 受管技能被清理，my-skill 與使用者設定保留；.claude/skills 生成，工具清單僅記 claude

##### Example: 共享選集轉換

| 原 tools | 新 tools | .agents/skills 受管技能 |
| --- | --- | --- |
| codex,copilot | copilot | 保留，本文不變 |
| copilot | codex | 保留，本文不變 |
| codex | codex,copilot | 一份，本文不變 |
| codex,copilot | claude | 清理，保留使用者自有技能 |

#### Scenario: 不同選取順序不改產物

- **WHEN** 先同步 [codex,copilot] 再同步 [copilot,codex] 並重試
- **THEN** 共享技能檔案與內容相同，不因順序或重試誤刪；update 成功摘要沿既有格式，更新工具名各列一次


<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->

---
### Requirement: 工作區補齊入口

引擎 SHALL 提供冪等的工作區補齊（adopt）入口：對「已有 openspec/ 但無 `.speclink.yaml`」的目錄，補 openspec/ 骨架缺件（specs/ 與 changes/archive/ 目錄；config.yaml 僅在不存在時寫入範本）、於專案根寫入 `.speclink.yaml` 記錄所選 tools、為每個所選工具生成技能檔，並確保專案根 `.gitignore` 涵蓋 `.speclink/` 工作資料夾。既有 openspec/ 文件（specs、changes、discussions 及既有 config.yaml）SHALL 零觸碰。入口 SHALL NOT 受 init 的「已初始化即擋下」限制；tools 空清單 SHALL 回錯誤。重複執行 SHALL 冪等收斂於相同結果。

#### Scenario: 補齊工作區檔且既有內容零觸碰

- **WHEN** 對含 openspec/（內有規格文件與自訂 config.yaml）但無 .speclink.yaml 的目錄以 tools=[claude] 執行 adopt
- **THEN** 專案根產生 .speclink.yaml（tools 含 claude）與 .claude/skills/ 技能檔，openspec/ 內既有文件與 config.yaml 位元級不變，且不生成 CLAUDE.md

#### Scenario: 骨架缺件補齊

- **WHEN** 對 openspec/ 內缺 specs/ 目錄與 config.yaml 的未啟用目錄執行 adopt
- **THEN** specs/、changes/archive/ 目錄建立，config.yaml 以範本寫入

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


<!-- @trace
source: remove-marker-injection
updated: 2026-08-23
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

- **WHEN** 對「已有 openspec/ 但無 .speclink.yaml」的目錄以 tools=[claude] 執行工作區補齊（adopt）
- **THEN** 工作區檔照常補齊（.speclink.yaml、技能檔、.gitignore 條目），`.claude/settings.json` 不存在


<!-- @trace
source: remove-marker-injection
updated: 2026-08-23
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

技能足跡生成（speclink init 與 speclink update）SHALL 依 openspec/config.yaml 的 worktree 檔值過濾兩顆 worktree 技能（speclink-apply-with-worktree 與 speclink-worktree-merge）：值為 true 時生成，false 或未設時不生成，且既有生成物依既有清理生命週期移除（技能目錄移除，因而變空的目錄一併移除）。生成判定 SHALL NOT 納入 SPECLINK_WORKTREE 環境變數（環境層僅影響執行期政策，不影響足跡）。其餘技能的生成集合不受 worktree 政策影響。此過濾對 claude、codex 與自訂描述子工具一視同仁。

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


<!-- @trace
source: remove-marker-injection
updated: 2026-08-23
-->

---
### Requirement: 受管檔再生的降級守門

引擎的受管檔再生 SHALL 於任何寫入之前判定方向：即將被新建或改寫的技能檔中，任一檔的 frontmatter 版號數值領先當前產物層版號時 SHALL 拒絕執行——輸出單行英文說明（含工作區領先的版號與引擎現版版號）、exit code 非零、SHALL NOT 寫入任何檔案（含設定檔）。判定目標 SHALL 取自該次再生的實際寫入集（tools 清單選集、無清單時的目錄偵測、自訂描述子的技能檔），SHALL NOT 僅取內建工具。守門 SHALL 一體適用於所有經再生入口的路徑：`speclink update`、`speclink init --force` 的重建、工具選集收斂、`workflow-config` 寫入後的技能足跡同步（CLI 與桌面設定頁）、桌面的技能檔更新動作。旗標 `--allow-downgrade`（`speclink update`）SHALL 為唯一的明示越過入口；`--force` SHALL NOT 被視為同意降級。缺失、過期、現版與無法判定情境下的既有行為 SHALL 不變。守門訊息 SHALL 為英文單行（與 update 既有輸出語言一致），不隨 locale 設定變化。

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


<!-- @trace
source: remove-marker-injection
updated: 2026-08-23
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
- **THEN** 回報缺失；逐工具資訊含 tool 為 cursor 且 missing 為真的一項；差異清單含以 `.cursor/skills/speclink-` 起頭的描述子技能檔路徑（for_codex 子集，worktree 政策關閉時不含兩顆 worktree 技能），不含任何 .claude/skills/ 路徑

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

#### Scenario: 共享目錄的逐工具結果與差異去重

- **WHEN** tools=[codex,copilot] 的共享技能為舊版，桌面取得技能探測回應
- **THEN** status 為 stale，tools 含 codex 與 copilot 各一筆、workspaceVersion 與方向資訊相同；differingFiles 每個 .agents/skills/speclink-*/SKILL.md 相對路徑只列一次，探測不寫檔

#### Scenario: 缺失與領先一致

- **WHEN** tools=[codex,copilot]，共享目錄沒有技能，或共享技能版號領先引擎
- **THEN** 前者 status=missing 且兩工具 missing=true，後者 status=newer 且兩工具 newer=true；相同目錄不因工具名不同而分類分裂

##### Example: 探測回應型別

| 欄位 | 型別與共享行為 |
| --- | --- |
| status | string，維持 missing／stale／newer／current／unknown |
| currentVersion | string |
| tools | array，項目 tool:string、workspaceVersion:string或null、stale/newer/missing:boolean；共選時 codex、copilot 各一項 |
| differingFiles | string array，repo-root 相對路徑去重，僅換行差異仍不列出 |


<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->

---
### Requirement: update 清除孤兒技能目錄

每一個再生入口——`speclink update`、`speclink init` 與 `speclink init --force`（filesystem 與 Remote Store）、工具選集收斂、工作區補齊、`workflow-config` 寫入後的技能足跡同步、桌面的技能檔更新動作——於各生成目標（內建工具與自訂描述子）完成技能生成後，SHALL 清除該目標 skills 目錄下名稱以 speclink- 為前綴、且不屬於該目標本次應生成集合的目錄。本次應生成集合 SHALL 依既有規則計算：claude 為 registry 全集、codex 與自訂描述子為 for_codex 子集，worktree 政策關閉時排除兩顆 worktree 技能。名稱非 speclink- 前綴的目錄 SHALL NOT 被移除。任一目錄刪除失敗時該入口 SHALL 以非零 exit code（或單行錯誤）結束，已生成的檔案保留；重跑 SHALL 收斂到同一終態。本清理 SHALL 與既有三條清理路徑（工具自 tools 下架、自訂描述子移除、worktree 政策關閉）並存，不改變其行為。`speclink init --force` 的選集 SHALL 視為內建工具的完整期望狀態：未選工具的 speclink- 技能目錄 SHALL 移除，自訂描述子的足跡記錄 SHALL 隨設定檔重寫歸零，兩個內建指令檔的遺留 `SPECLINK:START..END` 區塊無論選取與否 SHALL 剝除（使用者內容保留）。init 的 stdout 摘要 SHALL 維持既有兩行，不新增清理明細。

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


<!-- @trace
source: workspace-sync-entrypoints
updated: 2026-09-07T21:17:45+08:00
-->

---
### Requirement: 共享技能的實際執行工具標記

Codex／Copilot 共享技能 SHALL 指示執行 new change 與 review／verify stamp 時按代理自身身分追加 --agent codex 或 --agent copilot；不能確認時 SHALL 省略 --agent，不得從 tools 勾選或生成目標冒認執行者。共享命令範例 SHALL 不寫死單一工具、不得殘留未解析模板。CLI 動詞本身的 stdout／stderr、exit code、JSON 欄位、認證與 revision 契約 SHALL 維持，這次只修改技能指示；有明確 agent 時仍使用既有欄位記錄，無明確 agent 時不新增假標記。技能呼叫 SHALL 不等同執行 CLI apply／propose；引用 SHALL 使用技能名稱，操作 SHALL 使用實際存在的 CLI 動詞。

#### Scenario: 兩代理建立同一類變更

- **WHEN** 在 tools=[codex,copilot] 的 fixture，Codex 依共享 propose 建立 codex-demo、Copilot 建立 copilot-demo
- **THEN** 兩次均用 speclink new change、對應 --agent codex／copilot，exit 0；各自 .openspec.yaml 的 created_with 為 codex／copilot，既有桌面變更 metadata 回應的 createdWith:string 相同（CLI show JSON 不新增此欄位），stdout 沿既有摘要格式

#### Scenario: 無法確認工具身分

- **WHEN** 讀入共享技能的代理無法確認自身工具名稱
- **THEN** 使用有效 CLI 命令且省略 --agent，不寫成 codex／copilot；CLI 仍沿既有無 agent 的 metadata 與回應契約

#### Scenario: 工單蓋章與缺席

- **WHEN** Copilot 依共享 review／verify 執行具有效工單的 stamp，或對不存在的變更／工單執行查詢
- **THEN** stamp 追加 --agent copilot，成功結果記錄既有 agent 欄位；不存在時沿既有非零 exit、stderr 與 JSON 錯誤契約，不因共享技能改為成功或捏造工單

<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->

---
### Requirement: 共享受管目錄的更新守門

生成、同步、差異清單與降級守門 SHALL 使用相同的共享目錄及期望內容。較新的 .agents/skills 技能 SHALL 在 init／adopt／工具同步／update 預設路徑受既有降級 guard 保護，不因只選 copilot 或同時選兩工具而繞過。只有 update 的既有 --allow-downgrade 可顯性放行；guard 拒絕 SHALL 在任何檔案修改前發生。共享寫入 SHALL 不創建 .github/skills、AGENTS.md、Copilot 指令檔或使用者工具設定；不因同步建立變更 proposal、delta spec、task evidence 或 snapshot，這些仍屬之後流程操作。

#### Scenario: Copilot 單選不能繞過較新技能守門

- **WHEN** .agents/skills 有領先引擎的受管技能，tools 只選 copilot，執行 update 而未提供 --allow-downgrade
- **THEN** exit code 為 1，stderr 指出新版資產與既有例外選項；配置、共享檔與足跡不變

#### Scenario: 顯性允許降級

- **WHEN** 同一 fixture 執行 update --allow-downgrade
- **THEN** exit code 為 0，共享技能再生為引擎版本，兩個選取工具讀取同一份產物，成功摘要格式與 --no-color 行為維持

#### Scenario: 遠端錯誤不改本機綁定語意

- **WHEN** 使用共享技能的 Copilot 在 remote checkout 呼叫既有遠端讀寫動詞，遇 revision 衝突、離線或認證失效
- **THEN** 動詞維持既有非零 exit／stdout JSON與stderr錯誤語意，不以本地 openspec 回寫、不改 remote 綁定，不將規格存入第二份可寫真相

<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->