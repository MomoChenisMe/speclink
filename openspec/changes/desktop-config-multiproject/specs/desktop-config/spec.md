## ADDED Requirements

### Requirement: 執行期切換專案 root

桌面 app SHALL 提供「開啟專案」操作：經原生資料夾選擇器選定目錄後，app SHALL 以該目錄為起點向上探索 speclink 專案根（與啟動時的探索語意一致），命中即於執行期切換專案 root——看板、已封存與設定頁 SHALL 全數改為呈現新專案內容，SHALL NOT 要求重啟；頂欄分頁列 SHALL 以 active 分頁標示目前專案（root 目錄名）。開啟失敗（路徑不存在或不可讀）時 app SHALL 顯示單行錯誤訊息並維持原專案不變。

#### Scenario: 選定既有專案後全視圖切換

- **WHEN** 使用者於專案 A 中經「開啟專案」選定含 openspec/ 的專案 B 根目錄
- **THEN** 看板顯示專案 B 的 change、分頁列新增專案 B 的分頁並標示為 active，過程中 app 未重啟，且專案 A 與 B 的檔案內容均未被此操作改動

#### Scenario: 自子目錄向上探索至專案根

- **WHEN** 使用者選定專案 B 根目錄下的子目錄（子目錄本身不含 openspec/）
- **THEN** app 切換至向上探索命中的專案 B 根目錄，行為與直接選定專案根一致

#### Scenario: 開啟失敗維持原專案

- **WHEN** 使用者嘗試開啟一個已被刪除或不可讀的路徑
- **THEN** app 顯示單行錯誤訊息，目前專案與畫面內容維持不變

### Requirement: 未初始化目錄經確認後自動初始化

所選目錄向上探索未命中任何 speclink 專案時，app SHALL NOT 逕行寫入，而 SHALL 顯示初始化確認對話框（含 AI 工具多選 claude／codex，預設勾選 claude）。使用者確認後 app SHALL 執行與 speclink init 等效的初始化（openspec/ 骨架含 specs/、changes/archive/ 與 config.yaml、專案根的 .speclink.yaml 記錄所選 tools、為每個所選工具生成指令檔 marker 區塊與 skills 檔），隨即切換至該專案；使用者取消時 app SHALL 維持原專案，且目標目錄 SHALL NOT 產生任何寫入。初始化失敗時 app SHALL 顯示單行錯誤訊息且 SHALL NOT 切換 root。

#### Scenario: 確認後初始化並切入新專案

- **WHEN** 使用者選定不含任何 speclink 標記的空目錄，於確認對話框保持預設（claude）並確認
- **THEN** 該目錄產生 openspec/（含 specs/、changes/archive/、config.yaml）、.speclink.yaml（tools 含 claude）、CLAUDE.md 的 SPECLINK marker 區塊與 .claude/skills/ 技能檔，且 app 切換至該專案並於看板顯示空清單

#### Scenario: 勾選 codex 時生成對應工具檔

- **WHEN** 使用者於確認對話框加勾 codex 後確認
- **THEN** 目標目錄除 claude 對應檔案外，另產生 AGENTS.md 的 SPECLINK marker 區塊與 .agents/skills/ 技能檔，.speclink.yaml 的 tools 同時記錄 claude 與 codex

#### Scenario: 取消初始化則零寫入

- **WHEN** 使用者於確認對話框取消
- **THEN** app 維持原專案，所選目錄內容與選擇前完全相同（無任何新檔案或目錄）

### Requirement: 專案分頁列存於 app 本機

app 頂欄 SHALL 以分頁列呈現開啟過的專案（路徑與顯示名，上限 10 個分頁）：分頁 SHALL 跨啟動持久化於 app 本機狀態（含順序與最後活躍分頁），SHALL NOT 寫入任何專案目錄。點擊分頁 SHALL 以該路徑執行與「開啟專案」相同的切換語意；同一專案再次開啟 SHALL 去重並移至既有分頁；關閉分頁 SHALL 將其自持久化清單移除。分頁 SHALL 顯示該專案進行中變更數的徽章——活躍分頁隨看板刷新即時更新，背景分頁 SHALL 於 app 啟動時各查詢一次、之後保留最後已知值。分頁指向已不存在的路徑時 SHALL 以錯誤態呈現，點擊 SHALL 顯示錯誤並提供自分頁移除，SHALL NOT 切換專案。無任何分頁時 app SHALL 顯示「開啟專案」空狀態引導頁而非空看板。app SHALL 支援 Ctrl+Tab 循環切換與 Ctrl+1..9 直達第 N 個分頁。

#### Scenario: 成功開啟後記入分頁並去重上移

- **WHEN** 使用者依序開啟專案 A、B，再次開啟專案 A
- **THEN** 分頁列僅含 A、B 各一個分頁且 A 為 active，A 與 B 的專案目錄內均無因分頁列而新增的檔案；重啟 app 後分頁列還原為相同內容

#### Scenario: 點擊分頁切換專案

- **WHEN** 使用者於專案 A 為 active 時點擊專案 B 的分頁（或按 Ctrl+Tab 循環至 B）
- **THEN** 看板、已封存與設定頁改為呈現專案 B 內容，B 的分頁轉為 active，行為與經「開啟專案」選定 B 一致

#### Scenario: 分頁徽章顯示進行中變更數

- **WHEN** 分頁列含專案 A（active，3 個進行中變更）與背景專案 B（啟動時查得 2 個進行中變更）
- **THEN** A 的分頁徽章顯示 3 並隨看板刷新更新，B 的分頁徽章顯示 2（最後已知值）；hover 徽章顯示「N 個進行中變更」說明

