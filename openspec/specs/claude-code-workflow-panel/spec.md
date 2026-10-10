# claude-code-workflow-panel Specification

## Purpose

讓 Claude Code 使用者在側邊面板查看變更、討論與未結品質工單，並將名稱填入輸入框。面板讀取既有 Speclink 查詢結果，不取代 CLI 的生命週期與遠端認證契約。

## Requirements

### Requirement: 面板開關與名稱填入

使用者點面板按鈕或送出 `/speclink-panel` 時，面板 SHALL 在開啟與關閉間切換；斜線指令沒有額外旗標或 stdin，也不是新增的 speclink CLI 子指令。開啟時 SHALL 取得看板資料。點擊變更、下一步、討論或工單名稱 SHALL 將名稱設為輸入框最前方指令的參數；沒有指令時 SHALL 接在現有文字後方，不自動執行。

此 UI SHALL 沿用 claude-code-skill-bar 的語言選擇：明選 en／zh-TW 優先，auto 對中文、chinese、zh 作弱偵測，其餘（含日文與未設定）選英文，不讀工作流 `tw`／`ja`／`en`。面板操作 SHALL 不建立或修改 proposal.md、delta spec、.openspec.yaml、任務 evidence 或 snapshots。本基準 SHALL 記錄既有呈現，不變更 CLI 人眼或 JSON 位元級輸出；面板沒有自己的 stdout／stderr、exit code、`--json` 或 `--no-color` 模式。

#### Scenario: 開關回應

- **WHEN** 繁中介面送出 `/speclink-panel`
- **THEN** 未開啟時開啟並回覆 `speclink 面板已開啟。`，已開啟時關閉並回覆 `speclink 面板已關閉。`

#### Scenario: 點擊名稱

- **WHEN** 輸入框是 `/speclink-apply old-one`，點擊 add-auth
- **THEN** 輸入框成為 `/speclink-apply add-auth`；輸入框原為 `look at` 時則成為 `look at add-auth`

Evidence: `integrations/claude-code/speclink-skills/hooks/register.tsx`、`integrations/claude-code/speclink-skills/hooks/skills.ts`、`integrations/claude-code/speclink-skills/hooks/text.ts`；`integrations/claude-code/speclink-skills/tests/register.test.tsx` 的面板開啟與名稱填入測試。關閉回應由已讀取程式確認。

### Requirement: 看板順序與變更詳情

面板 SHALL 依 plan 結果提供下一步、波次、階段及阻擋關係，任務數與 worktree 資料 SHALL 來自變更清單。變更 SHALL 分為提案中、進行中、已就緒；使用者 SHALL 能收合各區塊。展開變更時 SHALL 讀取該變更的文件完成狀態與任務；有 worktree 時 SHALL 從它的目錄讀取。任務 SHALL 依 Markdown 標題分組、區分完成與未完成、以「手動」標示 `[M]`，並隱藏行尾的任務 ID 註解。

#### Scenario: 進度與阻擋

- **WHEN** plan 指出 add-auth 在第一波進行中、refactor-store 在第二波提案中且被 add-auth 阻擋，清單回報 add-auth 完成 5／12 個任務
- **THEN** 面板下一步為 add-auth，呈現其進度 5／12，並顯示 refactor-store 等 add-auth

#### Scenario: 從 worktree 展開文件

- **WHEN** add-auth 的 worktree 為 `/repo/.worktrees/add-auth`，使用者展開該變更
- **THEN** 在該目錄讀取 show 與 status；呈現 `✓ 提案  ✓ 設計  ✓ 規格  ○ 任務`、任務分組與完成狀態，手動任務顯示 `[手動]`，不顯示 `speclink-task` 註解

#### Scenario: 收合區塊

- **WHEN** 使用者收合進行中區塊
- **THEN** add-auth 不再顯示於該區塊，提案中的 refactor-store 仍可見

Evidence: `integrations/claude-code/speclink-skills/hooks/board.ts`、`integrations/claude-code/speclink-skills/hooks/register.tsx`；`integrations/claude-code/speclink-skills/tests/register.test.tsx` 的面板資料、任務解析及 worktree 展開測試。

### Requirement: 討論分組與內文

討論分頁 SHALL 排除已轉出的討論，分開呈現討論中與其餘討論；討論中 SHALL 顯示輪數，已結論 SHALL 顯示已結論。展開討論 SHALL 顯示內文，移除開頭 YAML metadata 與 HTML 註解並統一換行。

#### Scenario: 排除已轉出討論

- **WHEN** 清單含 open 的 hold-auto-close、concluded 的 ship-it、promoted 的 old-idea
- **THEN** 分別顯示討論中與已結論各一筆，不顯示 old-idea；展開 hold-auto-close 時只顯示討論內文

#### Scenario: 移除文件附帶資訊

- **WHEN** 討論內容含開頭 YAML、CRLF 與 HTML 註解
- **THEN** 面板 Markdown 不顯示 YAML 與註解，內文使用一致換行

Evidence: `integrations/claude-code/speclink-skills/hooks/board.ts`、`integrations/claude-code/speclink-skills/hooks/register.tsx`；`integrations/claude-code/speclink-skills/tests/register.test.tsx` 的討論分頁與內文測試。

