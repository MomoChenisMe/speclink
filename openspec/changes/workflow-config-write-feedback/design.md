## Context

workflow-config 的寫入子指令（set、context、rules、review）全部集中在 speclink-cli 的 workflow-config 動詞模組（crates/adapters/speclink-cli/src/verbs/config.rs），分本機臂 `workflow_config_fs` 與 remote 臂 `remote_workflow_config` 兩條路徑；兩臂共用同一個純文字改寫（`plan_workflow_config_edit`）與同一個差異產生函式（`unified_diff`）。

- 審查規則由審查技能在執行時以 `speclink workflow-config show --json` 讀取。技能檔是 `speclink update` 依引擎技能範本再生的產物；工作區技能檔若早於支援審查規則的版本，舊版審查技能不會讀 review 區塊。context、rules 與政策欄位則由 CLI 在 `speclink instructions` 注入，與技能檔新舊無關。
- 引擎已有唯讀的技能檔過期探測 `probe_assets`（speclink-core 的 workspace init 模組，workspace-tools 規格「技能檔過期探測」）：依 .speclink.yaml 的 tools 清單回報整體五態（缺失、過期、較新、現版、無法判定）與逐工具的 stale／newer／missing 方向。desktop 開專案時已用它顯示「指令檔過期提示」；CLI 目前沒有任何寫入路徑消費它。
- 現行 `unified_diff` 只去掉新舊文字的共同前綴與共同後綴，夾在中間的整段一律先列舊行（`-`）再列新行（`+`）。改動若有兩處（例如 rules 清單中改了第一條、在尾端加了三條），兩處之間沒改的條目也會成對出現在 `-` 與 `+`，使用者看不出哪幾行真的改了。
- remote 模式的 `RemoteCtx` 帶有解析 store 模式時找到的本機工作區（`ws`），remote 臂可以拿到本機根目錄。

## Goals / Non-Goals

**Goals:**

- 審查規則實際寫入成功、而本機技能檔過期時，讓使用者在同一次指令就看到「規則可能不會被套用、要跑 speclink update」。
- `--dry-run` 的差異只把真正刪除與新增的行標成 `-`／`+`，未改的行以 context 行呈現。
- 兩項都只動 CLI 呈現層；本機與 remote 兩臂共用同一份實作。

**Non-Goals:**

- 不在 context、rules、政策欄位的寫入加過期提示——它們由 CLI 注入，技能檔新舊不影響生效。
- 不在 `workflow-config show` 或其他指令加過期提示；不自動執行 `speclink update`。
- 不改 `probe_assets` 的判定、不改 workspace-tools 規格；不改 desktop（desktop 已有指令檔過期提示）。
- 不改 config 技能文字（屬討論 config-skill-rule-routing 的第 2 刀）；不升技能版號。
- 差異預覽不拆成多個 hunk、不引入外部 diff 套件。

## Decisions

### 過期提示只掛在 review 寫入、走 stderr

提示只在 `review <list> --stdin` 與 `review baseline <mode>` 實際寫入成功之後輸出，印到 stderr，格式沿用 CLI 既有的 stderr 警告前綴 `speclink: warning: `，單行、不帶 ANSI 色彩。stdout 維持 workflow-config 規格既有的「單行成功訊息」，exit code 維持 0。

- 為什麼只掛 review：只有審查規則要靠技能檔讀取；其餘寫入由 CLI 注入，提示對它們是誤導。
- 為什麼走 stderr：規格明定 stdout 為單行成功訊息，腳本可能逐行比對；提示屬於診斷資訊，放 stderr 不破壞 stdout 契約。
- 替代方案：印在 stdout 第二行——破壞「stdout 單行」契約，被否決；寫入前擋下——過期不代表規則錯，擋寫入會讓「先寫規則、再 update」的正當順序走不通，被否決。

### 以逐工具的「過期」方向判定，整體較新或無法判定時不提示

呼叫 `probe_assets(<工作區根目錄>)`：整體狀態為較新（Newer）或無法判定（Unknown）時不提示；其餘情況下，逐工具結果中 `stale` 為真的工具若至少一個，就提示並列出這些工具與其技能版號，以及探測回報的引擎現版號。

- 不看整體狀態而看逐工具：整體狀態的聚合優先序是「較新 > 缺失 > 過期」，一個工具缺失、另一個過期時整體為缺失，只看整體會漏掉過期的那個工具。
- 整體較新不提示：探測規格規定有任何檔案領先引擎時不提供會改寫它的動作，叫使用者 update 會降級。
- 無法判定不提示：無從得知技能檔版本，不猜。
- 缺失不算：工具技能檔整組不在，代表那個工具從未安裝，談不上「舊版技能不讀規則」；提示它只會在只宣告工具、沒裝技能的專案（例如 CLI 整合測試的臨時專案）造成雜訊。

### 兩臂共用一個提示函式，remote 臂探測本機工作區

新增一個供兩臂呼叫的函式（暫名 `warn_if_review_skills_stale`，參數為工作區根目錄與這次寫入），寫入意圖不是 review 時直接返回。本機臂在 `core::util::write_file` 成功並印出成功訊息後呼叫；remote 臂在 `put_config` 成功並印出成功訊息後，以 `RemoteCtx` 的 `ws.root` 呼叫。`--dry-run` 兩臂都在寫入前就返回，不會走到提示。

- 為什麼 remote 也探測本機：技能檔永遠在使用者本機的 checkout，remote 模式只把 config 存到 server。
- 直接呼叫 speclink-core 的 `probe_assets`：同一模組的 worktree 寫入已直接呼叫 `core::init::update` 同步技能，屬既有的 CLI 對 core 呼叫形態；探測是唯讀、零寫入。

### 差異改為「共同前後綴＋中段逐行比對」，仍為單一 hunk

