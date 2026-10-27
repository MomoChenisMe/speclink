## ADDED Requirements

### Requirement: 產出流程頁

專案設定 SHALL 提供「產出流程」分類頁（分類標籤與頁標題皆為「產出流程」；原頁籤標籤直出「Schema」的例外隨頁籤消失而退場）。頁內第一張卡 SHALL 為「產出政策」：locale 與 spec_locale 兩列為列右側下拉、tdd／audit／worktree 三列為列右側開關、底部為儲存與訊息列（寫入語意見需求「設定頁圖形化讀寫兩層設定」「產出政策的 worktree 開關」）。第二張卡 SHALL 為「產出流程」：列出每個可解析的 schema（顯示名稱、來源層級、artifact 圖），並可點入唯讀詳情——含每個 artifact 的 description、instruction 與 template 全文。詳情 SHALL 為唯讀；內建 schema 的內容不可在 desktop 編輯。清單資料 SHALL 由 desktop core 以引擎的解析函式在本地組裝，不經 server 端點。專案說明頁與產出規則頁 SHALL NOT 含產出流程內容。

#### Scenario: 產出流程自成一頁

- **WHEN** 使用者開啟專案設定（local 模式）並點分類欄「產出流程」
- **THEN** 頁標題為「產出流程」，頁內依序為產出政策卡（兩列下拉、三列開關）與產出流程卡（schema 清單）；專案說明頁與產出規則頁內無此內容

#### Scenario: 清單列出可解析的 schema

- **WHEN** 使用者開啟產出流程頁（local 模式，專案有一個自訂 schema）
- **THEN** 產出流程卡顯示內建 spec-driven 與該自訂 schema，各自帶來源層級與 artifact 圖

#### Scenario: 詳情唯讀呈現內容

- **WHEN** 使用者點入 spec-driven 的詳情
- **THEN** 顯示四個 artifact 各自的 description、instruction 與 template 全文，無任何編輯入口

##### Example: 清單一列的形狀

| 欄位 | 值（內建為例） |
|------|----------------|
| 名稱 | spec-driven |
| 來源層級 | 內建 |
| artifact 圖 | proposal → design → specs → tasks |

### Requirement: 外觀偏好

應用程式設定的「一般」頁與「語言與外觀」頁 SHALL 提供「外觀」列三選：跟隨系統、淺色、深色。偏好 SHALL 存於 app 本機（與 UI 語言偏好同層），SHALL NOT 寫入任何專案檔案；切換 SHALL 即時生效於主視窗與系統匣面板，重啟後維持。跟隨系統時 SHALL 依系統的淺色／深色偏好；明示淺色時即使系統為深色 SHALL 呈現淺色，明示深色反之。淺色與深色的全部色值 SHALL 取自共用 theme 的同一組 token（深色 token 對系統偏好與明示深色兩種觸發 SHALL 相同）。server-web SHALL NOT 受此偏好影響。

#### Scenario: 明示深色即時生效並持久化

- **WHEN** 系統為淺色，使用者於一般頁把外觀改為「深色」
- **THEN** 主視窗與系統匣面板立即改為深色 token；重啟 app 後仍為深色；config.yaml 與 .speclink.yaml 未被改動

#### Scenario: 跟隨系統

- **WHEN** 外觀為「跟隨系統」且使用者把系統外觀自淺色切為深色
- **THEN** 主視窗與系統匣面板隨之改為深色，無需重啟

#### Scenario: 明示淺色壓過系統深色

- **WHEN** 系統為深色，使用者把外觀改為「淺色」
- **THEN** 主視窗與系統匣面板呈現淺色 token

##### Example: 有效主題判定

| 偏好 | 系統 | 有效主題 |
| ---- | ---- | -------- |
| 跟隨系統 | 淺色 | 淺色 |
| 跟隨系統 | 深色 | 深色 |
| 淺色 | 深色 | 淺色 |
| 深色 | 淺色 | 深色 |

## MODIFIED Requirements

### Requirement: 設定頁圖形化讀寫兩層設定

<!-- REMOVED-SCENARIO: 兩頁分工與預設簽 -->
<!-- REMOVED-SCENARIO: 遠端分頁的 Workflow 簽編輯 -->
<!-- REMOVED-SCENARIO: 解析失敗簽級警示 -->

