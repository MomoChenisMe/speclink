## Context

工作流政策欄位 locale／spec_locale 的正典值是語系代碼（tw／ja／en，spec_locale 另有 auto），但整條寫入鏈沒有任何一層驗證值域：CLI set 動詞任意字串照單全收、共用改寫 seam 只驗 YAML 可解析、server 端點同樣只驗可解析。實際事故：/speclink-config 技能把「繁體中文」字面寫入 remote store，桌面下拉因對不上固定選項而靜默空白。tdd／audit 已有嚴格值驗證先例（policy_bool 連 yes／1 都拒收），locale 補上同等防線即是對齊既有慣例。

寫入路徑盤點（全部匯流於同一 seam）：

- CLI 本地：cmd_workflow_config → plan_workflow_config_edit → speclink-core 的 update_workflow_config_text
- CLI 遠端：同上改寫後以完整文件 PUT 至 server
- 桌面本地：write_workflow_config command → speclink-desktop-core settings 的 rewrite → 同一 update_workflow_config_text
- 桌面遠端：speclink-desktop-core settings 以同 seam 組新文字後經 server 端點寫入
- server 端點：speclink-server routes 的 policy 寫入，現行引擎 fail-closed 驗證僅涵蓋 WorkflowConfig::from_text 可解析性

## Goals / Non-Goals

**Goals**

- 官方寫入動詞與端點拒絕非法 locale 代碼，錯誤訊息列出合法集合
- server 成為最終防線（沿用 server-policy-write 既有原則：client 驗證僅為 UX）
- 桌面設定頁對未知儲存值顯性呈現，不再靜默空白
- config 技能明文規定寫入代碼而非顯示名稱

**Non-Goals**

- 不動讀取端寬容語意（from_text 與 locale_display 的 echo-back 不變）
- 不做既有髒資料自動遷移；不擴充語系集合；不動環境變數層

## Decisions

### D1：值域驗證落在 speclink-core 的 update_workflow_config_text

驗證函式歸 speclink-core（純領域規則，無 I/O 無 ANSI）：locale 合法集合 {tw, ja, en}、spec_locale 合法集合 {tw, ja, en, auto}，None（移除鍵）恆合法；非法值回 Err，訊息含欄位名、收到的值、合法集合。update_workflow_config_text 在組字前先驗 fields，使 CLI 本地／遠端與桌面本地／遠端四條路徑一次覆蓋。

- 替代一：各呼叫端（set_policy_field、桌面 settings）各自驗證——拒絕：四處重複、漏一處即破功，正是本次事故的形狀。
- 替代二：WorkflowConfig::from_text 讀取時嚴格驗證——拒絕：既有髒值專案升級後直接讀不了，且違反「讀取寬容」的既有設計（echo-back verbatim 是文件化行為）。
- CLI 端 set_policy_field 不另做前置驗證，錯誤自然從 seam 冒出（避免兩份訊息措辭漂移）；CLI 錯誤出口維持既有非零 exit code＋stderr 慣例。

### D2：server 端點擴充引擎 fail-closed 驗證至 locale 值域

routes 的 policy 寫入在既有 WorkflowConfig::from_text 可解析性檢查後，對解析出的 locale／spec_locale 呼叫同一 speclink-core 驗證函式，非法即以既有 invalid_config 錯誤家族拒絕且不落盤（訊息帶欄位與合法集合）。錯誤 envelope 沿用既有 status／reason／message 結構，不新增 reason 值——舊版 client 顯示 message 即可理解，無 wire 相容問題。

- 替代：只驗 client 端、信任官方 client——拒絕：server-policy-write 規格明定 client 驗證不得被信任為防線；未升級 CLI 或原始 HTTP client 仍能寫髒值。

### D3：合法集合凍結為 locale_display 的 frozen mapping