保留現行的共同前綴、共同後綴裁切與前後各三行 context；對夾在中間的中段，以最長共同子序列（LCS）逐行比對，依序輸出：兩邊相同的行為 context 行（前綴一個空白），只在舊文字的行為 `-`，只在新文字的行為 `+`；同一位置同時有刪除與新增時，先列 `-` 再列 `+`。整段仍輸出成一個 hunk，hunk 標頭的起點與行數算法不變（中段的 context 行同時計入新舊兩側行數）。

- 為什麼單一 hunk：config.yaml 規模小，單一 hunk 加中段 context 行已是合法的 unified diff，可被 patch 套用；拆多個 hunk 只多出標頭計算的複雜度。
- 為什麼不引入套件：repo 目前沒有任何 diff 套件；中段逐行比對以動態規劃實作即可，config.yaml 的行數下記憶體與時間都可忽略。
- 只有單一連續改動時，中段沒有相同行，輸出與現行位元級相同。

## Implementation Contract

**Behavior**

- 本機工作區的 tools 清單中至少一個工具的技能檔判為過期、且探測整體不是較新或無法判定時：`speclink workflow-config review <critical|warning|suggestion> --stdin` 與 `speclink workflow-config review baseline <extend|replace>` 寫入成功後，stdout 仍為單行成功訊息、exit code 0，stderr 恰多一行 `speclink: warning: ` 開頭的提示，內容含每個過期工具的名稱與技能版號、引擎現版號、可能不套用審查規則的說明，以及 `speclink update`。
- 技能檔為現版、整組缺失、整體較新或無法判定時，stderr 不出現該提示。
- 帶 `--dry-run` 的 review 子指令、以及 set、context、rules 的任何寫入，都不出現該提示。
- remote 模式寫入成功後，以本機工作區的技能檔判定，條件與本機模式相同。
- 所有 workflow-config 寫入子指令的 `--dry-run`：stdout 仍以 `--- a/<label>`、`+++ b/<label>` 與一個 `@@ -<舊起點,行數> +<新起點,行數> @@` 標頭開頭；新舊文字都有的行不出現在 `-` 或 `+` 行，只以 context 行出現；無變更時輸出為空。

**Interface / data shape**

- 提示行格式（單行，無 ANSI）：

  ```text
  speclink: warning: skill files are older than this engine (<工具> <技能版號>[, <工具> <技能版號>…]; engine <引擎現版號>) — an older review skill may not apply these rules; run `speclink update`
  ```
- 差異函式維持 `unified_diff(label, old, new) -> String` 的呼叫形狀，兩臂呼叫點不變。

**Failure modes**

- 探測失敗或結果無法判定一律靜默不提示，不影響寫入結果與 exit code（探測規格本就要求零寫入、失敗不阻斷）。
- 提示只是診斷：寫入已完成，config 是正典，不回滾、不改 exit code。

**Acceptance criteria**

- crates/adapters/speclink-cli/tests/it/workflow_config.rs 新增整合測試：臨時專案放一份 frontmatter 版號舊於引擎的 claude 技能檔，執行 review 寫入 → stdout 單行成功、stderr 含提示與工具名；同一專案帶 `--dry-run` → 無提示；技能檔整組不在（既有臨時專案的預設）→ 無提示；技能檔版號新於引擎 → 無提示；set、rules 寫入在過期技能檔下 → 無提示。
- 同檔新增 dry-run 測試：rules 清單改第一條並在尾端新增一條，diff 中未改的中間條目只以 context 行出現；既有 set、rules、review 的 dry-run 測試維持通過。
- 差異函式的單元測試覆蓋：相同文字為空、單一連續改動與現行輸出相同、兩處改動夾未改行時中間行為 context、hunk 標頭行數與內容行數一致。
- 驗證指令：`cargo test -p speclink-cli --test it workflow_config` 與 speclink-cli 的單元測試；收尾跑 `node --test scripts/*.test.mjs scripts/*/*.test.mjs`（docs 中英對照與詞彙守門）。

**Scope boundaries**

- In scope：workflow-config 動詞模組的提示函式與差異函式、兩臂的呼叫點、對應的 CLI 整合與單元測試、docs/configuration.md 與 docs/configuration.zh-TW.md 的說明。
- Out of scope：speclink-core 的探測邏輯、desktop、server、技能範本與 ASSET_VERSION、golden 對照檔、其他動詞的輸出。

## Risks / Trade-offs

- [回歸對照] 既有 dry-run 測試若斷言了「夾在中間的未改行以 `-`／`+` 出現」，行為改變後會紅 → 實作時先跑既有 workflow_config 測試，凡因中段 context 化而失敗者逐一確認屬刻意變更並同批更新；只有單一連續改動的測試應保持不變。golden 對照檔不涵蓋 workflow-config 輸出，不受影響。
- [跨平台] Windows 上 config.yaml 可能是 CRLF：差異沿用現行以行切分的方式（`\r\n` 與 `\n` 都視為行尾），比對只看行內容，輸出一律 `\n`，與現行一致；提示行只走 stderr 的單行輸出，無路徑分隔差異。技能檔探測的路徑處理由 speclink-core 既有實作負責，跨平台行為不變。
- [誤報] 工作區技能檔較舊但已支援審查規則時（例如引擎再升版、技能檔停在支援審查規則之後的版本），提示仍會出現 → 文案寫「可能不套用」而非斷言，且執行 update 本就是正確動作。
- [效能] 中段逐行比對的成本與中段新舊行數乘積成正比；config.yaml 規模下可忽略，不另設上限或回退。
- [storage 解耦] 本變更不碰規格儲存或流程抽象；remote 模式的寫入路徑與版本衝突處理維持原樣，只在成功後多一次本機唯讀探測。
