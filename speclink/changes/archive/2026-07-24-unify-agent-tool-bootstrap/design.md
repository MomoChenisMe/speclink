## Context

目前 `speclink-cli` 在未提供 `--tools` 時呼叫 Core 的 footprint 偵測，沒有互動選擇；空白專案會默認 Claude。Desktop 的本機 init 已有 Claude／Codex checkbox，但 Remote checkout 的 `bind_checkout` 只驗證 Git／marker 並寫 remote section，沒有同步 `.speclink.yaml.tools`、Skills 或 `AGENTS.md`／`CLAUDE.md` 指令區塊。已有相符 marker 的路徑還會提早返回，因此舊 checkout 無法補齊。

這個變更橫跨 `speclink-core`、`speclink-cli`、Desktop core、Tauri IPC 與 React chooser。主要使用者是透過 Claude／Codex 執行 SDD 的開發者、PO 與 PM；限制是 Core 不承擔終端呈現、Remote checkout 不建立 `openspec/`、設定改寫必須保留既有 remote／spec_dir／未知欄位與自訂工具描述子，而且使用者文件內容不得被覆寫。

## Goals / Non-Goals

**Goals:**

- 讓所有 init 入口取得至少一個明示選擇的內建 Agent 工具，不再以 footprint 或 Claude 作為靜默 fallback。
- 讓 CLI filesystem／Remote Store init、Desktop 本機 init 與 Remote checkout 綁定共用 Core 的 built-in tools 同步與清理語意。
- 讓新舊 Remote checkout 在開啟 Workspace 前收斂到 `.speclink.yaml` 指定的 Claude／Codex 期望狀態。
- 保留自訂工具描述子、其他設定欄位、非 Speclink Skills 與指令區塊外的使用者內容。
- 以 TDD 覆蓋互動、非互動、設定轉換、Desktop 成功／失敗與安全重試。

**Non-Goals:**

- 不讓 CLI／Desktop picker 建立或編輯自訂工具描述子。
- 不修改 `speclink link`、`speclink update` 的介面、Remote handshake、capability 解鎖或規格遷移流程。
- 不新增 Server API、資料庫欄位、Credential 儲存或 Remote Protocol payload。
- 不啟動外部 CLI process，不建立新的 Host／Store abstraction，也不為跨檔寫入引入交易框架。

## Decisions

### Core 單一 Workspace 工具同步入口

在 `speclink-core` 的 init 模組建立可供 CLI 與 Desktop 共用的 built-in tool reconciliation（收斂）入口：先以 `update_app_config_tools_text` 將 Claude／Codex 選集寫入 `.speclink.yaml`，再沿用既有 `update` 的 generate／prune 行為同步 Skills 與指令區塊。filesystem init 與 Remote init 仍各自負責是否建立規格樹及 remote section，但都把已解析的工具選集交給相同入口。

替代方案是讓 CLI、Desktop settings 與 checkout 各自組合 config 寫入及 `update`；這會維持現有重複邏輯與不同失敗語意，因此拒絕。另一替代是把 init 放進 Host command runtime；init 是本機 Workspace bootstrap，且不讀寫抽象 Store 文件，為它新增 Host command 只會形成空轉接層，暫不採用。此決策維持規格儲存與本機 Agent 設定解耦：Core 只處理 Workspace 產物，Remote 文件仍由 Store 路徑管理。

### CLI 互動解析停留在 speclink-cli

`InitArgs.tools` 維持 `Option<String>`。`cmd_init` 在 filesystem／remote 分流之前統一解析工具：有 `--tools` 就驗證逗號分隔的 claude／codex；沒有旗標且 stdin 是互動終端時，透過可測試的讀寫 helper 依序詢問 Claude 與 Codex，至少選一項；沒有旗標且 stdin 非終端時，回傳單行錯誤且不呼叫任何 Core 寫入。

提示寫到 stderr，成功摘要仍由既有 stdout 路徑輸出；不讀取 redirected／piped stdin 當作選項，也不新增 JSON payload。先以標準函式庫的行輸入與 `IsTerminal` 實作，避免為兩個布林選項引入 raw-mode 多選依賴；輸入無效或兩者皆否時在互動終端重新提示。

替代方案是加入 `dialoguer`／`inquire` 類依賴提供方向鍵 checkbox；視覺較完整，但增加跨平台 terminal 行為與依賴面，兩個選項不值得。保留 footprint 自動偵測則違反明示選擇及非互動可重現性，因此拒絕。