設定 SHALL 拆分為兩種模式：**專案設定**（跟隨 active 專案，自專案欄底部「專案設定」進入）與**應用程式設定**（與任何專案無關，自圖示列齒輪進入）。兩種模式 SHALL 以左側分類欄取代頁簽：點分類即切換主區為該分類的單一頁，分類欄 SHALL 記住上次所在分類；每頁 SHALL 以共用設定零件排版——24px 頁標題＋灰字說明＋右側頁級動作、14px 小標＋16px 圓角卡、卡內一列一筆細線分隔（左標籤與說明、右控制項）、列右側下拉。

專案設定的分類欄 SHALL 分兩組：「openspec/config.yaml」組依序為專案說明、產出規則、產出流程；「專案」組為 .speclink.yaml；預設落在專案說明。

- **專案說明**頁 SHALL 含專案說明卡、**產出規則**頁 SHALL 含產出規則卡（行為見需求「設定頁編輯專案說明與產出規則」）。
- **產出流程**頁 SHALL 含「產出政策」卡——locale、spec_locale（列右側下拉）與 tdd、audit、worktree（列右側開關）——與「產出流程」卡（行為見需求「產出流程頁」）。
- **.speclink.yaml** 頁 SHALL 含「檔案」列（等寬路徑，唯讀）與「AI 工具」卡——內建工具 claude／codex／copilot 每個一列附開關，自訂工具描述子原樣呈現為不可編輯列。

應用程式設定的分類欄 SHALL 為「應用程式」組：一般、語言與外觀、更新、CLI、伺服器；預設落在一般；零專案時 SHALL 照常可進入且分類欄照常呈現。

- **一般**頁 SHALL 含「介面語言」卡（顯示語言列、外觀列；行為見需求「UI 介面語言支援 zh-TW 與 en」「外觀偏好」）、「軟體更新」卡（目前版本、檢查更新、更新日誌三列）、「CLI 指令」卡（一列：狀態與安裝動作）與「鍵盤快捷鍵」卡，並 SHALL 以頁標題下的註記說明其內容僅存於此裝置、不寫入版本庫。
- **語言與外觀**頁 SHALL 含顯示語言列與外觀列。
- **更新**頁行為見 desktop-app 需求「桌面自動更新」「更新日誌彈窗」。
- **CLI**頁行為見 desktop-app 需求「安裝 CLI 指令到 PATH」。
- **伺服器**頁行為見 desktop-connections 需求「伺服器管理最小面」。

專案設定每頁的頁標題下 SHALL 以等寬字註記對應檔案路徑與鍵（如 `openspec/config.yaml · context`）。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，分類欄中掛該檔的分類項（config.yaml 掛專案說明、產出規則、產出流程；.speclink.yaml 掛 .speclink.yaml）SHALL 帶警示點、頁內 SHALL 浮出解析失敗說明且該頁表單 SHALL 停用；應用程式設定 SHALL NOT 受任何專案設定檔解析失敗影響。

遠端 workspace 為 active 專案時，專案設定的分類欄 SHALL 只有「openspec/config.yaml」組的三項（無 .speclink.yaml——tools 屬本機 checkout 概念），三頁內容來自 server 的 policy 文件，頁標題下的註記 SHALL 以等寬字顯示 policy revision；鍵保留語意與本地一致（未觸及鍵原樣保留、設回預設移除鍵）。儲存 SHALL 帶 expected revision；收到 revision 衝突時 SHALL 原樣保留使用者輸入並於頁頂浮出逐欄位對照（server 現值｜我的輸入），僅提供「以 server 版重載」與「檢視後以最新 revision 重新提交」兩出口——SHALL NOT 提供未經對照的強制覆寫。policy 寫入權為假（reader）時三頁 SHALL 唯讀、儲存停用附繁中角色說明。應用程式設定不受 active 專案種類影響。

#### Scenario: 兩種設定模式與預設分類

