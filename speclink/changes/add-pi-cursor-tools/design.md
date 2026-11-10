## Context

speclink 目前有三個內建 Agent 工具：claude（`.claude/skills/`，叫用前綴 `/speclink-`）、codex（`.agents/skills/`，`$speclink-`）、copilot（`.github/skills/`，`/speclink-`）。add-copilot-tool 已把全集收斂到 speclink-core 的 `Tool::ALL`：生成目標、取消勾選清單、描述子的保留名與保留目錄、assets.lock 指紋輸入、錯誤訊息裡的工具清單（`Tool::supported_names`）都走它。仍然手寫工具清單的地方：

- speclink-core：`Tool` 列舉本身與它的 match（`parse`、`name`、`skills_dir`、`plan_dir`、`slash_replacement`）、init 模組的 `instructions_path`（窮舉 match）、模組與函式註解裡的「claude, codex and copilot」。
- speclink-cli 的 init：`prompt_for_tools` 的三題、`TOOLS_HINT`、重問提示、`--tools` 的 help 範例。
- speclink-node 的 render：target 錯誤訊息與 `index.d.ts` 的型別聯集。
- desktop：前端 `BUILTIN_TOOLS` 常數（四處勾選共用）、Tauri 殼 checkout 綁定的空選集訊息、`CheckoutInspection` 的註解。
- Herdr 面板 integrations/herdr/mods/speclink/model.js：`SKILL_PREFIX`、`SPECLINK_TOOL`、`SKILL_FOLDERS` 三張對照表。pi 目前對到 codex。
- 測試面：render golden 每個內建工具一份快照；skill_authoring 守門禁止寫死的 `--agent` 值；init 測試把名為 `cursor` 的描述子當探測夾具，並把 `tools: [claude, cursor]` 當未知工具範例。

查證過的 pi 與 Cursor 行為（討論 pi-cursor-tool-support 第 1 輪）：

- pi（repo 已搬到 earendil-works/pi）：原始碼 packages/coding-agent/src/core/package-manager.ts 的掃描順序為 `.pi/skills` → `.agents/skills`（從工作目錄往上找到 repo 根）→ `~/.pi/agent/skills` → `~/.agents/skills`；同名技能保留先找到的並顯示警告；專案層兩個目錄只在專案被信任時才讀。叫用語法 `/skill:<名稱>`，可接參數。認得的 frontmatter 為 name、description、license、compatibility、metadata、allowed-tools、disable-model-invocation；name 規則為小寫英數與連字號、最多 64 字。pi 本身沒有子代理（README：「skips features like sub-agents and plan mode」）。
- Cursor（cursor.com/docs/skills、cursor.com/docs/subagents）：專案層讀 `.agents/skills/`、`.cursor/skills/`，相容讀 `.claude/skills/`、`.codex/skills/`；文件沒寫同名時用哪份。叫用語法 `/<名稱>`。文件列的 frontmatter 為 name（必須等於目錄名）、description、paths、disable-model-invocation、icon、color、metadata，沒列 license、compatibility。Cursor 有子代理（Task 工具），主代理派子代理時可指定模型。

## Goals / Non-Goals

**Goals:**

- `pi`、`cursor` 成為第四、第五個內建工具，在 CLI init、`.speclink.yaml`、update、工具選集收斂、工作區補齊、desktop 四處勾選、Node SDK render 與 Herdr 面板都可選。
- pi 與 Cursor 拿到的技能內容正確：叫用前綴分別為 `/skill:speclink-`、`/speclink-`，agent 分別記為 `pi`、`cursor`，沒有 Claude 專用欄位。
- claude、codex、copilot 的技能內容逐位元不變（只有產物層版號行因遞增而改變）。

**Non-Goals:**