### Desktop checkout 採先檢查、後同步的兩階段 IPC

Tauri connection adapter 新增唯讀 `inspect_checkout`：驗證資料夾、Git 與既有 marker 是否符合所選 origin／project／repo，並回傳 `{ root, tools }`；`tools` 僅含現有 `.speclink.yaml` 的 claude／codex，缺少 tools 清單時只依實際 Claude／Codex footprint 預選，不再補 Claude fallback。現有 `bind_checkout` 改為接收非空 `tools: string[]`，重做相同邊界驗證後，寫入 remote binding 並執行 Core reconciliation，成功才回傳 root。

Workspace chooser 在使用者選資料夾後先顯示檢查結果與 Claude／Codex checkbox；「開啟 Workspace」在至少一項被選取且同步未進行時才可按。既有 marker 但缺少工具選集的直接開啟路徑 SHALL 導回同一 chooser checkout 步驟；已有非空工具選集的持久化分頁可依記錄狀態自動 reconciliation，成功後才 handshake。

替代方案是選取資料夾時立即寫 marker，再另開工具設定；這會在使用者尚未確認工具前改動磁碟。只擴充單一 `bind_checkout` 也無法在寫入前取得現有選集，因此採用有實質讀寫分工的兩階段 IPC，而非多加薄 wrapper。

### Built-in 選擇收斂且保留自訂描述子

Claude／Codex checkbox 是內建工具的完整期望狀態：被選取者生成或更新 Speclink Skills 與對應指令區塊；未選取者移除 `speclink-*` Skills 與 `SPECLINK:START..END` 區塊，區塊外仍有內容時保留檔案，全空才刪檔。`.speclink.yaml` 中的 custom descriptor、unknown tool entry、remote、spec_dir 與未知頂層欄位原樣保留；picker 不顯示也不刪除自訂描述子。

替代方案是整份覆寫 tools 清單；這會刪除 picker 無法呈現的自訂描述子。只追加不 prune 則讓 config 與可載入 Skills 分歧。兩者均拒絕。serde 仍讀取現有 `ToolEntry` union，沒有 schema 或預設值變更；文字改寫沿用 raw mapping，維持向後可讀，接受既有的註解於重序列化時不保留之既知限制。

### 失敗不開啟 Workspace並以可重試收斂取代跨檔回滾

所有可以在寫入前完成的驗證（非空工具選集、未知工具、壞 YAML、marker 不一致、非 Git）先完成。Desktop sync 任一步驟失敗時 chooser 保持開啟、顯示單行且帶階段的錯誤，不建立 tab／checkout locator、不進行 handshake；再次提交相同期望狀態會以 reconciliation 補齊或清理 Speclink 受管產物。

不嘗試回滾跨越 `.speclink.yaml`、指令檔與多個 Skills 目錄的部分寫入，因為回滾可能覆蓋同步期間由使用者修改的內容；相反地，所有受管寫入保持可重複，且 remote section／tools 已存在時仍可再次同步。CLI 非互動缺少 `--tools` 是更強的零寫入前置失敗；其他既有檔案系統錯誤延續單行錯誤與非零 exit code。

替代方案是建立暫存 Workspace 後整批 rename；跨平台目錄置換、使用者檔案合併與 Windows rename 限制會大幅增加風險，不符合本次範圍。

## Implementation Contract

### CLI observable behavior

- filesystem 與 Remote Store 的 init 共用同一工具解析入口。
- 顯式 `--tools claude`、`--tools codex`、`--tools claude,codex` 成功；重複名稱去重。空字串、未知名稱或解析後零工具以非零 exit code 結束，任何 init 檔案均不存在或維持原文。
- 未傳 `--tools` 且 stdin 為互動終端時，stderr 逐一詢問 Claude／Codex；至少一項為 yes 才進入 init。成功後 stdout 的 Initialized 與 Generated files 摘要沿用既有格式。
- 未傳 `--tools` 且 stdin 非互動終端時，stderr 輸出單行訊息，須包含 `--tools` 及 claude／codex 可用值；exit code 非零、stdout 為空、`.speclink.yaml`／`openspec/`／Skills／指令檔／`.gitignore` 均不建立。
- 這是相對 Spectra 2.3.1 與目前自動偵測行為的刻意分歧；有顯式 `--tools` 的既有成功輸出、`--no-color` 行為及所有 `--json` shape 維持位元級基線。init 不新增 stdin payload 或 `--json`。