- **WHEN** 使用者於 local 專案分別點專案欄「專案設定」與圖示列齒輪
- **THEN** 專案設定的分類欄為「openspec/config.yaml」組（專案說明、產出規則、產出流程）與「專案」組（.speclink.yaml），預設在專案說明頁（頁標題下等寬註記 `openspec/config.yaml · context`）；切到產出流程頁見產出政策卡與產出流程卡、切到 .speclink.yaml 頁見檔案列與 AI 工具卡；應用程式設定的分類欄為「應用程式」組（一般、語言與外觀、更新、CLI、伺服器），預設在一般頁（含介面語言、軟體更新、CLI 指令、鍵盤快捷鍵四卡與「僅存於此裝置」註記）

#### Scenario: 遠端專案的專案設定分類

- **WHEN** active 專案為遠端 workspace 且使用者具 policy 寫入權，開啟專案設定修改產出流程頁的產出政策並儲存
- **THEN** 分類欄只有專案說明、產出規則、產出流程三項（頁標題下等寬顯示 policy revision），儲存成功後 revision 前進，server 端 config 反映新政策且未觸及鍵原樣保留

#### Scenario: 遠端儲存遇 revision 衝突

- **WHEN** 兩個 client 同時編輯同一 scope 的 policy，後儲存者收到 revision 衝突
- **THEN** 後儲存者的輸入原樣保留，頁頂浮出 server 現值與我的輸入的逐欄位對照，可選以 server 版重載或以最新 revision 重新提交；無任何未經對照的強制覆寫路徑

#### Scenario: reader 唯讀

- **WHEN** role 為 reader 的使用者於遠端專案開啟專案設定
- **THEN** 三頁呈現現值但唯讀，儲存停用並附角色說明

#### Scenario: 寫入政策欄位且未觸及鍵原樣保留

- **WHEN** config.yaml 原含 rules 區塊與 context 文字，使用者於產出流程頁將 tdd 開關切為開啟並儲存
- **THEN** 重新讀取 config.yaml 可見 tdd: true，且 rules 與 context 內容與寫入前逐字元一致

#### Scenario: 設回預設值即移除鍵

- **WHEN** config.yaml 原含 locale: tw，使用者於產出流程頁將 locale 改回「未設定（English）」並儲存
- **THEN** 重新讀取 config.yaml 已無 locale 鍵，且引擎解析該檔的有效 locale 為預設 English

##### Example: 政策欄位寫入效果

| 操作前檔案狀態 | 表單操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| 無 tdd 鍵 | tdd 切開啟 | 新增 tdd: true |
| tdd: true | tdd 切關閉 | tdd 鍵被移除（預設即 false） |
| locale: tw、含 rules 區塊 | spec_locale 選 auto | 新增 spec_locale: auto，locale 與 rules 原樣保留 |

#### Scenario: tools 變更後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者於 .speclink.yaml 頁開啟 codex 的開關並儲存
- **THEN** .speclink.yaml 的 tools 記錄 claude 與 codex，且專案根新增 .agents/skills/ 技能檔而無 AGENTS.md

#### Scenario: 加選與取消 copilot 後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者開啟 copilot 並儲存，之後再關閉 copilot 並儲存
- **THEN** 第一次儲存後 tools 記錄 claude 與 copilot、專案根新增 .github/skills/ 技能檔；第二次儲存後 tools 僅含 claude，.github/skills/ 下 speclink- 前綴目錄全數移除，.github/ 下其他檔案位元級不變

#### Scenario: 自訂工具描述子原樣保留

- **WHEN** .speclink.yaml 的 tools 含一個自訂描述子物件，使用者於 .speclink.yaml 頁變更內建工具開關並儲存
- **THEN** 寫入後的 tools 清單仍含該描述子且欄位內容不變，頁內將其呈現為不可編輯列

#### Scenario: 解析失敗分類級警示

- **WHEN** config.yaml 被外部改壞為無法解析，使用者開啟專案設定
- **THEN** 分類欄的專案說明、產出規則、產出流程三項帶警示點；進入任一頁可見解析失敗說明，產出政策卡表單與專案說明、產出規則兩頁的編輯鈕停用；應用程式設定的顯示語言列仍可正常使用

### Requirement: 設定頁編輯專案說明與產出規則

<!-- REMOVED-SCENARIO: 唯讀優先與各卡就地編輯 -->
<!-- REMOVED-SCENARIO: 各卡儲存僅寫對應鍵 -->