### Requirement: 品質分頁只列未結工單

品質分頁 SHALL 查詢變更的審查與驗證工單，只呈現至少有一份工單的變更。每份工單 SHALL 只使用最後一輪，顯示輪數及各嚴重度數量；展開時 SHALL 顯示發現的嚴重度、路徑與文字。查詢非零結束且 stderr 含 `no review ticket` 或 `no verify ticket` 時 SHALL 視為沒有工單，其餘失敗 SHALL 視為讀取錯誤。

#### Scenario: 最後一輪摘要與發現

- **WHEN** add-auth 審查工單最後一輪是第 2 輪，含一條 CRITICAL 與一條 WARNING，沒有驗證工單
- **THEN** 顯示 `審查  第 2 輪 · 1 CRITICAL 1 WARNING` 與驗證沒有工單；展開後可讀 `token 沒有檢查過期`，兩份工單都沒有的 refactor-store 不列出

#### Scenario: 缺席與錯誤分開

- **WHEN** 工單讀取非零結束，stderr 是 `Error: no verify ticket for change 'add-auth'`
- **THEN** 顯示沒有工單；`Error: Change not found` 則走讀取錯誤，不當成工單缺席

Evidence: `integrations/claude-code/speclink-skills/hooks/board.ts`、`integrations/claude-code/speclink-skills/hooks/register.tsx`；`integrations/claude-code/speclink-skills/tests/register.test.tsx` 的工單判定與品質分頁測試。

### Requirement: 唯讀查詢輸入與錯誤顯示

面板 SHALL 以以下既有 CLI 呼叫取得資料，全部附帶 `--json`，不提供 stdin。成功時 SHALL 從 stdout 讀 JSON；非零 exit code SHALL 從 stderr 取得錯誤，stderr 空白時顯示 exit code。Windows SHALL 經 `cmd.exe /d /c speclink`，其他平台 SHALL 直接執行 speclink。

| 呼叫 | 面板消費的 JSON 欄位與型別 |
| --- | --- |
| `speclink list --json` | `changes`: array；項目 `name`: string、`completedTasks`／`totalTasks`: number；選填 `worktree`: object，含 `path`／`branch`: string |
| `speclink plan --json` | `changes`: array；項目 `name`／`stage`: string、`wave`: number、`blockedBy`: string array；`next`: string 或 null；`skipped`: array，項目 `change`: string |
| `speclink discuss list --json` | `discussions`: array；項目 `slug`／`topic`／`status`: string、`rounds`: number |
| `speclink show <name> --json` | `tasks`: string |
| `speclink status --change <name> --json` | `artifacts`: array；項目 `id`／`status`: string |
| `speclink discuss show <slug> --json` | `content`: string |
| `speclink review show <name> --json`、`speclink verify show <name> --json` | `lastRound`: object；`index`: number、`findings`: array；項目 `severity`／`path`／`text`: string |

以上只列面板實際使用的欄位，不刪除原 CLI payload 的其他欄位。面板 SHALL 不讀人眼輸出或解析 ANSI 色彩；不傳 `--no-color`，亦不改動原指令的人眼輸出。重新整理、每輪完成與 Speclink Bash 呼叫完成時，已開啟面板 SHALL 更新資料；展開中的變更與討論 SHALL 重讀，品質分頁開啟時 SHALL 重讀工單。

看板讀取失敗 SHALL 保留上一份資料並顯示錯誤及 CLI PATH 提示；首次失敗沒有舊資料時 SHALL 顯示錯誤而非成功空清單。詳情讀取失敗 SHALL 在展開列顯示錯誤；品質讀取失敗 SHALL 保留上一份品質資料並顯示錯誤。遠端離線、認證失效，或查詢以 revision 衝突結束時 SHALL 沿用此非零結束處理；面板不另作認證、revision 寫入或本地回寫。

#### Scenario: 非零結束保留資料

- **WHEN** 已有看板資料後，重新整理的 CLI 查詢非零結束，stderr 為 `offline`
- **THEN** 舊看板資料保留，顯示包含該查詢與 `offline` 的錯誤及 CLI PATH 提示，不顯示成功的空清單

#### Scenario: 空白錯誤訊息

- **WHEN** 詳情查詢非零結束為 1，stderr 空白
- **THEN** 展開列顯示包含該查詢與 `exit 1` 的錯誤

#### Scenario: Windows 呼叫

- **WHEN** 在 Windows 取得變更清單
- **THEN** 呼叫參數為 `cmd.exe /d /c speclink list --json`；其他平台為 `speclink list --json`

Evidence: `integrations/claude-code/speclink-skills/hooks/register.tsx` 的查詢、重新整理與錯誤呈現；`integrations/claude-code/speclink-skills/hooks/board.ts` 的輸入欄位；`integrations/claude-code/speclink-skills/tests/register.test.tsx` 的 CLI 替身與 Windows 測試。錯誤保留與重新整理由已讀取程式確認，沒有宣稱已實跑遠端失敗情境。