- 不產生指令檔（`AGENTS.md`、`CLAUDE.md`、`.cursor/rules/`）。討論 init-marker-openspec-alignment 已定「不注入指令檔、路由由技能承載」。
- 不產生 `.cursor/agents/` 子代理檔，也不改技能內文的子代理段落。子代理的模型設定屬於討論 station-subagent-model；它的表格的列取自 `Tool::ALL`，會自動多兩列。pi 不能派子代理時，技能既有的「harness 不能派子代理時主對話自己跑檢查」承接。
- 不讓 pi 與 codex 共用 `.agents/skills`：叫用前綴互斥，統一就得改 codex 的 golden。
- 不寫程式避開同名技能：pi 的同名警告接受，Cursor 的同名規則以手動實測確認。
- 不產生使用者層技能（`~/.pi/agent/skills`、`~/.cursor/skills`）：所有內建工具都只產專案層。
- 不接受 `cursor-agent`、`pi-coding-agent` 等工具名別名。
- 不處理把 speclink 嵌進 pi 當 extension 或工具，那是討論 agent-tool-sdk-layering 的範圍。
- Herdr 面板不為 Cursor 分頁追蹤標籤（Claude、Codex 靠 hook，pi 靠 extension，Cursor 沒有接上的機制），只做「把技能送給 Cursor」。
- 不在本 repo 自己的 `.speclink.yaml` 加 pi 或 cursor。
- 不改寫 review-skill「首輪工單記錄套用的規則」裡「三種工具（claude、codex、copilot）生成」的情境：它對這三個工具仍然成立。

## Decisions

### 內建工具列舉加入 Pi 與 Cursor

speclink-core 的 `Tool` 列舉加 `Pi`、`Cursor` 兩個變體，`Tool::ALL` 改為 `[Claude, Codex, Copilot, Pi, Cursor]`（陣列長度 5）。

| 變體 | `name()` | `skills_dir()` | `plan_dir()` | `slash_replacement()` |
| --- | --- | --- | --- | --- |
| Pi | `pi` | `.pi/skills` | 空字串 | `/skill:speclink-` |
| Cursor | `cursor` | `.cursor/skills` | 空字串 | `/speclink-` |

`Tool::parse` 認 `pi`、`cursor`（大小寫不拘、前後空白修掉，與既有規則相同），不認別名。因為生成、取消勾選、描述子保留名與保留目錄、指紋輸入、錯誤訊息都已走 `Tool::ALL` 與 `supported_names`，這些地方不需改程式，錯誤訊息自動變成 `claude, codex, copilot, pi, cursor`。init 模組的 `instructions_path` 對 Pi、Cursor 回傳無值（從未有指令檔，剝除迴圈跳過）。改寫 skills.rs 與 init.rs 裡「claude, codex and copilot」的註解。

替代方案：pi 寫 `.agents/skills` 與 codex 共用。否決理由：codex 版寫 `$speclink-`，pi 要 `/skill:speclink-`，同一份檔案無法同時正確；統一前綴就要改 codex 的 golden。

### Pi 與 Cursor 渲染沿用 codex 路徑只換前綴

`managed_skills` 對所有非 claude 的內建目標取非 Claude 子集，`render_skill_file_for` 只在 claude 時加 Claude 專用欄位與 fork 前言，所以 pi、cursor 直接走同一條路，不加分支：frontmatter 與 codex 相同、子集與 codex 相同、`{{PLAN_DIR}}` 為空、`{{TOOL}}` 為 `pi` 或 `cursor`，唯一差別是 `/speclink:` 的代換值。pi 的代換結果例如 `/skill:speclink-apply`；這個字串不含 `/speclink-`，所以 pi 技能與 codex 技能一樣不含斜線形式的技能引用。

替代方案：用中性渲染（自訂描述子那條路）。否決理由：中性渲染把引用改成純技能名並加 Invocation 前言，但 pi 與 Cursor 都支援斜線叫用，拿掉反而失去可點的入口。

### footprint 偵測不推斷 pi 與 cursor

`detect_footprint_tools` 不因 `.pi/`、`.pi/skills/`、`.cursor/`、`.cursor/skills/` 存在而推斷新工具。很多 repo 有 `.cursor/rules/` 只為 Cursor 的規則；以它推斷會在 desktop checkout 預選時替使用者亂勾。pi、cursor 一律要使用者明示選取。

### 描述子驗證保留 pi、cursor 名稱與目錄

