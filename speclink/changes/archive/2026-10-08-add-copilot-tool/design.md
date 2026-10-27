## Context

speclink 目前只有兩個內建 Agent 工具：claude（技能寫到 `.claude/skills/`，叫用前綴 `/speclink-`）與 codex（`.agents/skills/`，前綴 `$speclink-`）。工具列舉散在幾個地方：

- speclink-core 的 workspace 模組：技能註冊與渲染裡的 `Tool` 列舉、`managed_skills` 的子集判定、`SyncPlan::resolve` 的生成目標與取消勾選清單、遺留 marker 剝除迴圈、footprint 偵測、`parse_tool_names` 的錯誤訊息、`ToolSelection` 對未知工具的 note、`ToolDescriptor::validate` 的保留名與保留目錄。
- speclink-cli 的 init：`TOOLS_HINT` 與 `prompt_for_tools` 的兩題詢問。
- speclink-node 的 render：target 解析的錯誤訊息與 `index.d.ts` 的型別聯集。
- desktop：前端四處寫死 `["claude", "codex"]` 的勾選（初始化確認、啟用確認、checkout 綁定、設定頁 AI 工具卡），以及 Tauri 殼 checkout 綁定的空選集錯誤訊息。desktop-core 與 Tauri 殼都透過 `parse_tool_names`／`Tool::parse` 解析工具名，不另有列舉。
- 測試面：render golden 的 claude、codex、neutral 樣本與 assets.lock 指紋；指紋輸入列舉了 claude、codex 與一個自訂描述子三種渲染目標。

查證過的 Copilot 行為（討論 github-copilot-tool-support 第 2 輪）：

- Copilot 各產品都讀專案的 `.github/skills/`、`.agents/skills/`、`.claude/skills/`。Copilot CLI 依序 `.github/skills` → `.agents/skills` → `.claude/skills`，同名「first found wins」。VS Code 1.125 起依技能名稱去重，但三個目錄的先後未寫明。
- 叫用語法是 `/技能名`。name 必須等於目錄名、小寫英數與連字號、最多 64 字；description 最多 1024 字。speclink 的輸出都符合。
- Copilot 認得 `argument-hint`、`user-invocable`、`disable-model-invocation`、`allowed-tools`；VS Code 另有實驗性的 `context: fork`。Claude 的 `agent`、`disallowedTools` 不在 Copilot 的欄位清單。

## Goals / Non-Goals

**Goals:**

- `copilot` 成為第三個內建工具，在 CLI init、`.speclink.yaml`、update、工具選集收斂、工作區補齊、desktop 四處勾選與 Node SDK render 都可選。
- Copilot 拿到的技能內容正確：叫用前綴 `/speclink-`、agent 記錄為 `copilot`、沒有 Claude 專用欄位。
- claude 與 codex 的技能內容逐位元不變（只有產物層版號行因遞增而改變）。

**Non-Goals:**

- 不產生 `.github/copilot-instructions.md` 或 `AGENTS.md`。討論 init-marker-openspec-alignment 已定「不注入指令檔、路由由技能承載」。
- 不加 Copilot 專用 frontmatter：`context: fork`（VS Code 標示實驗性、要另開設定，CLI 文件未列）、`allowed-tools`（官方警告預先核准 shell 會讓被注入的技能直接執行任意指令）、`argument-hint`（只是輸入框提示，codex 版也沒有）。
- 不接受 `github-copilot` 別名（gh skill 用的 agent id）。目前沒有需求。
- 不處理 Copilot 雲端 agent 的執行環境（在 copilot-setup-steps 裝 speclink CLI）。
- 不處理把 speclink 嵌進 Copilot SDK 當工具，那是討論 agent-tool-sdk-layering 的範圍。
- 不寫程式避開同名技能重複：交給 Copilot 的優先序與去重。
- 不改 codex 的目錄或叫用語法，也不把 codex 與 copilot 合併成一份中性渲染。
- 不在本 repo 自己的 `.speclink.yaml` 加 copilot。

## Decisions

### 內建工具列舉加入 Copilot 並集中為 Tool::ALL

在 speclink-core 的 `Tool` 列舉加 `Copilot` 變體：`name()` 回 `copilot`、`skills_dir()` 回 `.github/skills`、`plan_dir()` 回空字串、`slash_replacement()` 回 `/speclink-`；`Tool::parse` 只認 `copilot`（大小寫不拘、前後空白修掉，與既有規則相同）。

新增一個關聯常數 `Tool::ALL`（依 claude、codex、copilot 順序），取代所有手寫的 `[Tool::Claude, Tool::Codex]` 全集迴圈：`SyncPlan::resolve` 的生成目標與取消勾選清單、`ToolDescriptor::validate` 的保留目錄、render golden 的 assets.lock 指紋輸入。順序固定讓 `Generated files for:` 與探測結果的工具順序穩定。