專案設定的「專案說明」頁與「產出規則」頁各 SHALL 呈現一張卡，各頁預設唯讀、各持編輯態——一頁進入編輯 SHALL NOT 影響另一頁的唯讀呈現與可用性。專案說明唯讀 SHALL 以 markdown 渲染呈現 context 內容，超過固定高度 SHALL 收合並提供顯示更多展開；未設定時 SHALL 顯示空狀態提示。產出規則唯讀 SHALL 僅列出有條目的 artifact 鍵，鍵名為小節標題、條目為清單。各頁頁標題右側的編輯鈕 SHALL 切換該頁為編輯態並將按鈕列改為取消與儲存：專案說明為 raw markdown 多行文字區；產出規則 SHALL 以活躍 schema 的 artifact id 為固定鍵各渲染一個多行文字區，SHALL NOT 提供自由鍵輸入；一行即一條規則，行序 SHALL 即為寫入檔案的條目順序（亦即指令注入順序）。儲存時 SHALL 逐行去除頭尾空白，空行 SHALL NOT 寫入；專案說明清空儲存 SHALL 移除 context 鍵，某鍵文字區清空 SHALL 移除該 artifact 鍵，全部鍵皆空 SHALL 移除 rules 鍵。專案說明頁儲存 SHALL 僅寫入 context、產出規則頁儲存 SHALL 僅寫入 rules——各頁儲存 SHALL NOT 改動另一頁對應的鍵。取消 SHALL 還原該頁唯讀呈現且 SHALL NOT 寫入檔案。寫入 SHALL 經與政策欄位相同的雙重解析驗證流程；序列化 SHALL 為以 YAML 保留字元（反引號、at 符號等）開頭的條目自動加引號——寫出檔案 SHALL 可被引擎解析且條目值逐字元還原。config.yaml 解析失敗時，兩頁 SHALL 依分類的解析失敗呈現停用編輯（見需求「設定頁圖形化讀寫兩層設定」）。

#### Scenario: 唯讀優先與各頁獨立編輯

- **WHEN** 使用者開啟專案設定的專案說明頁
- **THEN** 專案說明以 markdown 渲染唯讀（超長時收合並可顯示更多）；點頁標題右側的編輯後該頁變為可編輯（按鈕列為取消與儲存）；切到產出規則頁，該頁維持唯讀、只列有條目的鍵，且仍可獨立進入自身的編輯態

#### Scenario: 各頁儲存僅寫對應鍵

- **WHEN** config.yaml 原含 context 與 rules，使用者僅於產出規則頁進入編輯、修改後儲存
- **THEN** 重新讀取 config.yaml 的 rules 依編輯更新，而 context 值與寫入前逐字元一致

#### Scenario: 編輯專案說明並儲存

- **WHEN** 使用者於專案說明頁進入編輯，於文字區輸入多行 markdown 並儲存
- **THEN** 重新讀取 config.yaml 解析出的 context 值與輸入逐字元一致、其餘鍵原樣保留，該頁回唯讀並渲染新內容，重開專案設定呈現同一文字

#### Scenario: 以保留字元開頭的規則條目寫入後仍可解析

- **WHEN** 使用者於產出規則頁某鍵文字區新增一行以 YAML 保留字元開頭的規則並儲存
- **THEN** 寫出的 config.yaml 可被引擎解析（必要引號由寫入自動加上）、該條目值逐字元還原，整份工作流政策未退回預設

##### Example: 保留字元條目自動加引號

- **GIVEN** rules 原含 proposal 節一條「提案必須列出影響的 crates」
- **WHEN** 於 tasks 文字區新增一行「@完成後執行全部測試」並儲存
- **THEN** 重讀 config.yaml 可解析，rules 的 tasks 節含值「@完成後執行全部測試」（逐字元一致），proposal 節與 schema 等其餘鍵原樣保留

#### Scenario: 清空即移除鍵

- **WHEN** 使用者於專案說明頁清空文字區全部行並儲存，再於產出規則頁清空某鍵文字區並儲存
- **THEN** 重新讀取 config.yaml 已無 context 鍵與該 artifact 鍵；其餘鍵原樣保留