`ToolDescriptor::validate` 經 `Tool::parse` 與 `Tool::ALL` 自動拒絕名為 `pi`、`cursor` 的描述子，以及 skills_dir 正規化後（不分大小寫）等於 `.pi/skills`、`.cursor/skills` 的描述子（含 `./.cursor/skills`、`.cursor/skills/.`、`.cursor//skills`、`.wad/../.pi/skills` 等拼法）。`.cursor/rules`、`.pi/agent` 這類不同目錄照常接受。程式不需改，只補測試。

這是唯一的 **BREAKING**。遷移是刪掉描述子、在 tools 寫 `pi` 或 `cursor`。

### 測試夾具與規格範例的描述子改名

init 測試的 `CURSOR_DESCRIPTOR` 夾具（name `cursor`、skills_dir `.cursor/skills`）與規格「技能檔過期探測」的對應情境、Example，改為 name `my-harness`、skills_dir `.my-harness/skills`（與 skills.rs 單元測試既有的描述子夾具同名）。`tool_selection_notes_an_unknown_builtin_name` 的未知工具範例由 `cursor` 改為 `windsurf`，訊息的工具清單改為五個。改名只換字面值，測試的判定邏輯不變。

### CLI init 五題與五工具錯誤訊息

speclink-cli 的 `prompt_for_tools` 依序問 Claude、Codex、Copilot、Pi、Cursor 五題（文字沿用 `Generate Speclink files for {label}? (y/n): `），五題都答 no 時印一行 `Pick at least one tool: claude, codex, copilot, pi, cursor, or any combination.` 並從 Claude 重問。`TOOLS_HINT` 改為 `no AI tool selected — pass --tools with one or more of claude, codex, copilot, pi, cursor (comma-separated, e.g. --tools claude,cursor)`。`--tools` 的 help 範例改為 `(e.g., claude, codex, copilot, pi, cursor)`。

旗標、stdin、exit code、stdout／stderr 分流都不變：互動詢問只寫 stderr、非互動缺 `--tools` 時零寫入非零結束、redirected stdin 不被當答案。既有以三題答案餵入的測試改成五題。

### desktop 勾選清單加入 pi 與 cursor

前端 `BUILTIN_TOOLS` 改為 `["claude", "codex", "copilot", "pi", "cursor"] as const`。初始化確認、啟用確認、checkout 綁定、設定頁 `.speclink.yaml` 頁的「AI 工具」卡都讀這個常數，所以四處自動多兩列；標籤照舊顯示原始工具名。預設勾選不變：初始化與啟用預設只勾 claude；checkout 依既有選集或 footprint 預選。沿用既有的勾選元件，不新增 @speclink/ui 元件或變體。

Tauri 殼 checkout 綁定的空選集訊息改成「請至少選擇一個內建工具：claude、codex、copilot、pi、cursor」，`preselected_tools` 與 `CheckoutInspection.tools` 的註解改成五個工具。desktop-core 透過 `parse_tool_names` 自動接受 pi、cursor，不需改邏輯，只補測試。

### Node SDK target 加入 pi 與 cursor

speclink-node 的 target 解析已經用 `Tool::parse`，pi、cursor 自動通過；只改錯誤訊息為 `target '<x>' must be 'claude', 'codex', 'copilot', 'pi', 'cursor', or 'neutral'` 與模組註解，`index.d.ts` 的 `target` 聯集加 `'pi' | 'cursor'`。回傳內容與 CLI 生成的 `.pi/skills/speclink-<name>/SKILL.md`、`.cursor/skills/speclink-<name>/SKILL.md` 逐字相同（同一個渲染函式）。

### Herdr 面板對到 pi 與 cursor 工具

integrations/herdr/mods/speclink/model.js 的三張對照表：

- `SPECLINK_TOOL.pi` 由 `codex` 改為 `pi`，缺技能提示變成 `speclink init --tools pi`；新增 `SPECLINK_TOOL.cursor = "cursor"`。
- `SKILL_PREFIX` 新增 `cursor: "/"`，面板因此把 Herdr 的 cursor 程式（match `cursor-agent`）列為可送達的 agent，送出 `/speclink-<技能>`。
- `SKILL_FOLDERS` 新增 cursor：專案 `.cursor/skills`、`.agents/skills`、`.claude/skills`、`.codex/skills`，個人 `~/.cursor/skills`、`~/.agents/skills`、`~/.claude/skills`、`~/.codex/skills`。pi 的四個目錄不變。
- 檔頭註解「Pi 讀 .agents/skills，和 Codex 共用技能檔」改為 pi 有自己的 `.pi/skills`。