替代方案：每處各自加一個 `Tool::Copilot`。否決理由：下次加工具又要找一遍，而且漏一處就會出現「生成了但取消勾選時不清」這類不對稱。`Tool::ALL` 是全集的唯一來源。

錯誤與提示文字裡的工具清單（`parse_tool_names` 的 unknown tool、`ToolSelection` 的 unknown tool note、描述子名稱衝突）改成列 `claude, codex, copilot`。

### Copilot 渲染沿用 codex 路徑只換前綴

`render_skill_file_for` 已經只在 claude 時加 `context`／`agent`／`disallowedTools` 與 fork 前言，`managed_skills` 已經對「非 claude」的內建目標取非 Claude 子集（registry 中 `claude_only` 為假的技能；原欄位名 `for_codex` 已名不副實，quality 輪改名並反轉語意）。所以 copilot 直接走同一條路，不加任何分支：frontmatter 與 codex 相同、子集與 codex 相同、`{{PLAN_DIR}}` 為空、`{{TOOL}}` 為 `copilot`，唯一差別是 `/speclink:` 換成 `/speclink-`。

替代方案：Copilot 版改用中性渲染（自訂描述子那條路）。否決理由：中性渲染會把 `/speclink-` 引用改成純技能名並加 Invocation 前言，但 Copilot 本來就支援 `/` 斜線叫用，拿掉反而失去可點的入口。

### footprint 偵測不推斷 copilot

`detect_footprint_tools`（desktop checkout 預選與無 tools 清單時的遺留回退）不因 `.github/` 或 `.github/skills/` 存在而推斷 copilot。大多數 GitHub repo 都有 `.github/workflows/`，以它推斷會在 checkout 預選時替使用者亂勾 copilot。copilot 一律要使用者明示選取。

### 遺留 marker 剝除只對 claude 與 codex

同步流程第 1 步「剝除內建指令檔的遺留 SPECLINK 區塊」只適用曾經被注入過的 `CLAUDE.md` 與 `AGENTS.md`。copilot 從未有指令檔，不剝除任何檔案。把內建指令檔路徑的對照改成回傳可選值（claude → `CLAUDE.md`、codex → `AGENTS.md`、copilot → 無），剝除迴圈走 `Tool::ALL` 並跳過無值者，讓列舉維持窮舉、編譯器會在下次加工具時提醒。

### 描述子驗證保留 copilot 名稱與 .github/skills 目錄

`ToolDescriptor::validate` 的保留名從 `claude, codex, agents` 擴為加上 `copilot`；保留目錄改走 `Tool::ALL` 的 `skills_dir()`，因此 `.github/skills` 與其等價拼法（`./.github/skills`、`.github/skills/.`、`.github//skills`、`.wad/../.github/skills`）一律被拒。理由與既有規則相同：兩個目標指向同一目錄時，探測會對同一份 SKILL.md 比兩種期望內容而永遠回報過期，清理也會互刪。`.github/skills-extra`、`.github/agents` 這類不同目錄照常接受。

這是唯一的 **BREAKING**：既有名為 `copilot` 或指向 `.github/skills` 的描述子，升級後 update 以非零結束並印單行錯誤。遷移是刪掉描述子、在 tools 寫 `copilot`。

### CLI init 第三題與三工具錯誤訊息

speclink-cli 的 `prompt_for_tools` 依序問 Claude、Codex、Copilot 三題（文字沿用 `Generate Speclink files for {label}? (y/n): `，label 為 `Copilot`），三題都答 no 時印一行 `Pick at least one tool: claude, codex, copilot, or any combination.` 並重問三題。`TOOLS_HINT` 改為 `no AI tool selected — pass --tools with one or more of claude, codex, copilot (comma-separated, e.g. --tools claude,copilot)`。`--tools` 的 help 範例改為 `(e.g., claude, codex, copilot)`。

旗標、stdin、exit code、stdout／stderr 分流都不變：互動詢問只寫 stderr、非互動缺 `--tools` 時零寫入非零結束、redirected stdin 不被當答案。

### desktop 四處勾選共用 BUILTIN_TOOLS 常數

前端新增 `apps/desktop/src/builtinTools.ts`，匯出 `BUILTIN_TOOLS = ["claude", "codex", "copilot"] as const`。初始化確認、啟用確認、checkout 綁定、設定頁 AI 工具卡四處的勾選改讀它，標籤照舊顯示原始工具名（與 claude、codex 一致，不翻譯）。預設勾選不變：初始化與啟用預設只勾 claude；checkout 依既有選集或 footprint 預選。

替代方案：從 Rust 經 IPC 取工具清單。否決理由：只是四處靜態清單，多一個 IPC 命令沒有行為可藏，過不了刪除測試。