##### Example: 鍵移除語意

| 操作前檔案狀態 | 編輯操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| context: 舊說明、rules 含 tasks 兩行 | 專案說明頁清空並儲存 | context 鍵被移除，rules.tasks 原樣保留 |
| rules 含 proposal 與 tasks 兩節 | 產出規則頁清空 tasks 文字區並儲存 | rules 僅餘 proposal 節 |
| rules 僅含 tasks 一節 | 產出規則頁清空該文字區並儲存 | rules 鍵整個被移除 |

#### Scenario: 行序即寫入順序

- **WHEN** 使用者於產出規則頁某鍵文字區將第二行搬至第一行並儲存
- **THEN** 重新讀取 config.yaml 該節條目順序對調，後續該 artifact 的指令注入依新順序呈現規則

##### Example: 行對調

- **GIVEN** tasks 節依序含條目「先寫失敗測試」「更新文件」
- **WHEN** 於 tasks 文字區將「更新文件」一行搬到「先寫失敗測試」之前並儲存
- **THEN** config.yaml 的 tasks 節依序為「更新文件」「先寫失敗測試」

#### Scenario: 固定鍵分節不可自由輸入

- **WHEN** 使用者於使用 spec-driven schema 的專案進入產出規則頁的編輯
- **THEN** 文字區恰為 proposal、design、specs、tasks 四個固定鍵各一，介面不提供自由新增分節鍵的輸入；回唯讀後僅有條目的鍵被列出

#### Scenario: 取消放棄編輯

- **WHEN** 使用者於專案說明頁進入編輯、修改內容後點取消
- **THEN** 該頁還原為編輯前的唯讀呈現，config.yaml 逐字元未變，且產出規則頁全程不受影響

#### Scenario: 解析失敗停用編輯

- **WHEN** config.yaml 被外部改壞為無法解析後使用者開啟專案設定
- **THEN** 專案說明頁與產出規則頁浮出解析失敗說明、編輯鈕停用，不提供任何寫入途徑

### Requirement: UI 介面語言支援 zh-TW 與 en

app 的 UI 介面語言 SHALL 支援 zh-TW 與 en:未設定偏好時 SHALL 跟隨系統語言(系統語言以 zh 開頭判為 zh-TW,其餘判為 en);應用程式設定的「一般」頁與「語言與外觀」頁的「顯示語言」列 SHALL 提供「跟隨系統／zh-TW／en」三選下拉,切換 SHALL 即時對全介面生效並持久化於 app 本機;macOS 的原生選單列 SHALL 同樣跟隨 UI 語言,切換時 SHALL 立即重建。UI 語言偏好與 config.yaml 的 locale(AI artifacts 產出語言)SHALL 互不影響。兩語言字典的 key 集合 SHALL 相等;查無 key 時 SHALL 顯示 key 本身而非另一語言的字串。

#### Scenario: 未設定偏好時跟隨系統語言

- **WHEN** app 於 UI 語言偏好未設定的狀態下啟動
- **THEN** 系統語言以 zh 開頭時全介面(含 macOS 選單列)呈現 zh-TW,否則呈現 en

##### Example: 系統語言判定

| 系統語言 | UI 語言 |
| -------- | ------- |
| zh-TW | zh-TW |
| zh-CN | zh-TW |
| en-US | en |
| ja-JP | en |

#### Scenario: 手動切換即時生效並持久化

- **WHEN** 使用者於一般頁的顯示語言列將 UI 語言由 zh-TW 切為 en
- **THEN** 全介面(標題列、圖示列、專案欄、看板、對話框、macOS 選單列)即時改為英文,重啟 app 後仍為英文,且 config.yaml 內容未被此操作改動

#### Scenario: UI 語言與 artifacts 產出語言互不影響

- **WHEN** config.yaml 設定 locale: tw,使用者將 UI 語言切為 en
- **THEN** UI 呈現英文,而 config.yaml 的 locale 仍為 tw(引擎產出 artifacts 的語言政策不受 UI 語言影響)

### Requirement: 設定頁政策下拉的未知值顯性呈現