只有 codex 技能、沒有 pi 技能的專案：pi 仍讀得到 `.agents/skills` 的 codex 版，面板的「有沒有 speclink 技能」判定照樣命中，不跳缺技能提示；這種專案的 pi 用的是 `$` 前綴的內容，屬既有狀況，文件建議改勾 pi。integrations/herdr/README.md 與 README.en.md 的叫用前綴、技能目錄、PATH 段落同步。本變更不把改好的 mod 套用到使用者本機的 Herdr。

### 技能原稿守門加入 pi 與 cursor 的 --agent 值

crates/engine/speclink-core/tests/it/skill_authoring.rs 的寫死引用偵測，`--agent` 值的正規式由 `claude|codex|copilot` 擴為 `claude|codex|copilot|pi|cursor`，並補一個 `--agent cursor` 被抓到的測試。目前沒有原稿寫死這兩個值，守門應維持綠燈。

### 產物層版號遞增與 golden 再生

assets.lock 的指紋輸入走 `Tool::ALL`，加入兩個工具後指紋必然改變。依「內嵌資產版本鎖定紀律」，`ASSET_VERSION` 從 v1.53.0 遞增為 v1.54.0，再於乾淨樹用 `UPDATE_ASSETS_LOCK=1` 重生 assets.lock。

render golden 新增 `pi_rendering_is_bit_identical_to_golden` 與 `cursor_rendering_is_bit_identical_to_golden`：以 `[Tool::Pi]`、`[Tool::Cursor]` init 後分別快照 `.pi/skills`、`.cursor/skills`，存成 `pi.snapshot.md`、`cursor.snapshot.md`。既有六份 golden 用 `UPDATE_GOLDEN=1` 重生，diff 必須只有 frontmatter 的 `version: "v1.53.0"` → `version: "v1.54.0"`；出現任何其他差異就代表既有渲染被誤改，停下來查。

版號遞增後先 `cargo build -p speclink-cli`，確認 `./target/debug/speclink --version` 顯示 `engine v1.54.0`，再用新 binary 跑 `speclink update`，讓本 repo 的 `.claude/skills/` 與 `.agents/skills/` 技能檔只改版號行。

### 文件同步

中英文同步改寫，內容與結構對等：

- README：`--tools` 說明列五個工具；說明 pi 輸入 `/skill:speclink-propose <變更名稱>`、Cursor 輸入 `/speclink-propose <變更名稱>`。
- getting-started：入口表加 pi 與 Cursor 兩列；init 產物說明加 `.pi/skills/`、`.cursor/skills/`；凡是「在 Codex：」的分支說明 pi 用 `/skill:` 前綴、Cursor 與 Claude 寫法相同，兩者都沒有 `/speclink-analyze`；寫明 pi 要先信任專案才讀專案技能，同時勾 codex 時 pi 會顯示同名警告但用的是 pi 版。
- configuration：內建工具表加 `pi` → `.pi/skills/`、`cursor` → `.cursor/skills/`；描述子 name 不得為 `claude`、`codex`、`agents`、`copilot`、`pi`、`cursor`，並寫明舊 pi／cursor 描述子的遷移方式；tools 欄位說明列五個工具；內建工具不受描述子影響的句子列五個工具。
- workflow：技能名寫法說明加 pi 的 `/skill:` 與 Cursor 的 `/`；`analyze` 只有 Claude 有。
- sdk-node：target 值域加 `pi`、`cursor`。
- product-status：Agent 技能列加 pi `/skill:speclink-*` 與 Cursor `/speclink-*`。

### 同名技能的手動實測