### Core and configuration contract

- 共用 reconciliation 接受 project root、非空且已驗證的 built-in `Tool` 選集，以及由 config remote section 決定的 `StoreKind`。
- `.speclink.yaml.tools` 中 claude／codex 的最終集合與請求相等；自訂描述子、未知 entries、remote、spec_dir 與其他鍵保持可解析且值不變。
- filesystem init 建立既有 `openspec/` tree；Remote init／Desktop Remote checkout SHALL NOT 建立 `openspec/`，且 Remote 指令區塊使用既有 remote wording。
- Claude 對應 `.claude/skills/speclink-*` 與 `CLAUDE.md`，Codex 對應 `.agents/skills/speclink-*` 與 `AGENTS.md`。選取時補齊／更新，未選取時只 prune Speclink 受管內容。

### Desktop IPC and UI contract

- `inspect_checkout(path, origin, project, repo)` 為零寫入，成功回傳 camelCase JSON `{ root: string, tools: string[] }`；marker 不一致、非 Git 或壞 YAML時 rejected Promise 的單行錯誤沿用既有繁中原因。
- `bind_checkout(path, origin, project, repo, tools)` 接受至少一個 claude／codex；未知、空選集或檢查失敗時 rejected Promise，成功回傳 checkout root。IPC payload 不含 credential、token 或 Server 新資料。
- Checkout 畫面在 folder mode 顯示 Claude／Codex checkbox；Open 在 path 與非空選集齊備前 disabled。同步期間維持 busy；失敗後畫面、path 與選集留存供重試，tab 不建立。
- 既有 marker 及既有 tab 的開啟不得繞過 reconciliation：有工具選集可自動收斂；缺少選集則回到可見選擇。

### Acceptance criteria and scope boundaries

- Core 測試證明 built-in 切換、custom descriptor／unknown key／remote section 保留、Speclink marker 外內容保留、prune 空檔與 remote mode 不建 `openspec/`。
- CLI 測試證明顯式三種選集、空／未知輸入、非互動零寫入、互動 helper 的單選／雙選／全否重試，以及 filesystem／remote 共用行為；現有 parity／color／golden 測試仍通過或只在已記載刻意分歧 fixture 更新。
- Desktop Rust 測試證明 inspect 零寫入、marker 驗證、既有 config 補齊／切換、同步失敗不回傳成功；React 測試證明 checkbox、disabled、busy、錯誤留存與成功後才呼叫 openRemote。
- 手動驗收以新 Git checkout、已有相符 marker 但缺 Skills、Claude 切換 Codex三條流程檢查實際檔案與 Workspace 開啟順序。
- 本次範圍只涵蓋 built-in Claude／Codex 的 init 與 Desktop Remote checkout bootstrap；自訂描述子 UI、`link`／`update` 介面、Server／Protocol、Remote 規格內容與 capability 解鎖均不變。

## Risks / Trade-offs

- [Breaking：既有 CI 省略 --tools] → 錯誤訊息提供三種顯式值；文件與測試 fixture 同步改為傳入 --tools。
- [部分檔案已寫入後同步失敗] → 先做所有可預檢查、tab 不開啟、受管操作保持冪等並以相同選集重試收斂，不回滾使用者檔案。
- [自訂描述子被 built-in picker 誤刪] → 重用 raw mapping 的 built-in-only 更新函式並加 descriptor 保留測試。
- [互動判斷或換行跨平台差異] → 使用標準 `IsTerminal` 與 line input，提示斷言正規化 CRLF；不依賴 raw terminal mode。
- [Spectra parity 回歸] → 將無 --tools 視為明載刻意分歧，只更新相關 fixture；有 --tools 的人眼輸出、no-color 與 golden 保持不變。
- [直接開啟既有 marker 繞過 chooser] → 在 remote binding／tab recovery 入口加入 reconciliation gate，測試每個入口在成功前都不建立 session。

## Migration Plan

- 更新 repository 內所有非互動 init 呼叫與測試 fixture，顯式加入 `--tools`。
- 發佈說明標示 breaking 行為，提供 claude、codex、claude,codex 三種遷移值。
- 既有 `.speclink.yaml` 無需 schema migration；Desktop 首次重新連接時依現有 built-in tools 或 footprint 預選並收斂。
- 若需回滾程式版本，已生成的 tools 清單、Skills 與 marker 均為舊版本可讀格式；回滾不需資料轉換。

## Open Questions

無。