專案設定產出流程頁的 locale 與 spec_locale 下拉（local 與遠端專案皆適用）在儲存值非空且不在合法選項集（locale：tw／ja／en；spec_locale：tw／ja／en／auto）時，SHALL 於下拉顯示該原始值並帶無效標註與警示樣式，且該欄位下方 SHALL 顯示引導改選合法代碼的提示文字；SHALL NOT 呈現為空白，SHALL NOT 於讀取時自動清空或改寫儲存值（寫入嚴格、讀取寬容）。使用者改選合法選項並儲存後，SHALL 以所選代碼覆蓋原值，下拉 SHALL 恢復正常呈現且提示文字 SHALL 消失。儲存值為空（未設定）或在合法選項集內時，本需求 SHALL NOT 改變既有呈現。

#### Scenario: 未知儲存值顯性呈現且不被改寫

- **WHEN** 專案的 locale 儲存值為「繁體中文」（合法選項集外的字串），使用者開啟專案設定的產出流程頁
- **THEN** locale 下拉顯示「繁體中文」並帶無效標註與警示樣式，欄位下方出現改選合法代碼的提示文字；未執行任何儲存動作前，儲存端的值仍為「繁體中文」

#### Scenario: 改選合法代碼即修復

- **WHEN** 於上述狀態，使用者將 locale 下拉改選 tw 並儲存
- **THEN** 儲存端的 locale 值成為 tw，下拉正常顯示 tw 選項，無效標註與提示文字消失

#### Scenario: 合法值與未設定不受影響

- **WHEN** 專案的 locale 儲存值為 tw、spec_locale 未設定，使用者開啟專案設定的產出流程頁
- **THEN** locale 下拉正常顯示 tw、spec_locale 顯示未設定預設狀態，無任何無效標註或提示文字

### Requirement: 產出政策的 worktree 開關

local workspace 的產出流程頁產出政策卡 SHALL 顯示 worktree 開關（列右側開關），文案於 zh-TW 與 en 介面語言下均直出「worktree」一詞（LANGUAGE.md 明文例外）。存檔 SHALL 以開關的畫面實值寫入 openspec/config.yaml 的 worktree 鍵（汰換「UI 無此欄位時恆回填原值」的保值行為），寫入成功後 SHALL 觸發與 CLI workflow-config set 同一技能足跡同步；設定頁載入時 SHALL 反映 config 現值。由開改關存檔且存在活躍 linked worktree 時，SHALL 拒絕寫入並浮出擋下訊息——列出各活躍 worktree 的 change 名、分支與路徑，提示先執行 worktree-merge 收尾——config 維持不變。remote 工作區的設定頁 SHALL NOT 顯示此開關。

#### Scenario: 開關顯示且存檔生效

- **WHEN** local workspace 於產出流程頁將 worktree 開關切為開啟並存檔
- **THEN** openspec/config.yaml 的 worktree 鍵為 true，技能足跡出現兩顆 worktree 技能，設定頁重新載入後開關維持開啟

#### Scenario: 關閉遇活躍 worktree 浮出擋下

- **WHEN** 存在活躍 linked worktree 時於設定頁將 worktree 開關切為關閉並存檔
- **THEN** 存檔失敗浮出，訊息列出各 worktree 的 change 名、分支與路徑及先收尾的指引，openspec/config.yaml 不變，開關回復為開啟

#### Scenario: CLI 先行寫入後設定頁不吃鍵

- **WHEN** 以 CLI 將 worktree 設為 true 後，開啟設定頁再存檔
- **THEN** 設定頁載入時開關即呈現開啟，存檔後 openspec/config.yaml 的 worktree 鍵維持 true

#### Scenario: remote 工作區不顯示開關

- **WHEN** remote 工作區開啟產出流程頁的產出政策卡
- **THEN** 不顯示 worktree 開關，其餘政策欄位照常