Cursor 對 `.cursor/skills` 與 `.agents/skills`、`.claude/skills` 同名技能的先後沒有文件，也不確定 Cursor 是否接受 license、compatibility 欄位，pi 的同名警告也要親眼確認不影響叫用，所以列為 [M] 手動任務。Cursor 若用到別的版本，不改程式，在 getting-started 的 Cursor 段落寫明「同時勾 claude 或 codex 時 Cursor 可能讀到其他版本，Cursor 使用者建議只勾 cursor」。

## Implementation Contract

**行為**

- `speclink init --tools pi`（filesystem 或 Remote Store）：exit 0，stdout 的 `Generated files for:` 行含 `pi`，`.speclink.yaml` 的 tools 為 `[pi]`，`.pi/skills/speclink-<name>/SKILL.md` 依非 Claude 子集生成；不產生 `.claude/`、`.agents/`、`.github/`、`.cursor/`、`AGENTS.md`、`CLAUDE.md`。`--tools cursor` 同理，只生成 `.cursor/skills/`，不產生 `.cursor/agents/`、`.cursor/rules/`。
- 生成的 pi 與 Cursor 技能 frontmatter 只有 `name`、`description`、`license`、`compatibility`、`metadata`（`author`、`version`、`generatedBy`）；pi 內文的技能引用寫 `/skill:speclink-<name>`，Cursor 寫 `/speclink-<name>`，兩者都沒有 `$speclink-` 與 `## Claude fork context`；propose 技能建立變更的指令分別帶 `--agent pi`、`--agent cursor`。
- 同一技能的 pi 內容把 `/skill:speclink-` 換成 `$speclink-`、`--agent pi` 換成 `--agent codex`，Cursor 內容把 `/speclink-` 換成 `$speclink-`、`--agent cursor` 換成 `--agent codex` 之後，與 codex 內容逐字相同。
- 取消勾選 pi 或 cursor（update、工具選集收斂、init --force、desktop 設定頁）：只移除該目錄下 `speclink-` 前綴的目錄，因而變空的 skills 目錄與其父目錄一併移除，同步前就是空的目錄保留；`.pi/` 或 `.cursor/` 下其他內容（`.cursor/rules/`、使用者自己的技能、pi 的 settings.json）不動。
- `.speclink.yaml` 無 tools 清單的遺留回退與 desktop checkout 預選，不會因 `.pi/` 或 `.cursor/` 存在而選 pi 或 cursor。
- 技能檔過期探測：tools 含 pi 或 cursor 時逐工具資訊多一項，判定規則與 codex 相同。
- `skills.render('apply', { target: 'pi' })` 與 `{ target: 'cursor' }` 回傳與 CLI 生成的對應 SKILL.md 相同的字串。
- Herdr 面板：`commandText('cursor', 'apply', 'a')` 為 `/speclink-apply a`；`missingSkillsPlan('Pi', 'pi')` 的提示含 `--tools pi`；`usableAgents` 對已安裝的 claude、codex、pi、cursor 都回傳。

**介面與資料形狀**

- `.speclink.yaml`：`tools` 清單元素多兩個合法字串 `pi`、`cursor`；沒有新欄位、不改 serde 結構，既有檔案照讀。
- CLI：`speclink init --tools <list>` 的值域加 `pi`、`cursor`；不新增旗標。
- Node SDK：`RenderOptions.target: 'claude' | 'codex' | 'copilot' | 'pi' | 'cursor' | 'neutral'`。
- 過期探測 `--json`／IPC：工具名多 `pi`、`cursor` 兩個值，欄位不變。

**失敗模式**

- `--tools` 含未知名稱（例如 `cursor-agent`）：非零結束，stderr 單行 `unknown tool: <name> (supported: claude, codex, copilot, pi, cursor)`，零寫入。
- 非互動缺 `--tools`：非零結束、stdout 空、stderr 為新的 `TOOLS_HINT`，零寫入。
- 描述子名為 `pi`、`cursor`，或 skills_dir 正規化後等於 `.pi/skills`、`.cursor/skills`：非零結束、單行錯誤指明衝突，該目錄下既有檔案逐字元不變。
- 舊版引擎讀到含 `pi` 或 `cursor` 的 `.speclink.yaml`：沿既有行為記一行 unknown tool note 並略過，不刪 `.pi/skills/`、`.cursor/skills/`。

**驗收**