Tauri 殼 checkout 綁定的空選集訊息改成「請至少選擇一個內建工具：claude、codex、copilot」。`CheckoutInspection.tools` 的註解改成三個工具。desktop-core 透過 `parse_tool_names` 自動接受 copilot，不需改邏輯，只補測試。

### Node SDK target 加入 copilot

speclink-node 的 `resolve_matrix` 已經用 `Tool::parse` 解析內建 target，copilot 自動通過；只改錯誤訊息為 `target '<x>' must be 'claude', 'codex', 'copilot', or 'neutral'`，`index.d.ts` 的 `target` 聯集加 `'copilot'`。回傳內容與 CLI 生成的 `.github/skills/speclink-<name>/SKILL.md` 逐字相同（同一個渲染函式）。

### 產物層版號遞增與 golden 再生

assets.lock 的指紋輸入改走 `Tool::ALL` 後納入 copilot 渲染，指紋必然改變。依「內嵌資產版本鎖定紀律」，`ASSET_VERSION` 從 v1.42.0 遞增為 v1.43.0，再於乾淨樹用 `UPDATE_ASSETS_LOCK=1` 重生 assets.lock。

render golden 新增 `copilot_rendering_is_bit_identical_to_golden`：以 `[Tool::Copilot]` init 後快照 `.github/skills`，存成 `copilot.snapshot.md`。既有五份 golden 用 `UPDATE_GOLDEN=1` 重生，diff 必須只有 frontmatter 的 `version: "v1.42.0"` → `version: "v1.43.0"`；出現任何其他差異就代表 claude／codex 渲染被誤改，停下來查。

版號遞增後先 `cargo build -p speclink-cli`，再用新 binary 跑 `speclink update`，讓本 repo 的 `.claude/skills/` 與 `.agents/skills/` 技能檔只改版號行。

### 文件同步

中英文同步改寫，內容與結構對等：

- README：功能列與「可用」列加 Copilot；入門指令示範 `speclink init --tools claude,codex,copilot` 或在說明中列出 copilot；說明 Copilot 輸入 `/speclink-propose <變更名稱>`。
- getting-started：入口表加一列 Copilot（`/speclink-propose add-csv-export`，`/` 加技能名）；說明 init 產生的技能目錄加 `.github/skills/`；凡是「在 Codex：」的分支說明 Copilot 與 Claude 寫法相同。
- configuration：內建工具表加 `copilot` → `.github/skills/`；描述子 name 不得為 `claude`、`codex`、`agents`、`copilot`；tools 欄位說明列三個工具。
- workflow：技能名寫法說明加 Copilot 與 Claude 同為 `/`；`analyze` 只有 Claude 有（Codex 與 Copilot 直接用 CLI）。
- sdk-node：target 值域加 `copilot`。
- product-status：Agent 技能列加 Copilot `/speclink-*`。

### VS Code 同名技能的手動實測

VS Code 對 `.github/skills` 與 `.claude/skills` 同名技能的先後沒有文件，程式碼也驗證不了，所以列為 [M] 手動任務：在同時勾 claude＋copilot 的專案，VS Code Copilot Chat 的 `/` 選單只出現一份 `speclink-propose`，而且叫用後 agent 用 `--agent copilot` 建立變更。結果若是用到 Claude 版，不改程式，在文件記錄並建議 Copilot 使用者不要同時勾 claude（或用 Copilot CLI 的 `ignoredSkillsLocations` 排除 `.claude/skills`）。

## Implementation Contract

**行為**

- `speclink init --tools copilot`（filesystem 或 Remote Store）：exit 0，stdout 的 `Generated files for:` 行含 `copilot`，`.speclink.yaml` 的 tools 為 `[copilot]`，`.github/skills/speclink-<name>/SKILL.md` 依非 Claude 子集生成，不產生 `.claude/`、`.agents/`、`AGENTS.md`、`CLAUDE.md`、`.github/copilot-instructions.md`。
- 生成的 Copilot 技能 frontmatter 只有 `name`、`description`、`license`、`compatibility`、`metadata`（`author`、`version`、`generatedBy`）；內文的技能引用寫 `/speclink-<name>`，沒有 `$speclink-`；propose 技能建立變更的指令為 `speclink new change "<name>" --agent copilot`；沒有 `## Claude fork context`。
- 取消勾選 copilot（update、工具選集收斂、init --force、desktop 設定頁）：只移除 `.github/skills/` 下 `speclink-` 前綴的目錄，因而變空的 `.github/skills/` 一併移除，同步前就是空的目錄保留；`.github/` 下其他內容（workflows、使用者自己的技能）不動。skills 目錄本身是 symlink（例如指向 `.claude/skills`）時不透過它清除任何目錄——穿過去會刪掉另一個工具剛寫好的技能。描述子 skills_dir 與內建目錄的比對不分大小寫。
- `.speclink.yaml` 無 tools 清單的遺留回退與 desktop checkout 預選，不會因 `.github/` 存在而選 copilot。
- 技能檔過期探測：tools 含 copilot 時逐工具資訊多一項 `tool: "copilot"`，判定規則與 codex 相同。
- `skills.render('apply', { target: 'copilot' })` 回傳與 CLI 生成的 `.github/skills/speclink-apply/SKILL.md` 相同的字串。