### Requirement: 產出流程的切換寫入
產出流程頁的產出流程卡 SHALL 提供下拉切換專案 schema：選定後把 schema 鍵寫入 openspec/config.yaml，寫入 SHALL 複用引擎的 byte-preserving setter（set_workflow_schema_text）——其餘內容逐位元組保留、無法解析的文件拒寫。local 模式直寫檔案；remote 模式 SHALL 走既有 revision 守門的 config 寫入通道，revision 落後時顯性失敗。切換成功後產出規則分節的固定鍵 SHALL 隨新 schema 的 artifact 圖更新——例外：產出規則正在編輯中時，編輯面 SHALL 凍結在開編輯當下的分節（草稿不因換集而丟棄或清空），固定鍵於該次編輯儲存或取消後才跟上；編輯期間換入的新固定鍵其既有規則 SHALL 在儲存時原樣保留。

#### Scenario: 切換寫入且其餘內容保留
- **WHEN** 使用者把專案 schema 從 spec-driven 切到自訂 schema
- **THEN** config.yaml 的 schema 鍵更新為該名稱，檔內其餘既有內容逐位元組不變，產出規則分節改列新 schema 的 artifact id

#### Scenario: 壞檔拒寫顯性失敗
- **WHEN** config.yaml 無法解析且使用者嘗試切換
- **THEN** 寫入被拒、錯誤浮出於表單，檔案一個位元組不變

### Requirement: 產出流程的客製 fork
產出流程頁的產出流程卡 SHALL 提供 fork 動作（僅 local 模式顯示）：把選中的 schema 複製到專案 openspec/schemas/ 下，複用引擎既有的 fork 函式（複本名為引擎預設 <source>-custom，不收自訂名）；成功後清單 SHALL 即時反映新的專案層 schema。同名跨層時 fork 動作 SHALL 只出現在引擎解析會命中的那一層（project→user→內建的第一命中，含壞檔——引擎的層命中只看檔案存在）：被 shadow 的清單項不提供 fork，避免複製到前層內容。remote 模式 SHALL 不顯示 fork 動作。

#### Scenario: fork 產出專案層複本
- **WHEN** 使用者在 local 模式對 spec-driven 按下 fork
- **THEN** openspec/schemas/spec-driven-custom/ 建立（schema.yaml 與 templates 目錄），清單新增該專案層項目

#### Scenario: remote 模式無 fork 入口
- **WHEN** 工作區連線 remote store 且使用者開啟產出流程頁
- **THEN** 介面不出現 fork 動作

### Requirement: 產出流程的建立
產出流程頁的產出流程卡 SHALL 提供建立動作（僅 local 模式顯示）：收 kebab-case 名稱，呼叫引擎既有的 init_schema 在專案 openspec/schemas/ 下產出預設骨架（schema.yaml 與 templates/ 內每個 artifact 的範本檔）；成功後清單 SHALL 即時反映新的專案層 schema。名稱驗證 SHALL 由引擎承擔（前端不重複規則）：名稱不合法或目標已存在時 SHALL 浮出引擎的錯誤訊息且磁碟不變。建立 SHALL NOT 提供 artifact 佈局輸入——骨架佈局用引擎預設，內容客製交外部編輯器。remote 模式 SHALL 不顯示建立動作。

#### Scenario: 建立產出專案層骨架
- **WHEN** 使用者在 local 模式輸入名稱 my-flow 並送出建立
- **THEN** openspec/schemas/my-flow/ 建立（schema.yaml 與 templates/ 內引擎預設 artifact 的範本檔），清單新增該專案層項目

#### Scenario: 不合法名稱顯性失敗
- **WHEN** 使用者輸入非 kebab-case 名稱（如 My Flow）並送出建立
- **THEN** 引擎的名稱錯誤訊息浮出於表單，openspec/schemas/ 無任何新目錄

##### Example: 建立輸入與結果
| 輸入名稱 | 結果 |
|----------|------|
| my-flow | openspec/schemas/my-flow/ 骨架建立，清單新增專案層項目 |
| My Flow | 拒絕：引擎 kebab-case 錯誤浮出，磁碟不變 |
| my-flow（已存在） | 拒絕：引擎 already exists 錯誤浮出，磁碟不變 |

#### Scenario: remote 模式無建立入口
- **WHEN** 工作區連線 remote store 且使用者開啟產出流程頁
- **THEN** 介面不出現建立動作