- `cargo test -p speclink-core`：新 pi、cursor golden、版號鎖定測試、init／config 單元測試（生成、取消勾選清理、footprint 不推斷、描述子保留名與保留目錄、改名後的探測夾具）與 skill_authoring 守門全過。
- `cargo test -p speclink-cli verbs::init` 與 `cargo test -p speclink-cli --test it init_tools`：`--tools pi`、`--tools cursor`、`--tools claude,codex,copilot,pi,cursor`、互動五題、五題皆 no 重問、新錯誤訊息全過。
- speclink-node 的 render 測試：target pi、cursor 與 CLI 生成逐字相同、未知 target 錯誤訊息含 pi、cursor。
- `cargo test -p speclink-desktop-core` 與 `cargo test -p speclink-desktop --lib connections`：設定頁加選 pi、cursor 生成對應目錄、取消勾選清掉、checkout 預選與空選集訊息。
- desktop vitest：四處勾選都出現 pi 與 cursor。
- `node --test scripts/herdr/panel-model.test.mjs`：pi 與 cursor 的前綴、目錄、缺技能提示。
- 文件守門：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全過（中英對等、連結、CLI help 一致）。
- [M] Cursor 與 pi 的同名技能實測。

**範圍**

- In scope：上述 speclink-core workspace 模組、speclink-cli init、speclink-node render、desktop 四處勾選與 checkout 錯誤訊息、Herdr 面板的對照表與說明、golden 與 assets.lock、skill_authoring 守門、中英文件、本 repo 技能檔的版號再生。
- Out of scope：Goals / Non-Goals 列出的全部項目；claude、codex、copilot 的渲染內容；子代理模型設定；Herdr 面板的 Cursor 標籤追蹤。

## Risks / Trade-offs

- [golden 回歸] 版號遞增讓六份既有 golden 同時改變，容易掩蓋誤改 → 重生後逐份檢查 diff 只有 `version:` 一行；pi、cursor golden 另做內容抽查（無 `$speclink-`、無 `agent:`、無 `speclink-analyze`，pi 有 `/skill:speclink-` 與 `--agent pi`，cursor 有 `/speclink-` 與 `--agent cursor`）。
- [版號對撞] 平行的變更若也遞增 `ASSET_VERSION`，合併時版號行與 assets.lock 會衝突 → 合併時以重生衍生物解決（重新遞增、重生 lock 與 golden），不挑邊。
- [CLI 測試] 互動詢問從三題變五題，既有以三個答案餵入的測試會卡在第四題 → 同批改寫 init 的單元測試與 init_tools 整合測試的輸入序列。
- [既有夾具] `cursor` 成為內建名後，探測夾具與未知工具範例會因保留名被拒或不再是未知 → 同批改名為 `my-harness`、`windsurf`，只換字面值。
- [跨平台] `.pi/skills`、`.cursor/skills` 是正斜線邏輯路徑；Windows 上路徑比較與探測回報沿用既有 skills_dir 的正規化，不新增平台分支 → Windows CI 跑同一組 init／probe 測試確認。
- [Cursor 先後未知] Cursor 可能在同名時選到 `.claude/skills` 或 `.agents/skills` 的版本，也可能不認 license、compatibility → [M] 實測；選錯時文件建議 Cursor 使用者只勾 cursor。
- [pi 同名警告] 同時勾 codex 與 pi 時，pi 每次啟動對 18 份技能各顯示一則警告 → 文件寫明這是預期行為、pi 用的是 pi 版。
- [pi 信任] pi 未信任專案時不讀專案技能，使用者會以為技能沒產生 → 文件寫明要先信任專案。
- [BREAKING 描述子] 照舊規格範例設了 `cursor` 描述子的專案升級後 update 失敗 → 錯誤訊息直接指出衝突；configuration 文件寫明遷移方式。
- [舊版混用] 團隊裡舊版 speclink 會把 `pi`、`cursor` 當未知工具略過 → 可接受：只是少生成，不會損壞檔案；新版 update 會補齊。
- [儲存解耦] 本變更只動工作區生成（Local Repo 的技能檔），不碰規格儲存或流程抽象，與 storage 解耦的方向無關。