**介面與資料形狀**

- `.speclink.yaml`：`tools` 清單元素多一個合法字串 `copilot`；沒有新欄位、不改 serde 結構，既有檔案照讀。
- CLI：`speclink init --tools <list>` 的值域加 `copilot`；不新增旗標。
- Node SDK：`RenderOptions.target: 'claude' | 'codex' | 'copilot' | 'neutral'`。
- 過期探測 `--json`／IPC：工具名多 `copilot` 一個值，欄位不變。

**失敗模式**

- `--tools` 含未知名稱：非零結束，stderr 單行 `unknown tool: <name> (supported: claude, codex, copilot)`，零寫入。
- 非互動缺 `--tools`：非零結束、stdout 空、stderr 為新的 `TOOLS_HINT`，零寫入。
- 描述子名為 `copilot` 或 skills_dir 正規化後等於 `.github/skills`：非零結束、單行錯誤指明衝突，`.github/skills/` 下既有檔案逐字元不變。
- 舊版引擎讀到含 `copilot` 的 `.speclink.yaml`：沿既有行為記一行 unknown tool note 並略過，不刪 `.github/skills/`（舊引擎不認得它）。

**驗收**

- `cargo test -p speclink-core`：新 copilot golden、版號鎖定測試、init／config 單元測試（copilot 生成、取消勾選清理、footprint 不推斷、描述子保留名與保留目錄）全過。
- `cargo test -p speclink-cli --test it init_tools`：`--tools copilot`、`--tools claude,codex,copilot`、互動三題、三題皆 no 重問、新錯誤訊息全過。
- speclink-node 的 render 測試：target copilot 與 CLI 生成逐字相同、未知 target 錯誤訊息含 copilot。
- `cargo test -p speclink-desktop-core`：設定頁加選 copilot 生成 `.github/skills/`、取消勾選清掉。
- desktop vitest：四處勾選都出現 copilot。
- 文件守門：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全過（中英對等、連結、CLI help 一致）。
- [M] VS Code 同名技能實測。

**範圍**

- In scope：上述 speclink-core workspace 模組、speclink-cli init、speclink-node render、desktop 四處勾選與 checkout 錯誤訊息、golden 與 assets.lock、中英文件、本 repo 技能檔的版號再生。
- Out of scope：Goals / Non-Goals 列出的全部項目；codex 與 claude 的渲染內容；Copilot 雲端 agent 與 Copilot SDK。

## Risks / Trade-offs

- [golden 回歸] 版號遞增讓五份既有 golden 同時改變，容易掩蓋誤改 → 重生後逐份檢查 diff 只有 `version:` 一行；copilot golden 另做內容抽查（無 `$speclink-`、無 `agent:`、有 `--agent copilot`）。
- [版號對撞] 平行的變更若也遞增 `ASSET_VERSION`，合併時版號行與 assets.lock 會衝突 → 合併時以重生衍生物解決（重新遞增、重生 lock 與 golden），不挑邊。
- [CLI 測試] 互動詢問從兩題變三題，既有以「y、n」餵答案的測試會卡在第三題 → 同批改寫 init 的單元測試與 init_tools 整合測試的輸入序列。
- [跨平台] `.github/skills` 是正斜線邏輯路徑；Windows 上路徑比較與探測回報沿用既有 skills_dir 的正規化，不新增平台分支 → Windows CI 會跑同一組 init／probe 測試確認。
- [VS Code 先後未知] VS Code 可能在同名時選到 `.claude/skills` 的 Claude 版 → [M] 實測；若選錯，文件建議 Copilot 使用者不要同時勾 claude，或在 Copilot CLI 用 `ignoredSkillsLocations` 排除。
- [BREAKING 描述子] 少數專案可能已用名為 `copilot` 的自訂描述子指向 `.github/skills` → 錯誤訊息直接指出衝突；configuration 文件寫明遷移方式。
- [舊版混用] 團隊裡舊版 speclink 會把 `copilot` 當未知工具略過，不會生成或清理 Copilot 技能 → 可接受：只是少生成，不會損壞檔案；新版 update 會補齊。
- [儲存解耦] 本變更只動工作區生成（Local Repo 的技能檔），不碰規格儲存或流程抽象，與 storage 解耦的方向無關。