### Requirement: 產出流程的編輯入口
產出流程頁的清單項 SHALL 對有磁碟路徑的 schema（專案層與 user 層）提供「開啟所在資料夾」動作（僅 local 模式顯示）：按下後在系統檔案管理器顯示該 schema 的目錄（schema.yaml 與 templates/ 所在處），內容編輯交外部編輯器。內建 schema（內嵌於程式、無磁碟檔案）SHALL 不顯示此動作。快照的每個清單項 SHALL 帶其 schema 目錄的絕對路徑（內建為空）——user 層路徑由快照組裝端解析，前端不自行拼路徑。remote 模式 SHALL 不顯示此動作。

#### Scenario: 專案層項目開啟所在資料夾
- **WHEN** 使用者在 local 模式對建立出的專案層 schema 按下開啟所在資料夾
- **THEN** 系統檔案管理器顯示 openspec/schemas/<name>/ 目錄

#### Scenario: 內建項無編輯入口
- **WHEN** 使用者在 local 模式檢視內建 spec-driven 的清單項
- **THEN** 該項不出現開啟所在資料夾動作

### Requirement: 產出流程的刪除
產出流程頁的產出流程卡 SHALL 對專案層項目提供刪除動作（僅 local 模式顯示；內建無檔案、user 層跨專案共用，均不提供）：按下 SHALL 先開確認對話框，取消 SHALL 零變動；確認後移除專案 openspec/schemas/<name>/ 整個目錄，成功後清單 SHALL 即時反映。刪除目標 SHALL 由名稱固定解析為專案層目錄（不接受任意路徑）。config 的 schema 鍵正指著的 schema（使用中）SHALL 拒刪並浮出顯性錯誤、磁碟不變。remote 模式 SHALL 不顯示刪除動作。

#### Scenario: 刪除經確認後移除專案層目錄
- **WHEN** 使用者對非使用中的專案層 schema 按刪除並在確認對話框按下確認
- **THEN** openspec/schemas/<name>/ 整個目錄移除，清單不再列出該項

#### Scenario: 取消確認零變動
- **WHEN** 使用者按刪除後在確認對話框取消
- **THEN** 磁碟與清單皆無任何變動

#### Scenario: 使用中的 schema 拒刪
- **WHEN** config.yaml 的 schema 鍵指著 my-flow 且使用者確認刪除 my-flow
- **THEN** 錯誤浮出於表單、openspec/schemas/my-flow/ 原封不動

### Requirement: remote 模式的內建限縮與誤解析修正
remote 模式下產出流程頁 SHALL 只列內建 schema，切換下拉的可選目標 SHALL 只含內建（config 的名稱非內建時 SHALL 以停用項顯示現值——沿政策下拉未知值顯性呈現的既有模式，不可被選取）。remote 設定快照解析 schema 名稱時 SHALL NOT 讀取 client 本機的 user 層目錄——名稱為內建即以內嵌定義解析；非內建時 SHALL 顯性呈現「遠端自訂尚不支援」的狀態而非猜測，且產出規則分節不呈現猜測的固定鍵。

#### Scenario: remote 快照不讀本機 user 層
- **WHEN** remote 專案的 config 指定 schema 名稱 X，且 client 本機 user 層目錄恰有同名 schema
- **THEN** 設定快照不以本機定義解析 X；X 非內建時產出規則分節為空並顯示遠端自訂尚不支援的狀態

##### Example: remote 解析結果表
| config 的 schema 名稱 | 本機 user 層有同名 | 快照結果 |
|-----------------------|--------------------|----------|
| spec-driven | 否 | 內建定義解析，artifact 圖正常 |
| spec-driven | 是 | 內建定義解析（本機定義不參與） |
| my-flow | 是 | 不解析，顯示遠端自訂尚不支援 |

#### Scenario: remote 下拉僅內建
- **WHEN** 工作區連線 remote store 且使用者開啟切換下拉
- **THEN** 可選的切換目標只有內建 spec-driven；config 的名稱非內建時以停用項顯示現值（沿政策下拉未知值顯性呈現的既有模式），不可被選取

## REMOVED Requirements

### Requirement: 設定頁的產出流程頁籤

**Reason**: 設定改為分類欄＋單頁，頁籤與其「Schema」標籤例外不再存在

**Migration**: 改以「產出流程頁」承載清單與唯讀詳情，並補產出政策卡的落點