合法集合直接取自既有 frozen mapping（tw／ja／en）＋ spec_locale 的 auto，單一常數來源放 speclink-core，驗證與（未來若需要的）列舉共用。手動編輯 config.yaml 寫其他 BCP-47 代碼仍被讀取端原樣注入——逃生口保留，但官方寫入面一律受限。

- 替代：接受任意 BCP-47 代碼——拒絕：引擎無法為任意代碼渲染顯示名稱、桌面下拉是固定選項，接受了也是另一種「存得進顯示不出」。

### D4：桌面未知值顯性呈現

ProjectSettingsView 的兩個 Select 在儲存值非空且不在選項集時，動態插入一個帶警示樣式的 SelectItem（顯示原始值＋「無效值」標註），並於欄位下方顯示提示文字引導改選合法代碼；i18n 新增 zh-TW 與 en 字串。使用者改選合法代碼儲存即覆蓋髒值——這同時是既有髒資料的修復路徑。

- 替代一：維持空白——拒絕：本次事故的可見性根源。
- 替代二：開啟時自動清空或自動改寫——拒絕：讀取面靜默改資料，違反「寫入嚴格、讀取寬容」。

### D5：技能文件三處同步與 golden 再生

crates/speclink-core/assets/skills/config.md 的政策欄位段落加入：locale／spec_locale 只接受語系代碼、列出合法集合、把使用者自然語言回答映射為代碼（例：「繁體中文」→ tw）、禁止寫顯示名稱。`.claude/skills` 與 `.agents/skills` 的 speclink-config 實例同步同一內容；render golden 四份 snapshot 以 UPDATE_GOLDEN=1 再生。

- 約束（既知風險）：golden 再生必須在乾淨樹上進行——工作樹目前含未提交的其他變更時，須先完成該批提交再執行本變更的資產任務，否則未提交狀態會烙進 golden。

## Implementation Contract

- **驗證函式**：speclink-core config 模組新增 pub 驗證入口（fields 層級），簽名接受 WorkflowPolicyFields（或等價的兩個 Option<&str>），Err 訊息格式「欄位名＋收到值＋合法集合」；update_workflow_config_text 於組字前呼叫。
- **CLI 行為**：workflow-config set locale <非法值> → 非零 exit code、stderr 單行含合法集合、檔案逐位元不變；set locale tw／set locale ""（移除）行為不變；--dry-run 對非法值同樣拒絕（不印 diff）。
- **server 行為**：policy 寫入附非法 locale → 拒絕、store 內容與 revision 不變、錯誤 message 含欄位與合法集合；合法寫入行為不變。
- **桌面行為**：儲存值不在選項集 → 下拉顯示原始值與無效標註＋欄位提示文字；值在選項集 → 現行為不變；儲存合法代碼後 store 值更新且下拉正常。
- **技能內容**：渲染後的 speclink-config SKILL.md 含代碼映射指引；三處實例內容一致；golden 通過。
- **範圍邊界**：不動 WorkflowConfig::from_text、locale_display、環境變數解析、instructions 注入邏輯；不動 tdd／audit 的既有驗證。

## Risks / Mitigations

- **golden 乒乓**：dirty 樹再生會把未提交狀態烙進 snapshot（已發生過一次）——任務明定先確認 git status 乾淨（或僅含本變更 diff）再跑 UPDATE_GOLDEN=1，並審視 diff 僅含 config.md 段落。
- **CLI 輸出回歸**：set 動詞的成功輸出與既有錯誤格式不變，僅新增一種拒絕訊息；workflow_config.rs 整合測試釘住新訊息與檔案不變性。
- **server 相容**：不新增 reason 值、不改 envelope 結構，舊 client 無感；policy_write.rs 增測非法 locale 情境。
- **桌面測試環境**：projectSettingsView 測試在 jsdom 下驗證未知值渲染（Node 20 跑 desktop 測試的既有約束照舊）；Radix Select 在 jsdom 的互動限制以渲染斷言為主、不模擬下拉開合。