#### Scenario: 分頁路徑已消失時轉錯誤態

- **WHEN** 分頁列中專案 B 的目錄已被刪除，使用者點擊該分頁
- **THEN** 該分頁呈現錯誤態（警示標記），app 顯示單行錯誤訊息、維持原專案，並提供「自分頁移除」操作；執行後該分頁自分頁列與持久化清單消失

#### Scenario: 零分頁時顯示空狀態引導頁

- **WHEN** app 於無任何持久化分頁的狀態下啟動（如首次使用）
- **THEN** 主畫面顯示含「開啟專案」操作的空狀態引導頁（說明可選既有專案目錄或經確認初始化一般目錄），而非空白看板

### Requirement: 設定頁圖形化讀寫兩層設定

設定頁 SHALL 以結構化表單呈現並寫入兩層設定：.speclink.yaml 的 tools（內建工具 claude／codex 多選）與 openspec/config.yaml 的 locale、spec_locale（下拉）、tdd、audit（開關）。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（rules、context、remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 於表單原樣呈現為不可編輯項且寫入後保留。

#### Scenario: 寫入政策欄位且未觸及鍵原樣保留

- **WHEN** config.yaml 原含 rules 區塊與 context 文字，使用者於設定頁將 tdd 切為開啟並儲存
- **THEN** 重新讀取 config.yaml 可見 tdd: true，且 rules 與 context 內容與寫入前逐字元一致

#### Scenario: 設回預設值即移除鍵

- **WHEN** config.yaml 原含 locale: tw，使用者於設定頁將 locale 改回「未設定（English）」並儲存
- **THEN** 重新讀取 config.yaml 已無 locale 鍵，且引擎解析該檔的有效 locale 為預設 English

##### Example: 政策欄位寫入效果

| 操作前檔案狀態 | 表單操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| 無 tdd 鍵 | tdd 切開啟 | 新增 tdd: true |
| tdd: true | tdd 切關閉 | tdd 鍵被移除（預設即 false） |
| locale: tw、含 rules 區塊 | spec_locale 選 auto | 新增 spec_locale: auto，locale 與 rules 原樣保留 |

#### Scenario: tools 變更後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者加選 codex 並儲存
- **THEN** .speclink.yaml 的 tools 記錄 claude 與 codex，且專案根新增 AGENTS.md marker 區塊與 .agents/skills/ 技能檔

#### Scenario: 自訂工具描述子原樣保留

- **WHEN** .speclink.yaml 的 tools 含一個自訂描述子物件，使用者於設定頁變更內建工具勾選並儲存
- **THEN** 寫入後的 tools 清單仍含該描述子且欄位內容不變，設定頁將其呈現為不可編輯項

### Requirement: 設定寫入具解析驗證且失敗浮出

設定頁載入 SHALL 區分「檔案缺席或欄位未設定」與「檔案存在但解析失敗」：後者 SHALL 於設定頁顯示警告並停用該檔對應表單，app SHALL NOT 對解析失敗的檔案執行寫入。寫入流程 SHALL 於寫檔前驗證新內容可被對應設定解析器解析且目標欄位值正確，寫檔後 SHALL 回讀再次驗證；任一驗證失敗 SHALL 顯示指明檔案與階段的單行錯誤訊息，且磁碟上的檔案 SHALL 維持原內容——SHALL NOT 留下不可解析的設定檔。

#### Scenario: 解析失敗的檔案拒絕寫入

- **WHEN** 使用者手動將 config.yaml 改壞（YAML 語法錯誤）後開啟設定頁
- **THEN** 設定頁對該檔顯示解析失敗警告、對應表單停用，且儲存操作不可對該檔執行

#### Scenario: 寫入驗證失敗檔案不變

- **WHEN** 設定寫入流程於寫檔前驗證未通過
- **THEN** app 顯示指明檔案與失敗階段的單行錯誤訊息，磁碟上該檔內容與操作前逐字元一致

### Requirement: UI 介面語言支援 zh-TW 與 en

app 的 UI 介面語言 SHALL 支援 zh-TW 與 en：未設定偏好時 SHALL 跟隨系統語言（系統語言以 zh 開頭判為 zh-TW，其餘判為 en）；設定頁 SHALL 提供「跟隨系統／zh-TW／en」三選，切換 SHALL 即時對全介面生效並持久化於 app 本機。UI 語言偏好與 config.yaml 的 locale（AI artifacts 產出語言）SHALL 互不影響。兩語言字典的 key 集合 SHALL 相等；查無 key 時 SHALL 顯示 key 本身而非另一語言的字串。

#### Scenario: 未設定偏好時跟隨系統語言

- **WHEN** app 於 UI 語言偏好未設定的狀態下啟動
- **THEN** 系統語言以 zh 開頭時全介面呈現 zh-TW，否則呈現 en

##### Example: 系統語言判定

| 系統語言 | UI 語言 |
| -------- | ------- |
| zh-TW | zh-TW |
| zh-CN | zh-TW |
| en-US | en |
| ja-JP | en |

#### Scenario: 手動切換即時生效並持久化

- **WHEN** 使用者於設定頁將 UI 語言由 zh-TW 切為 en
- **THEN** 全介面（頂欄、側欄、看板、對話框）即時改為英文，重啟 app 後仍為英文，且 config.yaml 內容未被此操作改動

#### Scenario: UI 語言與 artifacts 產出語言互不影響

- **WHEN** config.yaml 設定 locale: tw，使用者將 UI 語言切為 en
- **THEN** UI 呈現英文，而 config.yaml 的 locale 仍為 tw（引擎產出 artifacts 的語言政策不受 UI 語言影響）
